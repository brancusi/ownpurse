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
         ["ManualJournals" :ManualJournalID "manual-journals" "manual-journal"]
         ["Items" :ItemID "items" "item"]
         ["Users" :UserID "users" "user"]
         ["LinkedTransactions" :LinkedTransactionID "linked-transactions" "linked-transaction"]
         ["BatchPayments" :BatchPaymentID "batch-payments" "batch-payment"]]))

(def pull-order
  "What a pull asks Xero for, in this order: the org and its lists first, then the documents."
  ["Organisation" "Accounts" "TaxRates" "TrackingCategories" "Currencies" "Contacts" "Invoices" "CreditNotes"
   "Payments" "Overpayments" "Prepayments" "BankTransactions" "BankTransfers" "ManualJournals" "Items" "Users"
   "LinkedTransactions" "BatchPayments"])

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

(def account-resources
  "The resources whose rows hold source accounts."
  #{"Organisation" "Accounts"})

(defn- source-account [{:keys [table]} kind row name]
  (cond-> {:xt/id (:xt/id row)
           :source "xero"
           :source-table (str (namespace table) "." (str/replace (clojure.core/name table) "-" "_"))
           :org (:org row)
           :kind kind
           :name name}
    (:code row) (assoc :code (:code row))
    (or (:currency-code row) (:base-currency row)) (assoc :currency (or (:currency-code row) (:base-currency row)))
    (:status row) (assoc :status (str/lower-case (:status row)))))

(defn source-accounts
  "The source accounts in rows of `resource` (from `rows`), as source_accounts rows: the org (kind org) and each
  account of type BANK (bank, or credit-card when Xero's BankAccountType is CREDITCARD), archived ones too. Other
  resources hold none. Pure."
  [resource rows]
  (let [described (describe resource)]
    (case resource
      "Organisation" (for [r rows]
                       (-> (source-account described "org" r (or (:name r) (:organisation-id r)))
                           (assoc :status "active")))
      "Accounts" (for [r rows :when (= "BANK" (:type r))]
                   (source-account described (if (= "CREDITCARD" (:bank-account-type r)) "credit-card" "bank")
                                   r (:name r)))
      [])))

(def line-resources
  "The resources whose rows are lines on a bank account. A bank account's lines are its bank transactions (the
  legs of a transfer among them) and the payments made from or into it; a bank transfer is not a line of its own,
  since its legs are bank transactions already."
  #{"BankTransactions" "Payments"})

(def ^:private money-in-payments
  "The payment types that bring money into the bank account; every other type takes it out."
  #{"ACCRECPAYMENT" "APCREDITPAYMENT" "APOVERPAYMENTPAYMENT" "APPREPAYMENTPAYMENT"})

(defn check-number
  "The check number in a reference that is only that: \"Check 1042\", \"Chk #1042\", \"#1042\" or \"1042\" (three
  digits or more). Leading zeros dropped. Any other reference, such as \"Loan 2207\", has none."
  [reference]
  (when (string? reference)
    (some->> (re-matches #"(?i)\s*(?:(?:check|cheque|chk|ck)\.?\s*(?:no\.?|number)?\s*)?#?\s*0*([1-9]\d{2,})\s*"
                         reference)
             second)))

(defn- line [table row account amount fields]
  (into {:xt/id (str "line:" (:xt/id row))
         :account account
         :source "xero"
         :source-table (str (namespace table) "." (str/replace (clojure.core/name table) "-" "_"))
         :source-row (:xt/id row)
         :org (:org row)
         :date (:date row)
         :amount amount
         :status "posted"}
        (filter (comp some? val))
        fields))

(defn bank-lines
  "The bank lines in rows of `resource` (from `rows`), as bank_lines rows: each authorised bank transaction and
  payment, its amount signed from the bank account's side (money in is positive). A deleted or voided one is no
  line. Other resources hold none. Pure."
  [resource rows]
  (let [{:keys [table]} (describe resource)
        authorised (filter #(= "AUTHORISED" (:status %)) rows)
        account (fn [r k] (row-id (:org r) "account" (get-in r [k :account-id])))]
    (case resource
      "BankTransactions"
      (for [r authorised
            :let [total (some-> (:total r) bigdec)]
            :when (and total (get-in r [:bank-account :account-id]))]
        (line table r (account r :bank-account)
              (if (str/starts-with? (str (:type r)) "SPEND") (.negate ^BigDecimal total) total)
              {:currency (:currency-code r)
               :counterparty (get-in r [:contact :name])
               :reference (:reference r)
               :check-number (check-number (:reference r))
               :description (some :description (:line-items r))}))

      "Payments"
      (for [r authorised
            :let [amount (some-> (or (:bank-amount r) (:amount r)) bigdec)]
            :when (and amount (get-in r [:account :account-id]))]
        (line table r (account r :account)
              (if (money-in-payments (:payment-type r)) amount (.negate ^BigDecimal amount))
              {:counterparty (or (get-in r [:invoice :contact :name]) (get-in r [:credit-note :contact :name]))
               :reference (:reference r)
               :check-number (check-number (:reference r))
               :description (get-in r [:invoice :invoice-number])}))

      [])))

(defn- month-end ^LocalDate [^LocalDate d]
  (.withDayOfMonth d (.lengthOfMonth d)))

(defn report-requests
  "The reports to ask Xero for, for the `months` months that end with the month of `today`, newest first: for each
  month, the trial balance and the balance sheet at its last day, the profit and loss for the month on an accrual
  basis and on a cash basis, and the bank summary for the month. Each is {:report name :params {param value}
  :month-end date}."
  [^LocalDate today months]
  (for [i (range months)
        :let [end (month-end (.minusMonths today i))
              start (.withDayOfMonth end 1)
              month {"fromDate" (str start) "toDate" (str end)}]
        request [{:report "TrialBalance" :params {"date" (str end)}}
                 {:report "BalanceSheet" :params {"date" (str end)}}
                 {:report "ProfitAndLoss" :params month}
                 {:report "ProfitAndLoss" :params (assoc month "paymentsOnly" "true")}
                 {:report "BankSummary" :params month}]]
    (assoc request :month-end end)))

(defn report-id
  "The row id of `report` with `params` in org `tenant`: the report and its parameters, in order, so that a later
  fetch of the same report is a new version of the same row."
  [tenant report params]
  (row-id tenant "report" (str report "?" (str/join "&" (map (fn [[k v]] (str k "=" v)) (sort params))))))

(defn report-row
  "Xero's answer to `report` with `params` for org `tenant` as a row of xero.reports: the report as Xero sent it
  (rows, cells and their values as strings), less what changes on every fetch (UpdatedDateUTC, ReportDate), with
  `report`, the dates it covers (`date`, or `from_date` and `to_date`), its `basis` for a profit and loss, and
  valid from `fetched-at`."
  [tenant report params answer ^Instant fetched-at]
  (let [[content] (:Reports answer)
        day #(some-> (get params %) LocalDate/parse)]
    (when-not content
      (throw (ex-info (str "Xero's answer to the report " report " holds no report.")
                      {:type :ownpurse/no-xero-report :report report :params params})))
    (cond-> (assoc (normalize-payload (read-dates (dissoc content :UpdatedDateUTC :ReportDate)))
                   :xt/id (report-id tenant report params)
                   :org tenant
                   :report report
                   :xt/valid-from fetched-at)
      (day "date") (assoc :date (day "date"))
      (day "fromDate") (assoc :from-date (day "fromDate") :to-date (day "toDate"))
      (= "ProfitAndLoss" report) (assoc :basis (if (= "true" (get params "paymentsOnly")) "cash" "accrual")))))

