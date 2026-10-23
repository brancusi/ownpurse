(ns ownpurse.xero.interface-test
  (:require
   [clojure.test :refer [deftest is testing]]
   [ownpurse.money.interface :as money]
   [ownpurse.schema.interface :as schema]
   [ownpurse.xero.core :as core]
   [ownpurse.xero.interface :as xero]))

(def sample
  {:record "samples/xero-axi/brackenridge/record.sqlite"
   :gl "samples/xero-axi/brackenridge/gl.json"})

(def ^:private source (delay (xero/read-source sample)))
(def ^:private facts (delay (xero/facts @source)))

(deftest read-source-test
  (is (= {:orgs 4 :accounts 49 :contacts 0 :documents 39 :gl 78}
         (update-vals (select-keys @source [:orgs :accounts :contacts :documents :gl]) count)))
  (is (re-matches #"[0-9a-f]{64}" (:chain-head @source)))
  (testing "a missing record or gl file says what to do"
    (is (= :ownpurse/no-record
           (try (xero/read-source (assoc sample :record "nowhere.sqlite"))
                (catch clojure.lang.ExceptionInfo e (:type (ex-data e))))))
    (is (= :ownpurse/no-gl
           (try (xero/read-source (assoc sample :gl "nowhere.json"))
                (catch clojure.lang.ExceptionInfo e (:type (ex-data e))))))))

(deftest facts-test
  (let [{:keys [sources books accounts parties documents entries]} @facts]
    (is (= {:sources 4 :books 4 :accounts 49 :parties 23 :documents 39 :entries 39}
           (update-vals @facts count)))
    (testing "every fact matches its schema"
      (doseq [[k xs] {:ownpurse/source sources :ownpurse/book books :ownpurse/account accounts
                      :ownpurse/party parties :ownpurse/document documents :ownpurse/entry entries}
              x xs]
        (is (schema/valid? k x) (pr-str (schema/explain k x)))))
    (testing "the four books, in US dollars with a calendar year"
      (is (= #{"Tallowmere Coffee Roasting Co." "Brackenridge Lane Holdings Ltd"
               "Quillfeather Design Consultancy LLP" "Saltmarsh Cupping Lab LLC"}
             (set (map :book/name books))))
      (is (every? #(= "USD" (:book/currency %)) books))
      (is (every? #(= {:month 12 :day 31} (:book/fiscal-year-end %)) books)))
    (testing "every posting's account is one of its book's accounts, and every entry has its document"
      (let [account-ids (set (map :account/id accounts))
            document-ids (set (map :document/id documents))]
        (is (every? account-ids (for [e entries p (:entry/postings e)] (:posting/account p))))
        (is (every? (comp document-ids :entry/document) entries))))
    (testing "each book's postings sum to zero"
      (doseq [[book es] (group-by :entry/book entries)]
        (is (money/balanced? (for [e es p (:entry/postings e)]
                               {:amount (:posting/amount p) :commodity (:posting/commodity p)}))
            book)))
    (testing "the same record gives the same facts"
      (is (= @facts (xero/facts (xero/read-source sample)))))))

(deftest normalize-payload-test
  (is (= {:bank-account {:account-id "a"} :line-items [{:line-amount 1.10M}] :updated-date-utc "x" :url "u"}
         (core/normalize-payload {:BankAccount {:AccountID "a"} :LineItems [{:LineAmount 1.10M :Tracking nil}]
                                  :UpdatedDateUTC "x" :URL "u" :SystemAccount nil}))))
