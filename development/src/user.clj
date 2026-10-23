(ns user
  "Loaded by every REPL. Kept small: (dev) loads the development namespace.")

(defn dev
  "Load the development namespace and switch to it."
  []
  (require 'dev)
  (in-ns 'dev))
