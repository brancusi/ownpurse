(ns ownpurse.config.core
  (:require
   [aero.core :as aero]
   [clojure.java.io :as io]))

(def ^:private profiles #{:dev :test :prod})

(defn read-config
  "Reads config/config.edn from the classpath for `profile`."
  [profile]
  (when-not (profiles profile)
    (throw (ex-info (str "Unknown profile " profile ". Use one of :dev, :test or :prod.")
                    {:type :ownpurse/unknown-profile :profile profile :profiles profiles})))
  (aero/read-config (io/resource "config/config.edn") {:profile profile}))
