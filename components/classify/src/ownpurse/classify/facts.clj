(ns ownpurse.classify.facts
  "Enrichment: the facts about a bank line that code can know for certain (its category from the merchant code, who
  held the card, where they were that day, which project was running), computed before any judgment. Dates are
  compared here, never by a model."
  (:require
   [clojure.string :as str])
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
  "Where `person` was on day `d`: on one of their trips (with its project, if any, and which day of it), else at home.
  :place is the country; :city is there when the trip or the home names one. A scenario's :home is a country, or
  {:country :city}."
  [{:keys [trips home]} person d]
  (if-let [{:keys [place city project from to]} (first (filter #(and (= person (:person %)) (within? d %)) trips))]
    (cond-> {:place place :trip (if project :project :personal) :project project
             :day (days-between from d) :days (days-between from to)}
      city (assoc :city city))
    (let [{:keys [country city]} (if (map? home) home {:country home})]
      (cond-> {:place country :trip nil}
        city (assoc :city city)))))

(defn upcoming-trips
  "`person`'s trips that start after day `d` and within `days` of it, soonest first: what a booking made on `d` may be
  for."
  [{:keys [trips]} person d days]
  (let [d (date d)
        until (.plusDays d days)]
    (->> trips
         (filter #(and (= person (:person %))
                       (.isAfter (date (:from %)) d)
                       (not (.isAfter (date (:from %)) until))))
         (sort-by :from)
         vec)))

(defn refund-of
  "The purchase a refund (a line with a negative amount) returns, among the earlier purchases in the scenario's
  :cases on the same card at the same merchant in the same currency: one of exactly the refund's amount, else the
  latest at least as large, else the latest; nil when there is none. Matched in code, so the refund can follow it."
  [{:keys [cases]} {:keys [amount card merchant currency] d :date}]
  (when (and amount (neg? amount))
    (let [back (- amount)
          earlier (->> cases
                       (filter #(and (= card (:card %)) (= merchant (:merchant %)) (pos? (:amount %))
                                     (or (nil? currency) (= currency (:currency %)))
                                     (.isBefore (date (:date %)) (date d))))
                       (sort-by :date))]
      (or (last (filter #(== back (:amount %)) earlier))
          (last (filter #(>= (:amount %) back) earlier))
          (last earlier)))))

(defn earlier-here
  "The latest other charge on the same card at the same merchant in the week up to a purchase (a line with a
  positive amount), or nil: a tip or a charge posted after the meal or stay it belongs to can be read beside it. A
  charge the same day counts when it comes first in the scenario's :cases."
  [{:keys [cases]} {:keys [amount card merchant] d :date :as line}]
  (when (and amount (pos? amount))
    (let [own (or (:line line) (:id line))
          order (into {} (map-indexed (fn [i c] [(:id c) i])) cases)
          day (date d)]
      (->> cases
           (filter #(let [cd (date (:date %))]
                      (and (= card (:card %)) (= merchant (:merchant %)) (pos? (:amount %)) (not= own (:id %))
                           (not (.isAfter cd day)) (not (.isBefore cd (.minusDays day 7)))
                           (or (.isBefore cd day) (< (order (:id %)) (order own))))))
           (sort-by (juxt :date (comp order :id)))
           last))))

(defn running-projects
  "The ids of the projects running on day `d`."
  [{:keys [projects]} d]
  (vec (sort (for [[id p] projects :when (within? d p)] id))))

(defn units
  "What is coded about `line`: the line itself, or, when a receipt or an order lists its items, each item, as the
  line narrowed to it (its own :id, :expect and :ok, and its :review or else the line's; :amount the item's,
  :line-amount the line's, :other-items the rest of the receipt)."
  [{:keys [id items amount] :as line}]
  (if (seq items)
    (vec (map-indexed
          (fn [i it]
            (merge (dissoc line :items :expect :ok :why)
                   (select-keys it [:item :amount :expect :ok :review :why])
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
         running (running-projects scenario date)
         refunded (refund-of scenario line)
         before (earlier-here scenario line)]
     (cond-> (merge (select-keys line [:id :line :date :merchant :amount :currency :country :city :channel :note
                                       :item :line-amount :items-from :other-items])
                    {:card card :card-name (:name card-info) :holder holder :card-entity entity
                     :whereabouts where
                     :projects running
                     :upcoming (upcoming-trips scenario holder date 90)
                     :traveller (boolean (some #(some #{holder} (get-in projects [% :travellers])) running))
                     :country-mismatch (boolean (and (= :in-store channel)
                                                     (or (and country (not= country (:place where)))
                                                         (and (:city line) (:city where)
                                                              (not= (:city line) (:city where))))))})
       refunded (assoc :refund-of (assoc (select-keys refunded [:id :date :amount :currency :note])
                                         :whereabouts (whereabouts scenario holder (:date refunded))))
       before (assoc :earlier-here (select-keys before [:id :date :amount :currency :note]))
       mcc (assoc :mcc mcc :category (get categories mcc))
       (and mcc (not (:mcc line))) (assoc :category-from :enrichment)))))

(defn- where-text [{:keys [projects]} {:keys [city trip project day days] :as where}]
  (let [place (if city (str city ", " (:place where)) (:place where))]
    (case trip
      :project (str "in " place ", on a project trip for " (get-in projects [project :name])
                    " (day " day " of " days ")")
      :personal (str "in " place ", on a personal trip (no project)")
      (str "at home in " place))))

(defn state
  "What a judgment sees about a line: its facts in words, named fields only, and nothing it does not need."
  [{:keys [entities projects] :as scenario}
   {:keys [merchant category amount currency country city channel note card-name holder whereabouts country-mismatch
           item line-amount items-from other-items]
    :as facts}]
  {:transaction (cond-> {:merchant merchant
                         :amount (str currency " " (.toPlainString ^BigDecimal (or line-amount amount)))
                         :how (case channel :in-store "in person, in a shop" :online "online" :transfer "a transfer"
                                    (name channel))}
                  category (assoc :category category)
                  country (assoc :merchant_country country)
                  city (assoc :merchant_city city)
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
                                              (str name ", in " place (some->> work (str ": ")))))
                     :travellers_of_running_projects
                     (vec (for [p (:projects facts)
                                :let [{:keys [name travellers]} (get projects p)]
                                :when (seq travellers)]
                            (str (str/join " and " (map #(get-in entities [% :name]) travellers)) " travel for " name
                                 "; anyone else is not one of its travellers")))}
              (:traveller facts) (assoc :cardholder_travels_for_a_running_project true)
              (seq (:upcoming facts))
              (assoc :cardholder_upcoming_trips
                     (vec (for [{:keys [place city from to project]} (:upcoming facts)]
                            (str (if city (str city ", " place) place) ", " from " to " to ": "
                                 (if project (str "for " (get-in projects [project :name])) "a personal trip, no project")))))
              (= :in-store channel) (assoc :shop_is_where_cardholder_was (not country-mismatch))
              (:earlier-here facts)
              (assoc :earlier_charge_here
                     (let [{:keys [date amount currency note]} (:earlier-here facts)]
                       (str "another charge at this merchant on this card on " date ", " currency " "
                            (.toPlainString ^BigDecimal amount) (some->> note (str ", note: ")))))
              (:refund-of facts)
              (assoc :refund_of
                     (let [{:keys [date amount currency note whereabouts]} (:refund-of facts)]
                       (str "a purchase at this merchant on this card on " date ", " currency " "
                            (.toPlainString ^BigDecimal amount) (some->> note (str ", note: "))
                            ", when the cardholder was " (where-text scenario whereabouts)))))})
