(ns ownpurse.json.core
  (:require
   [jsonista.core :as j])
  (:import
   (com.fasterxml.jackson.core JsonGenerator$Feature)
   (com.fasterxml.jackson.databind ObjectMapper)))

(def ^ObjectMapper mapper
  (doto (j/object-mapper {:decode-key-fn true :bigdecimals true})
    (.configure JsonGenerator$Feature/WRITE_BIGDECIMAL_AS_PLAIN true)))

(defn parse [source]
  (j/read-value source mapper))

(defn write-str [x]
  (j/write-value-as-string x mapper))
