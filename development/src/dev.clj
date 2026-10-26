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
   [ownpurse.mercury.interface :as mercury]
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

(defn backup
  "Snapshots the running system's record into a new folder under `dir` while it runs (see docs/BACKUP.md)."
  [dir]
  (store/snapshot! (node) (:store (config/read-config :dev)) dir))

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
  of an org is about one call per resource per 1000 items; later pulls ask only for what changed. Reports cost
  five calls a month: 65 on the first pull, 10 on later ones (see xero/default-reports). Documents' history and
  attachments cost a call each, at most 100 a pull, and their files one each, none unless the config asks (see
  xero/default-history). Files are kept only in the running system's record, not in another :node."
  ([] (pull-xero {}))
  ([{:keys [local] :as opts}]
   (let [{:keys [xero] default-local :local} (config/read-config :dev)]
     (xero/pull-orgs! (or (:node opts) (node))
                      {:cli (:cli xero) :orgs (xero/read-orgs (or local default-local))
                       :reports (xero/read-reports (or local default-local))
                       :history (xero/read-history (or local default-local))
                       :keep-file (when-not (:node opts) #(store/keep-file! (:store (config/read-config :dev)) %))}))))

(defn keep-mercury-answers
  "Pulls a folder of saved Mercury answers (samples/mercury/pull-1) into the running system, through the stand-in
  mercury-cli the tests use, as of the instant `now` (default: now)."
  ([dir] (keep-mercury-answers dir (java.time.Instant/now)))
  ([dir now]
   (mercury/pull! (node) {:cli ["bash" "components/mercury/test/ownpurse/mercury/fake-mercury-cli.sh" dir "0"
                                "/dev/null"]
                          :alias "tallowmere"
                          :now now})))

(defn pull-mercury
  "Fetches each Mercury org in a local config through mercury-cli, and keeps what changed. Without arguments: the
  running system's record and config.local.edn (never committed). `opts` may name another :node and another
  :local config file. The first pull of an org asks for every transaction; later ones for the recent and the
  pending ones."
  ([] (pull-mercury {}))
  ([{:keys [local] :as opts}]
   (let [{:keys [mercury] default-local :local} (config/read-config :dev)]
     (mercury/pull-orgs! (or (:node opts) (node))
                         {:cli (:cli mercury) :orgs (mercury/read-orgs (or local default-local))}))))

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
