(ns ownpurse.xero.pull
  "Fetching through xero-cli and keeping what Xero said, as source rows in the xero schema."
  (:require
   [clojure.edn :as edn]
   [clojure.java.io :as io]
   [clojure.string :as str]
   [ownpurse.json.interface :as json]
   [ownpurse.store.interface :as store]
   [ownpurse.xero.cli :as cli]
   [ownpurse.xero.source :as source])
  (:import
   (java.time Instant ZonedDateTime)))

(defn keep!
  "Keeps Xero's answer to `resource` for org `tenant`, writing only the rows that changed, in one transaction that
  names the tool, the org, the resource and when Xero answered. Returns {:items n :written n}."
  [node tenant resource answer {:keys [tool-version modified-since now]}]
  (let [fetched-at (source/fetched-at answer (or now (Instant/now)))
        tables (source/rows tenant resource answer fetched-at)
        metadata (cond-> {:source "xero-cli" :why "fetch" :org tenant :resource resource
                          :fetched-at (str fetched-at)}
                   tool-version (assoc :tool-version tool-version)
                   modified-since (assoc :modified-since (str modified-since)))
        {:keys [written]} (store/put-changed! node tables metadata)]
    {:items (count (val (first tables))) :written (reduce + 0 (vals written))}))

(defn- last-update
  "The latest UpdatedDateUTC the store holds for `resource` in org `tenant`, or nil (no rows yet, or a resource
  without one): the --modified-since of the next fetch."
  [node tenant resource]
  (let [table (:table (source/describe resource))
        sql (str "SELECT MAX(updated_date_utc) AS since FROM " (namespace table) "."
                 (str/replace (name table) "-" "_") " WHERE org = ?")]
    (try
      (some-> (store/q node [sql tenant]) first :since (#(.toInstant ^ZonedDateTime %)))
      (catch Exception _ nil))))

(defn- ordered
  "A map of `pairs` that keeps their order, however many there are (into (array-map) stops at eight)."
  [pairs]
  (apply array-map (mapcat identity pairs)))

(defn pull!
  "Fetches each of `resources` for org `tenant` through xero-cli and keeps it. After the first fetch, a resource
  that has UpdatedDateUTC asks only for what changed since the latest one held. Returns {resource {:items
  :written}}. Stops at the first error xero-cli reports (see ownpurse.xero.cli/exit-types); what was kept before
  it stays kept."
  [node {:keys [cli tenant resources]}]
  (let [tool-version (cli/version cli)]
    (ordered (for [resource resources
                   :let [since (last-update node tenant resource)
                         answer (cli/get-resource cli tenant resource {:modified-since since})]]
               [resource (keep! node tenant resource answer {:tool-version tool-version :modified-since since})]))))

(defn read-orgs
  "The orgs a local config file names: {:xero {:orgs [{:tenant \"<tenant id>\" :name \"...\"}]}}. Says how to make
  the file when it is missing."
  [path]
  (let [f (io/file path)]
    (when-not (.isFile f)
      (throw (ex-info (str "No config at " path ". Make it with the orgs to read: "
                           "{:xero {:orgs [{:tenant \"<tenant id>\" :name \"<a name>\"}]}} (xero-cli orgs lists them).")
                      {:type :ownpurse/no-local-config :path (str path)})))
    (let [orgs (get-in (edn/read-string (slurp f)) [:xero :orgs])]
      (when-not (and (seq orgs) (every? (comp string? :tenant) orgs))
        (throw (ex-info (str path " names no orgs. Add {:xero {:orgs [{:tenant \"<tenant id>\" :name \"<a name>\"}]}}.")
                        {:type :ownpurse/no-local-config :path (str path)})))
      orgs)))

(defn pull-orgs!
  "Pulls every resource in `source/pull-order` for each of `orgs` ({:tenant :name}), one org after another.
  Returns {name-or-tenant {resource {:items :written}}}."
  [node {:keys [cli orgs]}]
  (ordered (for [{:keys [tenant name]} orgs]
             [(or name tenant) (pull! node {:cli cli :tenant tenant :resources source/pull-order})])))

(defn read-answers
  "Reads a folder of Xero answers saved as files, one per resource and named for its list (Invoices.json,
  Organisations.json), in the shape xero-cli prints. Returns {resource answer}. For the synthetic samples."
  [dir]
  (into (sorted-map)
        (for [^java.io.File f (.listFiles (io/file dir))
              :let [[_ list-name] (re-matches #"(.+)\.json" (.getName f))
                    resource (some-> list-name keyword source/resource-of-list)]
              :when resource]
          [resource (json/parse f)])))
