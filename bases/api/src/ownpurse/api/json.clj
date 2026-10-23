(ns ownpurse.api.json
  "JSON in and out. Every number with a fraction is read as a BigDecimal and written in plain notation, never as a
  double."
  (:require
   [jsonista.core :as j]
   [muuntaja.core :as m])
  (:import
   (com.fasterxml.jackson.core JsonGenerator$Feature)
   (com.fasterxml.jackson.databind ObjectMapper)))

(def ^ObjectMapper mapper
  "The one JSON mapper: keyword keys, BigDecimal fractions, plain BigDecimal output."
  (doto (j/object-mapper {:decode-key-fn true :bigdecimals true})
    (.configure JsonGenerator$Feature/WRITE_BIGDECIMAL_AS_PLAIN true)))

(def muuntaja
  "Content negotiation for the API: JSON only, through `mapper`."
  (m/create (-> m/default-options
                (update :formats select-keys ["application/json"])
                (assoc-in [:formats "application/json" :opts] {:mapper mapper}))))

(defn read-str
  "Reads a JSON string."
  [s]
  (j/read-value s mapper))

(defn write-str
  "Writes `x` as a JSON string."
  [x]
  (j/write-value-as-string x mapper))
