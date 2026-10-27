(ns ownpurse.config.core
  (:require
   [aero.core :as aero]
   [clojure.java.io :as io]
   [clojure.java.shell :as shell]
   [clojure.string :as str])
  (:import
   (java.io IOException)))

(def ^:private profiles #{:dev :test :prod})

(defn keychain
  "The password the macOS Keychain holds for `service` (a generic password), or nil when there is none, or no
  Keychain on this machine."
  [service]
  (try
    (let [{:keys [exit out]} (shell/sh "security" "find-generic-password" "-s" service "-w")]
      (when (zero? exit) (not-empty (str/trim out))))
    (catch IOException _ nil)))

;; #keychain "service" in config.edn reads a secret from the macOS Keychain.
(defmethod aero/reader 'keychain [_ _ service]
  (keychain service))

(defn- with-secrets
  "Takes each secret the config left empty from the secrets file, then drops the file's contents, so the secrets
  travel no further than the keys that use them. A missing file holds no secrets."
  [{:keys [secrets] :as config}]
  (let [secrets (if (:aero/missing-include secrets) {} secrets)]
    (-> config
        (update-in [:jev :api-key] #(not-empty (or (not-empty %) (get-in secrets [:typesafe :api-key]))))
        (dissoc :secrets))))

(defn read-config
  "Reads config/config.edn from the classpath for `profile`."
  [profile]
  (when-not (profiles profile)
    (throw (ex-info (str "Unknown profile " profile ". Use one of :dev, :test or :prod.")
                    {:type :ownpurse/unknown-profile :profile profile :profiles profiles})))
  (with-secrets (aero/read-config (io/resource "config/config.edn") {:profile profile})))
