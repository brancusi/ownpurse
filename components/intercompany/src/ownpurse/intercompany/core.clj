(ns ownpurse.intercompany.core
  (:require
   [clojure.string :as str]
   [ownpurse.ledger.interface :as ledger]
   [ownpurse.money.interface :as money]
   [ownpurse.store.interface :as store])
  (:import
   (java.time LocalDate YearMonth)
   (java.time.temporal ChronoUnit)))

(def ^:private metadata {:source "ownpurse" :why "intercompany pairs"})

(defn- fail! [type message data]
  (throw (ex-info message (assoc data :type type))))

(defn pair-id [ref] (str "intercompany-pair:" ref))

(defn- check-side! [ref k {:keys [book accounts narration assigned opening]}]
  (when (or (str/blank? book) (empty? accounts))
    (fail! :ownpurse/invalid (str "Side " (name k) " of pair " ref " needs a book and at least one account.")
           {:ref ref :side k}))
  (when narration
    (try (re-pattern narration)
         (catch java.util.regex.PatternSyntaxException e
           (fail! :ownpurse/invalid (str "Side " (name k) " of pair " ref ": the narration pattern does not read: "
                                         (.getDescription e) ".")
                  {:ref ref :side k}))))
  (when (some (comp str/blank? :entry) assigned)
    (fail! :ownpurse/invalid (str "Side " (name k) " of pair " ref ": each assigned line names its entry, and "
                                  "optionally its line: {entry, line}. To assign lines by their narration, give the "
                                  "side a narration pattern instead.")
           {:ref ref :side k}))
  (when (and opening (nil? (:amount opening)))
    (fail! :ownpurse/invalid (str "Side " (name k) " of pair " ref ": an opening needs an amount.") {:ref ref :side k})))

(defn- side-out [side]
  (cond-> side
    (:opening side) (update-in [:opening :amount] money/amount)))

