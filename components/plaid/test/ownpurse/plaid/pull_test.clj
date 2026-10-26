(ns ownpurse.plaid.pull-test
  (:require
   [clojure.string :as str]
   [clojure.test :refer [deftest is testing]]
   [ownpurse.plaid.interface :as plaid]
   [ownpurse.store.interface :as store])
  (:import
   (java.io File)
   (java.time Instant LocalDate ZonedDateTime)))

(def ^:private item "item-ostley-0001")
(def ^:private pull-1 "samples/plaid/pull-1")
(def ^:private pull-2 "samples/plaid/pull-2")
(def ^:private fake "components/plaid/test/ownpurse/plaid/fake-plaid-reader.sh")
(def ^:private day-1 (Instant/parse "2026-03-20T17:00:00Z"))
(def ^:private day-2 (Instant/parse "2026-04-02T09:00:00Z"))

(defn- txn [n] (format "plaid:%s:transaction:txn-ostley-%04d" item n))
(defn- line [n] (str "line:" (txn n)))

(defn- with-node [f]
  (let [node (store/start {:mode :in-memory})]
    (try (f node) (finally (store/stop node)))))

(defn- temp-log []
  (doto (File/createTempFile "fake-plaid-reader" ".log") .deleteOnExit))

(defn- cli [dir code log]
  ["bash" fake dir (str code) (str log)])

(defn- pull! [node dir now]
  (let [log (temp-log)]
    {:counts (plaid/pull! node {:cli (cli dir 0 log) :alias "ostley" :now now})
     :calls (rest (str/split-lines (slurp log)))}))

(defn- error [f]
  (try (f) nil (catch clojure.lang.ExceptionInfo e (ex-data e))))

(defn- lines [node]
  (map (juxt :account (comp str :date) :amount :status)
       (store/q node "SELECT account, date, amount, status FROM bank_lines ORDER BY date, _id")))

