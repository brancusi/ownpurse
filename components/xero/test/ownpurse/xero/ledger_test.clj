(ns ownpurse.xero.ledger-test
  (:require
   [clojure.test :refer [deftest is testing]]
   [ownpurse.ledger.interface :as ledger]
   [ownpurse.store.interface :as store]
   [ownpurse.xero.interface :as xero]
   [ownpurse.xero.source :as source]
   [ownpurse.xero.trial-balance :as tb])
  (:import
   (java.time Instant LocalDate)))

(def ^:private tenant "c0ffee00-0000-4000-8000-000000000000")
(def ^:private pull-1 "samples/xero/pull-1")
(def ^:private pull-2 "samples/xero/pull-2")
(def ^:private fake "components/xero/test/ownpurse/xero/fake-xero-cli.sh")

(defn- acct [n] (format "c0ffee01-0000-4000-8000-%012d" n))
(defn- id [n] (source/row-id tenant "account" (acct n)))

(def ^:private accounts
  "Part of the fixture's chart: 1010 bank, 1020 bank, 1200 receivables, 2000 payables, 2200 sales tax, 2500 loan,
  4000 sales, 5000 green coffee, 6100 meals, 6200 rent."
  (for [[n code system] [[1 "1010"] [2 "1020"] [3 "1200" "DEBTORS"] [6 "2000" "CREDITORS"] [7 "2200" "GST"] [8 "2500"]
                         [9 "4000"] [11 "5000"] [12 "6100"] [13 "6200"]]]
    (cond-> {:account-id (acct n) :code code} system (assoc :system-account system))))

(defn- day [s] (LocalDate/parse s))

