(ns ownpurse.json.interface
  "JSON in and out. Every number with a fraction is read as a BigDecimal and written in plain notation; JSON never
  turns into a double here."
  (:require
   [ownpurse.json.core :as core]))

(def mapper
  "The one Jackson mapper: keyword keys, BigDecimal fractions, plain BigDecimal output. For libraries that take a
  mapper (Muuntaja)."
  core/mapper)

(defn parse
  "Reads JSON from a string, a java.io.File, a reader or an input stream."
  [source]
  (core/parse source))

(defn write-str
  "Writes `x` as a JSON string."
  [x]
  (core/write-str x))
