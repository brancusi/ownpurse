(ns ownpurse.mercury.pull
  "Fetching through mercury-cli and keeping what Mercury said, as source rows in the mercury schema."
  (:require
   [clojure.edn :as edn]
   [clojure.java.io :as io]
   [ownpurse.mercury.cli :as cli]
   [ownpurse.mercury.source :as source]
   [ownpurse.store.interface :as store])
  (:import
   (java.time Instant LocalDate ZoneOffset)))

(def default-window-days
  "How far back a later pull asks for transactions by creation date. Mercury has no modified-since: a transaction
  that changes after this many days, other than a pending one, is seen only by a full pull."
  60)

(defn keep!
  "Keeps Mercury's answer to `resource` for Mercury org `org`, writing only the rows that changed, in one
  transaction that names the tool, the org, the resource and when it was fetched. Returns {:items n :written n}."
  [node org resource answer {:keys [tool-version alias account start now]}]
  (let [fetched-at (or now (Instant/now))
        tables (source/rows org resource answer fetched-at account)
        metadata (cond-> {:source "mercury-cli" :why "fetch" :org org :resource resource
                          :fetched-at (str fetched-at)}
                   alias (assoc :alias alias)
                   account (assoc :account account)
                   tool-version (assoc :tool-version tool-version)
                   start (assoc :start (str start)))
        {:keys [written]} (store/put-changed! node tables metadata)]
    {:items (count (val (first tables))) :written (reduce + 0 (vals written))}))

(defn- holds-transactions?
  "Whether the store holds any transaction of Mercury org `org`: after the first pull, later ones ask for less."
  [node org]
  (try
    (seq (store/q node ["SELECT _id FROM mercury.transactions WHERE org = ? LIMIT 1" org]))
    (catch Exception _ nil)))

(defn- ordered
  "A map of `pairs` that keeps their order, however many there are (into (array-map) stops at eight)."
  [pairs]
  (apply array-map (mapcat identity pairs)))

(defn- add [a b]
  (merge-with + a b))

(defn- fetch-credit
  "Credit accounts are for business orgs: Mercury refuses /credit for a personal one, which has none."
  [cli alias]
  (try
    (cli/get-resource cli alias "credit" {})
    (catch clojure.lang.ExceptionInfo e
      (if (= :mercury/refused (:type (ex-data e)))
        {:accounts []}
        (throw e)))))

(defn- fetch-transactions
  "Every transaction on the first pull. Later: those created in the last `window-days`, and every pending one,
  whenever it was created."
  [cli alias first? window-days now]
  (if first?
    [(cli/get-resource cli alias "transactions" {}) nil]
    (let [start (.minusDays (LocalDate/ofInstant now ZoneOffset/UTC) window-days)
          recent (cli/get-resource cli alias "transactions" {:start start})
          pending (cli/get-resource cli alias "transactions" {:status "pending"})
          by-id (into (array-map) (map (juxt :id identity)) (concat (:transactions pending) (:transactions recent)))]
      [{:transactions (vec (vals by-id))} start])))

(defn pull!
  "Fetches and keeps what ownpurse reads of one Mercury org, named by its mercury-cli alias: the org, its deposit
  accounts (archived ones too), its credit accounts, its transactions (every account, credit cards included, every
  status) and the statements of each account. The first pull asks for every transaction; later ones for those
  created in the last `window-days` (default-window-days) and every pending one. Returns {resource {:items
  :written}}. Stops at the first error mercury-cli reports (see ownpurse.mercury.cli/exit-types); what was kept
  before it stays kept."
  [node {:keys [cli alias window-days now]}]
  (let [now (or now (Instant/now))
        window-days (or window-days default-window-days)
        tool-version (cli/version cli)
        opts {:tool-version tool-version :alias alias :now now}
        organization (cli/get-resource cli alias "organization" {})
        org (get-in organization [:organization :id])
        _ (when-not org
            (throw (ex-info (str "Mercury's answer for " alias " names no organization id.")
                            {:type :mercury/failed :alias alias})))
        first? (not (holds-transactions? node org))
        accounts (cli/get-resource cli alias "accounts" {})
        credit (fetch-credit cli alias)
        [transactions start] (fetch-transactions cli alias first? window-days now)
        ;; Statements are asked for one account at a time; the counts add up over the accounts.
        statements (fn [resource account-resource account-answer]
                     (reduce add {:items 0 :written 0}
                             (for [{:keys [id status]} (source/items account-resource account-answer)
                                   :when (not= "deleted" status)]
                               (keep! node org resource (cli/get-resource cli alias resource {:account id})
                                      (assoc opts :account id)))))]
    (ordered [["organization" (keep! node org "organization" organization opts)]
              ["accounts" (keep! node org "accounts" accounts opts)]
              ["credit" (keep! node org "credit" credit opts)]
              ["transactions" (keep! node org "transactions" transactions (assoc opts :start start))]
              ["statements" (statements "statements" "accounts" accounts)]
              ["credit-statements" (statements "credit-statements" "credit" credit)]])))

(defn read-orgs
  "The Mercury orgs a local config file names, by their mercury-cli alias: {:mercury {:orgs [{:alias \"...\" :name
  \"...\"}] :window-days 60}}. Returns [] when it names none. Says how to make the file when it is missing."
  [path]
  (let [f (io/file path)]
    (when-not (.isFile f)
      (throw (ex-info (str "No config at " path ". Make it with the orgs to read: "
                           "{:mercury {:orgs [{:alias \"<mercury-cli alias>\" :name \"<a name>\"}]}} "
                           "(mercury-cli orgs lists them).")
                      {:type :ownpurse/no-local-config :path (str path)})))
    (let [{:keys [orgs window-days]} (:mercury (edn/read-string (slurp f)))]
      (when-not (every? (comp string? :alias) orgs)
        (throw (ex-info (str path " names a Mercury org without an alias. Write {:mercury {:orgs [{:alias \"<mercury-cli alias>\" :name \"<a name>\"}]}}.")
                        {:type :ownpurse/no-local-config :path (str path)})))
      ;; An org's own :window-days wins over the one for every org.
      (mapv #(merge (when window-days {:window-days window-days}) %) orgs))))

(defn pull-orgs!
  "Pulls each of `orgs` ({:alias :name :window-days}), one after another. Returns {name-or-alias {resource {:items
  :written}}}."
  [node {:keys [cli orgs now]}]
  (ordered (for [{:keys [alias name window-days]} orgs]
             [(or name alias) (pull! node {:cli cli :alias alias :window-days window-days :now now})])))
