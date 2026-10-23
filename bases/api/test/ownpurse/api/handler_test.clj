(ns ownpurse.api.handler-test
  (:require
   [clojure.test :refer [deftest is testing]]
   [ownpurse.api.handler :as handler]
   [ownpurse.json.interface :as json]
   [ownpurse.store.interface :as store]))

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

(deftest openapi-test
  (let [{:keys [status body]} (request (handler/handler {:node nil}) :get "/openapi.json")]
    (is (= 200 status))
    (is (contains? (:paths body) (keyword "/health")))))
