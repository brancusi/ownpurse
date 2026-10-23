(ns ownpurse.api.json
  "Content negotiation for the API: JSON only, through ownpurse's one mapper."
  (:require
   [muuntaja.core :as m]
   [ownpurse.json.interface :as json]))

(def muuntaja
  "Muuntaja for reitit: JSON only, BigDecimal-safe."
  (m/create (-> m/default-options
                (update :formats select-keys ["application/json"])
                (assoc-in [:formats "application/json" :opts] {:mapper json/mapper}))))
