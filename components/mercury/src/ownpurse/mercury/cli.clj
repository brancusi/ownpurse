(ns ownpurse.mercury.cli
  "Running mercury-cli, which talks to Mercury; ownpurse never does. I/O only: no mapping."
  (:require
   [clojure.java.process :as process]
   [clojure.string :as str]
   [ownpurse.json.interface :as json]))

(def exit-types
  "mercury-cli's exit codes, as the :type of the error ownpurse throws."
  {1 :mercury/usage
   2 :mercury/login-needed
   3 :mercury/rate-limited
   4 :mercury/refused
   5 :mercury/failed})

(defn- run
  "Runs `command` (a vector: the program and its arguments). Returns {:exit :out :err}."
  [command]
  (let [p (try
            (apply process/start {:out :pipe :err :pipe} command)
            (catch java.io.IOException e
              (throw (ex-info (str "Could not run " (pr-str (first command)) ": " (ex-message e)
                                   ". Install mercury-cli, or set OWNPURSE_MERCURY_CLI to its path.")
                              {:type :mercury/not-found :command command}))))
        err (future (slurp (process/stderr p)))
        out (slurp (process/stdout p))]
    {:exit (.waitFor ^Process p) :out out :err @err}))

(defn- error-json
  "mercury-cli's error, the last line of stderr that parses as {\"error\": ...}, or nil."
  [err]
  (some (fn [line]
          (when (str/starts-with? line "{")
            (:error (try (json/parse line) (catch Exception _ nil)))))
        (reverse (str/split-lines err))))

(defn- fail! [command {:keys [exit err]}]
  (let [{:keys [message hint] :as error} (error-json err)
        type (exit-types exit :mercury/failed)]
    (throw (ex-info (str "mercury-cli failed (exit " exit "): " (or message (str/trim err))
                         (when hint (str " " hint)))
                    {:type type :exit exit :error error :command command}))))

(defn- command [cli & args]
  (into (if (string? cli) [cli] (vec cli)) args))

(defn get-resource
  "Runs `mercury-cli get <resource> --org <alias> --all --format json` and returns Mercury's answer, every number
  exact. opts: :account (the account id, for statements), :start (a date: transactions created since) and :status
  (pending, sent, ...). `cli` is the program (a path, or a vector with leading arguments). Throws an error whose
  :type says what mercury-cli's exit code means (see `exit-types`), with mercury-cli's own error as data."
  [cli alias resource {:keys [account start status]}]
  (let [cmd (cond-> (command cli "get" resource "--org" alias "--all" "--format" "json")
              account (conj "--account" account)
              start (conj "--start" (str start))
              status (conj "--status" status))
        {:keys [exit out] :as result} (run cmd)]
    (if (zero? exit)
      (json/parse out)
      (fail! cmd result))))

(defn version
  "mercury-cli's version, from the first line of its help (`mercury-cli 0.1.0: ...`), or nil."
  [cli]
  (let [{:keys [exit out]} (run (command cli "--help"))]
    (when (zero? exit)
      (second (re-find #"^mercury-cli (\S+):" out)))))
