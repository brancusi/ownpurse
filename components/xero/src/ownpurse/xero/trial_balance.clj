(ns ownpurse.xero.trial-balance
  "Whether an org's book equals Xero's trial balance at a date, to the cent, account by account. ownpurse's balances
  are its postings: for a balance sheet account, its balance at the start of the financial year (from Xero's balance
  sheet at the day before, or from the book's own postings) and the year's postings; for an income or expense
  account, the year's postings alone. Xero's are its TrialBalance report's year-to-date columns, debit less credit."
  (:require
   [clojure.string :as str]
   [ownpurse.ledger.interface :as ledger]
   [ownpurse.store.interface :as store]
   [ownpurse.xero.ledger :as xero-ledger]
   [ownpurse.xero.source :as source])
  (:import
   (java.time LocalDate MonthDay)))

(defn year-start
  "The first day of the financial year that holds `date`, for a year that ends on `month`/`day`."
  [^LocalDate date month day]
  (let [end-this-year (.atYear (MonthDay/of (int month) (int day)) (.getYear date))
        end (if (.isAfter date end-this-year) (.plusYears end-this-year 1) end-this-year)]
    (.plusDays (.minusYears end 1) 1)))

(defn- figure
  "A report cell's value as a decimal: an empty cell is zero."
  [s]
  (if (str/blank? s) 0M (bigdec (str/replace s "," ""))))

(def ^:private no-account
  "The id Xero's reports give a computed row (Current Year Earnings) that is no account in the chart."
  "abababab-abab-abab-abab-abababababab")

(defn- account-of
  "The AccountID a report row names, or nil for a row that names none (or only Xero's placeholder)."
  [row]
  (some #(when (and (= "account" (:id %)) (not= no-account (:value %))) (:value %)) (mapcat :attributes (:cells row))))

(defn- report-rows
  "Every row of a report (rows within sections), each with the title of its section."
  [report]
  (for [section (:rows report)
        row (if (= "Section" (:row-type section)) (:rows section) [section])
        :when (= "Row" (:row-type row))]
    (assoc row :section (:title section))))

(defn trial-balance-figures
  "Xero's TrialBalance report (a row of xero.reports) as {:accounts {AccountID amount} :other [{:name :amount}]}:
  each row's year-to-date debit less its year-to-date credit; rows that name no account (Retained Earnings) under
  :other."
  [report]
  (let [header (some #(when (= "Header" (:row-type %)) (mapv :value (:cells %))) (:rows report))
        column #(.indexOf ^java.util.List header %)
        [debit credit] [(column "YTD Debit") (column "YTD Credit")]]
    (when (or (neg? debit) (neg? credit))
      (throw (ex-info "Xero's trial balance has no year-to-date columns." {:type :ownpurse/invalid :header header})))
    (reduce (fn [acc row]
              (let [cells (mapv :value (:cells row))
                    amount (- (figure (get cells debit)) (figure (get cells credit)))]
                (if-let [id (account-of row)]
                  (update-in acc [:accounts id] (fnil + 0M) amount)
                  (update acc :other conj {:name (first cells) :amount amount}))))
            {:accounts {} :other []}
            (report-rows report))))

(defn balance-sheet-figures
  "Xero's BalanceSheet report as {:accounts {AccountID amount} :other [{:name :amount}]}, debits positive: an asset's
  figure as it is, a liability's or equity's negated (`credit?` says which, by AccountID). Rows that name no account
  (Current Year Earnings) are under :other, as credits when in a section that is not Assets."
  [report credit?]
  (reduce (fn [acc row]
            (let [[name value] (mapv :value (:cells row))
                  id (account-of row)
                  credit (if id (credit? id) (not (re-find #"(?i)asset" (str (:section row)))))
                  amount (cond-> (figure value) credit -)]
              (if id
                (update-in acc [:accounts id] (fnil + 0M) amount)
                (update acc :other conj {:name name :amount amount}))))
          {:accounts {} :other []}
          (report-rows report)))

(defn- report [node tenant report date]
  (when (store/table-exists? node :xero/reports)
    (first (store/q node ["SELECT * FROM xero.reports WHERE _id = ?" (source/report-id tenant report {"date" (str date)})]))))

(defn- rows-of [node tenant resource]
  (let [{:keys [table]} (source/describe resource)]
    (if (store/table-exists? node table)
      (store/q node [(str "SELECT * FROM " (namespace table) "." (str/replace (name table) "-" "_") " WHERE org = ?")
                     tenant])
      [])))

(defn check
  "ownpurse's book of org `tenant` against Xero's trial balance at `date` (which the store must hold: a pull keeps
  one at the end of each month). `opening`: :balance-sheet (the default) takes the balance sheet accounts at the
  start of the year from Xero's balance sheet at the day before, which the store must hold, with the earnings
  rows that name no account added to the retained earnings account (so too the trial balance's); :ledger takes
  them from the book's postings before the year. `adjustments`, when given, is a function of {:from :to} that
  returns {account amount} (an account by its row id, AccountID or code): each account then has :adjustments,
  those dated up to `date` for a balance sheet account and within the year for an income or expense account, and
  :pro-forma, Xero's balance with them. Returns {:date :year-start :opening :tied? :accounts [{:account :code :name
  :xero :ownpurse :difference :adjustments :pro-forma}] :other [...]}, every account any column has a balance in,
  by code."
  [node tenant ^LocalDate date {:keys [opening adjustments] :or {opening :balance-sheet}}]
  (let [[org] (rows-of node tenant "Organisation")
        _ (when-not org
            (throw (ex-info (str "ownpurse holds no Xero organisation for " tenant ". Pull it first.")
                            {:type :ownpurse/not-found :tenant tenant})))
        start (year-start date (:financial-year-end-month org) (:financial-year-end-day org))
        before (.minusDays start 1)
        accounts (into {} (map (juxt :account-id identity)) (rows-of node tenant "Accounts"))
        profit-or-loss? #(#{"REVENUE" "EXPENSE"} (:class (accounts %)))
        ;; A credit card is a bank account of class ASSET, but the balance sheet shows it as a liability.
        credit? #(let [a (accounts %)]
                   (or (#{"LIABILITY" "EQUITY" "REVENUE"} (:class a)) (= "CREDITCARD" (:bank-account-type a))))
        tb (or (report node tenant "TrialBalance" date)
               (throw (ex-info (str "ownpurse holds no Xero trial balance at " date ". A pull keeps one at the end of "
                                    "each month it covers ({:xero {:reports {:months n}}}); choose a month end.")
                               {:type :ownpurse/not-found :report "TrialBalance" :date (str date)})))
        retained (some (fn [[id a]] (when (= "RETAINEDEARNINGS" (:system-account a)) id)) accounts)
        xero (cond-> (trial-balance-figures tb)
               ;; Xero's trial balance may show retained earnings on a row that names no account.
               retained (update-in [:accounts retained] (fnil + 0M) (reduce + 0M (map :amount (:other (trial-balance-figures tb))))))
        book (xero-ledger/book-id tenant)
        row-id #(source/row-id tenant "account" %)
        xero-id #(when (str/starts-with? % (row-id "")) (subs % (count (row-id ""))))
        by-xero-id (fn [balances] (into {} (keep (fn [[k v]] (when-let [id (xero-id k)] [id v]))) balances))
        year (by-xero-id (ledger/balances node book {:from start :to date}))
        opening-figures
        (case opening
          :ledger {:accounts (by-xero-id (ledger/balances node book {:to before})) :other []}
          :balance-sheet
          (let [bs (or (report node tenant "BalanceSheet" before)
                       (throw (ex-info (str "ownpurse holds no Xero balance sheet at " before ", the day before the "
                                            "financial year. Pull with {:xero {:reports {:months n}}} covering it, or "
                                            "check with the opening balances from the book's own postings (opening=ledger).")
                                       {:type :ownpurse/not-found :report "BalanceSheet" :date (str before)})))
                figures (balance-sheet-figures bs credit?)]
            (cond-> figures
              retained (update-in [:accounts retained] (fnil + 0M) (reduce + 0M (map :amount (:other figures)))))))
        opening-balances (into {} (remove (comp profit-or-loss? key)) (:accounts opening-figures))
        ours (merge-with + opening-balances year)
        ;; Adjustments name an account by its row id, AccountID or code; one the chart lacks keeps its own key.
        chart (xero-ledger/chart tenant (vals accounts))
        resolve #(or (xero-id %) (let [r (xero-ledger/resolve-account chart % %)] (or (xero-id r) r)))
        adjusted (when adjustments
                   (let [by-id #(reduce (fn [acc [k v]] (update acc (resolve k) (fnil + 0M) v)) {} %)
                         to-date (by-id (adjustments {:to date}))
                         this-year (by-id (adjustments {:from start :to date}))]
                     (into {} (for [id (into (set (keys to-date)) (keys this-year))]
                                [id (get (if (profit-or-loss? id) this-year to-date) id 0M)]))))
        ids (sort-by #(vector (or (:code (accounts %)) "~") %)
                     (-> (set (keys (:accounts xero))) (into (keys ours)) (into (keys adjusted))))
        rows (for [id ids
                   :let [x (get-in xero [:accounts id] 0M)
                         o (get ours id 0M)
                         a (get adjusted id 0M)]
                   :when (not (and (zero? x) (zero? o) (zero? a)))]
               (cond-> {:account (if (accounts id) (row-id id) id)
                        :name (or (:name (accounts id)) id)
                        :xero x
                        :ownpurse o
                        :difference (- o x)}
                 (:code (accounts id)) (assoc :code (:code (accounts id)))
                 adjustments (assoc :adjustments a :pro-forma (+ x a))))]
    {:date date
     :year-start start
     :opening (name opening)
     :tied? (every? (comp zero? :difference) rows)
     :accounts (vec rows)
     :other (:other xero)}))
