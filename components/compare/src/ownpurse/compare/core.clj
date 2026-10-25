(ns ownpurse.compare.core
  (:require
   [ownpurse.entity.interface :as entity]
   [ownpurse.store.interface :as store])
  (:import
   (java.time LocalDate)
   (java.time.temporal ChronoUnit)))

(def default-days
  "How many days apart two lines of the same amount may be and still be the same money: a payment dated in Xero
  the day it was sent, posted by the bank days later."
  5)

(def ^:private check-days
  "How far outside the range lines are read, so a check written near its edge finds the line where it cleared."
  120)

(defn- days-apart [^LocalDate a ^LocalDate b]
  (Math/abs (.between ChronoUnit/DAYS a b)))

(defn- amount-key
  "1.10 and 1.1 are the same amount; as keys they would not be."
  [^BigDecimal amount]
  (.stripTrailingZeros amount))

(defn- in-order [lines]
  (sort-by (juxt :date :xt/id) lines))

(defn- check-key [{:keys [check-number amount]}]
  (when check-number [check-number (amount-key amount)]))

(defn- match-checks
  "Pairs each evidence line with a book line of the same check number and amount, however far apart their dates:
  a check is written on one day and clears weeks later. Returns {:matched [[e b]] :evidence [left] :book [left]}."
  [evidence book]
  (let [{:keys [matched left]}
        (reduce (fn [{:keys [open] :as acc} e]
                  (if-let [b (some-> (check-key e) open first)]
                    (-> acc (update :matched conj [e b]) (update-in [:open (check-key e)] (comp vec rest)))
                    (update acc :left conj e)))
                {:open (group-by check-key (filter check-key book)) :matched [] :left []}
                (in-order evidence))
        used (set (map (comp :xt/id second) matched))]
    {:matched matched :evidence left :book (remove (comp used :xt/id) book)}))

(defn- match-by-date [evidence book days]
  (let [step (fn [{:keys [open] :as acc} e]
               (let [k (amount-key (:amount e))
                     apart #(days-apart (:date e) (:date %))
                     best (->> (get open k)
                               (filter #(<= (apart %) days))
                               (sort-by (juxt apart :date :xt/id))
                               first)]
                 (if best
                   (-> acc
                       (update :matched conj [e best])
                       (update-in [:open k] (fn [ls] (vec (remove #(= (:xt/id best) (:xt/id %)) ls)))))
                   (update acc :evidence-only conj e))))
        {:keys [matched evidence-only open]}
        (reduce step
                {:open (group-by (comp amount-key :amount) book) :matched [] :evidence-only []}
                (in-order evidence))]
    {:matched matched
     :evidence-only evidence-only
     :book-only (vec (in-order (mapcat val open)))}))

(defn match [evidence book days]
  (let [checks (match-checks evidence book)
        by-date (match-by-date (:evidence checks) (:book checks) days)]
    (update by-date :matched #(into (:matched checks) %))))

(defn- lines
  "The lines of `account` dated from `from` to `to` (either may be nil: no bound)."
  [node account ^LocalDate from ^LocalDate to]
  (if-not (store/table-exists? node :bank-lines)
    []
    (store/q node (cond-> ["SELECT * FROM bank_lines WHERE account = ?" account]
                    from (-> (update 0 str " AND date >= ?") (conj from))
                    to (-> (update 0 str " AND date <= ?") (conj to))))))

(defn- line-out [{:keys [xt/id date amount status counterparty description reference check-number source-row]}]
  (cond-> {:id id :date (str date) :amount (.toPlainString ^BigDecimal amount) :status status :source-row source-row}
    counterparty (assoc :counterparty counterparty)
    description (assoc :description description)
    reference (assoc :reference reference)
    check-number (assoc :check-number check-number)))

(defn- total [lines]
  (.toPlainString ^BigDecimal (reduce + 0M (map :amount lines))))

(defn- compare-link
  [node {:keys [entity account evidence-for]} {:keys [^LocalDate from ^LocalDate to days]}]
  (let [days (or days default-days)
        ;; Lines outside the range are read too, so a line near its edge still finds its match; far enough for a
        ;; check that clears weeks after it was written. Matching by date still keeps to `days`.
        pad (max days check-days)
        padded #(lines node % (some-> from (.minusDays pad)) (some-> to (.plusDays pad)))
        in-range? (fn [{:keys [^LocalDate date]}]
                    (and (or (nil? from) (not (.isBefore date from)))
                         (or (nil? to) (not (.isAfter date to)))))
        evidence (padded account)
        book (padded evidence-for)
        {:keys [matched evidence-only book-only]} (match evidence book days)
        missing (filter in-range? evidence-only)
        name-of #(:name (entity/source-account node %))]
    (cond-> {:entity entity
             :account account
             :account-name (name-of account)
             :evidence-for evidence-for
             :evidence-for-name (name-of evidence-for)
             :days days
             :matched (count (filter (comp in-range? first) matched))
             :missing-from-book (mapv line-out (remove #(= "pending" (:status %)) missing))
             :missing-from-evidence (mapv line-out (filter in-range? book-only))
             :pending (mapv line-out (filter #(= "pending" (:status %)) missing))
             :totals {:evidence (total (filter #(and (in-range? %) (= "posted" (:status %))) evidence))
                      :book (total (filter in-range? book))}}
      from (assoc :from (str from))
      to (assoc :to (str to)))))

(defn compare-accounts [node {:keys [account] :as opts}]
  (let [links (filter #(and (= "evidence" (:role %)) (:evidence-for %)
                            (or (nil? account) (= account (:account %))))
                      (entity/links node))]
    (when (and account (empty? links))
      (throw (ex-info (str account " is not linked as evidence for an account. Link it first: PUT /links with "
                           "role evidence and evidence-for.")
                      {:type :ownpurse/not-found :account account})))
    (mapv #(compare-link node % opts) links)))
