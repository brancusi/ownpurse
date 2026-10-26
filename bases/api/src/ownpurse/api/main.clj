(ns ownpurse.api.main
  "Runs ownpurse: the record and the API, until the process is stopped."
  (:require
   [donut.system :as ds]
   [ownpurse.api.system :as system]
   [ownpurse.config.interface :as config]
   [ownpurse.store.interface :as store])
  (:gen-class))

(defn- serve []
  (let [running (ds/start (system/system (config/read-config :prod)))]
    (.addShutdownHook (Runtime/getRuntime) (Thread. ^Runnable #(ds/stop running)))
    (println (str "ownpurse is listening on http://127.0.0.1:" (system/http-port running)))
    (when-let [port (system/sql-port running)]
      (println (str "SQL, read-only: postgresql://127.0.0.1:" port "/xtdb")))))

(defn- restore [from dir]
  (try
    (let [{:keys [dir epoch transactions]} (store/restore! {:from from :dir dir})]
      (println (str "Restored " transactions " transactions into " dir ", log epoch " epoch ". Start ownpurse on it "
                    "with OWNPURSE_DATA_DIR=" dir ".")))
    (catch clojure.lang.ExceptionInfo e
      (binding [*out* *err*] (println (ex-message e)))
      (System/exit 1))))

(defn -main
  "With no arguments, starts the system with the :prod profile and stops it on shutdown. `restore <snapshot>
  [<data dir>]` restores a snapshot into a data directory that holds no record (default: OWNPURSE_DATA_DIR, else
  data), with ownpurse stopped."
  [& [command from dir]]
  (case command
    nil (serve)
    "restore" (if from
                (restore from (or dir (get-in (config/read-config :prod) [:store :dir])))
                (do (println "Usage: restore <snapshot folder> [<data dir>]") (System/exit 2)))
    (do (println (str "Unknown command " command ". Run with no arguments to serve, or restore <snapshot folder> "
                      "[<data dir>]."))
        (System/exit 2))))