(defn- postings
  "An entry's postings as [account-number amount], by the account's code, for reading."
  [entry]
  (let [code (into {} (map (juxt #(source/row-id tenant "account" (:account-id %)) :code)) accounts)]
    (mapv (juxt #(code (:posting/account %) (:posting/account %)) :posting/amount) (:entry/postings entry))))

(defn- only [docs]
  (let [{:keys [entries] :as result} (xero/entries tenant "USD" (assoc docs :accounts accounts))]
    (assoc result :postings (mapv postings entries))))

(deftest invoices-test
  (testing "a sale on account, tax exclusive: receivables debited with the total, sales and tax credited"
    (is (= [[["4000" -1701.50M] ["2200" -144.63M] ["1200" 1846.13M]]]
           (:postings (only {:invoices [{:invoice-id "i1" :type "ACCREC" :status "AUTHORISED" :date (day "2026-02-16")
                                         :line-amount-types "Exclusive" :total 1846.13M :invoice-number "INV-0412"
                                         :line-items [{:account-code "4000" :line-amount 1701.50M
                                                       :tax-amount 144.63M}]}]})))))
  (testing "a bill, tax inclusive: the line's amount holds its tax; payables credited"
    (is (= [[["5000" 100.00M] ["2200" 10.00M] ["2000" -110.00M]]]
           (:postings (only {:invoices [{:invoice-id "b1" :type "ACCPAY" :status "PAID" :date (day "2026-03-01")
                                         :line-amount-types "Inclusive" :total 110M
                                         :line-items [{:account-code "5000" :line-amount 110M :tax-amount 10M}]}]})))))
  (testing "a credit note reverses its side's invoice"
    (is (= [[["4000" 50.00M] ["1200" -50.00M]]]
           (:postings (only {:credit-notes [{:credit-note-id "c1" :type "ACCRECCREDIT" :status "AUTHORISED"
                                             :date (day "2026-03-02") :line-amount-types "NoTax" :total 50M
                                             :line-items [{:account-code "4000" :line-amount 50M}]}]})))))
  (testing "drafts and voided invoices post nothing"
    (is (= {"Invoices:DRAFT" 1 "Invoices:VOIDED" 1}
           (:skipped (only {:invoices [{:invoice-id "d" :type "ACCREC" :status "DRAFT"}
                                       {:invoice-id "v" :type "ACCREC" :status "VOIDED"}]})))))
  (testing "another currency, at the document's rate, the rounding on the control account"
    (is (= [[["4000" -33.33M] ["4000" -33.33M] ["1200" 66.66M]]]
           (:postings (only {:invoices [{:invoice-id "f1" :type "ACCREC" :status "AUTHORISED" :date (day "2026-03-03")
                                         :currency-code "EUR" :currency-rate 0.9M :line-amount-types "NoTax"
                                         :total 60M :line-items [{:account-code "4000" :line-amount 30M}
                                                                 {:account-code "4000" :line-amount 30M}]}]})))))
  (testing "a document that does not balance is listed, not posted"
    (let [{:keys [entries unbalanced]} (only {:invoices [{:invoice-id "u1" :type "ACCREC" :status "AUTHORISED"
                                                          :date (day "2026-03-04") :line-amount-types "NoTax"
                                                          :total 10.01M
                                                          :line-items [{:account-code "4000" :line-amount 10M}]}]})]
      (is (empty? entries))
      (is (= [{:resource "Invoices" :id "u1" :date (day "2026-03-04") :imbalance 0.01M}] unbalanced)))))

(deftest bank-test
  (testing "money spent: the line's account debited, the bank credited"
    (is (= [[["6100" 4.75M] ["1010" -4.75M]]]
           (:postings (only {:bank-transactions [{:bank-transaction-id "s1" :type "SPEND" :status "AUTHORISED"
                                                  :date (day "2026-03-06") :line-amount-types "NoTax" :total 4.75M
                                                  :bank-account {:account-id (acct 1)}
                                                  :line-items [{:account-code "6100" :line-amount 4.75M}]}]})))))
  (testing "money received: the bank debited"
    (is (= [[["4000" -20.00M] ["1010" 20.00M]]]
           (:postings (only {:bank-transactions [{:bank-transaction-id "r1" :type "RECEIVE" :status "AUTHORISED"
                                                  :date (day "2026-03-06") :line-amount-types "NoTax" :total 20M
                                                  :bank-account {:account-id (acct 1)}
                                                  :line-items [{:account-code "4000" :line-amount 20M}]}]})))))
  (testing "a transfer's legs are skipped; the transfer posts once"
    (let [{:keys [postings skipped]}
          (only {:bank-transactions [{:bank-transaction-id "t1" :type "SPEND-TRANSFER" :status "AUTHORISED"}]
                 :bank-transfers [{:bank-transfer-id "x1" :amount 5000M :date (day "2026-03-16")
                                   :from-bank-account {:account-id (acct 1)} :to-bank-account {:account-id (acct 2)}}]})]
      (is (= [[["1020" 5000.00M] ["1010" -5000.00M]]] postings))
      (is (= {"BankTransactions:SPEND-TRANSFER:AUTHORISED" 1} skipped))))
  (testing "payments: received against receivables, made against payables"
    (is (= [[["1200" -1846.13M] ["1010" 1846.13M]] [["2000" 110.00M] ["1010" -110.00M]]]
           (:postings (only {:payments [{:payment-id "p1" :payment-type "ACCRECPAYMENT" :status "AUTHORISED"
                                         :date (day "2026-03-02") :amount 1846.13M :account {:code "1010"}}
                                        {:payment-id "p2" :payment-type "ACCPAYPAYMENT" :status "AUTHORISED"
                                         :date (day "2026-03-05") :amount 110M :account {:code "1010"}}]})))))
  (testing "a manual journal posts its lines; a line to an account the chart lacks goes to a placeholder"
    (is (= [[["6200" 100.00M] [(source/row-id tenant "account" "(uncoded):9999") -100.00M]]]
           (:postings (only {:manual-journals [{:manual-journal-id "j1" :status "POSTED" :date (day "2026-03-31")
                                                :journal-lines [{:account-code "6200" :line-amount 100M}
                                                                {:account-code "9999" :line-amount -100M}]}]}))))))

(defn- with-node [f]
  (let [node (store/start {:mode :in-memory})]
    (try (f node) (finally (store/stop node)))))

(defn- pull! [node dir]
  (xero/pull! node {:cli ["bash" fake dir "0" "/dev/null"] :tenant tenant :resources xero/pull-order
                    :reports {:months 0} :history {:calls 0}}))

(deftest post-test
  (with-node
    (fn [node]
      (testing "a pull posts the org's entries, every one balanced"
        (let [{:strs [Ledger]} (pull! node pull-1)]
          (is (pos? (:items Ledger)))
          (is (= (:items Ledger) (:written Ledger)))
          (is (nil? (:unbalanced Ledger))))
        (is (= [0.00M] (map :net (store/q node "SELECT SUM(amount) AS net FROM postings")))))
      (testing "a second pull of the same documents writes no entry"
        (is (= 0 (get-in (pull! node pull-1) ["Ledger" :written]))))
      (testing "after the documents change, the changed entries are written and those that no longer hold end"
        (let [{:strs [Ledger]} (pull! node pull-2)]
          (is (pos? (:written Ledger)))
          (is (pos? (:ended Ledger))))
        (is (= [0.00M] (map :net (store/q node "SELECT SUM(amount) AS net FROM postings"))))))))

(defn- report-answer
  "A Xero report answer: `header` and sections of [name account-id & values]."
  [header sections]
  {:Reports [{:ReportID "x" :Rows (into [{:RowType "Header" :Cells (mapv #(hash-map :Value %) header)}]
                                        (for [[title rows] sections]
                                          {:RowType "Section" :Title title
                                           :Rows (vec (for [[name account & values] rows]
                                                        {:RowType "Row"
                                                         :Cells (mapv (fn [v] (cond-> {:Value v}
                                                                                account (assoc :Attributes [{:Id "account" :Value account}])))
                                                                      (cons name values))}))}))}]})

(defn- keep-report! [node report date answer]
  (store/put-changed! node {:xero/reports [(source/report-row tenant report {"date" date} answer (Instant/now))]}
                      {:source "test" :why "fixture"}))

(deftest trial-balance-test
  (with-node
    (fn [node]
      (pull! node pull-1)
      ;; A retained earnings account, as Xero's charts have.
      (store/put-changed! node {:xero/accounts [{:xt/id (id 30) :org tenant :account-id (acct 30) :code "3200"
                                                 :name "Retained Earnings" :class "EQUITY"
                                                 :system-account "RETAINEDEARNINGS"}]}
                          {:source "test" :why "fixture"})
      (let [year (into {} (keep (fn [[k v]] (when-let [[_ x] (re-find #":account:(.+)$" k)] [x v])))
                       (ledger/balances node (xero/book-id tenant) {:from (day "2026-01-01") :to (day "2026-03-31")}))
            opening {(acct 1) 10000M (acct 4) 18000M (acct 8) -10000M (acct 30) -12000M}
            plain #(.toPlainString (.abs ^BigDecimal %))
            expected (merge-with + opening year {(acct 30) -6000M (acct 13) 0.01M})
            tb-answer (report-answer ["Account" "Debit" "Credit" "YTD Debit" "YTD Credit"]
                                     [["Accounts" (for [[a v] expected]
                                                    [a a "" "" (if (pos? v) (plain v) "") (if (neg? v) (plain v) "")])]])
            bs-answer (report-answer ["" "31 Dec 2025"]
                                     [["Assets" [["Operating account" (acct 1) "10000.00"]
                                                 ["Roasting equipment" (acct 4) "18000.00"]]]
                                      ["Liabilities" [["Equipment loan" (acct 8) "10000.00"]]]
                                      ["Equity" [["Retained Earnings" (acct 30) "12000.00"]
                                                 ["Current Year Earnings" nil "6000.00"]]]])]
        (testing "without the trial balance, the check says which month ends it can use"
          (is (= :ownpurse/not-found
                 (:type (ex-data (try (xero/check-trial-balance node tenant (day "2026-03-31") {})
                                      (catch clojure.lang.ExceptionInfo e e)))))))
        (keep-report! node "TrialBalance" "2026-03-31" tb-answer)
        (testing "without the balance sheet before the year, it says how to get one, or to open from the ledger"
          (is (= "BalanceSheet"
                 (:report (ex-data (try (xero/check-trial-balance node tenant (day "2026-03-31") {})
                                        (catch clojure.lang.ExceptionInfo e e)))))))
        (keep-report! node "BalanceSheet" "2025-12-31" bs-answer)
        (let [{:keys [tied? accounts year-start]} (xero/check-trial-balance node tenant (day "2026-03-31") {})
              by-code (into {} (map (juxt :code identity)) accounts)]
          (testing "the year starts after the year end Xero's org names"
            (is (= (day "2026-01-01") year-start)))
          (testing "balance sheet accounts open at Xero's balance sheet; last year's earnings join retained earnings"
            (is (= {:xero -18000M :ownpurse -18000M :difference 0M}
                   (select-keys (by-code "3200") [:xero :ownpurse :difference])))
            (is (zero? (:difference (by-code "1010")))))
          (testing "an account that differs is shown with its difference, and the book does not tie"
            (is (false? tied?))
            (is (= [["6200" -0.01M]] (keep #(when-not (zero? (:difference %)) [(:code %) (:difference %)]) accounts)))))
        (testing "opening from the book's own postings needs no balance sheet"
          (is (= "ledger" (:opening (xero/check-trial-balance node tenant (day "2026-03-31") {:opening :ledger})))))))))

(deftest year-start-test
  (is (= (day "2026-01-01") (tb/year-start (day "2026-03-31") 12 31)))
  (is (= (day "2025-07-01") (tb/year-start (day "2026-03-31") 6 30)))
  (is (= (day "2026-07-01") (tb/year-start (day "2026-07-01") 6 30)))
  (is (= (day "2025-07-01") (tb/year-start (day "2026-06-30") 6 30))))
