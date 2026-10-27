(ns ownpurse.config.core
  (:require
   [aero.core :as aero]
   [clojure.java.io :as io]))

(def ^:private profiles #{:dev :test :prod})

(defn- with-secrets
  "Takes each secret the config left empty from the secrets file, then drops the file's contents, so the secrets
  travel no further than the keys that use them. A missing file holds no secrets."
  [{:keys [secrets] :as config}]
  (let [secrets (if (:aero/missing-include secrets) {} secrets)]
    (-> config
        (update-in [:jev :api-key] #(or % (get-in secrets [:typesafe :api-key])))
        (dissoc :secrets))))

(defn read-config
  "Reads config/config.edn from the classpath for `profile`."
  [profile]
  (when-not (profiles profile)
    (throw (ex-info (str "Unknown profile " profile ". Use one of :dev, :test or :prod.")
                    {:type :ownpurse/unknown-profile :profile profile :profiles profiles})))
  (with-secrets (aero/read-config (io/resource "config/config.edn") {:profile profile})))
