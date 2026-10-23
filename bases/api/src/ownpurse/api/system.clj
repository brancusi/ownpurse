(ns ownpurse.api.system
  "The running system: the record and the HTTP API, started and stopped by donut.system."
  (:require
   [donut.system :as ds]
   [ownpurse.api.handler :as handler]
   [ownpurse.api.http :as http]
   [ownpurse.store.interface :as store]))

(defn system
  "The system for `config`, as read by ownpurse.config.interface/read-config."
  [config]
  {::ds/defs
   {:ownpurse
    {:store #::ds{:start (fn [{::ds/keys [config]}] (store/start config))
                  :stop (fn [{::ds/keys [instance]}] (store/stop instance))
                  :config (:store config)}
     :handler #::ds{:start (fn [{{:keys [node]} ::ds/config}] (handler/handler {:node node}))
                    :config {:node (ds/local-ref [:store])}}
     :http #::ds{:start (fn [{::ds/keys [config]}] (http/start config))
                 :stop (fn [{::ds/keys [instance]}] (http/stop instance))
                 :config (assoc (:http config) :handler (ds/local-ref [:handler]))}}}})

(defn node
  "The XTDB node of a running system."
  [running]
  (get-in running [::ds/instances :ownpurse :store]))

(defn http-port
  "The HTTP port of a running system."
  [running]
  (http/port (get-in running [::ds/instances :ownpurse :http])))

(defn sql-port
  "The read-only SQL port of a running system, or nil when it is off."
  [running]
  (store/sql-port (node running)))
