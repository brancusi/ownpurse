(ns ownpurse.api.main
  "Runs ownpurse: the record and the API, until the process is stopped."
  (:require
   [donut.system :as ds]
   [ownpurse.api.system :as system]
   [ownpurse.config.interface :as config])
  (:gen-class))

(defn -main
  "Starts the system with the :prod profile and stops it on shutdown."
  [& _]
  (let [running (ds/start (system/system (config/read-config :prod)))]
    (.addShutdownHook (Runtime/getRuntime) (Thread. ^Runnable #(ds/stop running)))
    (println (str "ownpurse is listening on http://127.0.0.1:" (system/http-port running)))
    (when-let [port (system/sql-port running)]
      (println (str "SQL, read-only: postgresql://127.0.0.1:" port "/xtdb")))))
