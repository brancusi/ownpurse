(ns ownpurse.plaid.source
  "Plaid's answers as source rows: the item, its accounts and its transactions, each in a table of the plaid schema,
  kept as Plaid sent them; the cursor a sync ended at; and from them, source accounts and bank lines. Pure."
  (:require
   [clojure.string :as str]
   [clojure.walk :as walk])
  (:import
   (java.time Instant LocalDate OffsetDateTime ZoneOffset ZonedDateTime)))

(def tables
  "The tables ownpurse keeps Plaid's answers in."
  {:items :plaid/items
   :accounts :plaid/accounts
   :transactions :plaid/transactions
   :cursors :plaid/cursors})

(defn sql-table
  "A table's name in SQL: :plaid/transactions is plaid.transactions."
  [table]
  (str (namespace table) "." (str/replace (name table) "-" "_")))

(defn row-id
  "The id of the row for `id` of `kind` in Plaid item `item`: derived from Plaid's own, so a second fetch finds it."
  [item kind id]
  (str "plaid:" item ":" kind ":" id))

(defn cursor-id
  "The id of the row holding item `item`'s cursor."
  [item]
  (str "plaid:" item ":cursor"))

(defn- kebab
  "account_id → :account-id."
  [k]
  (keyword (str/replace (name k) "_" "-")))

