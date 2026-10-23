(ns ownpurse.xero.interface
  "Xero, as xero-axi holds it. The only brick that knows about Xero: it reads a xero-axi record and the postings
  xero-axi derives from it, and turns them into ownpurse's facts. It never calls Xero."
  (:require
   [ownpurse.xero.core :as core]
   [ownpurse.xero.record :as record]))

(defn read-source
  "Reads a xero-axi record (SQLite, opened read-only) and its postings (the output of `xero-axi --json --full gl
  all`). `paths` is {:record path, :gl path}. Returns the raw rows, untouched."
  [paths]
  (record/read-source paths))

(defn facts
  "Turns what `read-source` returned into ownpurse's facts: {:sources :books :accounts :parties :documents
  :entries}, each a vector of maps that match the schemas. Pure."
  [source]
  (core/facts source))

(defn metadata
  "What a load of `source` records about itself in the transaction: where it came from and the record's chain
  head when it was read."
  [source]
  (core/metadata source))
