(ns ownpurse.plaid.cli
  "Running plaid-reader, which talks to Plaid; ownpurse never does. I/O only: no mapping."
  (:require
   [clojure.java.process :as process]
   [clojure.string :as str]
   [ownpurse.json.interface :as json]))

(def exit-types
  "plaid-reader's exit codes, as the :type of the error ownpurse throws."
  {1 :plaid/usage
   2 :plaid/login-needed
   3 :plaid/rate-limited
   4 :plaid/refused
   5 :plaid/failed})

(defn- run
  "Runs `command` (a vector: the program and its arguments). Returns {:exit :out :err}."
  [command]
  (let [p (try
            (apply process/start {:out :pipe :err :pipe} command)
            (catch java.io.IOException e
              (throw (ex-info (str "Could not run " (pr-str (first command)) ": " (ex-message e)
                                   ". Install plaid-reader, or set OWNPURSE_PLAID_CLI to its path.")
                              {:type :plaid/not-found :command command}))))
        err (future (slurp (process/stderr p)))
        out (slurp (process/stdout p))]
    {:exit (.waitFor ^Process p) :out out :err @err}))

(defn- error-json
  "plaid-reader's error, the last line of stderr that parses as {\"error\": ...}, or nil."
  [err]
  (some (fn [line]
          (when (str/starts-with? line "{")
            (:error (try (json/parse line) (catch Exception _ nil)))))
        (reverse (str/split-lines err))))

(defn- fail! [command {:keys [exit err]}]
  (let [{:keys [message hint] :as error} (error-json err)
        type (exit-types exit :plaid/failed)]
    (throw (ex-info (str "plaid-reader failed (exit " exit "): " (or message (str/trim err))
                         (when hint (str " " hint)))
                    {:type type :exit exit :error error :command command}))))

(defn- command [cli & args]
  (into (if (string? cli) [cli] (vec cli)) args))

(defn- answer! [cmd]
  (let [{:keys [exit out] :as result} (run cmd)]
    (if (zero? exit)
      (json/parse out)
      (fail! cmd result))))

(defn item
  "Runs `plaid-reader item --item <alias> --format json`: Plaid's /item/get answer. `cli` is the program (a path, or
  a vector with leading arguments). Throws an error whose :type says what plaid-reader's exit code means (see
  `exit-types`), with plaid-reader's own error as data."
  [cli alias]
  (answer! (command cli "item" "--item" alias "--format" "json")))

(defn sync-all
  "Runs `plaid-reader sync --item <alias> --all --format json`, from `cursor` when there is one: every transaction
  added, modified and removed since it, the item's accounts, and the cursor to start from next time."
  [cli alias cursor]
  (answer! (cond-> (command cli "sync" "--item" alias "--all" "--format" "json")
             cursor (conj "--cursor" cursor))))

(defn version
  "plaid-reader's version, from the first line of its help (`plaid-reader 0.1.0: ...`), or nil."
  [cli]
  (let [{:keys [exit out]} (run (command cli "--help"))]
    (when (zero? exit)
      (second (re-find #"^plaid-reader (\S+):" out)))))
