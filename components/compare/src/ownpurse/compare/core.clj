(ns ownpurse.compare.core
  (:require
   [clojure.string :as str]
   [ownpurse.entity.interface :as entity]
   [ownpurse.store.interface :as store])
  (:import
   (java.time Instant LocalDate ZoneOffset ZonedDateTime)
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

(defn- pairs [by matched]
  (map (fn [[e b]] {:evidence e :book b :by by}) matched))

(defn match
  ([evidence book days] (match evidence book days []))
  ([evidence book days manual]
   (let [index #(into {} (map (juxt :xt/id identity)) %)
         evidence-by (index evidence)
         book-by (index book)
         asserted (for [[e b] manual
                        :let [el (evidence-by e) bl (book-by b)]
                        :when (and el bl)]
                    [el bl])
         used (set (mapcat #(map :xt/id %) asserted))
         checks (match-checks (remove (comp used :xt/id) evidence) (remove (comp used :xt/id) book))
         by-date (match-by-date (:evidence checks) (:book checks) days)]
     {:matched (vec (concat (pairs "manual" asserted) (pairs "check" (:matched checks))
                            (pairs "amount" (:matched by-date))))
      :evidence-only (:evidence-only by-date)
      :book-only (:book-only by-date)})))

(defn- lines
  "The lines of `account` dated from `from` to `to` (either may be nil: no bound)."
  [node account ^LocalDate from ^LocalDate to]
  (if-not (store/table-exists? node :bank-lines)
    []
    (store/q node (cond-> ["SELECT * FROM bank_lines WHERE account = ?" account]
                    from (-> (update 0 str " AND date >= ?") (conj from))
                    to (-> (update 0 str " AND date <= ?") (conj to))))))

(defn- lines-by-id [node ids]
  (if (and (seq ids) (store/table-exists? node :bank-lines))
    (store/q node (into [(str "SELECT * FROM bank_lines WHERE _id IN (" (str/join ", " (repeat (count ids) "?")) ")")]
                        ids))
    []))

(defn- line-id [source-row] (str "line:" source-row))

(defn- source-row-of [line-id] (str/replace-first line-id #"^line:" ""))

(defn- match-out [{:keys [evidence book note xt/valid-from]}]
  (cond-> {:evidence (source-row-of evidence) :book (source-row-of book)}
    note (assoc :note note)
    valid-from (assoc :from (str (LocalDate/ofInstant (.toInstant ^ZonedDateTime valid-from) ZoneOffset/UTC)))))

(defn- matches-of
  "The matches a person asserted for the lines of evidence account `account`, now."
  [node account]
  (if (store/table-exists? node :line-matches)
    (store/q node ["SELECT *, _valid_from FROM line_matches WHERE account = ?" account])
    []))

(defn- match-by [node column line]
  (when (store/table-exists? node :line-matches)
    (first (store/q node [(str "SELECT *, _valid_from FROM line_matches WHERE " column " = ?") line]))))

(defn- fail! [type message data]
  (throw (ex-info message (assoc data :type type))))

(def ^:private metadata {:source "ownpurse" :why "matches"})

(defn- start-of [^LocalDate date]
  (.toInstant (.atStartOfDay date ZoneOffset/UTC)))

(defn put-match! [node {:keys [evidence book note from]}]
  (let [[e b] (map #(first (lines-by-id node [(line-id %)])) [evidence book])]
    (doseq [[row line] [[evidence e] [book b]] :when (nil? line)]
      (fail! :ownpurse/not-found (str "There is no line with source row " (pr-str row) ". The compare lists each "
                                      "line's source-row.")
             {:source-row row}))
    (when-not (some #(and (= "evidence" (:role %)) (= (:account e) (:account %)) (= (:account b) (:evidence-for %)))
                    (entity/links node))
      (fail! :ownpurse/invalid-match (str "The two lines are not on an evidence account and the account it is "
                                          "evidence for: " (:account e) " and " (:account b) ". Link them first.")
             {:evidence evidence :book book}))
    (when-not (zero? (.compareTo ^BigDecimal (:amount e) ^BigDecimal (:amount b)))
      (fail! :ownpurse/invalid-match (str "The two lines are not the same amount: " (.toPlainString ^BigDecimal (:amount e))
                                          " and " (.toPlainString ^BigDecimal (:amount b)) ".")
             {:evidence evidence :book book}))
    (doseq [[column line other] [["evidence" (:xt/id e) :book] ["book" (:xt/id b) :evidence]]
            :let [held (match-by node column line)]
            :when (and held (not= (get held other) (if (= :book other) (:xt/id b) (:xt/id e))))]
      (fail! :ownpurse/match-conflict (str (source-row-of line) " is matched with " (source-row-of (get held other))
                                           ". End that match first: DELETE /matches?evidence="
                                           (source-row-of (:evidence held)) ".")
             {:source-row (source-row-of line)}))
    (store/put-changed! node {:line-matches [(cond-> {:xt/id (str "match:" (:xt/id e)) :evidence (:xt/id e)
                                                      :book (:xt/id b) :account (:account e)
                                                      :evidence-for (:account b)}
                                               note (assoc :note note)
                                               from (assoc :xt/valid-from (start-of from)))]}
                        metadata)
    (match-out (match-by node "evidence" (:xt/id e)))))

(defn end-match! [node {:keys [evidence on]}]
  (let [held (or (match-by node "evidence" (line-id evidence))
                 (fail! :ownpurse/not-found (str evidence " is not matched by hand.") {:source-row evidence}))
        at (if on (start-of on) (Instant/now))]
    (when (.isBefore ^Instant at (.toInstant ^ZonedDateTime (:xt/valid-from held)))
      (fail! :ownpurse/invalid-match (str "The match of " evidence " starts later, so it cannot end then.")
             {:source-row evidence}))
    (store/delete! node :line-matches [(:xt/id held)] {:valid-from at} metadata)
    (assoc (match-out held) :until (str (LocalDate/ofInstant at ZoneOffset/UTC)))))

(defn- line-out [{:keys [xt/id date amount status counterparty description reference check-number source-row]}]
  (cond-> {:id id :date (str date) :amount (.toPlainString ^BigDecimal amount) :status status :source-row source-row}
    counterparty (assoc :counterparty counterparty)
    description (assoc :description description)
    reference (assoc :reference reference)
    check-number (assoc :check-number check-number)))

(defn- total [lines]
  (.toPlainString ^BigDecimal (reduce + 0M (map :amount lines))))

(defn- compare-link
  [node {:keys [entity account evidence-for]} {:keys [^LocalDate from ^LocalDate to days] with-pairs :pairs}]
  (let [days (or days default-days)
        ;; Lines outside the range are read too, so a line near its edge still finds its match; far enough for a
        ;; check that clears weeks after it was written. Matching by date still keeps to `days`.
        pad (max days check-days)
        padded #(lines node % (some-> from (.minusDays pad)) (some-> to (.plusDays pad)))
        in-range? (fn [{:keys [^LocalDate date]}]
                    (and (or (nil? from) (not (.isBefore date from)))
                         (or (nil? to) (not (.isAfter date to)))))
        asserted (matches-of node account)
        ;; A line matched by hand may lie further out than the lines read: read it too.
        with-partners (fn [lines column]
                        (let [held (set (map :xt/id lines))]
                          (into (vec lines) (lines-by-id node (vec (remove held (map column asserted)))))))
        evidence (with-partners (padded account) :evidence)
        book (with-partners (padded evidence-for) :book)
        notes (into {} (map (juxt :evidence :note)) asserted)
        {:keys [matched evidence-only book-only]}
        (match evidence book days (map (juxt :evidence :book) asserted))
        matched (filter (comp in-range? :evidence) matched)
        missing (filter in-range? evidence-only)
        name-of #(:name (entity/source-account node %))]
    (cond-> {:entity entity
             :account account
             :account-name (name-of account)
             :evidence-for evidence-for
             :evidence-for-name (name-of evidence-for)
             :days days
             :matched (count matched)
             :matched-by (merge {:manual 0 :check 0 :amount 0} (update-keys (frequencies (map :by matched)) keyword))
             :missing-from-book (mapv line-out (remove #(= "pending" (:status %)) missing))
             :missing-from-evidence (mapv line-out (filter in-range? book-only))
             :pending (mapv line-out (filter #(= "pending" (:status %)) missing))
             :totals {:evidence (total (filter #(and (in-range? %) (= "posted" (:status %))) evidence))
                      :book (total (filter in-range? book))}}
      from (assoc :from (str from))
      to (assoc :to (str to))
      with-pairs (assoc :pairs (vec (for [{:keys [evidence book by]} matched]
                                      (cond-> {:by by :evidence (line-out evidence) :book (line-out book)}
                                        (and (= "manual" by) (notes (:xt/id evidence)))
                                        (assoc :note (notes (:xt/id evidence))))))))))

(defn compare-accounts [node {:keys [account] :as opts}]
  (let [links (filter #(and (= "evidence" (:role %)) (:evidence-for %)
                            (or (nil? account) (= account (:account %))))
                      (entity/links node))]
    (when (and account (empty? links))
      (throw (ex-info (str account " is not linked as evidence for an account. Link it first: PUT /links with "
                           "role evidence and evidence-for.")
                      {:type :ownpurse/not-found :account account})))
    (mapv #(compare-link node % opts) links)))
