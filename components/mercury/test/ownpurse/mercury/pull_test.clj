(ns ownpurse.mercury.pull-test
  (:require
   [clojure.java.io :as io]
   [clojure.string :as str]
   [clojure.test :refer [deftest is testing]]
   [ownpurse.json.interface :as json]
   [ownpurse.mercury.interface :as mercury]
   [ownpurse.store.interface :as store])
  (:import
   (java.io File)
   (java.time Instant LocalDate)))

(def ^:private org "4d0c5e11-0000-4000-8000-000000000001")
(def ^:private pull-1 "samples/mercury/pull-1")
(def ^:private pull-2 "samples/mercury/pull-2")
(def ^:private fake "components/mercury/test/ownpurse/mercury/fake-mercury-cli.sh")
(def ^:private day-1 (Instant/parse "2026-03-20T17:00:00Z"))
(def ^:private day-2 (Instant/parse "2026-04-02T09:00:00Z"))

(defn- txn-id [n] (str "4d0c5e11-0000-4000-8000-000000000" n))

(defn- with-node [f]
  (let [node (store/start {:mode :in-memory})]
    (try (f node) (finally (store/stop node)))))

(defn- temp-log []
  (doto (File/createTempFile "fake-mercury-cli" ".log") .deleteOnExit))

(defn- cli [dir code log]
  ["bash" fake dir (str code) (str log)])

(defn- pull! [node dir now]
  (let [log (temp-log)]
    {:counts (mercury/pull! node {:cli (cli dir 0 log) :alias "tallowmere" :now now})
     :calls (rest (str/split-lines (slurp log)))}))

(defn- error [f]
  (try (f) nil (catch clojure.lang.ExceptionInfo e (ex-data e))))

(deftest rows-test
  (let [answer (json/parse (io/file pull-1 "transactions.json"))
        {rows :mercury/transactions} (mercury/rows org "transactions" answer day-1)
        by-id (into {} (map (juxt :id identity)) rows)
        wire (by-id (txn-id 102))
        card (by-id (txn-id 103))]
    (is (= 8 (count rows)))
    (testing "the row id comes from Mercury's ids, and the org is a column"
      (is (= (str "mercury:" org ":transaction:" (txn-id 102)) (:xt/id wire)))
      (is (= org (:org wire))))
    (testing "fields keep Mercury's names in kebab case; amounts stay exact, whole ones become decimals"
      (is (= -2318.40M (:amount wire)))
      (is (= 2 (.scale ^BigDecimal (:amount wire))))
      (is (= {:converted-from-currency "USD" :converted-to-currency "EUR" :converted-from-amount 2318.40M
              :converted-to-amount 2000.00M :exchange-rate 0.8627M :fee-amount 11.59M :fee-percentage 0.5M}
             (:currency-exchange-info wire)))
      (is (= -412M (:amount (by-id (txn-id 104)))))
      (is (decimal? (:amount (by-id (txn-id 104))))))
    (testing "times become instants; nils are dropped"
      (is (= (Instant/parse "2026-03-05T08:30:00Z") (:posted-at wire)))
      (is (not (contains? wire :note))))
    (testing "signed links are not kept"
      (is (= [{:file-name "copper-pot-2026-03-12.jpg" :attachment-type "receipt"}] (:attachments card))))
    (testing "a transaction is valid from the time its status says"
      (is (= (:posted-at wire) (:xt/valid-from wire)) "sent: posted")
      (is (= (Instant/parse "2026-03-19T11:20:00Z") (:xt/valid-from (by-id (txn-id 104)))) "pending: created")
      (is (= (Instant/parse "2026-03-10T13:00:00Z") (:xt/valid-from (by-id (txn-id 108)))) "failed: failed"))
    (testing "never later than the fetch"
      (let [early (Instant/parse "2026-03-03T00:00:00Z")]
        (is (= early (:xt/valid-from (some #(when (= (txn-id 102) (:id %)) %)
                                           (:mercury/transactions (mercury/rows org "transactions" answer early))))))))))

