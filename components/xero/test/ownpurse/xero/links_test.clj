(ns ownpurse.xero.links-test
  (:require
   [clojure.test :refer [deftest is testing]]
   [ownpurse.xero.links :as links]
   [ownpurse.xero.source :as source]))

(def ^:private tenant "c0ffee00-0000-4000-8000-000000000000")

(deftest links-test
  (let [docs {"Invoices" [{:invoice-id "i-1" :type "ACCPAY"} {:invoice-id "i-2" :type "ACCREC"}]
              "Payments" [{:payment-id "p-1" :invoice {:invoice-id "i-1"}}]
              "BankTransactions" [{:bank-transaction-id "b-1"}]
              "ManualJournals" [{:manual-journal-id "m-1"}]}
        l (links/links tenant "!aBc12" docs)
        id #(source/row-id tenant %1 %2)]
    (testing "a bill and a sales invoice open in their own ledger, through the org's login"
      (is (= (str "https://go.xero.com/organisationlogin/default.aspx?shortcode=!aBc12&redirecturl="
                  "%2FAccountsPayable%2FView.aspx%3FInvoiceID%3Di-1")
             (l (id "invoice" "i-1"))))
      (is (re-find #"AccountsReceivable" (l (id "invoice" "i-2")))))
    (testing "a payment links to what it pays"
      (is (= (l (id "invoice" "i-1")) (l (id "payment" "p-1")))))
    (testing "bank transactions and journals have links of their own; without a short code, a plain link"
      (is (re-find #"bankTransactionID%3Db-1" (l (id "bank-transaction" "b-1"))))
      (is (= "https://go.xero.com/Journal/View.aspx?invoiceID=m-1"
             ((links/links tenant nil docs) (id "manual-journal" "m-1")))))))
