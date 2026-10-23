(ns dev
  "The REPL system: (go) starts it, (halt) stops it, (reset) stops it, reloads changed namespaces and starts
  it again. The running system is in donut.system.repl.state/system."
  (:require
   [clojure.java.io :as io]
   [clojure.string :as str]
   [clojure.test :as test]
   [clojure.tools.namespace.find :as find]
   [donut.system :as ds]
   [donut.system.repl :as dsr]
   [donut.system.repl.state :as state]
   [ownpurse.api.system :as system]
   [ownpurse.config.interface :as config]
   [ownpurse.ledger.interface :as ledger]
   [ownpurse.xero.interface :as xero]))

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

(defn load-xero
  "Loads a xero-axi record and its gl into the running system: {:record path, :gl path}. Returns what was
  written; a second load of the same files writes nothing."
  [paths]
  (let [source (xero/read-source paths)]
    (ledger/load! (node) (xero/facts source) (xero/metadata source))))

(defn load-sample
  "Loads the synthetic Brackenridge group (samples/xero-axi/brackenridge)."
  []
  (load-xero {:record "samples/xero-axi/brackenridge/record.sqlite"
              :gl "samples/xero-axi/brackenridge/gl.json"}))

(defn tests
  "Loads every test namespace in the bricks and runs them in this REPL. The shell run (clojure -M:dev:test:kaocha)
  is still the truth before a commit."
  []
  (let [namespaces (->> ["components" "bases"]
                        (mapcat #(find/find-namespaces-in-dir (io/file %)))
                        (filter #(str/ends-with? (name %) "-test"))
                        sort)]
    (apply require namespaces)
    (apply test/run-tests namespaces)))
