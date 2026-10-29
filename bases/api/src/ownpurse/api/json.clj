(ns ownpurse.api.json
  "Content negotiation for the API: JSON through ownpurse's one mapper, and Transit for the admin, which reads the
  components' data as they hold it (keywords, sets, decimals)."
  (:require
   [cognitect.transit :as transit]
   [muuntaja.core :as m]
   [ownpurse.json.interface :as json]))

(def ^:private transit-handlers
  "Times as their ISO strings: the admin reads them as text. The store gives back an instant it kept as a
  ZonedDateTime, so that is an instant too."
  {java.time.Instant (transit/write-handler "instant" str)
   java.time.ZonedDateTime (transit/write-handler "instant" #(str (.toInstant ^java.time.ZonedDateTime %)))
   java.time.OffsetDateTime (transit/write-handler "instant" #(str (.toInstant ^java.time.OffsetDateTime %)))
   java.time.LocalDate (transit/write-handler "date" str)})

(def muuntaja
  "Muuntaja for reitit: JSON (BigDecimal-safe, the default) and Transit JSON."
  (m/create (-> m/default-options
                (update :formats select-keys ["application/json" "application/transit+json"])
                (assoc-in [:formats "application/json" :opts] {:mapper json/mapper})
                (assoc-in [:formats "application/transit+json" :encoder-opts] {:handlers transit-handlers}))))
