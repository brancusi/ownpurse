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
   (java.time Instant LocalDate)))

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

(deftest pull-test
  (with-node
    (fn [node]
      (let [log (temp-log)
            opts {:cli ["bash" fake pull-1 "0" (str log)] :tenant tenant :resources ["Organisation" "Invoices"]}]
        (testing "pull! runs xero-cli for each resource and keeps what it prints"
          (is (= {"Organisation" {:items 1 :written 1} "Invoices" {:items 7 :written 7}} (xero/pull! node opts))))
        (testing "the next pull asks only for what changed since the latest update held"
          (is (= {"Organisation" {:items 1 :written 0} "Invoices" {:items 7 :written 0}} (xero/pull! node opts)))
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
                                              :orgs (xero/read-orgs (str config))})]
            (is (= ["Tallowmere"] (keys result)))
            (is (= xero/pull-order (keys (result "Tallowmere"))))))))))

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
