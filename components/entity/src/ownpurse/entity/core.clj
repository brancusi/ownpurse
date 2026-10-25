(ns ownpurse.entity.core
  (:require
   [clojure.string :as str]
   [ownpurse.schema.interface :as schema]
   [ownpurse.store.interface :as store])
  (:import
   (java.time Instant LocalDate ZoneOffset ZonedDateTime)))

(def ^:private metadata {:source "ownpurse" :why "entities"})

(defn- entity-id [slug] (str "entity:" slug))

(defn- slug [id] (str/replace-first id #"^entity:" ""))

(defn- link-id [account] (str "link:" account))

(defn- start-of [^LocalDate date]
  (.toInstant (.atStartOfDay date ZoneOffset/UTC)))

(defn- day-of [t]
  (when t (LocalDate/ofInstant (if (instance? ZonedDateTime t) (.toInstant ^ZonedDateTime t) t) ZoneOffset/UTC)))

(defn- fail! [type message data]
  (throw (ex-info message (assoc data :type type))))

(defn- rows
  "The rows `sql` returns, or none when `table` has never been written: XTDB refuses a query on a table it has not
  seen."
  [node table sql]
  (if (store/table-exists? node table) (store/q node sql) []))

(defn- one [node table sql]
  (first (rows node table sql)))

(defn- entity-out [{:keys [xt/id name kind]}]
  {:id (slug id) :name name :kind kind})

(defn- account-out [account]
  (-> account
      (dissoc :xt/id)
      (assoc :id (:xt/id account))))

(defn- link-out [{:keys [entity account role evidence-for xt/valid-from]}]
  (cond-> {:entity (slug entity) :account account :role role}
    evidence-for (assoc :evidence-for evidence-for)
    valid-from (assoc :from (str (day-of valid-from)))))

(defn source-account [node id]
  (one node :source-accounts ["SELECT * FROM source_accounts WHERE _id = ?" id]))

(defn- entity-row [node slug]
  (one node :entities ["SELECT * FROM entities WHERE _id = ?" (entity-id slug)]))

(defn- link-of [node account]
  (one node :entity-links ["SELECT *, _valid_from FROM entity_links WHERE _id = ?" (link-id account)]))

(defn- links-of [node entity]
  (rows node :entity-links ["SELECT *, _valid_from FROM entity_links WHERE entity = ? ORDER BY account" entity]))

(defn put-entity! [node e]
  (let [{:keys [id name kind]} (schema/check! :ownpurse/entity e)]
    (store/put-changed! node {:entities [{:xt/id (entity-id id) :name name :kind kind}]} metadata)
    {:id id :name name :kind kind}))

(defn entities [node]
  (let [links (group-by :entity (rows node :entity-links "SELECT *, _valid_from FROM entity_links ORDER BY account"))]
    (vec (for [e (rows node :entities "SELECT * FROM entities ORDER BY _id")]
           (assoc (entity-out e) :links (mapv link-out (links (:xt/id e))))))))

(def ^:private kind-order {"org" 0 "bank" 1 "credit-card" 2})

(defn source-accounts [node {:keys [source unlinked]}]
  (let [links (into {} (map (juxt :account identity))
                    (rows node :entity-links "SELECT *, _valid_from FROM entity_links"))]
    (vec (for [a (sort-by (juxt :source :org (comp kind-order :kind) :name)
                          (rows node :source-accounts "SELECT * FROM source_accounts"))
               :when (or (nil? source) (= source (:source a)))
               :let [link (links (:xt/id a))]
               :when (not (and unlinked link))]
           (cond-> (account-out a)
             link (assoc :link (dissoc (link-out link) :account)))))))

(defn- system-of-record
  "The entity's system-of-record link and the org it names, or nil."
  [node entity]
  (when-let [link (some #(when (= "system-of-record" (:role %)) %) (links-of node entity))]
    {:link link :org (source-account node (:account link))}))

(defn- check-link! [node {:keys [entity account role evidence-for]}]
  (when-not (entity-row node entity)
    (fail! :ownpurse/not-found (str "There is no entity " (pr-str entity) ". Make it first: PUT /entities/" entity ".")
           {:entity entity}))
  (let [acct (or (source-account node account)
                 (fail! :ownpurse/not-found (str "There is no source account " (pr-str account) ". GET /source-accounts "
                                                 "lists them; a source's accounts appear after its first pull.")
                        {:account account}))
        held (link-of node account)]
    (when (and held (not= (entity-id entity) (:entity held)))
      (fail! :ownpurse/link-conflict (str account " belongs to " (slug (:entity held)) ". End that link first: "
                                          "DELETE /links?account=" account ".")
             {:account account :entity (slug (:entity held))}))
    (case role
      "system-of-record"
      (do
        (when-not (= "org" (:kind acct))
          (fail! :ownpurse/invalid-link (str "A system of record is an organisation, and " account " is a "
                                             (:kind acct) " account. Link it as evidence instead.")
                 {:account account :kind (:kind acct)}))
        (when evidence-for
          (fail! :ownpurse/invalid-link "Only an evidence link names the account it is evidence for."
                 {:account account}))
        (when-let [{other :link} (system-of-record node (entity-id entity))]
          (when (not= account (:account other))
            (fail! :ownpurse/link-conflict (str entity " already has a system of record, " (:account other)
                                                ". End that link first: DELETE /links?account=" (:account other) ".")
                   {:entity entity :account (:account other)}))))

      "evidence"
      (do
        (when (= "org" (:kind acct))
          (fail! :ownpurse/invalid-link (str account " is an organisation. Link it as the system of record, or link "
                                             "its accounts as evidence.")
                 {:account account}))
        (when evidence-for
          (let [target (or (source-account node evidence-for)
                           (fail! :ownpurse/not-found (str "There is no source account " (pr-str evidence-for) ".")
                                  {:account evidence-for}))
                {:keys [org]} (or (system-of-record node (entity-id entity))
                                  (fail! :ownpurse/invalid-link
                                         (str entity " has no system of record, so " account " cannot be evidence for "
                                              "one of its accounts. Link its system of record first.")
                                         {:entity entity}))]
            (when-not (and (= (:source org) (:source target)) (= (:org org) (:org target))
                           (not= "org" (:kind target)))
              (fail! :ownpurse/invalid-link (str evidence-for " is not an account in " entity "'s system of record, "
                                                 (:xt/id org) ".")
                     {:account evidence-for :system-of-record (:xt/id org)}))))))))

(defn link! [node l]
  (let [{:keys [entity account role evidence-for from] :as l} (schema/check! :ownpurse/link l)]
    (check-link! node l)
    (store/put-changed! node {:entity-links [(cond-> {:xt/id (link-id account) :entity (entity-id entity)
                                                      :account account :role role}
                                               evidence-for (assoc :evidence-for evidence-for)
                                               from (assoc :xt/valid-from (start-of from)))]}
                        metadata)
    (link-out (link-of node account))))

(defn unlink! [node {:keys [account on]}]
  (let [held (or (link-of node account)
                 (fail! :ownpurse/not-found (str account " is not linked to any entity.") {:account account}))
        at (if on (start-of on) (Instant/now))
        since (.toInstant ^ZonedDateTime (:xt/valid-from held))]
    (when (.isBefore ^Instant at since)
      (fail! :ownpurse/invalid-link (str "The link of " account " starts on " (day-of since)
                                         ", so it cannot end before that.")
             {:account account :from (str (day-of since))}))
    (store/delete! node :entity-links [(link-id account)] {:valid-from at} metadata)
    (assoc (link-out held) :until (str (day-of at)))))
