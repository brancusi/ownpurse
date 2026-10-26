(ns ownpurse.xero.pull-test
  (:require
   [clojure.java.io :as io]
   [clojure.string :as str]
   [clojure.test :refer [deftest is testing]]
   [ownpurse.store.interface :as store]
   [ownpurse.xero.interface :as xero]
   [ownpurse.xero.source :as source])
  (:import
   (java.io File)
   (java.nio.file Files)
   (java.nio.file.attribute FileAttribute)
   (java.time Instant LocalDate ZonedDateTime)))

(def ^:private tenant "c0ffee00-0000-4000-8000-000000000000")
(def ^:private pull-1 "samples/xero/pull-1")
(def ^:private pull-2 "samples/xero/pull-2")
(def ^:private fake "components/xero/test/ownpurse/xero/fake-xero-cli.sh")

(def ^:private changed-in-pull-2
  "Items of pull-2 that are new or differ from pull-1, counted from the files with jq."
  {"Accounts" 1 "BankTransactions" 5 "Invoices" 3 "ManualJournals" 1 "Payments" 1})

(defn- with-node [f]
  (let [node (store/start {:mode :in-memory})]
    (try (f node) (finally (store/stop node)))))

(defn- keep-all! [node dir]
  (into (sorted-map)
        (for [[resource answer] (xero/read-answers dir)]
          [resource (xero/keep! node tenant resource answer {})])))

(defn- tx-count [node]
  (:n (first (store/q node "SELECT COUNT(*) AS n FROM xt.txs"))))

(defn- error [f]
  (try (f) nil (catch clojure.lang.ExceptionInfo e (ex-data e))))

(defn- temp-log []
  (doto (File/createTempFile "fake-xero-cli" ".log") .deleteOnExit))

