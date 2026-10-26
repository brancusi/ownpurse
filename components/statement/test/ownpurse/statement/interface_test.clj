(ns ownpurse.statement.interface-test
  (:require
   [clojure.test :refer [deftest is testing]]
   [ownpurse.statement.interface :as statement]
   [ownpurse.store.interface :as store])
  (:import
   (java.time LocalDate)))

(def ^:private formats
  {"bank-export" {:skip 1 :date "Posted Date" :date-format "MM/dd/yyyy" :debit "Debit" :credit "Credit"
                  :description "Description" :id "Reference" :balance "Balance"}
   "card-export" {:delimiter ";" :decimal "," :date "Transaction Date" :amount "Amount" :sign "debit-positive"
                  :description "Description"}
   "petty-cash" {:date "Date" :amount "Amount" :description "Description" :balance "Balance"}})

(def ^:private accounts
  [{:account "operating" :name "Operating checking" :currency "USD"}
   {:account "card" :name "Company card" :kind "credit-card" :currency "USD"}
   {:account "petty-cash" :name "Petty cash" :currency "USD"}])

(def ^:private q1 {:path "samples/statement/bank-2025-q1.csv" :account "operating" :format "bank-export"
                   :opening "1000.00" :closing "1002.90" :from "2025-01-01" :to "2025-03-31"})

(def ^:private config
  {:formats formats :accounts accounts
   :files [q1
           {:path "samples/statement/card-2025-01.csv" :account "card" :format "card-export"}
           {:path "samples/statement/petty-cash.csv" :account "petty-cash" :format "petty-cash" :opening "0.00"
            :closing "181.35"}]})

(defn- day [s] (LocalDate/parse s))

(defn- with-node [f]
  (let [node (store/start {:mode :in-memory})]
    (try (f node) (finally (store/stop node)))))

(defn- error [f]
  (try (f) nil (catch clojure.lang.ExceptionInfo e (ex-data e))))

(deftest parse-test
  (testing "debit and credit columns, a heading above the header, and the file's own dates"
    (let [{:keys [lines]} (statement/parse (formats "bank-export") (slurp "samples/statement/bank-2025-q1.csv"))]
      (is (= [[(day "2025-01-03") -42.10M "ACME PAPER CO, ORDER 1182" "TX1001" 957.90M 3]
              [(day "2025-01-15") 1250.00M "CUSTOMER DEPOSIT TALLOWMERE" "TX1002" 2207.90M 4]]
             (map (juxt :date :amount :description :reference :balance :row) (take 2 lines))))))
  (testing "semicolons, a decimal comma, and charges written positive"
    (is (= [-1234.56M 1000.00M -3.50M -3.50M]
           (map :amount (:lines (statement/parse (formats "card-export") (slurp "samples/statement/card-2025-01.csv")))))))
  (testing "a column the file lacks is named, with the columns it has"
    (is (re-find #"no column \"Posted\".*\"Posted Date\""
                 (try (statement/parse (assoc (formats "bank-export") :date "Posted")
                                       (slurp "samples/statement/bank-2025-q1.csv"))
                      (catch clojure.lang.ExceptionInfo e (ex-message e))))))
  (testing "a date that does not read says which line"
    (is (= 2 (:row (error #(statement/parse {:date "Date" :amount "Amount"} "Date,Amount\n02/01/2025,1.00\n"))))))
  (testing "parentheses are a negative amount; a total line with no date is skipped"
    (is (= [-12.00M] (map :amount (:lines (statement/parse {:date "Date" :amount "Amount"}
                                                           "Date,Amount\n2025-01-01,(12.00)\n,\n")))))))

(deftest import-test
  (with-node
    (fn [node]
      (testing "an import keeps every line once, with a source account and a bank line for each"
        (is (= {:files 3 :lines 11 :written 11} (statement/import! node config {})))
        (is (= #{"statement:operating:account" "statement:card:account" "statement:petty-cash:account"}
               (set (map :xt/id (store/q node "SELECT _id FROM source_accounts WHERE source = 'statement'")))))
        (is (= 11 (count (store/q node "SELECT _id FROM bank_lines WHERE source = 'statement'")))))
      (testing "two identical lines in one file are two lines"
        (is (= 2 (count (store/q node ["SELECT _id FROM bank_lines WHERE account = ? AND amount = -3.50"
                                       "statement:card:account"])))))
      (testing "a second import of the same files writes nothing"
        (is (= 0 (:written (statement/import! node config {})))))
      (testing "an overlapping export adds only the lines the record lacks"
        (let [more (update config :files conj {:path "samples/statement/bank-2025-q1-q2.csv" :account "operating"
                                               :format "bank-export"})]
          (is (= 1 (:written (statement/import! node more {}))))
          (is (= 12 (count (store/q node "SELECT _id FROM bank_lines WHERE source = 'statement'"))))))
      (testing "each file says whether it ties: its opening and lines make its closing, its running balance follows"
        (let [by-name (into {} (map (juxt :name identity)) (statement/statements node))]
          (is (true? (:tied? (by-name "bank-2025-q1.csv"))))
          (is (= 2.90M (:total (by-name "bank-2025-q1.csv"))))
          (is (true? (:tied? (by-name "petty-cash.csv"))))
          (is (false? (:tied? (by-name "card-2025-01.csv"))))))
      (testing "a closing the lines do not make, or a broken running balance, does not tie"
        (statement/import! node (assoc config :files [(assoc q1 :closing "1000.00" :opening "999.00")]) {})
        (let [q1 (first (filter #(= "bank-2025-q1.csv" (:name %)) (statement/statements node)))]
          (is (false? (:tied? q1)))
          (is (= [{:row 3 :expected 956.90M :balance 957.90M}] (:breaks q1)))))
      (testing "a file naming an account or format the config lacks is refused"
        (is (= :ownpurse/invalid
               (:type (error #(statement/import! node (assoc config :files [(assoc q1 :account "other")]) {})))))))))
