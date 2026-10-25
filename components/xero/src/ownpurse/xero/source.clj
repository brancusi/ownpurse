(ns ownpurse.xero.source
  "Xero's answers as source rows: one row per item, in a table of the xero schema, kept as Xero sent it. Pure."
  (:require
   [clojure.string :as str]
   [clojure.walk :as walk])
  (:import
   (java.time Instant LocalDate ZoneOffset)))

(def resources
  "The resources ownpurse keeps, by Xero's name: the key of the list in Xero's answer, the field that holds an
  item's id, the table its rows go to and the kind named in each row's id."
  (into {}
        (map (fn [[resource id-field table kind list-key]]
               [resource {:list-key (or list-key (keyword resource))
                          :id-field id-field
                          :table (keyword "xero" table)
                          :kind kind}]))
        [["Organisation" :OrganisationID "organisations" "organisation" :Organisations]
         ["Accounts" :AccountID "accounts" "account"]
         ["Contacts" :ContactID "contacts" "contact"]
         ["TaxRates" :TaxType "tax-rates" "tax-rate"]
         ["TrackingCategories" :TrackingCategoryID "tracking-categories" "tracking-category"]
         ["Currencies" :Code "currencies" "currency"]
         ["Invoices" :InvoiceID "invoices" "invoice"]
         ["CreditNotes" :CreditNoteID "credit-notes" "credit-note"]
         ["Payments" :PaymentID "payments" "payment"]
         ["Overpayments" :OverpaymentID "overpayments" "overpayment"]
         ["Prepayments" :PrepaymentID "prepayments" "prepayment"]
         ["BankTransactions" :BankTransactionID "bank-transactions" "bank-transaction"]
         ["BankTransfers" :BankTransferID "bank-transfers" "bank-transfer"]
         ["ManualJournals" :ManualJournalID "manual-journals" "manual-journal"]]))

(def pull-order
  "What a pull asks Xero for, in this order: the org and its lists first, then the documents."
  ["Organisation" "Accounts" "TaxRates" "TrackingCategories" "Currencies" "Contacts" "Invoices" "CreditNotes"
   "Payments" "Overpayments" "Prepayments" "BankTransactions" "BankTransfers" "ManualJournals"])

(defn- kebab
  "BankAccount → :bank-account, AccountID → :account-id, UpdatedDateUTC → :updated-date-utc."
  [k]
  (-> (name k)
      (str/replace #"([a-z0-9])([A-Z])" "$1-$2")
      (str/replace #"([A-Z]+)([A-Z][a-z])" "$1-$2")
      str/lower-case
      keyword))

(defn normalize-payload
  "Xero's item as XTDB keeps it: kebab-case keys (XTDB folds the case of nested keys) and no nil values (XTDB
  drops them)."
  [payload]
  (walk/postwalk (fn [x]
                   (if (map? x)
                     (into {} (keep (fn [[k v]] (when (some? v) [(kebab k) v]))) x)
                     x))
                 payload))

(defn describe
  "What ownpurse knows of `resource` (Xero's name), or an error naming the resources it keeps."
  [resource]
  (or (resources resource)
      (throw (ex-info (str "ownpurse does not keep the Xero resource " (pr-str resource) ". It keeps: "
                           (str/join ", " (sort (keys resources))) ".")
                      {:type :ownpurse/unknown-resource :resource resource}))))

(defn resource-of-list
  "The resource whose answers hold their items under `list-key` (Invoices, Organisations, ...), or nil."
  [list-key]
  (some (fn [[resource {k :list-key}]] (when (= k list-key) resource)) resources))

(defn row-id
  "The id of the row for item `id` of `kind` in org `tenant`: derived from Xero's own, so a second fetch finds it."
  [tenant kind id]
  (str "xero:" tenant ":" kind ":" id))

(def ^:private xero-date #"^/Date\((-?\d+)([+-]\d{4})?\)/$")

(defn- instant [s]
  (some->> (re-find xero-date s) second parse-long Instant/ofEpochMilli))

(defn- date-value
  "Xero writes dates as /Date(ms+0000)/. A field whose name ends in UTC is an instant; any other is a calendar
  date, which Xero sends as midnight UTC."
  [k v]
  (if-let [t (and (string? v) (instant v))]
    (if (str/ends-with? (name k) "UTC") t (LocalDate/ofInstant t ZoneOffset/UTC))
    v))

(defn- read-dates [x]
  (cond
    (map? x) (into {} (map (fn [[k v]] [k (read-dates (date-value k v))])) x)
    (vector? x) (mapv read-dates x)
    :else x))

(defn fetched-at
  "When Xero answered: the DateTimeUTC of its answer, or `default` when the answer has none."
  [answer default]
  (or (some-> (:DateTimeUTC answer) instant) default))

(defn- valid-from
  "When this version became true in Xero: its UpdatedDateUTC, never later than the fetch."
  [item ^Instant fetched-at]
  (when-let [t (some-> (:UpdatedDateUTC item) instant)]
    (if (.isAfter t fetched-at) fetched-at t)))

(defn- row [resource {:keys [id-field kind]} tenant fetched-at item]
  (let [id (get item id-field)
        from (valid-from item fetched-at)]
    (when-not id
      (throw (ex-info (str "An item in Xero's " resource " has no " (name id-field)
                           ", so ownpurse cannot keep it apart from the others.")
                      {:type :ownpurse/no-xero-id :resource resource :item item})))
    (cond-> (assoc (normalize-payload (read-dates item))
                   :xt/id (row-id tenant kind id)
                   :org tenant)
      from (assoc :xt/valid-from from))))

(defn rows
  "Xero's answer to `resource` for org `tenant`, as rows for its table: each item with its fields in kebab case
  (InvoiceNumber is the column invoice_number), Xero's dates as dates, no nils, its id as :xt/id, the org as
  `org`, and valid from its UpdatedDateUTC when it has one. Returns {table [rows]}."
  [tenant resource answer fetched-at]
  (let [{:keys [list-key table] :as described} (describe resource)]
    {table (mapv (partial row resource described tenant fetched-at) (get answer list-key []))}))
