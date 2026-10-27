(ns ownpurse.config.interface-test
  (:require
   [clojure.test :refer [deftest is testing]]
   [ownpurse.config.core :as core]
   [ownpurse.config.interface :as config]))

(deftest read-config-test
  (testing "tests get an in-memory store, no SQL port and a random HTTP port"
    (let [{:keys [store http]} (config/read-config :test)]
      (is (= :in-memory (:mode store)))
      (is (nil? (:sql store)))
      (is (= 0 (:port http)))))
  (testing "everything listens on loopback only"
    (doseq [profile [:dev :prod]
            :let [{:keys [store http]} (config/read-config profile)]]
      (is (= "127.0.0.1" (:host http) (get-in store [:sql :host])))))
  (testing "tests have no Jev key, and no config carries the secrets file itself"
    (is (nil? (get-in (config/read-config :test) [:jev :api-key])))
    (doseq [profile [:test :dev :prod]]
      (is (not (contains? (config/read-config profile) :secrets)))))
  (testing "a Keychain service that holds nothing reads as no secret"
    (is (nil? (core/keychain "ownpurse-test-no-such-service"))))
  (testing "an unknown profile says which ones exist"
    (is (= :ownpurse/unknown-profile
           (try (config/read-config :staging)
                (catch clojure.lang.ExceptionInfo e (:type (ex-data e))))))))
