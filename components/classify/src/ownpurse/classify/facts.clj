(ns ownpurse.classify.facts
  "Enrichment: the facts about a bank line that code can know for certain (its category from the merchant code, who
  held the card, where they were that day, which project was running), computed before any judgment. Dates are
  compared here, never by a model."
  (:import
   (java.time LocalDate)
   (java.time.temporal ChronoUnit)))

(defn- date ^LocalDate [s]
  (if (instance? LocalDate s) s (LocalDate/parse s)))

(defn- within? [d {:keys [from to]}]
  (let [d (date d)]
    (not (or (.isBefore d (date from)) (.isAfter d (date to))))))

(defn- days-between [from to]
  (inc (.between ChronoUnit/DAYS (date from) (date to))))

(defn whereabouts
  "Where `person` was on day `d`: on one of their trips (with its project, if any, and which day of it), else at home."
  [{:keys [trips home]} person d]
  (if-let [{:keys [place project from to]} (first (filter #(and (= person (:person %)) (within? d %)) trips))]
    {:place place :trip (if project :project :personal) :project project
     :day (days-between from d) :days (days-between from to)}
    {:place home :trip nil}))

(defn running-projects
  "The ids of the projects running on day `d`."
  [{:keys [projects]} d]
  (vec (sort (for [[id p] projects :when (within? d p)] id))))

(defn units
  "What is coded about `line`: the line itself, or, when a receipt or an order lists its items, each item, as the
  line narrowed to it (its own :id, :expect and :ok; :amount the item's, :line-amount the line's, :other-items the
  rest of the receipt)."
  [{:keys [id items amount] :as line}]
  (if (seq items)
    (vec (map-indexed
          (fn [i it]
            (merge (dissoc line :items :expect :ok :why)
                   (select-keys it [:item :amount :expect :ok :why])
                   {:id (str id "/" (inc i)) :line id :line-amount amount
                    :other-items (vec (keep-indexed #(when (not= i %1) (:item %2)) items))}))
          items))
    [line]))

(defn enrich
  "The facts about `line` (or one unit of it) in `scenario`. A line with no merchant code takes its category from
  `category`, the code an enrichment judgment chose (or nil when none fits)."
  ([scenario line] (enrich scenario line nil))
  ([{:keys [cards categories projects] :as scenario} {:keys [date card mcc country channel] :as line} category]
   (let [{:keys [holder entity] :as card-info} (get cards card)
         mcc (or mcc category)
         where (whereabouts scenario holder date)
         running (running-projects scenario date)]
     (cond-> (merge (select-keys line [:id :line :date :merchant :amount :currency :country :channel :note
                                       :item :line-amount :items-from :other-items])
                    {:card card :card-name (:name card-info) :holder holder :card-entity entity
                     :whereabouts where
                     :projects running
                     :traveller (boolean (some #(some #{holder} (get-in projects [% :travellers])) running))
                     :country-mismatch (boolean (and (= :in-store channel) country
                                                     (not= country (:place where))))})
       mcc (assoc :mcc mcc :category (get categories mcc))
       (and mcc (not (:mcc line))) (assoc :category-from :enrichment)))))

(defn- where-text [{:keys [projects]} {:keys [place trip project day days]}]
  (case trip
    :project (str "in " place ", on a project trip for " (get-in projects [project :name])
                  " (day " day " of " days ")")
    :personal (str "in " place ", on a personal trip (no project)")
    (str "at home in " place)))

(defn state
  "What a judgment sees about a line: its facts in words, named fields only, and nothing it does not need."
  [{:keys [entities projects] :as scenario}
   {:keys [merchant category amount currency country channel note card-name holder whereabouts country-mismatch
           item line-amount items-from other-items]
    :as facts}]
  {:transaction (cond-> {:merchant merchant
                         :amount (str currency " " (.toPlainString ^BigDecimal (or line-amount amount)))
                         :how (case channel :in-store "in person, in a shop" :online "online" :transfer "a transfer"
                                    (name channel))}
                  category (assoc :category category)
                  country (assoc :merchant_country country)
                  note (assoc :note note)
                  item (assoc :item (cond-> {:name item
                                             :amount (str currency " " (.toPlainString ^BigDecimal amount))
                                             :listed_on (case items-from :order "the online order" "the scanned receipt")}
                                      (seq other-items) (assoc :other_items_on_it other-items))))
   :context (cond-> {:paid_with card-name
                     :cardholder (get-in entities [holder :name])
                     :cardholder_was (where-text scenario whereabouts)
                     :projects_running (vec (for [p (:projects facts)
                                                  :let [{:keys [name place work]} (get projects p)]]
                                              (str name ", in " place (some->> work (str ": ")))))}
              (:traveller facts) (assoc :cardholder_travels_for_a_running_project true)
              (= :in-store channel) (assoc :shop_is_where_cardholder_was (not country-mismatch)))})
