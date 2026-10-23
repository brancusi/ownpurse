(ns dev
  "The REPL system: (go) starts it, (halt) stops it, (reset) stops it, reloads changed namespaces and starts
  it again."
  (:require
   [donut.system :as ds]
   [donut.system.repl :as dsr]))

(def system
  "Empty until the store and the API exist (step 1)."
  {::ds/defs {}})

(defmethod ds/named-system ::ds/repl
  [_]
  system)

(defn go [] (dsr/start))
(defn halt [] (dsr/stop))
(defn reset [] (dsr/restart))
