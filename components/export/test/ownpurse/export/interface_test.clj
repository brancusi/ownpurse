(ns ownpurse.export.interface-test
  (:require
   [clojure.data.csv :as csv]
   [clojure.java.io :as io]
   [clojure.string :as str]
   [clojure.test :refer [deftest is testing]]
   [ownpurse.close.interface :as close]
   [ownpurse.export.interface :as export]
   [ownpurse.ledger.interface :as ledger]
   [ownpurse.store.interface :as store])
  (:import
   (java.io File)
   (java.nio.file Files)
   (java.nio.file.attribute FileAttribute)
   (java.time LocalDate)
   (java.util.zip ZipFile)))

(def ^:private book "book:test:tallowmere")
(defn- day [s] (LocalDate/parse s))

(def ^:private chart
  {"acct:1010" {:code "1010" :name "Operating account" :class "ASSET"}
   "acct:3000" {:code "3000" :name "Owner's equity" :class "EQUITY"}
   "acct:4000" {:code "4000" :name "Sales" :class "REVENUE"}
   "acct:6200" {:code "6200" :name "Rent" :class "EXPENSE"}})

(defn- entry [id date document & postings]
  {:entry/id id :entry/book book :entry/date (day date) :entry/narration (str "Entry " id) :entry/document document
   :entry/postings (vec (map-indexed (fn [i [account amount]]
                                       {:posting/line i :posting/account account :posting/amount amount
                                        :posting/commodity "USD" :posting/party "party:cafe"})
                                     postings))})

(def ^:private trial-balance
  ;; As the trial balance check returns it at the year end, with adjustments.
  {:accounts [{:account "acct:1010" :code "1010" :name "Operating account" :xero 1300.00M :ownpurse 1300.00M
               :difference 0M :adjustments 0M :pro-forma 1300.00M}
              {:account "acct:3000" :code "3000" :name "Owner's equity" :xero -500.00M :ownpurse -500.00M
               :difference 0M :adjustments 0M :pro-forma -500.00M}
              {:account "acct:4000" :code "4000" :name "Sales" :xero -1200.00M :ownpurse -1200.00M
               :difference 0M :adjustments 0M :pro-forma -1200.00M}
              {:account "acct:6200" :code "6200" :name "Rent" :xero 400.00M :ownpurse 400.00M :difference 0M
               :adjustments -100.00M :pro-forma 300.00M}]})

(defn- read-csv [f] (with-open [r (io/reader f)] (doall (csv/read-csv r))))

(deftest export-test
  (let [node (store/start {:mode :in-memory})
        dir (str (Files/createTempDirectory "ownpurse-export" (make-array FileAttribute 0)))]
    (try
      (ledger/put-entries! node book [(entry "e1" "2025-02-01" "doc:invoice:1" ["acct:1010" 1200.00M] ["acct:4000" -1200.00M])
                                      (entry "e2" "2025-03-01" "doc:bill:2" ["acct:6200" 400.00M] ["acct:1010" -400.00M])]
                           {:source "test" :why "fixture"})
      (close/put-item! node {:ref "C-01" :book book :title "Prepaid rent" :status "accountant decision"
                             :evidence ["doc:bill:2"]
                             :journals [{:ref "J1" :date (day "2025-12-31") :narration "Prepaid"
                                         :postings [{:account "acct:1400" :amount 100.00M :commodity "USD"}
                                                    {:account "acct:6200" :amount -100.00M :commodity "USD"}]}]})
      (close/put-item! node {:ref "C-02" :book book :title "Old deposit" :status "excluded" :reason "Below materiality"})
      (let [opts {:dir dir :name "Tallowmere Coffee" :book book :from (day "2025-01-01") :to (day "2025-12-31")
                  :trial-balance trial-balance :accounts chart :parties {"party:cafe" {:name "Corner Cafe"}}
                  :links {"doc:invoice:1" "https://example.test/invoice/1"}
                  :payees {:threshold 600.00M :payees [{:party "party:cafe" :name "Corner Cafe" :paid {"bank" 750.00M}
                                                        :payments 2 :candidate? true}]}}
            {:keys [workbook sheets] out :dir} (export/export! node opts)
            csv-dir (io/file out "tallowmere-coffee-2025-12-31-csv")]
        (testing "a workbook with a README first and a sheet per report, and a CSV of each beside it"
          (is (str/ends-with? workbook "tallowmere-coffee-2025-12-31/tallowmere-coffee-2025-12-31.xlsx"))
          (is (= ["README" "Trial balance" "Balance sheet" "Profit and loss" "General ledger" "Adjustments" "Exclusions"
                  "Bank reconciliation" "Unreconciled lines" "Intercompany" "Payees"]
                 (map :name sheets)))
          (is (= 11 (count (.listFiles csv-dir)))))
        (testing "the general ledger opens each account, runs its balance, and names each line's document and link"
          (let [rows (read-csv (io/file csv-dir "general-ledger.csv"))
                rent (filter #(= "6200" (first %)) rows)]
            (is (= ["code" "account" "date" "narration" "description" "party" "amount" "balance" "document" "link"
                    "entry" "account id"]
                   (first rows)))
            (is (= [["Opening balance" "" "0.00"] ["Entry e2" "400.00" "400.00"]] (map #(vector (nth % 3) (nth % 6) (nth % 7)) rent)))
            (is (some #(= ["doc:invoice:1" "https://example.test/invoice/1" "Corner Cafe"] [(nth % 8) (nth % 9) (nth % 5)])
                      rows))))
        (testing "the balance sheet carries the year's earnings and sums to zero; the P&L nets them"
          (let [rows (read-csv (io/file csv-dir "balance-sheet.csv"))]
            (is (= ["Current year earnings (from the profit and loss)" "-800.00" "-100.00" "-900.00"]
                   (let [r (first (filter #(str/starts-with? (second %) "Current year") rows))] [(second r) (nth r 3) (nth r 4) (nth r 5)])))
            (is (= ["0.00" "-100.00" "-100.00"] (subvec (vec (last rows)) 3 6)))))
        (testing "set-aside items are in the exclusions with their reason, not among the adjustments"
          (is (= [["C-02" "Old deposit" "excluded"]] (map #(subvec % 0 3) (rest (read-csv (io/file csv-dir "exclusions.csv"))))))
          (is (= #{"C-01"} (set (map first (rest (read-csv (io/file csv-dir "adjustments.csv"))))))))
        (testing "the workbook's amounts are numbers written as their exact decimals, its text inline"
          (with-open [z (ZipFile. ^String workbook)]
            (let [sheet #(slurp (.getInputStream z (.getEntry z (str "xl/worksheets/sheet" % ".xml"))))]
              (is (str/includes? (sheet 2) "<v>1300.00</v>"))
              (is (str/includes? (sheet 1) "Record transaction"))
              (is (not (str/includes? (sheet 2) "<f>"))))))
        (testing "an export never overwrites one"
          (is (= :ownpurse/export-exists (:type (ex-data (try (export/export! node opts)
                                                              (catch clojure.lang.ExceptionInfo e e))))))))
      (finally
        (store/stop node)
        (doseq [^File f (reverse (file-seq (io/file dir)))] (.delete f))))))
