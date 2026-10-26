(ns ownpurse.api.handler-test
  (:require
   [clojure.java.io :as io]
   [clojure.string :as str]
   [clojure.test :refer [deftest is testing]]
   [ownpurse.api.handler :as handler]
   [ownpurse.json.interface :as json]
   [ownpurse.ledger.interface :as ledger]
   [ownpurse.store.interface :as store])
  (:import
   (java.io File)))

(defn- request
  ([app method uri] (request app method uri nil))
  ([app method uri body]
   (let [[path query] (str/split uri #"\?" 2)
         {:keys [status body]} (app (cond-> {:request-method method :uri path :query-string query
                                             :headers {"accept" "application/json"
                                                       "content-type" "application/json"}}
                                      body (assoc :body (java.io.ByteArrayInputStream.
                                                         (.getBytes ^String (json/write-str body) "UTF-8")))))]
     {:status status :body (json/parse (if (string? body) body (slurp body)))})))

(deftest health-test
  (let [node (store/start {:mode :in-memory})]
    (try
      (is (= {:status 200 :body {:status "ok"}}
             (request (handler/handler {:node node}) :get "/health")))
      (finally (store/stop node))))
  (testing "a closed record is unavailable"
    (let [node (doto (store/start {:mode :in-memory}) store/stop)]
      (is (= {:status 503 :body {:status "unavailable"}}
             (request (handler/handler {:node node}) :get "/health"))))))

(deftest backup-test
  (let [tmp (.toFile (java.nio.file.Files/createTempDirectory "ownpurse-api-backup"
                                                              (make-array java.nio.file.attribute.FileAttribute 0)))
        store-config {:mode :disk :dir (str (io/file tmp "record"))}
        node (store/start store-config)
        app (handler/handler {:node node :store store-config})]
    (try
      (testing "health says nothing of snapshots until there is one"
        (is (= {:status "ok"} (:body (request app :get "/health")))))
      (testing "a folder given as a relative path is refused"
        (is (= {:status 400 :type "ownpurse/invalid"}
               (let [{:keys [status body]} (request app :post "/backup" {:dir "backups"})]
                 {:status status :type (get-in body [:error :type])}))))
      (store/put! node {:probes [{:xt/id "p1" :amount 1.00M}]} {:source "test" :why "fixtures"})
      (let [{:keys [status body]} (request app :post "/backup" {:dir (str (io/file tmp "backups"))})
            snapshot (:snapshot body)]
        (testing "a snapshot of the running record, with what it holds"
          (is (= 200 status))
          (is (= {:epoch 0 :transactions 1} (select-keys snapshot [:epoch :transactions])))
          (is (.exists (io/file (:dir snapshot) "snapshot.edn"))))
        (testing "health then says when the last snapshot was taken"
          (is (= (select-keys snapshot [:dir :taken-at :block :transactions])
                 (:last-snapshot (:body (request app :get "/health")))))))
      (finally
        (store/stop node)
        (run! io/delete-file (reverse (file-seq tmp)))))))

(deftest errors-test
  (let [app (handler/handler {:node nil})]
    (is (= {:status 404 :body {:error {:type "not-found" :message "No such endpoint. GET /openapi.json lists them."}}}
           (request app :get "/nowhere")))
    (is (= 405 (:status (request app :post "/health"))))))

(deftest pull-test
  (let [node (store/start {:mode :in-memory})
        orgs (doto (File/createTempFile "orgs" ".edn") .deleteOnExit)
        log (doto (File/createTempFile "fake-cli" ".log") .deleteOnExit)
        sources (fn [local]
                  {:node node
                   :local local
                   :xero {:cli ["bash" "components/xero/test/ownpurse/xero/fake-xero-cli.sh" "samples/xero/pull-1"
                                "0" (str log)]}
                   :mercury {:cli ["bash" "components/mercury/test/ownpurse/mercury/fake-mercury-cli.sh"
                                   "samples/mercury/pull-1" "0" (str log)]}
                   :plaid {:cli ["bash" "components/plaid/test/ownpurse/plaid/fake-plaid-reader.sh"
                                 "samples/plaid/pull-1" "0" (str log)]}})]
    (try
      (testing "without a config naming the orgs, the pull says how to write one"
        (let [{:keys [status body]} (request (handler/handler (sources "/nowhere.edn")) :post "/pull")]
          (is (= 409 status))
          (is (= "ownpurse/no-local-config" (get-in body [:error :type]))))
        (spit orgs (pr-str {:xero {:orgs []}}))
        (is (= 409 (:status (request (handler/handler (sources (str orgs))) :post "/pull")))))
      (testing "with one, every org of every source is pulled and the counts come back"
        (spit orgs (pr-str {:xero {:orgs [{:tenant "c0ffee00-0000-4000-8000-000000000000" :name "Tallowmere"}]}
                            :mercury {:orgs [{:alias "tallowmere" :name "Tallowmere"}]}
                            :plaid {:items [{:alias "ostley" :name "Ostley"}]}}))
        (let [{:keys [status body]} (request (handler/handler (sources (str orgs))) :post "/pull")]
          (is (= 200 status))
          (is (= {:items 7 :written 7} (get-in body [:pulled :xero :Tallowmere :Invoices])))
          (is (= {:items 14 :written 14 :waiting 0} (get-in body [:pulled :xero :Tallowmere :History])))
          (is (= {:items 8 :written 8} (get-in body [:pulled :mercury :Tallowmere :transactions])))
          (is (= {:items 6 :written 6} (get-in body [:pulled :plaid :Ostley :transactions])))))
      (finally (store/stop node)))))

(deftest entities-test
  (let [node (store/start {:mode :in-memory})
        orgs (doto (File/createTempFile "orgs" ".edn") .deleteOnExit)
        log (doto (File/createTempFile "fake-cli" ".log") .deleteOnExit)
        tenant "c0ffee00-0000-4000-8000-000000000000"
        app (handler/handler
             {:node node
              :local (str orgs)
              :xero {:cli ["bash" "components/xero/test/ownpurse/xero/fake-xero-cli.sh" "samples/xero/pull-1"
                           "0" (str log)]}
              :mercury {:cli ["bash" "components/mercury/test/ownpurse/mercury/fake-mercury-cli.sh"
                              "samples/mercury/pull-1" "0" (str log)]}})
        xero-org (str "xero:" tenant ":organisation:c0ffee00-0000-4000-8000-0000000000a1")
        xero-1010 (str "xero:" tenant ":account:c0ffee01-0000-4000-8000-000000000001")
        operating "mercury:4d0c5e11-0000-4000-8000-000000000001:account:4d0c5e11-0000-4000-8000-0000000000a1"]
    (try
      (spit orgs (pr-str {:xero {:orgs [{:tenant tenant :name "Tallowmere"}]}
                          :mercury {:orgs [{:alias "tallowmere" :name "Tallowmere"}]}}))
      (is (= 200 (:status (request app :post "/pull"))))
      (testing "after a pull, every source's accounts are listed, none linked yet"
        (let [{:keys [status body]} (request app :get "/source-accounts?unlinked=true")]
          (is (= 200 status))
          (is (= {"mercury" 4 "xero" 3} (frequencies (map :source (:source-accounts body)))))
          (is (some #(= xero-org (:id %)) (:source-accounts body)))))
      (testing "an entity is made by its slug"
        (is (= {:status 200 :body {:entity {:id "tallowmere" :name "Tallowmere" :kind "business"}}}
               (request app :put "/entities/tallowmere" {:name "Tallowmere" :kind "business"})))
        (is (= 400 (:status (request app :put "/entities/Not%20a%20slug" {:name "x" :kind "business"})))))
      (testing "its Xero org is its system of record, and a Mercury account is evidence for Xero's 1010"
        (is (= {:status 200 :body {:link {:entity "tallowmere" :account xero-org :role "system-of-record"
                                          :from "2025-01-01"}}}
               (request app :put "/links" {:entity "tallowmere" :account xero-org :role "system-of-record"
                                           :from "2025-01-01"})))
        (is (= 200 (:status (request app :put "/links" {:entity "tallowmere" :account operating :role "evidence"
                                                        :evidence-for xero-1010 :from "2025-01-01"}))))
        (is (= [[xero-org nil] [operating xero-1010]]
               (->> (request app :get "/entities") :body :entities first :links
                    (map (juxt :account :evidence-for)) (sort-by first) reverse))))
      (testing "a broken rule is a structured error that says how to fix it"
        (let [{:keys [status body]} (request app :put "/links" {:entity "tallowmere" :account xero-1010
                                                                :role "system-of-record"})]
          (is (= 400 status))
          (is (= "ownpurse/invalid-link" (get-in body [:error :type]))))
        (is (= 404 (:status (request app :put "/links" {:entity "nobody" :account operating :role "evidence"})))))
      (testing "the compare lists, for March, what each side has that the other has not"
        (let [{:keys [status body]} (request app :get (str "/compare?account=" operating
                                                           "&from=2026-03-01&to=2026-03-31"))
              c (first (:compares body))]
          (is (= 200 status))
          (is (= [operating xero-1010 0] ((juxt :account :evidence-for :matched) c)))
          (is (= [3 8 1] (map (comp count c) [:missing-from-book :missing-from-evidence :pending])))
          (is (= {:evidence "8181.60" :book "-11715.24"} (:totals c)))
          (is (= {:manual 0 :check 0 :amount 0} (:matched-by c)))))
      (testing "a match of two lines of different amounts is refused, saying why"
        (let [{:keys [status body]}
              (request app :put "/matches"
                       {:evidence "mercury:4d0c5e11-0000-4000-8000-000000000001:transaction:4d0c5e11-0000-4000-8000-000000000101"
                        :book (str "xero:" tenant ":payment:c0ffee04-0000-4000-8000-000000000001")})]
          (is (= 400 status))
          (is (= "ownpurse/invalid-match" (get-in body [:error :type])))))
      (testing "a link ends from a day"
        (is (= {:status 200 :body {:ended {:entity "tallowmere" :account operating :role "evidence"
                                           :evidence-for xero-1010 :from "2025-01-01" :until "2025-07-01"}}}
               (request app :delete (str "/links?account=" operating "&on=2025-07-01"))))
        (is (= 404 (:status (request app :delete (str "/links?account=" operating))))))
      (finally (store/stop node)))))

(deftest openapi-test
  (let [{:keys [status body]} (request (handler/handler {:node nil}) :get "/openapi.json")]
    (is (= 200 status))
    (is (contains? (:paths body) (keyword "/health")))
    (is (contains? (:paths body) (keyword "/pull")))
    (is (contains? (:paths body) (keyword "/links")))
    (is (contains? (:paths body) (keyword "/compare")))
    (is (contains? (:paths body) (keyword "/matches")))))

(deftest trial-balance-test
  (let [node (store/start {:mode :in-memory})
        app (handler/handler {:node node})
        tenant "c0ffee00-0000-4000-8000-000000000000"]
    (try
      (testing "an org ownpurse does not hold is not found, and the error says to pull it"
        (let [{:keys [status body]} (request app :get (str "/trial-balance?org=" tenant "&date=2026-03-31"))]
          (is (= 404 status))
          (is (= "ownpurse/not-found" (get-in body [:error :type])))))
      (testing "an opening that is neither is refused"
        (is (= 400 (:status (request app :get (str "/trial-balance?org=" tenant "&date=2026-03-31&opening=x"))))))
      (testing "the route is in the OpenAPI document"
        (is (contains? (:paths (:body (request app :get "/openapi.json"))) (keyword "/trial-balance"))))
      (finally (store/stop node)))))

(deftest close-test
  (let [node (store/start {:mode :in-memory})
        app (handler/handler {:node node})
        item {:title "January rent paid in December" :status "accountant decision" :book "book:test:tallowmere"
              :amount "1200.00" :group "G-1"
              :journals [{:ref "J1" :date "2025-12-31" :narration "January rent is prepaid"
                          :postings [{:account "1400" :amount "1200.00" :commodity "USD"}
                                     {:account "6200" :amount "-1200.00" :commodity "USD"}]}]}]
    (try
      (testing "an item is put with its journals, amounts as decimal strings"
        (let [{:keys [status body]} (request app :put "/close/items/C-01" item)]
          (is (= 200 status))
          (is (= ["1200.00" "-1200.00"] (map :amount (get-in body [:item :journals 0 :postings]))))
          (is (= ["2025-12-31" false] [(get-in body [:item :journals 0 :date]) (get-in body [:item :approved])]))))
      (testing "a journal that does not balance is refused"
        (is (= 400 (:status (request app :put "/close/items/C-02"
                                     (assoc-in item [:journals 0 :postings 0 :amount] "1200.01"))))))
      (testing "a group gathers the items that name it"
        (is (= ["C-01"] (get-in (request app :put "/close/groups/G-1" {:title "Prepayments"}) [:body :group :items]))))
      (testing "a sealed package is approved with its hash; its items are then approved"
        (let [{:keys [hash]} (:package (:body (request app :post "/close/packages"
                                                       {:ref "P-1" :items ["C-01"] :coverage {:mapped 1}})))]
          (is (= 409 (:status (request app :post "/close/packages/P-1/approval" {:hash "x" :by "owner"}))))
          (is (= "owner" (get-in (request app :post "/close/packages/P-1/approval" {:hash hash :by "owner"})
                                 [:body :package :approval :by])))
          (is (= [true] (map :approved (:items (:body (request app :get "/close/items?book=book:test:tallowmere"))))))))
      (testing "posting records the reference"
        (is (= "MJ-1" (get-in (request app :post "/close/items/C-01/journals/J1/posted" {:reference "MJ-1"})
                              [:body :item :journals 0 :posted-reference]))))
      (testing "the routes are in the OpenAPI document"
        (is (every? (set (keys (:paths (:body (request app :get "/openapi.json")))))
                    (map keyword ["/close/items" "/close/packages" "/close/groups"]))))
      (finally (store/stop node)))))

(deftest intercompany-test
  (let [node (store/start {:mode :in-memory})
        app (handler/handler {:node node})
        entry (fn [book id date & postings]
                {:entry/id id :entry/book book :entry/date (java.time.LocalDate/parse date)
                 :entry/postings (vec (map-indexed (fn [i [account amount]]
                                                     {:posting/line i :posting/account account :posting/amount amount
                                                      :posting/commodity "USD"})
                                                   postings))})]
    (try
      (ledger/put-entries! node "book:a" [(entry "book:a" "a1" "2025-03-01" ["a:1800" 250.00M] ["a:1010" -250.00M])]
                           {:source "test" :why "fixture"})
      (ledger/put-entries! node "book:b" [(entry "book:b" "b1" "2025-03-02" ["b:1010" 250.00M] ["b:2800" -250.00M])]
                           {:source "test" :why "fixture"})
      (testing "a pair is put, amounts as decimal strings"
        (is (= 200 (:status (request app :put "/intercompany/pairs/loan"
                                     {:a {:book "book:a" :accounts ["a:1800"]} :b {:book "book:b" :accounts ["b:2800"]}
                                      :from "2025-01-01" :explained [{:ref "x" :amount "0.00"}]}))))
        (is (= 400 (:status (request app :put "/intercompany/pairs/loan" {:a {:book "book:a" :accounts []}
                                                                          :b {:book "book:b" :accounts ["b:2800"]}})))))
      (testing "the report reconciles it"
        (let [[pair] (:pairs (:body (request app :get "/intercompany?from=2025-01-01&to=2025-12-31")))]
          (is (= ["250.00" "-250.00" "0.00" true 1]
                 [(get-in pair [:a :balance]) (get-in pair [:b :balance]) (:difference pair) (:reconciled pair)
                  (:matched pair)]))))
      (finally (store/stop node)))))

(deftest payees-test
  (let [node (store/start {:mode :in-memory})
        orgs (doto (File/createTempFile "orgs" ".edn") .deleteOnExit)
        log (doto (File/createTempFile "fake-cli" ".log") .deleteOnExit)
        tenant "c0ffee00-0000-4000-8000-000000000000"
        app (handler/handler
             {:node node
              :local (str orgs)
              :xero {:cli ["bash" "components/xero/test/ownpurse/xero/fake-xero-cli.sh" "samples/xero/pull-1"
                           "0" (str log)]}})]
    (try
      (spit orgs (pr-str {:xero {:orgs [{:tenant tenant :name "Tallowmere"}]}}))
      (is (= 200 (:status (request app :post "/pull"))))
      (let [{:keys [status body]} (request app :get (str "/payees?org=" tenant "&from=2026-01-01&to=2026-12-31"
                                                         "&threshold=1"))
            {:keys [threshold payees]} (:payees body)]
        (testing "a Xero org's payees, by contact name, paid by bank, with the threshold given"
          (is (= 200 status))
          (is (= "1" threshold))
          (is (seq payees))
          (is (every? #(and (:name %) (not= (:name %) (:party %))) payees))
          (is (= {:bank "3215.47"} (:paid (first payees))))
          (is (every? :candidate (filter #(get-in % [:paid :bank]) payees))))
        (testing "a payee paid less than the threshold is no candidate"
          (let [{:keys [payees]} (:payees (:body (request app :get (str "/payees?org=" tenant
                                                                        "&from=2026-01-01&to=2026-12-31"))))]
            (is (= [true false] (map :candidate [(first payees) (last payees)])))
            (is (every? #(seq (:paid %)) payees)))))
      (testing "dates are required"
        (is (= 400 (:status (request app :get (str "/payees?org=" tenant))))))
      (finally (store/stop node)))))

(deftest export-test
  (let [node (store/start {:mode :in-memory})
        orgs (doto (File/createTempFile "orgs" ".edn") .deleteOnExit)
        log (doto (File/createTempFile "fake-cli" ".log") .deleteOnExit)
        dir (str (java.nio.file.Files/createTempDirectory "ownpurse-export"
                                                          (make-array java.nio.file.attribute.FileAttribute 0)))
        tenant "c0ffee00-0000-4000-8000-000000000000"
        app (handler/handler
             {:node node
              :local (str orgs)
              :xero {:cli ["bash" "components/xero/test/ownpurse/xero/fake-xero-cli.sh" "samples/xero/pull-1"
                           "0" (str log)]}})]
    (try
      (spit orgs (pr-str {:xero {:orgs [{:tenant tenant :name "Tallowmere"}]}}))
      (is (= 200 (:status (request app :post "/pull"))))
      (testing "an export of the org's book for the year writes the workbook and its CSV files"
        (let [{:keys [status body]} (request app :post "/export" {:org tenant :date "2026-02-28" :dir dir
                                                                  :opening "ledger"})]
          (is (= 200 status))
          (is (str/ends-with? (get-in body [:export :workbook]) ".xlsx"))
          (is (.isFile (io/file (get-in body [:export :workbook]))))
          (is (pos? (:rows (first (filter #(= "General ledger" (:name %)) (get-in body [:export :sheets]))))))))
      (testing "the general ledger links each document back to Xero"
        (let [gl (slurp (first (filter #(= "general-ledger.csv" (.getName ^File %)) (file-seq (io/file dir)))))]
          (is (str/includes? gl "https://go.xero.com/"))))
      (testing "an export never overwrites one; a relative folder is refused"
        (is (= 409 (:status (request app :post "/export" {:org tenant :date "2026-02-28" :dir dir :opening "ledger"}))))
        (is (= 400 (:status (request app :post "/export" {:org tenant :date "2026-02-28" :dir "relative"
                                                          :opening "ledger"})))))
      (finally
        (store/stop node)
        (doseq [^File f (reverse (file-seq (io/file dir)))] (.delete f))))))
