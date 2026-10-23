(ns ownpurse.api.system-test
  (:require
   [clojure.test :refer [deftest is]]
   [donut.system :as ds]
   [ownpurse.api.json :as json]
   [ownpurse.api.system :as system]
   [ownpurse.config.interface :as config])
  (:import
   (java.net URI)
   (java.net.http HttpClient HttpRequest HttpResponse$BodyHandlers)))

(defn- get-json [port path]
  (let [response (.send (HttpClient/newHttpClient)
                        (.build (HttpRequest/newBuilder (URI. (str "http://127.0.0.1:" port path))))
                        (HttpResponse$BodyHandlers/ofString))]
    {:status (.statusCode response) :body (json/read-str (.body response))}))

(deftest system-test
  (let [running (ds/start (system/system (config/read-config :test)))]
    (try
      (is (= {:status 200 :body {:status "ok"}} (get-json (system/http-port running) "/health")))
      (is (nil? (system/sql-port running)))
      (finally (ds/stop running)))))