(deftest pull-test
  (with-node
    (fn [node]
      (let [{:keys [counts calls]} (pull! node pull-1 day-1)]
        (testing "the first pull keeps every item and asks for every transaction"
          (is (= {"organization" {:items 1 :written 1} "accounts" {:items 2 :written 2} "credit" {:items 1 :written 1}
                  "transactions" {:items 8 :written 8} "statements" {:items 1 :written 1}
                  "credit-statements" {:items 1 :written 1}}
                 counts))
          (is (some #{"get transactions --org tallowmere --all --format json"} calls))
          (is (every? #(str/includes? % "--format json") calls)))
        (testing "deposit and credit accounts, archived ones too, in their own tables"
          (is (= #{["Operating" "active"] [nil "archived"]}
                 (set (map (juxt :nickname :status) (store/q node "SELECT nickname, status FROM mercury.accounts")))))
          (is (= [-2349.80M] (map :current-balance (store/q node "SELECT current_balance FROM mercury.credit_accounts")))))
        (testing "the org, its deposit accounts and its credit card accounts are source accounts"
          (is (= [["org" "Tallowmere Coffee Roasting Co." "active"]
                  ["bank" "Mercury Savings ••3304" "archived"]
                  ["bank" "Operating (Mercury Checking ••2210)" "active"]
                  ["credit-card" "Credit card" "active"]]
                 (map (juxt :kind :name :status)
                      (store/q node "SELECT kind, name, status FROM source_accounts
                                     ORDER BY CASE kind WHEN 'org' THEN 0 WHEN 'bank' THEN 1 ELSE 2 END, name")))))
        (testing "pending and sent transactions are lines, on their deposit or credit card account; a failed one is not"
          (is (= [["account:4d0c5e11-0000-4000-8000-0000000000a1" "2026-03-02" 12500.0M "posted"]
                  ["account:4d0c5e11-0000-4000-8000-0000000000a1" "2026-03-05" -2318.4M "posted"]
                  ["account:4d0c5e11-0000-4000-8000-0000000000a1" "2026-03-15" -2000.0M "posted"]
                  ["account:4d0c5e11-0000-4000-8000-0000000000a1" "2026-03-20" -1200.0M "pending"]
                  ["credit-account:4d0c5e11-0000-4000-8000-0000000000c1" "2026-03-13" -86.4M "posted"]
                  ["credit-account:4d0c5e11-0000-4000-8000-0000000000c1" "2026-03-15" 2000.0M "posted"]
                  ["credit-account:4d0c5e11-0000-4000-8000-0000000000c1" "2026-03-19" -412M "pending"]]
                 (map (juxt #(subs (:account %) (count (str "mercury:" org ":"))) (comp str :date) :amount :status)
                      (store/q node "SELECT account, date, amount, status FROM bank_lines ORDER BY account, date")))))
        (testing "statements name their account"
          (is (= ["4d0c5e11-0000-4000-8000-0000000000a1"]
                 (map :account (store/q node "SELECT account FROM mercury.statements"))))))
      (testing "a second pull of the same answers writes nothing, and asks only for recent and pending transactions"
        (let [{:keys [counts calls]} (pull! node pull-1 day-1)]
          (is (every? zero? (map :written (vals counts))))
          (is (some #{"get transactions --org tallowmere --all --format json --start 2026-01-19"} calls))
          (is (some #{"get transactions --org tallowmere --all --format json --status pending"} calls))))
      (testing "a later pull writes only what changed: a pending card charge posted, a pending payment cancelled, a new charge, balances, the March statements; a new signed link is no change"
        (let [{:keys [counts]} (pull! node pull-2 day-2)]
          (is (= {"organization" {:items 1 :written 0} "accounts" {:items 2 :written 1} "credit" {:items 1 :written 1}
                  "transactions" {:items 9 :written 3} "statements" {:items 2 :written 1}
                  "credit-statements" {:items 2 :written 1}}
                 counts))))
      (testing "pending then posted is two versions, each valid from its own time"
        (is (= [["pending" (Instant/parse "2026-03-19T11:20:00Z")] ["sent" (Instant/parse "2026-03-22T06:00:00Z")]]
               (map (juxt :status #(.toInstant ^java.time.ZonedDateTime (:xt/valid-from %)))
                    (store/q node [(str "SELECT status, _valid_from FROM mercury.transactions FOR ALL VALID_TIME "
                                        "WHERE _id = ? ORDER BY _valid_from")
                                   (str "mercury:" org ":transaction:" (txn-id 104))])))))
      (testing "after pull-2, the cancelled payment's line has ended, its history kept; the posted charge moved to
                the day it posted; the new charge is a line"
        (let [line #(str "line:mercury:" org ":transaction:" (txn-id %))]
          (is (= [] (store/q node ["SELECT _id FROM bank_lines WHERE _id = ?" (line 105)])))
          (is (= 1 (count (store/q node ["SELECT _id FROM bank_lines FOR ALL VALID_TIME WHERE _id = ?" (line 105)]))))
          (is (= [{:date (LocalDate/parse "2026-03-22") :status "posted"}]
                 (store/q node ["SELECT date, status FROM bank_lines WHERE _id = ?" (line 104)])))
          (is (= 7 (count (store/q node "SELECT _id FROM bank_lines"))))))
      (testing "a cancelled payment is valid from the fetch that saw it"
        (is (= day-2 (some-> (store/q node ["SELECT _valid_from FROM mercury.transactions WHERE _id = ?"
                                            (str "mercury:" org ":transaction:" (txn-id 105))])
                             first :xt/valid-from (#(.toInstant ^java.time.ZonedDateTime %)))))))))

(deftest check-number-test
  (with-node
    (fn [node]
      (testing "a check's number goes to its line"
        (mercury/keep! node org "transactions"
                       {:transactions [{:id (txn-id 120) :accountId "4d0c5e11-0000-4000-8000-0000000000a1"
                                        :amount -850.00M :status "sent" :kind "checkDeposit"
                                        :createdAt "2026-03-20T10:00:00Z" :postedAt "2026-03-24T10:00:00Z"
                                        :counterpartyName "Brindle & Oake Supply" :checkNumber "001043"}]}
                       {:now day-2})
        (is (= [{:check-number "1043" :date (LocalDate/parse "2026-03-24")}]
               (store/q node "SELECT check_number, date FROM bank_lines")))))))

(deftest refusals-test
  (with-node
    (fn [node]
      (testing "a personal org, whose /credit Mercury refuses, has no credit accounts"
        (let [counts (mercury/pull! node {:cli (cli pull-1 "credit:4" (temp-log)) :alias "tallowmere" :now day-1})]
          (is (= {:items 0 :written 0} (counts "credit")))
          (is (= {:items 0 :written 0} (counts "credit-statements")))))
      (testing "any other failure stops the pull with mercury-cli's exit as the error's type"
        (doseq [[code type] mercury/exit-types]
          (is (= type (:type (error #(mercury/pull! node {:cli (cli pull-1 code (temp-log)) :alias "tallowmere"})))))))
      (testing "a missing program says how to fix it"
        (is (= :mercury/not-found
               (:type (error #(mercury/pull! node {:cli "/nowhere/mercury-cli" :alias "tallowmere"})))))))))

(deftest read-orgs-test
  (let [f (doto (File/createTempFile "local" ".edn") .deleteOnExit)]
    (testing "orgs by alias, with the window for every org unless an org names its own"
      (spit f (pr-str {:mercury {:orgs [{:alias "tallowmere" :name "Tallowmere"} {:alias "home" :window-days 30}]
                                 :window-days 90}}))
      (is (= [{:alias "tallowmere" :name "Tallowmere" :window-days 90} {:alias "home" :window-days 30}]
             (mercury/read-orgs (str f)))))
    (testing "a config with no Mercury orgs names none"
      (spit f (pr-str {:xero {:orgs []}}))
      (is (= [] (mercury/read-orgs (str f)))))
    (testing "a missing file says how to make one"
      (is (= :ownpurse/no-local-config (:type (error #(mercury/read-orgs "/nowhere.edn"))))))))
