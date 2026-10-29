(ns ownpurse.export.core
  (:require
   [clojure.data.csv :as csv]
   [clojure.java.io :as io]
   [clojure.string :as str]
   [ownpurse.close.interface :as close]
   [ownpurse.compare.interface :as cmp]
   [ownpurse.export.xlsx :as xlsx]
   [ownpurse.intercompany.interface :as ic]
   [ownpurse.ledger.interface :as ledger]
   [ownpurse.store.interface :as store])
  (:import
   (java.time Instant)))

(defn- value
  "A cell as a table holds it: an amount stays a BigDecimal; a boolean is yes or no; a list is joined; nothing is
  empty; anything else is its text."
  [v]
  (cond
    (nil? v) ""
    (decimal? v) v
    (boolean? v) (if v "yes" "no")
    (sequential? v) (str/join " | " (map value v))
    :else (str v)))

(defn- text [v]
  (if (decimal? v) (.toPlainString ^BigDecimal v) v))

(defn- table [columns rows]
  {:columns (mapv first columns)
   :rows (mapv (fn [r] (mapv (fn [[_ f]] (value (f r))) columns)) rows)})

(defn- sum [xs] (reduce + 0M (keep identity xs)))

(defn- record-state
  "The record's last transaction, and the last time each source wrote, by source and org, from the transactions'
  metadata."
  [node]
  (let [txs (store/q node "SELECT _id, system_time, user_metadata FROM xt.txs")
        last-tx (last (sort-by :xt/id txs))
        at #(or (get-in % [:user-metadata :fetched-at]) (get-in % [:user-metadata :imported-at])
                (str (.toInstant ^java.time.ZonedDateTime (:system-time %))))]
    {:tx-id (:xt/id last-tx)
     :tx-time (some-> ^java.time.ZonedDateTime (:system-time last-tx) .toInstant str)
     :freshness (->> txs
                     (filter #(#{"fetch" "import"} (get-in % [:user-metadata :why])))
                     (group-by (juxt #(get-in % [:user-metadata :source]) #(get-in % [:user-metadata :org])))
                     (map (fn [[[source org] ts]] {:source source :org org :at (last (sort (map at ts)))}))
                     (sort-by (juxt :source :org)))}))

(defn readme
  "What the export is: whose book, which period, as of which transaction of the record, and how fresh each
  source is."
  [node {:keys [name book from to basis]}]
  (let [{:keys [tx-id tx-time freshness]} (record-state node)]
    (table [["" first] ["" second]]
           (concat [["Entity" name]
                    ["Book" book]
                    ["Period" (str from " to " to)]
                    ["As of" (str to)]
                    ["Record transaction" (str tx-id)]
                    ["Record transaction time" tx-time]
                    ["Generated" (str (Instant/now))]
                    ["Adjustments counted" (case basis
                                             "proposed" "the journals of close items not set aside (a draft)"
                                             (nil "approved") "the journals of close items approved as they stand"
                                             (str "the layers of scenario " basis " (a draft)"))]
                    ["Amounts" "Exact decimals, debits positive and credits negative"]]
                   (for [{:keys [source org at]} freshness]
                     [(str "Last " (if (= "statement-files" source) "import" "fetch") ": " source
                           (when org (str " (" org ")")))
                      at])))))

(defn trial-balance
  "The trial balance check as a table: each account's figure in the book's system, in ownpurse, their difference,
  and the adjustments and the pro-forma."
  [{:keys [accounts]}]
  (table [["code" :code] ["account" :name] ["account id" :account] ["system of record" :xero] ["ownpurse" :ownpurse]
          ["difference" :difference] ["adjustments" :adjustments] ["pro-forma" :pro-forma]]
         accounts))

(def ^:private profit-or-loss #{"REVENUE" "EXPENSE"})

(defn- statement-rows [rows classes]
  (concat
   (for [class classes
         :let [in-class (filter #(= class (:class %)) rows)]
         :when (seq in-class)
         row (concat (sort-by (juxt #(or (:code %) "~") :name) in-class)
                     [{:name (str "Total " (str/lower-case class)) :xero (sum (map :xero in-class))
                       :adjustments (sum (map :adjustments in-class)) :pro-forma (sum (map :pro-forma in-class))}])]
     row)))

(defn- with-class [chart accounts]
  (map #(assoc % :class (:class (chart (:account %)))) accounts))

(defn balance-sheet
  "The balance sheet accounts at the end of the period, the system's figures beside the pro-forma, with the
  period's earnings so that the whole sums to zero."
  [{:keys [trial-balance accounts]}]
  (let [rows (with-class accounts (:accounts trial-balance))
        pl (filter (comp profit-or-loss :class) rows)
        bs (remove (comp profit-or-loss :class) rows)
        earnings {:name "Current year earnings (from the profit and loss)" :xero (sum (map :xero pl))
                  :adjustments (sum (map :adjustments pl)) :pro-forma (sum (map :pro-forma pl))}]
    (table [["code" :code] ["account" :name] ["class" :class] ["system of record" :xero] ["adjustments" :adjustments]
            ["pro-forma" :pro-forma]]
           (concat (statement-rows bs ["ASSET" "LIABILITY" "EQUITY" nil])
                   [earnings
                    {:name "Total (zero when it balances)" :xero (sum (map :xero rows))
                     :adjustments (sum (map :adjustments rows)) :pro-forma (sum (map :pro-forma rows))}]))))

(defn profit-and-loss
  "The income and expense accounts for the period, the system's figures beside the pro-forma; the net is the
  earnings, a profit negative (a credit)."
  [{:keys [trial-balance accounts]}]
  (let [pl (filter (comp profit-or-loss :class) (with-class accounts (:accounts trial-balance)))]
    (table [["code" :code] ["account" :name] ["class" :class] ["system of record" :xero] ["adjustments" :adjustments]
            ["pro-forma" :pro-forma]]
           (concat (statement-rows pl ["REVENUE" "EXPENSE"])
                   [{:name "Net (a profit is negative)" :xero (sum (map :xero pl))
                     :adjustments (sum (map :adjustments pl)) :pro-forma (sum (map :pro-forma pl))}]))))

(defn general-ledger
  "Every posting of the period by account, by date, each account opening at its balance before the period (its
  balance at the end, as the trial balance check has it, less the period's postings) with a running balance, and
  the document each posting comes from, with a link to it."
  [node {:keys [book from to accounts parties links trial-balance]}]
  (let [closing (into {} (map (juxt :account :ownpurse)) (:accounts trial-balance))
        moved (ledger/balances node book {:from from :to to})
        ids (sort-by #(vector (or (:code (accounts %)) "~") %) (into (set (keys moved)) (keys closing)))]
    (table [["code" :code] ["account" :name] ["date" :date] ["narration" :narration] ["description" :description]
            ["party" :party-name] ["amount" :amount] ["balance" :balance] ["document" :document]
            ["link" #(some-> (:document %) links)] ["entry" :entry] ["account id" :account]]
           (for [id ids
                 :let [{:keys [code name]} (accounts id)
                       opening (- (get closing id (get moved id 0M)) (get moved id 0M))
                       head {:code code :name (or name id) :account id}
                       postings (ledger/postings node book id {:from from :to to})]
                 row (cons (assoc head :narration "Opening balance" :balance opening)
                           (map (fn [p balance]
                                  (merge head p {:party-name (some-> (:party p) parties :name) :balance balance}))
                                postings
                                (rest (reductions + opening (map :amount postings)))))]
             row))))

(defn adjustments
  "The close register's items for the book that are not set aside, one row per journal posting (an item with no
  journal, one row): what is proposed, its status, whether it is approved as it stands, its evidence, and where it
  was posted."
  [node {:keys [book accounts]}]
  (table [["item" :ref] ["title" :title] ["category" :category] ["status" :status] ["approved" :approved]
          ["action" :action] ["journal" :journal] ["date" :date] ["narration" :narration] ["code" :code]
          ["account" :account-name] ["amount" :amount] ["evidence" :evidence] ["posted as" :posted-reference]]
         (for [item (close/items node {:book book})
               :when (not (close/set-aside (:status item)))
               :let [head (assoc (select-keys item [:ref :title :category :status :action])
                                 :approved (close/approved? node item))
                     journals (filter #(= book (:book %)) (:journals item))]
               row (if (empty? journals)
                     [(assoc head :evidence (:evidence item))]
                     (for [j journals p (:postings j)]
                       (assoc head :journal (:ref j) :date (:date j) :narration (:narration j)
                              :code (:code (accounts (:account p)))
                              :account-name (or (:name (accounts (:account p))) (:account p))
                              :amount (:amount p) :evidence (concat (:evidence item) (:evidence j))
                              :posted-reference (:posted-reference j))))]
           row)))

(defn exclusions
  "The close items set aside (excluded or rejected), with why."
  [node {:keys [book]}]
  (table [["item" :ref] ["title" :title] ["status" :status] ["category" :category] ["amount" :amount]
          ["reason" :reason] ["note" :note]]
         (filter #(close/set-aside (:status %)) (close/items node {:book book}))))

(defn- in-book [accounts compares]
  (filter #(contains? accounts (:evidence-for %)) compares))

(defn- named [accounts c]
  (assoc c :book-name (or (:name (accounts (:evidence-for c))) (:evidence-for-name c) (:evidence-for c))
         :account-name (or (:account-name c) (:account c))))

(defn bank-reconciliation
  "Each evidence account linked to one of the book's accounts, compared with it over the period: what matched, and
  the count and total of what each side lacks."
  [compares]
  (let [total #(sum (map (comp bigdec :amount) %))]
    (table [["book account" :book-name] ["evidence account" :account-name] ["matched" :matched]
            ["missing from book" (comp count :missing-from-book)]
            ["missing from book, total" (comp total :missing-from-book)]
            ["missing from evidence" (comp count :missing-from-evidence)]
            ["missing from evidence, total" (comp total :missing-from-evidence)]
            ["pending" (comp count :pending)] ["evidence total" (comp bigdec :evidence :totals)]
            ["book total" (comp bigdec :book :totals)]]
           compares)))

(defn unreconciled-lines
  "The lines each compare leaves unmatched, to clear one by one."
  [compares]
  (table [["book account" :book-name] ["evidence account" :account-name] ["missing from" :side] ["date" :date]
          ["amount" (comp bigdec :amount)] ["description" :description] ["counterparty" :counterparty]
          ["source row" :source-row]]
         (for [c compares
               [side k] [["book" :missing-from-book] ["evidence" :missing-from-evidence]]
               line (get c k)]
           (merge line (select-keys c [:book-name :account-name]) {:side side}))))

(defn intercompany
  "Each intercompany pair with a side in the book, reconciled over the period."
  [node {:keys [book from to]}]
  (table [["pair" :ref] ["name" :name] ["book a" (comp :book :a)] ["balance a" (comp :balance :a)]
          ["book b" (comp :book :b)] ["balance b" (comp :balance :b)] ["difference" :difference]
          ["explained" (fn [r] (sum (map :amount (:explained r))))] ["unexplained" :unexplained]
          ["reconciled" :reconciled?] ["unmatched lines" (fn [r] (+ (count (get-in r [:unmatched :a]))
                                                                    (count (get-in r [:unmatched :b]))))]]
         (filter #(or (= book (get-in % [:a :book])) (= book (get-in % [:b :book])))
                 (ic/report node {:from from :to to}))))

(defn payees
  "Who the book paid, by party, by bank and by card, and the 1099 candidates."
  [{:keys [payees threshold]}]
  (table [["payee" :name] ["paid by bank" #(get-in % [:paid "bank"])] ["paid by card" #(get-in % [:paid "credit-card"])]
          ["payments" :payments] [(str "1099 candidate (by bank, " (text threshold) " or more)") :candidate?]
          ["tax number" :tax-number] ["party id" :party]]
         payees))

(defn- sheets
  "Every report of the export, in order, each as the function that makes it: [[name f] ...]."
  [node {:keys [from to accounts] :as opts}]
  (let [compares (delay (map (partial named accounts) (in-book accounts (cmp/compare-accounts node {:from from :to to}))))]
    (cond-> [["README" #(readme node opts)]
             ["Trial balance" #(trial-balance (:trial-balance opts))]
             ["Balance sheet" #(balance-sheet opts)]
             ["Profit and loss" #(profit-and-loss opts)]
             ["General ledger" #(general-ledger node opts)]
             ["Adjustments" #(adjustments node opts)]
             ["Exclusions" #(exclusions node opts)]
             ["Bank reconciliation" #(bank-reconciliation @compares)]
             ["Unreconciled lines" #(unreconciled-lines @compares)]
             ["Intercompany" #(intercompany node opts)]]
      (:payees opts) (conj ["Payees" #(payees (:payees opts))]))))

(defn reports
  "Every report of the export, in order: [[name table] ...]."
  [node opts]
  (mapv (fn [[name f]] [name (f)]) (sheets node opts)))

(defn report
  "One report of the export by its sheet's name, or nil when the export has none by that name."
  [node opts sheet]
  (some (fn [[name f]] (when (= sheet name) (f))) (sheets node opts)))

(defn- file-name [s]
  (str/replace (str/lower-case (str s)) #"[^a-z0-9._-]+" "-"))

(defn export! [node {:keys [dir name to] :as opts}]
  (let [base (io/file dir)]
    (when-not (.isAbsolute base)
      (throw (ex-info (str dir " is not an absolute path. Give the folder for the export in full.")
                      {:type :ownpurse/invalid :dir dir})))
    (let [label (str (file-name name) "-" to)
          folder (io/file base label)
          csv-folder (io/file folder (str label "-csv"))]
      (when (.exists folder)
        (throw (ex-info (str folder " already exists. Move it, or export to another folder.")
                        {:type :ownpurse/export-exists :dir (str folder)})))
      (.mkdirs csv-folder)
      (let [sheets (reports node opts)
            workbook (io/file folder (str label ".xlsx"))]
        (xlsx/write! workbook sheets)
        (doseq [[sheet {:keys [columns rows]}] sheets]
          (with-open [w (io/writer (io/file csv-folder (str (file-name sheet) ".csv")))]
            (csv/write-csv w (cons columns (map (partial map text) rows)))))
        {:dir (str folder)
         :workbook (str workbook)
         :sheets (vec (for [[sheet {:keys [rows]}] sheets] {:name sheet :rows (count rows)}))}))))