(defn- out [row]
  (cond-> (-> (dissoc row :xt/id :xt/valid-from :xt/valid-to) (update :a side-out) (update :b side-out))
    (:explained row) (update :explained (partial mapv #(update % :amount money/amount)))))

(defn pair [node ref]
  (when (store/table-exists? node :intercompany-pairs)
    (some-> (first (store/q node ["SELECT * FROM intercompany_pairs WHERE _id = ?" (pair-id ref)])) out)))

(defn put-pair! [node {:keys [ref a b from to explained] :as given}]
  (when-not (and (string? ref) (re-matches #"[A-Za-z0-9][A-Za-z0-9._-]{0,63}" ref))
    (fail! :ownpurse/invalid "A pair needs a reference of letters, digits, dots, dashes and underscores." {:ref ref}))
  (check-side! ref :a a)
  (check-side! ref :b b)
  (when (and from to (.isAfter ^LocalDate from to))
    (fail! :ownpurse/invalid (str "Pair " ref " ends before it starts.") {:ref ref}))
  (let [side #(cond-> {:book (:book %) :accounts (vec (distinct (:accounts %)))}
                (seq (:parties %)) (assoc :parties (vec (distinct (:parties %))))
                (seq (:assigned %)) (assoc :assigned (mapv (fn [a] (select-keys a [:entry :line])) (:assigned %)))
                (:narration %) (assoc :narration (:narration %))
                (:opening %) (assoc :opening (-> (select-keys (:opening %) [:amount :note])
                                                 (update :amount money/amount))))]
    (store/put-changed! node {:intercompany-pairs
                              [(into {:xt/id (pair-id ref) :ref ref :a (side a) :b (side b)
                                      :explained (mapv #(update % :amount money/amount) explained)}
                                     (remove (comp nil? val))
                                     (select-keys given [:name :from :to :note]))]}
                        metadata))
  (pair node ref))

(defn pairs [node]
  (if (store/table-exists? node :intercompany-pairs)
    (mapv out (store/q node ["SELECT * FROM intercompany_pairs ORDER BY ref"]))
    []))

(defn- later [^LocalDate x ^LocalDate y] (cond (nil? x) y (nil? y) x (.isAfter x y) x :else y))
(defn- earlier [^LocalDate x ^LocalDate y] (cond (nil? x) y (nil? y) x (.isBefore x y) x :else y))

(defn- assigned?
  "Whether a line that names no party is assigned to the side's parties: by its entry (and line, when given), or by
  its narration or description matching the side's pattern."
  [{:keys [assigned narration]}]
  (let [entries (set (keep #(when-not (:line %) (:entry %)) assigned))
        lines (set (keep #(when (:line %) [(:entry %) (:line %)]) assigned))
        pattern (some-> narration re-pattern)]
    (fn [{:keys [entry line] :as p}]
      (boolean (or (entries entry) (lines [entry line])
                   (and pattern (some #(and % (re-find pattern %)) [(:narration p) (:description p)])))))))

(defn- side-lines
  "The postings to a side's accounts up to `to`, each with its account; with `parties`, split into those of the
  parties (or assigned to them), and those with no party at all (which the side cannot attribute)."
  [node {:keys [book accounts parties] :as side} to]
  (let [lines (for [account accounts
                    p (ledger/postings node book account {:to to})]
                (assoc p :account account))
        assigned? (assigned? side)]
    (if (seq parties)
      {:lines (filterv #(or ((set parties) (:party %)) (and (nil? (:party %)) (assigned? %))) lines)
       :unattributed (filterv #(and (nil? (:party %)) (not (assigned? %))) lines)}
      {:lines (vec lines) :unattributed []})))

(defn- months
  "Each side's movement by month over the period, and their sum: what an accountant reads when one side books a
  month in one journal and the other in many lines."
  [as bs]
  (let [by-month (fn [lines] (reduce (fn [acc {:keys [date amount]}]
                                       (update acc (str (YearMonth/from date)) (fnil + 0M) amount))
                                     {} lines))
        ma (by-month as)
        mb (by-month bs)]
    (vec (for [m (sort (into (set (keys ma)) (keys mb)))
               :let [a (get ma m 0M) b (get mb m 0M)]]
           {:month m :a a :b b :difference (+ a b)}))))

(defn- within [from to lines]
  (filterv #(and (or (nil? from) (not (.isBefore ^LocalDate (:date %) from)))
                 (or (nil? to) (not (.isAfter ^LocalDate (:date %) to))))
           lines))

(defn- total [lines] (reduce + 0M (map :amount lines)))

(defn- days-apart [^LocalDate x ^LocalDate y]
  (Math/abs (.between ChronoUnit/DAYS x y)))

(defn- match-units
  "Pairs units of side a with units of side b of the opposite amount at most `days` apart, the nearest first.
  Returns [unmatched-a unmatched-b matched-count]."
  [as bs days]
  (loop [[u & more] (sort-by :date as) bs (vec bs) left [] n 0]
    (if-not u
      [left bs n]
      (let [candidates (keep-indexed (fn [i b] (when (and (zero? (+ (:amount u) (:amount b)))
                                                          (<= (days-apart (:date u) (:date b)) days))
                                                 [i (days-apart (:date u) (:date b))]))
                                     bs)]
        (if-let [[i] (first (sort-by second candidates))]
          (recur more (into (subvec bs 0 i) (subvec bs (inc i))) left (inc n))
          (recur more bs (conj left u) n))))))

(defn- by-entry
  "Lines summed by their entry: one unit per entry, its lines kept."
  [lines]
  (for [[entry ls] (group-by :entry lines)]
    {:entry entry :date (:date (first ls)) :amount (total ls) :lines ls}))

(defn- unmatched
  "The lines of each side over the period that the other side has no mirror for. A line matches a line of the
  opposite amount; what is left matches by entry (an entry split over several accounts on one side)."
  [as bs days]
  (let [[as bs n1] (match-units as bs days)
        [as bs n2] (match-units (by-entry as) (by-entry bs) days)]
    {:matched (+ n1 n2)
     :a (vec (sort-by (juxt :date :entry :line) (mapcat :lines as)))
     :b (vec (sort-by (juxt :date :entry :line) (mapcat :lines bs)))}))

(defn reconcile
  "One pair over `from` to `to`."
  [node {:keys [a b explained] :as pair} {:keys [from to days] :or {days 5}}]
  (let [period-from (later from (:from pair))
        period-to (earlier to (:to pair))
        side (fn [s]
               (let [{:keys [lines unattributed]} (side-lines node s to)
                     period (within period-from period-to lines)]
                 {:summary (cond-> (assoc (select-keys s [:book :accounts :parties :opening])
                                          :balance (+ (total lines) (get-in s [:opening :amount] 0M))
                                          :movement (total period))
                             (seq (:parties s)) (assoc :unattributed (within period-from period-to unattributed)))
                  :period period}))
        sa (side a)
        sb (side b)
        difference (+ (get-in sa [:summary :balance]) (get-in sb [:summary :balance]))
        explained-total (total explained)
        {:keys [matched] :as lines} (unmatched (:period sa) (:period sb) days)]
    (cond-> {:ref (:ref pair)
             :from period-from
             :to period-to
             :a (:summary sa)
             :b (:summary sb)
             :difference difference
             :explained explained
             :unexplained (- difference explained-total)
             :movement-difference (+ (get-in sa [:summary :movement]) (get-in sb [:summary :movement]))
             :months (months (:period sa) (:period sb))
             :matched matched
             :unmatched (select-keys lines [:a :b])
             :reconciled? (zero? (- difference explained-total))}
      (:name pair) (assoc :name (:name pair)))))

(defn report [node {:keys [from to] :as opts}]
  (vec (for [p (pairs node)
             :when (and (or (nil? to) (nil? (:from p)) (not (.isAfter ^LocalDate (:from p) to)))
                        (or (nil? from) (nil? (:to p)) (not (.isBefore ^LocalDate (:to p) from))))]
         (reconcile node p opts))))
