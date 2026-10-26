(ns ownpurse.xero.cli
  "Running xero-cli, which talks to Xero; ownpurse never does. I/O only: no mapping."
  (:require
   [clojure.java.process :as process]
   [clojure.string :as str]
   [ownpurse.json.interface :as json]))

(def exit-types
  "xero-cli's exit codes, as the :type of the error ownpurse throws."
  {1 :xero/usage
   2 :xero/login-needed
   3 :xero/daily-limit
   4 :xero/refused
   5 :xero/failed})

(defn- run
  "Runs `command` (a vector: the program and its arguments), asking for JSON on stdout and stderr (xero-cli prints
  TOON by default). Returns {:exit :out :err}."
  [command]
  (let [p (try
            (apply process/start {:out :pipe :err :pipe :env {"XERO_CLI_FORMAT" "json"}} command)
            (catch java.io.IOException e
              (throw (ex-info (str "Could not run " (pr-str (first command)) ": " (ex-message e)
                                   ". Install xero-cli, or set OWNPURSE_XERO_CLI to its path.")
                              {:type :xero/not-found :command command}))))
        err (future (slurp (process/stderr p)))
        out (slurp (process/stdout p))]
    {:exit (.waitFor ^Process p) :out out :err @err}))

(defn- error-json
  "xero-cli's error, the last line of stderr that parses as {\"error\": ...}, or nil."
  [err]
  (some (fn [line]
          (when (str/starts-with? line "{")
            (:error (try (json/parse line) (catch Exception _ nil)))))
        (reverse (str/split-lines err))))

(defn- fail! [command {:keys [exit err]}]
  (let [{:keys [message hint] :as error} (error-json err)
        type (exit-types exit :xero/failed)]
    (throw (ex-info (str "xero-cli failed (exit " exit "): " (or message (str/trim err))
                         (when hint (str " " hint)))
                    {:type type :exit exit :error error :command command}))))

(defn- command [cli & args]
  (into (if (string? cli) [cli] (vec cli)) args))

(defn get-resource
  "Runs `xero-cli get <resource> --org <tenant> --all [--modified-since <instant>]` and returns Xero's answer, every
  number exact. `cli` is the program (a path, or a vector with leading arguments). Throws an error whose :type
  says what xero-cli's exit code means (see `exit-types`), with xero-cli's own error, hint included, as data."
  [cli tenant resource {:keys [modified-since]}]
  (let [cmd (cond-> (command cli "get" resource "--org" tenant "--all")
              modified-since (conj "--modified-since" (str modified-since)))
        {:keys [exit out] :as result} (run cmd)]
    (if (zero? exit)
      (json/parse out)
      (fail! cmd result))))

(defn get-report
  "Runs `xero-cli report <report> --org <tenant> --param k=v ...` for each of `params` and returns Xero's answer,
  every number exact. Throws as `get-resource` does."
  [cli tenant report params]
  (let [cmd (into (command cli "report" report "--org" tenant)
                  (mapcat (fn [[k v]] ["--param" (str k "=" v)]) (sort params)))
        {:keys [exit out] :as result} (run cmd)]
    (if (zero? exit)
      (json/parse out)
      (fail! cmd result))))

(defn version
  "xero-cli's version, from the first line of its help (`xero-cli 0.1.0: ...`), or nil."
  [cli]
  (let [{:keys [exit out]} (run (command cli "--help"))]
    (when (zero? exit)
      (second (re-find #"^xero-cli (\S+):" out)))))
