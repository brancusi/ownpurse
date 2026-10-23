(ns ownpurse.config.interface
  "The configuration of a running ownpurse."
  (:require
   [ownpurse.config.core :as core]))

(defn read-config
  "Returns the configuration for `profile` (:dev, :test or :prod), with the environment applied."
  [profile]
  (core/read-config profile))
