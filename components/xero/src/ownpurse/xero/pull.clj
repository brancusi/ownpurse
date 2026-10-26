(ns ownpurse.xero.pull
  "Fetching through xero-cli and keeping what Xero said, as source rows in the xero schema."
  (:require
   [clojure.edn :as edn]
   [clojure.java.io :as io]
   [clojure.string :as str]
   [ownpurse.json.interface :as json]
   [ownpurse.schema.interface :as schema]
   [ownpurse.store.interface :as store]
   [ownpurse.xero.cli :as cli]
   [ownpurse.xero.source :as source])
  (:import
   (java.time Instant LocalDate ZoneOffset ZonedDateTime)))

(defn- with-held
  "`rows` over every row of `table` the store already holds for `org`: a fetch that asks only for what changed
  still yields every source account, and a record written before source_accounts existed fills it."
  [node table org rows]
  (let [held (when (store/table-exists? node table)
               (store/q node [(str "SELECT * FROM " (namespace table) "." (str/replace (name table) "-" "_")
                                   " WHERE org = ?")
                              org]))
        by-id #(into {} (map (juxt :xt/id identity)) %)]
    (vals (merge (by-id held) (by-id rows)))))

(defn- end-stale-lines!
  "Ends, by DELETE, the bank lines of `source-table` for `org` that are no longer among `ids`: a line whose payment
  was voided, or whose transaction failed, stops counting, and its history stays."
  [node source-table org ids metadata]
  (when (store/table-exists? node :bank-lines)
    (let [stale (into [] (comp (map :xt/id) (remove (set ids)))
                      (store/q node ["SELECT _id FROM bank_lines WHERE source_table = ? AND org = ?" source-table org]))]
      (when (seq stale)
        (store/delete! node :bank-lines stale {} (assoc metadata :why "fetch: lines that no longer hold"))))))

(defn keep!
  "Keeps Xero's answer to `resource` for org `tenant`, writing only the rows that changed, in one transaction that
  names the tool, the org, the resource and when Xero answered. Kept with them, from every row the store holds for
  the org and not only those fetched: the org and its bank accounts as source_accounts rows, and its bank
  transactions and payments as bank_lines rows; a line that no longer holds ends. Returns {:items n :written n},
  counting the resource's own rows."
  [node tenant resource answer {:keys [tool-version modified-since now]}]
  (let [fetched-at (source/fetched-at answer (or now (Instant/now)))
        [[table rows]] (seq (source/rows tenant resource answer fetched-at))
        accounts (when (source/account-resources resource)
                   (mapv #(schema/check! :ownpurse/source-account %)
                         (source/source-accounts resource (with-held node table tenant rows))))
        lines (when (source/line-resources resource)
                (mapv #(schema/check! :ownpurse/bank-line %)
                      (source/bank-lines resource (with-held node table tenant rows))))
        tables (cond-> {table rows} (seq accounts) (assoc :source-accounts accounts) lines (assoc :bank-lines lines))
        metadata (cond-> {:source "xero-cli" :why "fetch" :org tenant :resource resource
                          :fetched-at (str fetched-at)}
                   tool-version (assoc :tool-version tool-version)
                   modified-since (assoc :modified-since (str modified-since)))
        {:keys [written]} (store/put-changed! node tables metadata)]
    (when lines
      (end-stale-lines! node (str (namespace table) "." (str/replace (name table) "-" "_")) tenant (map :xt/id lines)
                        metadata))
    {:items (count rows) :written (get written table 0)}))

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

(def default-reports
  "How many months of reports a pull keeps (ending with this one), and how many of the latest it fetches again on
  every pull, since Xero's figures for a month change while its books are open. Older months are fetched once.
  Each month costs five calls of the org's daily 1000."
  {:months 13 :refresh 2})

(defn- held-ids [node table ids]
  (if (and (seq ids) (store/table-exists? node table))
    (set (map :xt/id (store/q node (into [(str "SELECT _id FROM " (namespace table) "." (name table) " WHERE _id IN ("
                                               (str/join ", " (repeat (count ids) "?")) ")")]
                                         ids))))
    #{}))

(defn pull-reports!
  "Fetches Xero's reports for org `tenant` (see `source/report-requests`) through xero-cli and keeps each one that
  changed as a new version of its row in xero.reports. The latest `:refresh` months are fetched on every pull, older
  ones only while the store does not hold them. `:today` is the day the months end with. Returns {:items n
  :written n}: the reports fetched, and those that changed."
  [node {:keys [cli tenant today tool-version] {:keys [months refresh]} :reports}]
  (let [requests (source/report-requests today months)
        refresh-from (.withDayOfMonth (.minusMonths ^LocalDate today (dec (max refresh 1))) 1)
        held (held-ids node :xero/reports (mapv #(source/report-id tenant (:report %) (:params %)) requests))
        wanted (filter (fn [{:keys [report params month-end]}]
                         (or (not (.isBefore ^LocalDate month-end refresh-from))
                             (not (held (source/report-id tenant report params)))))
                       requests)]
    (reduce (fn [acc {:keys [report params]}]
              (let [answer (cli/get-report cli tenant report params)
                    fetched-at (source/fetched-at answer (Instant/now))
                    row (source/report-row tenant report params answer fetched-at)
                    {:keys [written]} (store/put-changed! node {:xero/reports [row]}
                                                          (cond-> {:source "xero-cli" :why "fetch" :org tenant
                                                                   :resource (str "Reports/" report)
                                                                   :fetched-at (str fetched-at)}
                                                            tool-version (assoc :tool-version tool-version)))]
                (-> acc (update :items inc) (update :written + (get written :xero/reports 0)))))
            {:items 0 :written 0}
            wanted)))

(def default-history
  "How many calls a pull may spend, per org, on documents' history and attachment details, and on which documents
  (see `source/history-resources`). A call fetches one document's history, or the list of its attachments. Kept
  well under the org's daily 1000, so that an ordinary pull never runs out; what is left waits for the next pull."
  {:calls 100 :resources source/history-resources})

(defn- sql-table [table]
  (str (namespace table) "." (str/replace (name table) "-" "_")))

(defn- instant-of [t]
  (if (instance? ZonedDateTime t) (.toInstant ^ZonedDateTime t) t))

(defn- documents
  "The documents of `resource` the store holds for org `tenant`: {:xt/id :xero-id :updated :has-attachments}."
  [node tenant resource]
  (let [{:keys [table]} (source/describe resource)]
    (when (store/table-exists? node table)
      (for [r (store/q node [(str "SELECT _id, " (source/id-column resource) " AS xero_id, updated_date_utc, "
                                  "has_attachments FROM " (sql-table table) " WHERE org = ?")
                             tenant])]
        {:xt/id (:xt/id r)
         :xero-id (:xero-id r)
         :updated (instant-of (:updated-date-utc r))
         :has-attachments (true? (:has-attachments r))}))))

(defn- held-documents
  "What `table` (xero.history or xero.attachments) holds for org `tenant`, by document: {:updated :any?}."
  [node table tenant]
  (if (store/table-exists? node table)
    (into {}
          (map (fn [r] [(:document r) {:updated (instant-of (:document-updated r))
                                       :any? (boolean (seq (:attachments r)))}]))
          (store/q node [(str "SELECT document, document_updated"
                              (when (= :xero/attachments table) ", attachments")
                              " FROM " (sql-table table) " WHERE org = ?")
                         tenant]))
    {}))

(defn pull-history!
  "Fetches the history and the attachment details of org `tenant`'s documents through xero-cli, for each document
  the store holds a newer version of than the one its history was fetched for (`source/document-calls`), in the
  order of `:resources`, newest first, and at most `:calls` of them. Each answer is a row of xero.history or
  xero.attachments, a new version when it changed. A document Xero refuses is skipped and tried again on the next
  pull. Returns {:items n :written n :waiting n}, with :refused n when Xero refused any: :waiting counts the calls
  left for later pulls."
  [node {:keys [cli tenant tool-version] {:keys [calls resources]} :history}]
  (let [history (held-documents node :xero/history tenant)
        attachments (held-documents node :xero/attachments tenant)
        wanted (mapcat #(source/document-calls % (documents node tenant %) history attachments) resources)
        [now later] (split-at calls wanted)
        refused? #(= :xero/refused (:type (ex-data %)))]
    (-> (reduce (fn [acc {:keys [resource what] :as call}]
                  (if-let [answer (try (cli/get-path cli tenant (source/document-path call))
                                       (catch clojure.lang.ExceptionInfo e (when-not (refused? e) (throw e))))]
                    (let [fetched-at (source/fetched-at answer (Instant/now))
                          tables (source/document-row tenant call answer fetched-at)
                          {:keys [written]} (store/put-changed! node tables
                                                                (cond-> {:source "xero-cli" :why "fetch" :org tenant
                                                                         :resource (str resource "/" what)
                                                                         :fetched-at (str fetched-at)}
                                                                  tool-version (assoc :tool-version tool-version)))]
                      (-> acc (update :items inc) (update :written + (reduce + 0 (vals written)))))
                    (update acc :refused (fnil inc 0))))
                {:items 0 :written 0}
                now)
        (assoc :waiting (count later)))))

(defn pull!
  "Fetches each of `resources` for org `tenant` through xero-cli and keeps it. After the first fetch, a resource
  that has UpdatedDateUTC asks only for what changed since the latest one held. Then the reports (`pull-reports!`,
  with `:reports` over `default-reports`; {:months 0} fetches none), counted under \"Reports\", and the documents'
  history and attachments (`pull-history!`, with `:history` over `default-history`; {:calls 0} fetches none),
  counted under \"History\". Returns {resource {:items :written}}. Stops at the first error xero-cli reports (see
  ownpurse.xero.cli/exit-types); what was kept before it stays kept."
  [node {:keys [cli tenant resources reports history today]}]
  (let [tool-version (cli/version cli)
        reports (merge default-reports reports)
        history (merge default-history history)
        kept (vec (for [resource resources
                        :let [since (last-update node tenant resource)
                              answer (cli/get-resource cli tenant resource {:modified-since since})]]
                    [resource (keep! node tenant resource answer {:tool-version tool-version :modified-since since})]))]
    (ordered (cond-> kept
               (pos? (:months reports))
               (conj ["Reports" (pull-reports! node {:cli cli :tenant tenant :reports reports :tool-version tool-version
                                                     :today (or today (LocalDate/now ZoneOffset/UTC))})])
               (pos? (:calls history))
               (conj ["History" (pull-history! node {:cli cli :tenant tenant :history history
                                                     :tool-version tool-version})])))))

(defn read-orgs
  "The orgs a local config file names: {:xero {:orgs [{:tenant \"<tenant id>\" :name \"...\"}]}}, or [] when it
  names none. Says how to make the file when it is missing."
  [path]
  (let [f (io/file path)]
    (when-not (.isFile f)
      (throw (ex-info (str "No config at " path ". Make it with the orgs to read: "
                           "{:xero {:orgs [{:tenant \"<tenant id>\" :name \"<a name>\"}]}} (xero-cli orgs lists them).")
                      {:type :ownpurse/no-local-config :path (str path)})))
    (let [orgs (get-in (edn/read-string (slurp f)) [:xero :orgs])]
      (when-not (every? (comp string? :tenant) orgs)
        (throw (ex-info (str path " names a Xero org without a tenant. Write {:xero {:orgs [{:tenant \"<tenant id>\" :name \"<a name>\"}]}}.")
                        {:type :ownpurse/no-local-config :path (str path)})))
      (vec orgs))))

(defn read-reports
  "Which reports a local config file asks for, {:xero {:reports {:months n :refresh n}}}, over `default-reports`."
  [path]
  (let [f (io/file path)]
    (merge default-reports (when (.isFile f) (get-in (edn/read-string (slurp f)) [:xero :reports])))))

(defn read-history
  "How many calls a pull may spend on documents' history and attachments, and on which documents, as a local config
  file says, {:xero {:history {:calls n :resources [...]}}}, over `default-history`."
  [path]
  (let [f (io/file path)]
    (merge default-history (when (.isFile f) (get-in (edn/read-string (slurp f)) [:xero :history])))))

(defn pull-orgs!
  "Pulls every resource in `source/pull-order`, then the reports and the documents' history, for each of `orgs`
  ({:tenant :name}), one org after another. Returns {name-or-tenant {resource {:items :written}}}."
  [node {:keys [cli orgs reports history today]}]
  (ordered (for [{:keys [tenant name]} orgs]
             [(or name tenant) (pull! node {:cli cli :tenant tenant :resources source/pull-order :reports reports
                                            :history history :today today})])))

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
