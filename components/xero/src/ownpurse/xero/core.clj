(ns ownpurse.xero.core
  "From xero-axi's rows to ownpurse's facts. Pure."
  (:require
   [clojure.string :as str]
   [clojure.walk :as walk]
   [ownpurse.money.interface :as money])
  (:import
   (java.time Instant LocalDate)))

;; Ids are derived from Xero's own, so a second load finds the same facts.

(defn- book-id [tenant] (str "xero:" tenant))
(defn- source-id [tenant] (str "xero-axi:" tenant))
(defn- account-id [tenant id] (str "xero:" tenant ":account:" id))
(defn- document-id [tenant entity id] (str "xero:" tenant ":" entity ":" id))

(defn- party-id
  "By the contact's id when xero-axi has the contact, otherwise by its name (Xero keeps names unique in an org)."
  [tenant {:keys [id name]}]
  (str "xero:" tenant ":contact:" (if id id (str "name:" name))))

(def ^:private classes
  {"ASSET" :asset "LIABILITY" :liability "EQUITY" :equity "REVENUE" :income "EXPENSE" :expense})

(defn- kebab
  "BankAccount → :bank-account, AccountID → :account-id, UpdatedDateUTC → :updated-date-utc."
  [k]
  (-> (name k)
      (str/replace #"([a-z0-9])([A-Z])" "$1-$2")
      (str/replace #"([A-Z]+)([A-Z][a-z])" "$1-$2")
      str/lower-case
      keyword))

(defn normalize-payload
  "Xero's payload as XTDB keeps it: kebab-case keys (XTDB folds the case of nested keys) and no nil values (XTDB
  drops them). xero-axi's content hash still identifies the exact original."
  [payload]
  (walk/postwalk (fn [x]
                   (if (map? x)
                     (into {} (keep (fn [[k v]] (when (some? v) [(kebab k) v]))) x)
                     x))
                 payload))

(defn- blank->nil [s]
  (when-not (str/blank? s) s))

(defn- compact
  "Drops nil values, which XTDB would drop anyway."
  [m]
  (into {} (remove (comp nil? val)) m))

(defn- source [record chain-head {:keys [tenant_id]}]
  (compact {:source/id (source-id tenant_id)
            :source/tool "xero-axi"
            :source/org tenant_id
            :source/record record
            :source/chain-head chain-head}))

(defn- book [{:keys [tenant_id payload]}]
  (let [{:keys [Name BaseCurrency FinancialYearEndMonth FinancialYearEndDay]} payload]
    (compact {:book/id (book-id tenant_id)
              :book/name Name
              :book/currency BaseCurrency
              :book/source (source-id tenant_id)
              :book/fiscal-year-end (when (and FinancialYearEndMonth FinancialYearEndDay)
                                      {:month FinancialYearEndMonth :day FinancialYearEndDay})})))

(defn- account [{:keys [tenant_id xero_id payload]}]
  (let [{:keys [Code Name Class Status Type]} payload]
    (compact {:account/id (account-id tenant_id xero_id)
              :account/book (book-id tenant_id)
              :account/code (blank->nil Code)
              :account/name Name
              :account/class (or (classes Class)
                                 (throw (ex-info (str "Account " Name " has the class " (pr-str Class)
                                                      ", which ownpurse does not know.")
                                                 {:type :ownpurse/unknown-account-class :account xero_id
                                                  :class Class})))
              :account/status Status
              :account/source-type Type})))

(defn- document [{:keys [tenant_id entity xero_id version content_hash status observed_at payload date]}]
  (compact {:document/id (document-id tenant_id entity xero_id)
            :document/book (book-id tenant_id)
            :document/kind entity
            :document/source-id xero_id
            :document/version version
            :document/content-hash content_hash
            :document/status status
            :document/date (some-> date LocalDate/parse)
            :document/observed-at (Instant/parse observed_at)
            :document/payload (normalize-payload payload)}))

(defn- contacts->parties
  "Contacts xero-axi has, plus every other name the postings mention: [{:tenant :id :name}]."
  [contacts gl]
  (let [known (for [{:keys [tenant_id xero_id payload]} contacts]
                {:tenant tenant_id :id xero_id :name (:Name payload)})
        known-names (set (map (juxt :tenant :name) known))
        mentioned (for [{:keys [tenant_id contact]} gl
                        :when (and (not (str/blank? contact)) (not (known-names [tenant_id contact])))]
                    {:tenant tenant_id :name contact})]
    (concat known (distinct mentioned))))

(defn- party [{:keys [tenant id name] :as p}]
  (compact {:party/id (party-id tenant p)
            :party/book (book-id tenant)
            :party/name name
            :party/source-id id}))

(defn- entry
  "One entry per source document, from its postings in the gl."
  [currencies party-ids document-ids [[tenant source source-id] postings]]
  (let [{:keys [date kind reference]} (first postings)
        id (document-id tenant source source-id)]
    (compact {:entry/id id
              :entry/book (book-id tenant)
              :entry/date (LocalDate/parse date)
              :entry/kind kind
              :entry/narration (blank->nil reference)
              :entry/document (document-ids id)
              :entry/postings (->> postings
                                   (sort-by :line)
                                   (mapv (fn [{:keys [line account_id amount contact description]}]
                                           (compact {:posting/line line
                                                     :posting/account (account-id tenant account_id)
                                                     :posting/amount (money/amount amount)
                                                     :posting/commodity (currencies tenant)
                                                     :posting/party (party-ids [tenant contact])
                                                     :posting/description (blank->nil description)}))))})))

(defn facts [{:keys [record chain-head orgs accounts contacts documents gl]}]
  (let [books (mapv book orgs)
        currencies (into {} (map (juxt :tenant_id (comp :BaseCurrency :payload))) orgs)
        found (contacts->parties contacts gl)
        party-ids (into {} (map (fn [{:keys [tenant name] :as p}] [[tenant name] (party-id tenant p)])) found)
        documents (mapv document documents)
        document-ids (set (map :document/id documents))]
    {:sources (mapv (partial source record chain-head) orgs)
     :books books
     :accounts (mapv account accounts)
     :parties (mapv party found)
     :documents documents
     :entries (->> gl
                   (group-by (juxt :tenant_id :source :source_id))
                   (sort-by key)
                   (mapv (partial entry currencies party-ids document-ids)))}))

(defn metadata [{:keys [record chain-head]}]
  (compact {:source "xero-axi" :why "load" :record record :chain-head chain-head}))
