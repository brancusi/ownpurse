(ns ownpurse.schema.interface
  "The shapes of ownpurse's data, as Malli schemas: books, accounts, entries, postings. Every boundary (the API, a
  source, the store) checks data against them."
  (:require
   [ownpurse.schema.core :as core]))

(def registry
  "The registry: :ownpurse/amount, :ownpurse/commodity, :ownpurse/date, :ownpurse/book, :ownpurse/account,
  :ownpurse/posting, :ownpurse/entry and the pieces they are made of."
  core/registry)

(defn schema
  "The Malli schema for `k`, for reitit routes and generators."
  [k]
  (core/schema k))

(defn valid?
  "True when `x` matches the schema `k`."
  [k x]
  (core/valid? k x))

(defn explain
  "What is wrong with `x` against `k`, in words (Malli's humanized errors), or nil when it is valid."
  [k x]
  (core/explain k x))

(defn check!
  "Returns `x` when it matches `k`. Otherwise throws :ownpurse/invalid with the schema and the errors."
  [k x]
  (core/check! k x))

(defn coerce!
  "Decodes `x` from JSON-shaped data (decimal strings, ISO dates, keyword strings) into `k`, then `check!`s it. A
  float is never decoded into an amount: it fails the check."
  [k x]
  (core/coerce! k x))
