(ns ownpurse.mercury.source
  "Mercury's answers as source rows: one row per item, in a table of the mercury schema, kept as Mercury sent it.
  Pure."
  (:require
   [clojure.string :as str]
   [clojure.walk :as walk])
  (:import
   (java.time Instant LocalDate ZoneOffset ZonedDateTime)))

(def resources
  "The resources ownpurse keeps, by mercury-cli's name: the key of the list in Mercury's answer (or of the one item),
  the table its rows go to and the kind named in each row's id."
  {"organization" {:one-key :organization :table :mercury/organizations :kind "organization"}
   "accounts" {:list-key :accounts :table :mercury/accounts :kind "account"}
   "credit" {:list-key :accounts :table :mercury/credit-accounts :kind "credit-account"}
   "transactions" {:list-key :transactions :table :mercury/transactions :kind "transaction"}
   "statements" {:list-key :statements :table :mercury/statements :kind "statement" :per-account? true}
   "credit-statements" {:list-key :statements :table :mercury/credit-statements :kind "credit-statement"
                        :per-account? true}})

(defn describe
  "What ownpurse knows of `resource` (mercury-cli's name), or an error naming the resources it keeps."
  [resource]
  (or (resources resource)
      (throw (ex-info (str "ownpurse does not keep the Mercury resource " (pr-str resource) ". It keeps: "
                           (str/join ", " (sort (keys resources))) ".")
                      {:type :ownpurse/unknown-resource :resource resource}))))

(defn row-id
  "The id of the row for item `id` of `kind` in Mercury org `org`: derived from Mercury's own, so a second fetch
  finds it."
  [org kind id]
  (str "mercury:" org ":" kind ":" id))

