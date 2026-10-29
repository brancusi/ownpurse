(ns ownpurse.admin.scenarios.common
  "What every part of a scenario's page shares: its API paths, and its layers as the API takes them back."
  (:require
   [clojure.string :as str]))

(defn path [ref & more] (str "/scenarios/" (str/join "/" (map js/encodeURIComponent (cons ref more)))))

(def data-key
  "Where each kind of layer keeps what it reads."
  {"world" :world "coding" :coding "lines" :lines "journals" :journals "close" :basis})

(def templates
  "What a new layer of each kind starts with."
  {"world" {:trips [{:person "someone" :place "Country" :city "City" :from "2025-01-01" :to "2025-01-31" :project "a-project"}]}
   "coding" {:rules [{:id "R1" :code "a-bucket" :rule "The rule as a person would say it."
                      :when [["category" "in" ["5411"]]]}]}
   "lines" {:select {:accounts ["a source account id"] :from "2025-01-01" :to "2025-12-31"}}
   "journals" []
   "close" "proposed"})

(defn json [v] (js/JSON.stringify (clj->js v) nil 2))

(defn parse-json
  "[value nil] for the JSON text `s`, or [nil message] when it is not JSON."
  [s]
  (try [(js->clj (js/JSON.parse s)) nil]
       (catch :default e [nil (str "Not JSON: " (.-message e))])))

(defn layer-body
  "A layer as PUT /scenarios/{ref}/layers/{id} takes it."
  [layer]
  (select-keys layer [:kind :enabled :note :basis :journals :world :coding :lines]))

(defn scenario-body
  "A scenario as PUT /scenarios/{ref} takes it."
  [s]
  (-> (select-keys s [:name :note :base :layers])
      (update :layers (partial mapv #(select-keys % [:id :kind :enabled :note :basis :journals :world :coding :lines])))))
