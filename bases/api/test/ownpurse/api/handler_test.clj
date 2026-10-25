(ns ownpurse.api.handler-test
  (:require
   [clojure.test :refer [deftest is testing]]
   [ownpurse.api.handler :as handler]
   [ownpurse.json.interface :as json]
   [ownpurse.store.interface :as store])
  (:import
   (java.io File)))

(defn- request [app method uri]
  (let [{:keys [status body]} (app {:request-method method :uri uri :headers {"accept" "application/json"}})]
    {:status status :body (json/parse (if (string? body) body (slurp body)))}))

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
                                   "samples/mercury/pull-1" "0" (str log)]}})]
    (try
      (testing "without a config naming the orgs, the pull says how to write one"
        (let [{:keys [status body]} (request (handler/handler (sources "/nowhere.edn")) :post "/pull")]
          (is (= 409 status))
          (is (= "ownpurse/no-local-config" (get-in body [:error :type]))))
        (spit orgs (pr-str {:xero {:orgs []}}))
        (is (= 409 (:status (request (handler/handler (sources (str orgs))) :post "/pull")))))
      (testing "with one, every org of every source is pulled and the counts come back"
        (spit orgs (pr-str {:xero {:orgs [{:tenant "c0ffee00-0000-4000-8000-000000000000" :name "Tallowmere"}]}
                            :mercury {:orgs [{:alias "tallowmere" :name "Tallowmere"}]}}))
        (let [{:keys [status body]} (request (handler/handler (sources (str orgs))) :post "/pull")]
          (is (= 200 status))
          (is (= {:items 7 :written 7} (get-in body [:pulled :xero :Tallowmere :Invoices])))
          (is (= {:items 8 :written 8} (get-in body [:pulled :mercury :Tallowmere :transactions])))))
      (finally (store/stop node)))))

(deftest openapi-test
  (let [{:keys [status body]} (request (handler/handler {:node nil}) :get "/openapi.json")]
    (is (= 200 status))
    (is (contains? (:paths body) (keyword "/health")))
    (is (contains? (:paths body) (keyword "/pull")))))
