(ns dev
  "The REPL system: (go) starts it, (halt) stops it, (reset) stops it, reloads changed namespaces and starts
  it again. The running system is in donut.system.repl.state/system."
  (:require
   [clojure.java.io :as io]
   [clojure.test :as test]
   [clojure.tools.namespace.find :as find]
   [donut.system :as ds]
   [donut.system.repl :as dsr]
   [donut.system.repl.state :as state]
   [ownpurse.api.system :as system]
   [ownpurse.config.interface :as config]))

(defmethod ds/named-system ::ds/repl
  [_]
  (system/system (config/read-config :dev)))

(defn go [] (dsr/start) {:http (system/http-port state/system) :sql (system/sql-port state/system)})
(defn halt [] (dsr/stop))
(defn reset [] (dsr/restart))

(defn node
  "The XTDB node of the running system, for queries at the REPL: (xt/q (node) \"SELECT ...\")."
  []
  (system/node state/system))

(def ^:private test-dirs
  ["components/config/test" "components/store/test" "bases/api/test"])

(defn tests
  "Loads every test namespace and runs them in this REPL. The shell run (clojure -M:dev:test:kaocha) is still the
  truth before a commit."
  []
  (let [namespaces (mapcat #(find/find-namespaces-in-dir (io/file %)) test-dirs)]
    (apply require namespaces)
    (apply test/run-tests namespaces)))