(deftest rows-test
  (let [answer ((xero/read-answers pull-1) "Invoices")
        fetched-at (Instant/parse "2026-03-20T17:05:00Z")
        {rows :xero/invoices} (xero/rows tenant "Invoices" answer fetched-at)
        inv (first (filter #(= "INV-0412" (:invoice-number %)) rows))]
    (is (= 7 (count rows)))
    (testing "the row id comes from Xero's id, and the org is a column"
      (is (= (str "xero:" tenant ":invoice:c0ffee03-0000-4000-8000-000000000001") (:xt/id inv)))
      (is (= tenant (:org inv))))
    (testing "fields keep Xero's names in kebab case, money stays exact"
      (is (= 1846.13M (:total inv)))
      (is (= 2 (.scale ^BigDecimal (:total inv))))
      (is (= 25.0000M (-> inv :line-items first :quantity))))
    (testing "Xero's dates become dates, and *UTC fields instants"
      (is (= (LocalDate/parse "2026-02-16") (:date inv)))
      (is (= (Instant/parse "2026-03-02T15:20:00Z") (:updated-date-utc inv))))
    (testing "a row is valid from its UpdatedDateUTC, never later than the fetch"
      (is (= (:updated-date-utc inv) (:xt/valid-from inv)))
      (let [early (Instant/parse "2026-03-01T00:00:00Z")]
        (is (= early (->> (:xero/invoices (xero/rows tenant "Invoices" answer early))
                          (filter #(= "INV-0412" (:invoice-number %)))
                          first :xt/valid-from)))))
    (testing "an unknown resource says which ones ownpurse keeps"
      (is (= :ownpurse/unknown-resource (:type (error #(xero/rows tenant "Widgets" {} fetched-at))))))))

(deftest keep-test
  (with-node
    (fn [node]
      (let [first-load (keep-all! node pull-1)]
        (testing "pull-1 loads: every item of every resource"
          (is (= {"Accounts" 17 "BankTransactions" 6 "BankTransfers" 1 "Contacts" 10 "Currencies" 1 "Invoices" 7
                  "ManualJournals" 0 "Organisation" 1 "Payments" 3 "TaxRates" 2 "TrackingCategories" 1}
                 (update-vals first-load :items)))
          (is (= (update-vals first-load :items) (update-vals first-load :written))))
        (testing "the org and its bank accounts are source accounts"
          (is (= [["org" "Tallowmere Coffee Roasting Co." nil] ["bank" "Operating account" "1010"]
                  ["bank" "Reserve savings" "1020"]]
                 (map (juxt :kind :name :code)
                      (store/q node "SELECT kind, name, code FROM source_accounts ORDER BY kind DESC, code"))))
          (is (= (str "xero:" tenant ":account:c0ffee01-0000-4000-8000-000000000001")
                 (:xt/id (first (store/q node "SELECT _id FROM source_accounts WHERE code = '1010'"))))))
        (testing "a second load of pull-1 writes nothing"
          (let [before (tx-count node)]
            (is (every? zero? (map :written (vals (keep-all! node pull-1)))))
            (is (= before (tx-count node)))))
        (testing "pull-2 after pull-1 writes only what changed"
          (is (= changed-in-pull-2
                 (into {} (remove (comp zero? val)) (update-vals (keep-all! node pull-2) :written)))))
        (testing "an edited invoice keeps both versions, each valid from Xero's update"
          (is (= [929.63M 1084.57M]
                 (map :total (store/q node "SELECT total FROM xero.invoices FOR ALL VALID_TIME
                                            WHERE invoice_number = 'INV-0415' ORDER BY _valid_from")))))
        (testing "a voided invoice is a new version, not a deleted row"
          (is (= ["VOIDED"] (map :status (store/q node "SELECT status FROM xero.invoices
                                                         WHERE invoice_number = 'INV-0416'")))))
        (testing "each fetch's transaction names the org, the resource and when Xero answered"
          (is (some #(= {:source "xero-cli" :why "fetch" :org tenant :resource "Invoices"}
                        (select-keys % [:source :why :org :resource]))
                    (map :user-metadata (store/q node "SELECT user_metadata FROM xt.txs")))))))))

(deftest source-accounts-test
  (with-node
    (fn [node]
      (let [accounts (get (into {} (xero/read-answers pull-1)) "Accounts")
            reserve (first (filter #(= "1020" (:Code %)) (:Accounts accounts)))
            banks #(store/q node "SELECT code, status FROM source_accounts WHERE kind = 'bank' ORDER BY code")]
        (testing "a record kept before source_accounts existed: its accounts are held, none are source accounts"
          (store/put! node (xero/rows tenant "Accounts" accounts (Instant/parse "2026-03-20T17:00:00Z"))
                      {:source "test" :why "a record from before"})
          (is (not (store/table-exists? node :source-accounts))))
        (testing "a later fetch of only what changed makes every held bank account a source account, and an
                  archived account's status follows"
          (xero/keep! node tenant "Accounts" {:Accounts [(assoc reserve :Status "ARCHIVED")]} {})
          (is (= [{:code "1010" :status "active"} {:code "1020" :status "archived"}] (banks))))
        (testing "a fetch with nothing changed changes nothing"
          (xero/keep! node tenant "Accounts" {:Accounts []} {})
          (is (= [{:code "1010" :status "active"} {:code "1020" :status "archived"}] (banks))))))))

(deftest bank-lines-test
  (with-node
    (fn [node]
      (keep-all! node pull-1)
      (let [account #(str "xero:" tenant ":account:c0ffee01-0000-4000-8000-00000000000" %)
            lines #(store/q node ["SELECT amount FROM bank_lines WHERE account = ?" (account %)])
            sum #(reduce + 0M (map :amount %))]
        (testing "a bank account's lines are its bank transactions and payments, money in positive"
          (is (= 8 (count (lines 1))))
          (is (= -11715.24M (sum (lines 1))))
          (is (= [5000.0M] (map :amount (lines 2)))))
        (testing "a transfer is counted once a side: its legs are bank transactions, the transfer itself no line"
          (is (= [] (store/q node "SELECT _id FROM bank_lines WHERE source_table = 'xero.bank_transfers'"))))
        (testing "a deleted payment's line ends, and its history stays"
          (let [payments (get (into {} (xero/read-answers pull-1)) "Payments")
                deleted (assoc (last (:Payments payments)) :Status "DELETED")]
            (xero/keep! node tenant "Payments" {:Payments [deleted]} {})
            (is (= 7 (count (lines 1))))
            (is (= -9315.24M (sum (lines 1))))
            (is (= 8 (count (store/q node ["SELECT _id FROM bank_lines FOR ALL VALID_TIME WHERE account = ?"
                                           (account 1)]))))))))))

(deftest check-number-test
  (testing "a reference that is only a check number gives it; any other reference gives none"
    (is (= ["1042" "1042" "1042" "1042" "1042"]
           (map source/check-number ["Check 1042" "chk #1042" "#1042" "1042" "Cheque no. 001042"])))
    (is (every? nil? (map source/check-number ["Loan 2207" "Saltmarsh inv 57" "ACH Quillfeather" "42" nil]))))
  (testing "the payment for check 1042 carries its number to its line"
    (with-node
      (fn [node]
        (keep-all! node pull-1)
        (is (= [{:check-number "1042" :amount -3215.47M}]
               (store/q node "SELECT check_number, amount FROM bank_lines WHERE check_number IS NOT NULL")))))))

(deftest pull-test
  (with-node
    (fn [node]
      (let [log (temp-log)
            opts {:cli ["bash" fake pull-1 "0" (str log)] :tenant tenant :resources ["Organisation" "Invoices"]
                  :reports {:months 0} :history {:calls 0}}]
        (testing "pull! runs xero-cli for each resource and keeps what it prints, then posts the org's entries"
          (is (= {"Organisation" {:items 1 :written 1} "Invoices" {:items 7 :written 7}
                  "Ledger" {:items 7 :written 7 :ended 0}}
                 (xero/pull! node opts))))
        (testing "the next pull asks only for what changed since the latest update held"
          (is (= {"Organisation" {:items 1 :written 0} "Invoices" {:items 7 :written 0}
                  "Ledger" {:items 7 :written 0 :ended 0}}
                 (xero/pull! node opts)))
          (let [calls (str/split-lines (slurp log))]
            (is (= ["--help" (str "get Organisation --org " tenant " --all") (str "get Invoices --org " tenant " --all")]
                   (take 3 calls)))
            (is (= (str "get Invoices --org " tenant " --all --modified-since 2026-03-19T13:45:00Z") (last calls)))))
        (testing "the tool's version is in the metadata"
          (is (some #(= "0.0.0-fake" (:tool-version %))
                    (map :user-metadata (store/q node "SELECT user_metadata FROM xt.txs")))))))))

(deftest normalize-payload-test
  (is (= {:bank-account {:account-id "a"} :line-items [{:line-amount 1.10M}] :updated-date-utc "x" :url "u"}
         (source/normalize-payload {:BankAccount {:AccountID "a"} :LineItems [{:LineAmount 1.10M :Tracking nil}]
                                    :UpdatedDateUTC "x" :URL "u" :SystemAccount nil}))))

(deftest pull-orgs-test
  (let [config (doto (File/createTempFile "orgs" ".edn") .deleteOnExit)]
    (testing "a missing config says how to write it; one with no Xero orgs names none"
      (is (= :ownpurse/no-local-config (:type (error #(xero/read-orgs "/nowhere/config.local.edn")))))
      (spit config "{:mercury {:orgs []}}")
      (is (= [] (xero/read-orgs (str config))))
      (spit config "{:xero {:orgs [{:name \"no tenant\"}]}}")
      (is (= :ownpurse/no-local-config (:type (error #(xero/read-orgs (str config)))))))
    (testing "every org in the config is pulled, every resource in order"
      (spit config (pr-str {:xero {:orgs [{:tenant tenant :name "Tallowmere"}]}}))
      (with-node
        (fn [node]
          (let [log (temp-log)
                result (xero/pull-orgs! node {:cli ["bash" fake pull-1 "0" (str log)]
                                              :orgs (xero/read-orgs (str config))
                                              :reports {:months 1}
                                              :history {:calls 5}
                                              :today (LocalDate/parse "2026-03-20")})]
            (is (= ["Tallowmere"] (keys result)))
            (is (= (conj xero/pull-order "Reports" "History" "Ledger") (keys (result "Tallowmere"))))))))
    (testing "the config may say how many months of reports to keep"
      (is (= xero/default-reports (xero/read-reports "/nowhere/config.local.edn")))
      (spit config (pr-str {:xero {:orgs [] :reports {:months 24}}}))
      (is (= {:months 24 :refresh 2} (xero/read-reports (str config)))))
    (testing "and how many calls to spend on documents' history and attachments"
      (is (= xero/default-history (xero/read-history (str config))))
      (spit config (pr-str {:xero {:orgs [] :history {:calls 20}}}))
      (is (= (assoc xero/default-history :calls 20) (xero/read-history (str config)))))))

(deftest report-requests-test
  (let [requests (xero/report-requests (LocalDate/parse "2026-03-20") 2)]
    (is (= 10 (count requests)))
    (is (= [["TrialBalance" {"date" "2026-03-31"}]
            ["BalanceSheet" {"date" "2026-03-31"}]
            ["ProfitAndLoss" {"fromDate" "2026-03-01" "toDate" "2026-03-31"}]
            ["ProfitAndLoss" {"fromDate" "2026-03-01" "toDate" "2026-03-31" "paymentsOnly" "true"}]
            ["BankSummary" {"fromDate" "2026-03-01" "toDate" "2026-03-31"}]]
           (map (juxt :report :params) (take 5 requests))))
    (is (= [(LocalDate/parse "2026-02-28")] (distinct (map :month-end (drop 5 requests)))))))

(deftest reports-test
  (with-node
    (fn [node]
      (let [log (temp-log)
            opts {:cli ["bash" fake pull-1 "0" (str log)] :tenant tenant :resources []
                  :reports {:months 3 :refresh 1} :history {:calls 0} :today (LocalDate/parse "2026-03-20")}
            report-calls #(filter (fn [call] (str/starts-with? call "report ")) (str/split-lines (slurp log)))]
        (testing "a first pull fetches every report of every month, each a row of xero.reports"
          (is (= {"Reports" {:items 15 :written 15}} (xero/pull! node opts)))
          (is (= 15 (count (report-calls))))
          (is (some #{(str "report ProfitAndLoss --org " tenant " --param fromDate=2026-01-01 --param paymentsOnly=true "
                           "--param toDate=2026-01-31")}
                    (report-calls)))
          (is (= [["ProfitAndLoss" "accrual"] ["ProfitAndLoss" "cash"]]
                 (map (juxt :report :basis)
                      (store/q node ["SELECT report, basis FROM xero.reports WHERE report = 'ProfitAndLoss' AND from_date = ? ORDER BY basis"
                                     (LocalDate/parse "2026-02-01")])))))
        (testing "a trial balance keeps Xero's rows and cells, with the date it is at"
          (let [[tb] (store/q node ["SELECT * FROM xero.reports WHERE _id = ?"
                                    (str "xero:" tenant ":report:TrialBalance?date=2026-03-31")])
                cells (for [section (:rows tb) row (:rows section) :when (= "Row" (:row-type row))]
                        (mapv :value (:cells row)))]
            (is (= {:report "TrialBalance" :date (LocalDate/parse "2026-03-31") :org tenant}
                   (select-keys tb [:report :date :org])))
            (is (some #{["Operating account (1010)" "1846.13" "2400.00" "12480.25" ""]} cells))
            (is (nil? (:report-date tb)) "what changes on every fetch is not kept")))
        (testing "the next pull fetches only the latest month again, and keeps nothing when nothing changed"
          (is (= {"Reports" {:items 5 :written 0}} (xero/pull! node opts)))
          (is (= 20 (count (report-calls))))
          (is (= 15 (:n (first (store/q node "SELECT COUNT(*) AS n FROM xero.reports"))))))))))

(deftest fetch-errors-test
  (testing "xero-cli's exit code becomes the error's type, with xero-cli's own error as data"
    (doseq [[code type] xero/exit-types]
      (let [e (error #(xero/fetch ["bash" fake pull-1 (str code) (str (temp-log))] tenant "Invoices" {}))]
        (is (= type (:type e)))
        (is (= {:type "refused" :message "Xero refused the request." :hint "Check the org id."} (:error e))))))
  (testing "a missing xero-cli says how to fix it"
    (is (= :xero/not-found (:type (error #(xero/fetch "/nowhere/xero-cli" tenant "Invoices" {}))))))
  (testing "the samples are where the tests expect them"
    (is (.isDirectory (io/file pull-1)))))

(deftest smaller-lists-test
  (testing "items, users, linked transactions and batch payments are rows of their own tables, by Xero's id"
    (let [at (Instant/parse "2026-03-20T12:00:00Z")]
      (is (= {:xero/items [(str "xero:" tenant ":item:i1")]
              :xero/users [(str "xero:" tenant ":user:u1")]
              :xero/linked-transactions [(str "xero:" tenant ":linked-transaction:l1")]
              :xero/batch-payments [(str "xero:" tenant ":batch-payment:b1")]}
             (into {} (for [[resource answer] {"Items" {:Items [{:ItemID "i1" :Code "GREEN-1"}]}
                                               "Users" {:Users [{:UserID "u1" :IsSubscriber false}]}
                                               "LinkedTransactions" {:LinkedTransactions [{:LinkedTransactionID "l1"}]}
                                               "BatchPayments" {:BatchPayments [{:BatchPaymentID "b1"
                                                                                 :TotalAmount 920.00M}]}}
                            [table rows] (xero/rows tenant resource answer at)]
                        [table (map :xt/id rows)])))))))

(deftest history-test
  (with-node
    (fn [node]
      (let [log (temp-log)
            opts (fn [dir calls] {:cli ["bash" fake dir "0" (str log)] :tenant tenant :resources []
                                  :reports {:months 0} :history {:calls calls}})
            api-calls #(filter (fn [call] (str/starts-with? call "api ")) (str/split-lines (slurp log)))
            bt (fn [n] (str "c0ffee05-0000-4000-8000-00000000000" n))]
        (keep-all! node pull-1)
        (testing "a pull spends at most its calls, bank transactions first and the newest of them first"
          (is (= {"History" {:items 3 :written 3 :waiting 11}} (xero/pull! node (opts pull-1 3))))
          (is (= [(str "api /api.xro/2.0/BankTransactions/" (bt 6) "/History --org " tenant)
                  (str "api /api.xro/2.0/BankTransactions/" (bt 4) "/History --org " tenant)
                  (str "api /api.xro/2.0/BankTransactions/" (bt 5) "/History --org " tenant)]
                 (api-calls))))
        (testing "the next pull goes on where it stopped: the history of every document, and the attachments of one"
          (is (= {"History" {:items 11 :written 11 :waiting 0}} (xero/pull! node (opts pull-1 100))))
          (is (= {"BankTransactions" 6 "Invoices" 7}
                 (into {} (map (juxt :resource :n))
                       (store/q node "SELECT resource, COUNT(*) AS n FROM xero.history GROUP BY resource"))))
          (is (= [(str "api /api.xro/2.0/BankTransactions/" (bt 3) "/Attachments --org " tenant)]
                 (filter #(str/includes? % "/Attachments") (api-calls)))))
        (testing "a document's history is Xero's records, with the document it belongs to and its version"
          (let [[h] (store/q node ["SELECT * FROM xero.history WHERE _id = ?"
                                   (str "xero:" tenant ":history:bank-transaction:" (bt 3))])]
            (is (= {:resource "BankTransactions" :org tenant
                    :document (str "xero:" tenant ":bank-transaction:" (bt 3))}
                   (select-keys h [:resource :org :document])))
            (is (= ["Note" "Created"] (map :changes (:history-records h))))
            (is (= (Instant/parse "2026-03-13T09:00:00Z")
                   (.toInstant ^ZonedDateTime (:date-utc (second (:history-records h))))))))
        (testing "an attachment's details are kept, not its file"
          (is (= [{:file-name "receipt-sample-bags.pdf" :mime-type "application/pdf" :content-length 48213}]
                 (map #(select-keys % [:file-name :mime-type :content-length])
                      (:attachments (first (store/q node "SELECT attachments FROM xero.attachments")))))))
        (testing "with nothing changed, a pull makes no calls"
          (is (= {"History" {:items 0 :written 0 :waiting 0}} (xero/pull! node (opts pull-1 100)))))
        (testing "after the documents change, only the new versions are fetched"
          (keep-all! node pull-2)
          (let [before (count (api-calls))
                result (xero/pull! node (opts pull-1 100))
                calls (drop before (api-calls))]
            (is (= 10 (get-in result ["History" :items])))
            (is (= {"BankTransactions" 6 "ManualJournals" 1 "Invoices" 3}
                   (frequencies (map #(second (re-find #"^api /api.xro/2.0/(\w+)/" %)) calls))))))))))

(deftest history-refused-test
  (with-node
    (fn [node]
      (let [dir (.toFile (Files/createTempDirectory "fake-xero" (make-array FileAttribute 0)))
            refused "c0ffee05-0000-4000-8000-000000000006"]
        (io/make-parents dir "history" "x")
        (spit (io/file dir "history" (str refused ".refused")) "")
        (keep-all! node pull-1)
        (testing "a document Xero refuses is skipped, and tried again on the next pull"
          (is (= {"History" {:items 13 :written 13 :refused 1 :waiting 0}}
                 (xero/pull! node {:cli ["bash" fake (str dir) "0" (str (temp-log))] :tenant tenant :resources []
                                   :reports {:months 0} :history {:calls 100}})))
          (is (= {"History" {:items 0 :written 0 :refused 1 :waiting 0}}
                 (xero/pull! node {:cli ["bash" fake (str dir) "0" (str (temp-log))] :tenant tenant :resources []
                                   :reports {:months 0} :history {:calls 100}}))))))))

(deftest files-test
  (with-node
    (fn [node]
      (let [tmp (.toFile (Files/createTempDirectory "ownpurse record" (make-array FileAttribute 0)))
            empty-answers (.toFile (Files/createTempDirectory "fake-xero" (make-array FileAttribute 0)))
            keep-file #(store/keep-file! {:mode :disk :dir (str tmp)} %)
            opts (fn [dir history] {:cli ["bash" fake dir "0" (str (temp-log))] :tenant tenant :resources []
                                    :reports {:months 0} :history history :keep-file keep-file})
            attachment "c0ffee0a-0000-4000-8000-000000000001"]
        (keep-all! node pull-1)
        (is (= {"History" {:items 14 :written 14 :waiting 0}} (xero/pull! node (opts pull-1 {:files 0}))))
        (testing "a file Xero refuses is skipped and counted"
          (is (= {"Files" {:items 0 :written 0 :refused 1 :waiting 0}}
                 (xero/pull! node (opts (str empty-answers) {:calls 0 :files 5})))))
        (testing "each attachment's file is kept in the record, under its SHA-256, and named by a row of xero.files"
          (is (= {"Files" {:items 1 :written 1 :waiting 0}} (xero/pull! node (opts pull-1 {:calls 0 :files 5}))))
          (let [[row] (store/q node "SELECT * FROM xero.files")]
            (is (= {:xt/id (str "xero:" tenant ":file:" attachment)
                    :document (str "xero:" tenant ":bank-transaction:c0ffee05-0000-4000-8000-000000000003")
                    :attachment-id attachment
                    :file-name "receipt-sample-bags.pdf"
                    :mime-type "application/pdf"}
                   (select-keys row [:xt/id :document :attachment-id :file-name :mime-type])))
            (is (= (slurp (io/file pull-1 "files" attachment)) (slurp (io/file tmp (:path row)))))
            (is (= (.length (io/file pull-1 "files" attachment)) (:bytes row)))))
        (testing "a file already kept is not downloaded again"
          (is (= {"Files" {:items 0 :written 0 :waiting 0}} (xero/pull! node (opts pull-1 {:calls 0 :files 5})))))
        (testing "a record in memory cannot keep files"
          (is (= :ownpurse/no-disk
                 (:type (error #(xero/pull! node (assoc (opts pull-1 {:calls 0 :files 5}) :keep-file nil)))))))))))

(deftest history-first-test
  (with-node
    (fn [node]
      (let [log (temp-log)
            api-calls #(filter (fn [call] (str/starts-with? call "api ")) (str/split-lines (slurp log)))
            documents #(map (fn [call] (second (re-find #"^api /api.xro/2.0/(\w+/c0ffee0\d-[-0-9]+?\d)/" call))) %)
            pull #(xero/pull! node {:cli ["bash" fake pull-1 "0" (str log)] :tenant tenant :resources []
                                    :reports {:months 0} :history %})]
        (keep-all! node pull-1)
        (testing "documents dated in the :first range come before the rest, bank transactions first"
          (is (= {"History" {:items 6 :written 6 :waiting 8}}
                 (pull {:calls 6 :first {:from "2026-03-01" :to "2026-03-10"}})))
          (let [[in-range [next-one]] (split-at 5 (documents (api-calls)))]
            (is (= ["BankTransactions/c0ffee05-0000-4000-8000-000000000002"
                    "BankTransactions/c0ffee05-0000-4000-8000-000000000001"]
                   (take 2 in-range)))
            (is (= #{"Invoices/c0ffee03-0000-4000-8000-000000000002"
                     "Invoices/c0ffee03-0000-4000-8000-000000000005"
                     "Invoices/c0ffee03-0000-4000-8000-000000000006"}
                   (set (drop 2 in-range))))
            (is (= "BankTransactions/c0ffee05-0000-4000-8000-000000000006" next-one))))
        (testing "a range that is not two dates says how to write it"
          (is (= :ownpurse/invalid (:type (error #(pull {:calls 6 :first {:from "last year"}}))))))))))

(deftest history-recheck-test
  (with-node
    (fn [node]
      (let [log (temp-log)
            api-calls #(filter (fn [call] (str/starts-with? call "api ")) (str/split-lines (slurp log)))
            calls-of #(map (fn [call] (rest (re-find #"BankTransactions/c0ffee05-[-0-9]+?(\d)/(\w+)" call))) %)
            t0 (Instant/parse "2026-03-20T17:00:00Z")
            day #(.plus t0 (java.time.Duration/ofDays %))
            pull (fn [dir now history]
                   (let [before (count (api-calls))
                         result (xero/pull! node {:cli ["bash" fake dir "0" (str log)] :tenant tenant :resources []
                                                  :reports {:months 0} :now now
                                                  :history (merge {:resources ["BankTransactions"]} history)})]
                     [(result "History") (calls-of (drop before (api-calls)))]))]
        (keep-all! node pull-1)
        (testing "without a recheck, a pull fetches what is new and nothing else"
          (is (= {:items 7 :written 7 :waiting 0} (first (pull pull-1 t0 {:calls 100})))))
        (testing "a recheck looks at recent documents' attachments never looked at, the latest first"
          (is (= [{:items 3 :written 3 :waiting 0 :rechecked 3}
                  [["6" "Attachments"] ["4" "Attachments"] ["5" "Attachments"]]]
                 (pull pull-1 (day 1) {:calls 100 :recheck {:days 7 :every 7}}))))
        (testing "a document looked at within `every` days is not looked at again"
          (is (= [{:items 0 :written 0 :waiting 0 :rechecked 0} []]
                 (pull pull-1 (day 2) {:calls 100 :recheck {:days 7 :every 7}}))))
        (testing "later, the never-looked-at first, then the longest since, the latest documents first, within the cap;
                  an answer that did not change writes nothing but when it was asked"
          (is (= [{:items 4 :written 2 :waiting 0 :rechecked 4}
                  [["1" "Attachments"] ["2" "Attachments"] ["6" "History"] ["4" "History"]]]
                 (pull pull-1 (day 9) {:calls 4 :recheck {:days 30 :every 7}})))
          (is (= (day 9) (some-> (store/q node ["SELECT checked_at FROM xero.checks WHERE _id = ?"
                                                (str "xero:" tenant ":history:bank-transaction:"
                                                     "c0ffee05-0000-4000-8000-000000000006")])
                                 first :checked-at (#(.toInstant ^ZonedDateTime %))))))
        (testing "new versions of documents come before any recheck"
          (keep-all! node pull-2)
          (let [[counts calls] (pull pull-1 (day 10) {:calls 2 :recheck {:days 30 :every 7}})]
            (is (= 0 (:rechecked counts)))
            (is (pos? (:waiting counts)))
            (is (= 2 (count calls)))))
        (testing "a recheck that is not two whole numbers of days says how to write it"
          (is (= :ownpurse/invalid (:type (error #(pull pull-1 (day 11) {:calls 1 :recheck {:days "90"}}))))))))))
