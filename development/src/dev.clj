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
   [dev.watch :as watch]
   [ownpurse.api.system :as system]
   [ownpurse.config.interface :as config]
   [ownpurse.store.interface :as store]
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

(defn keep-xero-answers
  "Keeps a folder of saved Xero answers (samples/xero/pull-1) as org `tenant` in the running system."
  [tenant dir]
  (into (sorted-map)
        (for [[resource answer] (xero/read-answers dir)]
          [resource (xero/keep! (node) tenant resource answer {})])))

(defn pull-xero
  "Fetches every resource for each org in a local config through xero-cli, and keeps what changed. Without
  arguments: the running system's record and config.local.edn (never committed). `opts` may name another
  :node (a record opened with `open-record`) and another :local config file. Costs Xero calls: the first pull
  of an org is about one call per resource per 1000 items; later pulls ask only for what changed."
  ([] (pull-xero {}))
  ([{:keys [local] :as opts}]
   (let [{:keys [cli] default-local :local} (:xero (config/read-config :dev))]
     (xero/pull-orgs! (or (:node opts) (node)) {:cli cli :orgs (xero/read-orgs (or local default-local))}))))

(defn open-record
  "Opens a record on disk at `dir` beside the running system, with its read-only SQL on `sql-port` (or none):
  for a private record that lives outside this repository. Close it with (store/stop node)."
  ([dir] (open-record dir nil))
  ([dir sql-port]
   (store/start {:mode :disk :dir dir :sql (when sql-port {:host "127.0.0.1" :read-only-port sql-port})})))

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

(defn watch
  "Tests on save: every save of a brick's .clj file reloads it and runs the tests that cover it. Results print to
  the REPL's output and (last-run) keeps the latest. (unwatch) stops it."
  []
  (watch/start))

(defn unwatch [] (watch/stop))

(defn last-run
  "What the last save reloaded and tested, with the test output, or why the file did not load."
  []
  (dissoc (watch/last-run) :exception))