(deftest pull-test
  (with-node
    (fn [node]
      (let [{:keys [counts calls]} (pull! node pull-1 day-1)]
        (testing "the first pull syncs from the start, as JSON, and keeps the item, its accounts and its transactions"
          (is (= ["item --item ostley --format json" "sync --item ostley --all --format json"] calls))
          (is (= {"item" {:items 1 :written 1} "accounts" {:items 2 :written 2} "transactions" {:items 6 :written 6}}
                 counts)))
        (testing "the cursor is kept in the record"
          (is (= "cur-0001" (plaid/cursor node item))))
        (testing "fields keep Plaid's names in kebab case; amounts are decimals with two places"
          (let [[t] (store/q node ["SELECT * FROM plaid.transactions WHERE _id = ?" (txn 1)])]
            (is (= -2500.00M (:amount t)))
            (is (= 2 (.scale ^BigDecimal (:amount t))))
            (is (= "PAYROLL DEPOSIT" (:original-description t)))
            (is (= (LocalDate/parse "2026-03-02") (:date t)))
            (is (= item (:item t))))
          (is (= 86.40M (:amount (first (store/q node ["SELECT amount FROM plaid.transactions WHERE _id = ?"
                                                       (txn 3)]))))))
        (testing "an added transaction is valid from its date"
          (is (= (Instant/parse "2026-03-12T00:00:00Z")
                 (.toInstant ^ZonedDateTime (:xt/valid-from (first (store/q node ["SELECT _valid_from FROM plaid.transactions WHERE _id = ?" (txn 3)])))))))
        (testing "the deposit and credit card accounts are source accounts; the item is their org"
          (is (= [["bank" "Ostley Business Checking ••4410" item "USD"] ["credit-card" "Card ••8821" item "USD"]]
                 (map (juxt :kind :name :org :currency)
                      (store/q node "SELECT kind, name, org, currency FROM source_accounts ORDER BY kind")))))
        (testing "every transaction is a line, money in positive, pending ones too"
          (let [chk (str "plaid:" item ":account:acct-ostley-chk")
                card (str "plaid:" item ":account:acct-ostley-card")]
            (is (= [[chk "2026-03-02" 2500.00M "posted"]
                    [chk "2026-03-05" -850.50M "posted"]
                    [card "2026-03-12" -86.40M "posted"]
                    [chk "2026-03-15" -500.00M "posted"]
                    [card "2026-03-15" 500.00M "posted"]
                    [card "2026-03-19" -41.25M "pending"]]
                   (lines node)))))
        (testing "a check's number, the merchant and Plaid's original description go to the line"
          (is (= [{:check-number "1043" :description "CHECK 1043"}]
                 (store/q node ["SELECT check_number, description FROM bank_lines WHERE _id = ?" (line 2)])))
          (is (= [{:counterparty "Copper Pot"}]
                 (store/q node ["SELECT counterparty FROM bank_lines WHERE _id = ?" (line 3)])))))
      (testing "the next pull syncs from the cursor kept; the pending charge posted under a new id, a merchant
                changed, a new charge"
        (let [{:keys [counts calls]} (pull! node pull-2 day-2)]
          (is (some #{"sync --item ostley --all --format json --cursor cur-0001"} calls))
          (is (= {"item" {:items 1 :written 0} "accounts" {:items 2 :written 1} "transactions" {:items 4 :written 4}}
                 counts))
          (is (= "cur-0002" (plaid/cursor node item)))))
      (testing "the removed transaction's row stays, as a new version marked removed; its earlier version is kept"
        (is (= [{:removed true :amount 41.25M}]
               (store/q node ["SELECT removed, amount FROM plaid.transactions WHERE _id = ?" (txn 4)])))
        (is (= 2 (count (store/q node ["SELECT _id FROM plaid.transactions FOR ALL VALID_TIME WHERE _id = ?"
                                       (txn 4)])))))
      (testing "its line has ended, its history kept; the posted charge and the new one are lines"
        (is (= [] (store/q node ["SELECT _id FROM bank_lines WHERE _id = ?" (line 4)])))
        (is (= 1 (count (store/q node ["SELECT _id FROM bank_lines FOR ALL VALID_TIME WHERE _id = ?" (line 4)]))))
        (is (= 7 (count (lines node))))
        (is (= [{:date (LocalDate/parse "2026-03-21") :amount -41.25M :status "posted"}]
               (store/q node ["SELECT date, amount, status FROM bank_lines WHERE _id = ?" (line 7)]))))
      (testing "a modified transaction is a new version of its row and its line"
        (is (= [{:counterparty "Copper Pot Café"}]
               (store/q node ["SELECT counterparty FROM bank_lines WHERE _id = ?" (line 3)]))))
      (testing "a pull with nothing new writes nothing, and keeps the cursor"
        (let [{:keys [counts]} (pull! node pull-2 day-2)]
          (is (= {:items 0 :written 0} (counts "transactions")))
          (is (= 0 (:written (counts "accounts"))))
          (is (= "cur-0002" (plaid/cursor node item))))))))

(deftest failure-test
  (with-node
    (fn [node]
      (testing "a sync that fails keeps nothing of the item, so the next pull starts from the same cursor"
        (is (= :plaid/login-needed
               (:type (error #(plaid/pull! node {:cli (cli pull-1 "sync:2" (temp-log)) :alias "ostley"})))))
        (is (nil? (plaid/cursor node item)))
        (is (not (store/table-exists? node :plaid/transactions))))
      (testing "each exit code is the error's type"
        (doseq [[code type] plaid/exit-types]
          (is (= type (:type (error #(plaid/pull! node {:cli (cli pull-1 code (temp-log)) :alias "ostley"})))))))
      (testing "a missing program says how to fix it"
        (is (= :plaid/not-found (:type (error #(plaid/pull! node {:cli "/nowhere/plaid-reader" :alias "ostley"})))))))))

(deftest read-items-test
  (let [f (doto (File/createTempFile "local" ".edn") .deleteOnExit)]
    (testing "items by alias"
      (spit f (pr-str {:plaid {:items [{:alias "ostley" :name "Ostley Savings"}]}}))
      (is (= [{:alias "ostley" :name "Ostley Savings"}] (plaid/read-items (str f)))))
    (testing "a config with no Plaid items names none"
      (spit f (pr-str {:xero {:orgs []}}))
      (is (= [] (plaid/read-items (str f)))))
    (testing "an item without an alias"
      (spit f (pr-str {:plaid {:items [{:name "Ostley"}]}}))
      (is (= :ownpurse/no-local-config (:type (error #(plaid/read-items (str f)))))))
    (testing "a missing file says how to make one"
      (is (= :ownpurse/no-local-config (:type (error #(plaid/read-items "/nowhere.edn"))))))))
