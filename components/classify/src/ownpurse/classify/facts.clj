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

(defn enrich
  "The facts about `line` in `scenario`. A line with no merchant code takes its category from `category`, the code
  an enrichment judgment chose (or nil when none fits)."
  ([scenario line] (enrich scenario line nil))
  ([{:keys [cards categories] :as scenario} {:keys [date card mcc country channel] :as line} category]
   (let [{:keys [holder entity] :as card-info} (get cards card)
         mcc (or mcc category)
         where (whereabouts scenario holder date)]
     (cond-> (merge (select-keys line [:id :date :merchant :amount :currency :country :channel :note])
                    {:card card :card-name (:name card-info) :holder holder :card-entity entity
                     :whereabouts where
                     :projects (running-projects scenario date)
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
   {:keys [merchant category amount currency country channel note card-name holder whereabouts country-mismatch]
    :as facts}]
  {:transaction (cond-> {:merchant merchant
                         :amount (str currency " " (.toPlainString ^BigDecimal amount))
                         :how (case channel :in-store "in person, in a shop" :online "online" :transfer "a transfer"
                                    (name channel))}
                  category (assoc :category category)
                  country (assoc :merchant_country country)
                  note (assoc :note note))
   :context (cond-> {:paid_with card-name
                     :cardholder (get-in entities [holder :name])
                     :cardholder_was (where-text scenario whereabouts)
                     :projects_running (vec (for [p (:projects facts)
                                                  :let [{:keys [name place]} (get projects p)]]
                                              (str name ", in " place)))}
              (= :in-store channel) (assoc :shop_is_where_cardholder_was (not country-mismatch)))})