(defn- money-key?
  "Fields that hold amounts. Plaid writes them without trailing zeros, and a whole one without a fraction (162):
  each is kept as a decimal with at least two places, so a column holds one type and a cent is a cent."
  [k]
  (contains? #{"amount" "available" "current" "limit"} (name k)))

(defn money
  "`v` as a decimal with at least two places: 162 is 162.00, 12.1 is 12.10, 0.125 stays 0.125."
  [v]
  (let [d (bigdec v)]
    (if (< (.scale d) 2) (.setScale d 2) d)))

(defn- time-value
  "Plaid writes dates as 2026-03-02 and times as 2026-03-02T14:05:00Z (or with an offset)."
  [v]
  (cond
    (not (string? v)) v
    (re-matches #"\d{4}-\d{2}-\d{2}" v) (LocalDate/parse v)
    (re-matches #"\d{4}-\d{2}-\d{2}T.+" v) (try (.toInstant (OffsetDateTime/parse v)) (catch Exception _ v))
    :else v))

(defn- time-key? [k]
  (boolean (re-find #"(^date|_date|datetime|_time)$" (name k))))

(defn- field [k v]
  (cond
    (and (money-key? k) (number? v)) (money v)
    (time-key? k) (time-value v)
    :else v))

(defn normalize-payload
  "Plaid's object as XTDB keeps it: amounts as decimals of at least two places, dates and times as such, kebab-case
  keys (XTDB folds the case of nested keys), no nil values (XTDB drops them)."
  [x]
  (walk/postwalk (fn [x]
                   (if (map? x)
                     (into {} (keep (fn [[k v]] (when (some? v) [(kebab k) (field k v)]))) x)
                     x))
                 x))

(defn- start-of [^LocalDate date]
  (.toInstant (.atStartOfDay date ZoneOffset/UTC)))

(defn- not-after [^Instant t ^Instant limit]
  (if (.isAfter t limit) limit t))

(defn item-row
  "Plaid's /item/get answer as the item's row. Valid from when ownpurse writes it."
  [answer]
  (let [{:keys [item_id] :as item} (:item answer)]
    (when-not item_id
      (throw (ex-info "Plaid's answer for the item names no item id." {:type :plaid/failed})))
    (assoc (normalize-payload item) :xt/id (row-id item_id "item" item_id) :item item_id)))

(defn account-rows
  "The accounts of a sync's answer as rows, with the balances Plaid last saw. Valid from when ownpurse writes them."
  [item answer]
  (vec (for [{:keys [account_id] :as a} (:accounts answer)]
         (assoc (normalize-payload a) :xt/id (row-id item "account" account_id) :item item))))

(defn transaction-rows
  "The added and modified transactions of a sync's answer as rows. An added one is valid from its date (never later
  than `fetched-at`), a modified one from `fetched-at`."
  [item answer ^Instant fetched-at]
  (let [row (fn [from {:keys [transaction_id] :as t}]
              (when-not transaction_id
                (throw (ex-info "A transaction in Plaid's answer has no id, so ownpurse cannot keep it apart."
                                {:type :plaid/failed :item item})))
              (let [r (normalize-payload t)]
                (assoc r
                       :xt/id (row-id item "transaction" transaction_id)
                       :item item
                       :xt/valid-from (from r))))]
    (into (mapv (partial row #(if-let [d (:date %)] (not-after (start-of d) fetched-at) fetched-at)) (:added answer))
          (map (partial row (constantly fetched-at)))
          (:modified answer))))

(defn removed-rows
  "The transactions a sync's answer removes, as new versions of the rows `held` (by id) with `removed` true, valid
  from `fetched-at`: the row stays, its history too. A removed transaction ownpurse never held is skipped."
  [item answer held fetched-at]
  (vec (for [{:keys [transaction_id]} (:removed answer)
             :let [r (get held (row-id item "transaction" transaction_id))]
             :when r]
         (assoc r :removed true :xt/valid-from fetched-at))))

(defn cursor-row
  "Where item `item`'s sync ended: the cursor to start the next one from, and Plaid's word on how complete the
  item's history is."
  [item answer]
  (cond-> {:xt/id (cursor-id item) :item item :cursor (:next_cursor answer)}
    (:transactions_update_status answer) (assoc :update-status (:transactions_update_status answer))))

(def ^:private account-kinds
  "Plaid's account types that are source accounts. Loans and investments are not kept as accounts yet."
  {"depository" "bank" "credit" "credit-card"})

(defn source-accounts
  "The item's deposit and credit card accounts (rows from `account-rows`), as source_accounts rows: the item is
  their org. Named by Plaid's official name (or name) and the last digits."
  [rows]
  (for [{:keys [type mask balances] :as r} rows
        :let [kind (account-kinds type)]
        :when kind]
    (cond-> {:xt/id (:xt/id r)
             :source "plaid"
             :source-table (sql-table (:accounts tables))
             :org (:item r)
             :kind kind
             :name (str (or (:official-name r) (:name r) "Account") (when mask (str " ••" mask)))
             :status "active"}
      (:iso-currency-code balances) (assoc :currency (:iso-currency-code balances)))))

(defn- date-of [d]
  (cond
    (instance? LocalDate d) d
    (instance? ZonedDateTime d) (.toLocalDate ^ZonedDateTime d)
    (string? d) (LocalDate/parse d)
    :else nil))

(defn bank-lines
  "The transactions (rows held and fetched, by id) as bank_lines rows: each one not removed, on its account, on its
  date, pending or posted. Plaid's amounts are positive for money out; a line's are positive for money in, so they
  are negated, with two places. Pure."
  [rows]
  (for [{:keys [account-id amount pending removed] :as r} rows
        :let [date (date-of (:date r))]
        :when (and (not removed) account-id date amount)]
    (into {:xt/id (str "line:" (:xt/id r))
           :account (row-id (:item r) "account" account-id)
           :source "plaid"
           :source-table (sql-table (:transactions tables))
           :source-row (:xt/id r)
           :org (:item r)
           :date date
           :amount (money (.negate (bigdec amount)))
           :status (if pending "pending" "posted")}
          (filter (comp some? val))
          {:currency (:iso-currency-code r)
           :counterparty (or (:merchant-name r) (some :name (:counterparties r)))
           :description (or (:original-description r) (:name r))
           :check-number (some->> (:check-number r) str (re-find #"[1-9]\d*"))})))