(defn- kebab
  "accountId → :account-id, currencyExchangeInfo → :currency-exchange-info."
  [k]
  (-> (name k)
      (str/replace #"([a-z0-9])([A-Z])" "$1-$2")
      (str/replace #"([A-Z]+)([A-Z][a-z])" "$1-$2")
      str/lower-case
      keyword))

(defn- money-key?
  "Fields that hold amounts or rates. Mercury writes a whole amount without a fraction (-412), which would read as
  a long; these are kept as decimals, so a column holds one type."
  [k]
  (boolean (re-find #"(?i)amount|balance|rate|fee" (name k))))

(defn- time-key?
  [k]
  (boolean (re-find #"(At|Date)$" (name k))))

(defn- time-value
  "Mercury writes instants as 2026-03-02T14:05:00Z and dates as 2026-03-02."
  [v]
  (cond
    (not (string? v)) v
    (re-matches #"\d{4}-\d{2}-\d{2}" v) (LocalDate/parse v)
    (re-matches #"\d{4}-\d{2}-\d{2}T.+" v) (try (Instant/parse v) (catch Exception _ v))
    :else v))

(defn- field [k v]
  (cond
    (and (money-key? k) (integer? v)) (bigdec v)
    (time-key? k) (time-value v)
    :else v))

(defn- without-signed-links
  "Mercury's signed download links (a statement's downloadUrl, an attachment's url) expire and change on every
  fetch. They are not kept, so an unchanged item stays unchanged; the item's id fetches a fresh one."
  [item]
  (walk/postwalk (fn [x]
                   (if (map? x)
                     (cond-> (dissoc x :downloadUrl)
                       (vector? (:attachments x)) (update :attachments (partial mapv #(if (map? %) (dissoc % :url) %))))
                     x))
                 item))

(defn normalize-payload
  "Mercury's item as XTDB keeps it: amounts as decimals, times as instants and dates, kebab-case keys (XTDB folds
  the case of nested keys), no nil values (XTDB drops them) and no signed links."
  [item]
  (walk/postwalk (fn [x]
                   (if (map? x)
                     (into {} (keep (fn [[k v]] (when (some? v) [(kebab k) (field k v)]))) x)
                     x))
                 (without-signed-links item)))

(defn- not-after [^Instant t ^Instant limit]
  (when t (if (.isAfter t limit) limit t)))

(defn- valid-from
  "When this version of a transaction became true at Mercury: posted when sent, failed when failed, created while
  pending. A cancelled, reversed or blocked one carries no time for that, so it is valid from the fetch. Never later
  than the fetch. Other resources carry no time of change: nil, so they are valid from when ownpurse wrote them."
  [resource item fetched-at]
  (when (= "transactions" resource)
    (let [{:keys [status createdAt postedAt failedAt]} item
          at (fn [s] (some-> s time-value))]
      (not-after (case status
                   "pending" (at createdAt)
                   "sent" (at (or postedAt createdAt))
                   "failed" (at (or failedAt createdAt))
                   fetched-at)
                 fetched-at))))

(defn- row [resource {:keys [kind]} org account fetched-at item]
  (let [id (:id item)
        from (valid-from resource item fetched-at)]
    (when-not id
      (throw (ex-info (str "An item in Mercury's " resource " has no id, so ownpurse cannot keep it apart from the others.")
                      {:type :ownpurse/no-mercury-id :resource resource :item item})))
    (cond-> (assoc (normalize-payload item)
                   :xt/id (row-id org kind id)
                   :org org)
      account (assoc :account account)
      from (assoc :xt/valid-from from))))

(defn items
  "The items of Mercury's answer to `resource`."
  [resource answer]
  (let [{:keys [one-key list-key]} (describe resource)]
    (if one-key
      (some-> (get answer one-key) vector)
      (get answer list-key []))))

(defn rows
  "Mercury's answer to `resource` for Mercury org `org`, as rows for its table: each item with its fields in kebab
  case (accountId is the column account_id), amounts as decimals, times as instants, no nils and no signed links,
  its id as :xt/id, the org as `org`, the account asked about as `account` (statements), and a transaction valid
  from the time its status says. Returns {table [rows]}."
  ([org resource answer fetched-at] (rows org resource answer fetched-at nil))
  ([org resource answer fetched-at account]
   (let [{:keys [table] :as described} (describe resource)]
     {table (mapv (partial row resource described org account fetched-at) (items resource answer))})))

(def account-resources
  "The resources whose rows hold source accounts."
  #{"organization" "accounts" "credit"})

(defn- source-account [{:keys [table]} kind row name]
  {:xt/id (:xt/id row)
   :source "mercury"
   :source-table (str (namespace table) "." (str/replace (clojure.core/name table) "-" "_"))
   :org (:org row)
   :kind kind
   :name name
   :status (or (:status row) "active")})

(defn- account-name
  "An account's nickname, with Mercury's own name for it (which ends in the last digits), or that name alone."
  [{:keys [nickname name]} default]
  (cond
    (and nickname name) (str nickname " (" name ")")
    :else (or nickname name default)))

(defn source-accounts
  "The source accounts in rows of `resource` (from `rows`), as source_accounts rows: the org (kind org), each
  deposit account (bank) and each credit card account (credit-card), archived ones too. Other resources hold none.
  Pure."
  [resource rows]
  (let [described (describe resource)]
    (case resource
      "organization" (for [r rows] (source-account described "org" r (or (:legal-business-name r) (:id r))))
      "accounts" (for [r rows] (source-account described "bank" r (account-name r (:id r))))
      "credit" (for [r rows] (source-account described "credit-card" r (account-name r "Credit card")))
      [])))

(def line-resources
  "The resources whose rows are lines on an account."
  #{"transactions"})

(defn- utc-date
  "The day, in UTC, of an instant as `rows` makes it or as the store hands it back."
  [t]
  (cond
    (instance? Instant t) (LocalDate/ofInstant t ZoneOffset/UTC)
    (instance? ZonedDateTime t) (LocalDate/ofInstant (.toInstant ^ZonedDateTime t) ZoneOffset/UTC)
    :else nil))

(def ^:private line-status
  "A transaction is a line while pending and once sent. A failed, cancelled, reversed or blocked one moved no money."
  {"pending" "pending" "sent" "posted"})

(def ^:private transfer-kinds
  "The kinds of Mercury transaction that move money between accounts, not pay a merchant."
  #{"externalTransfer" "internalTransfer" "outgoingPayment" "incomingDomesticWire" "incomingInternationalWire"
    "checkDeposit"})

(defn- card-facts
  "What a card line says about the purchase: the card, the day it was used (in UTC), the merchant's category code,
  and the amount in the merchant's currency (Mercury gives it in hundredths). Nil for a line that is no card's."
  [{:keys [card-id merchant kind created-at]}]
  (cond
    card-id (let [{:keys [category-code currency amount]} merchant
                  used (utc-date created-at)]
              (cond-> {:channel "card" :card card-id}
                used (assoc :purchased-on used)
                (and category-code (re-matches #"[0-9]{4}" category-code)) (assoc :merchant-code category-code)
                (and currency amount (re-matches #"[A-Z]{3}" currency))
                (assoc :original-currency currency :original-amount (.movePointLeft (bigdec amount) 2))))
    (transfer-kinds kind) {:channel "transfer"}))

(defn bank-lines
  "The lines in rows of `resource` (from `rows`), as bank_lines rows: each pending or sent transaction, on its
  deposit account or, when its account is one of `credit-ids` (Mercury's ids of the org's credit card accounts),
  on that credit card account. Dated by the day it posted, in UTC, or the day it was made while pending. A card's
  line says its card, the merchant's category code and the amount in the merchant's currency; a transfer's says so.
  Other resources hold none. Pure."
  [resource rows credit-ids]
  (if-not (line-resources resource)
    []
    (let [{:keys [table]} (describe resource)
          credit? (set credit-ids)]
      (for [{:keys [account-id status] :as r} rows
            :let [status (line-status status)
                  date (utc-date (or (:posted-at r) (:created-at r)))]
            :when (and status account-id date (:amount r))]
        (merge
         (into {:xt/id (str "line:" (:xt/id r))
               :account (row-id (:org r) (if (credit? account-id) "credit-account" "account") account-id)
               :source "mercury"
               :source-table (str (namespace table) "." (str/replace (name table) "-" "_"))
               :source-row (:xt/id r)
               :org (:org r)
               :date date
               :amount (bigdec (:amount r))
               :status status}
              (filter (comp some? val))
              {:counterparty (:counterparty-name r)
               :description (or (:bank-description r) (:note r))
               :reference (:external-memo r)
               :check-number (some->> (:check-number r) str (re-find #"[1-9]\d*"))})
         (card-facts r))))))
