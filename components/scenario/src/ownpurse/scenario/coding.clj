(ns ownpurse.scenario.coding
  "The layers that code bank lines: `world` (who and where: entities, cards and accounts with their holders,
  projects, trips, merchant categories), `coding` (the buckets and the rules) and `lines` (which lines: a book's, read
  from the record, or lines given as they are). A scenario's layers compose, bases first, into the classifier's
  scenario: maps merge key by key, trips add up, and a rule replaces the rule of its id (`:enabled false` takes it
  out)."
  (:require
   [clojure.string :as str]
   [ownpurse.store.interface :as store])
  (:import
   (java.time LocalDate)))

(defn- fail! [message data]
  (throw (ex-info message (assoc data :type :ownpurse/invalid))))

(defn- kw [x] (if (string? x) (keyword x) x))

(defn- date! [where s]
  (try (LocalDate/parse (str s)) s
       (catch Exception _ (fail! (str where " needs a date such as 2025-09-22, not " (pr-str s) ".") {:date s}))))


(defn block
  "A rule's building block as the classifier reads it, from EDN or from JSON: the operator and keyword arguments as
  keywords, the values a block matches against as a set, a channel as a keyword."
  [[op & args]]
  (let [op (kw op)]
    (case op
      (:not) [op (block (first args))]
      (:any) (into [op] (map block args))
      (:category :card :channel) (let [[in vs] args
                                       vs (set (cond->> vs (= :channel op) (map kw)))]
                                   [op (kw in) vs])
      (:amount) (let [[cmp & ns] args] (into [op (kw cmp)] (map bigdec ns)))
      (:is :contains :present) (into [op (kw (first args))] (rest args))
      (:trip) [op (kw (first args))]
      (into [op] args))))

(defn- rule-in [codes {:keys [id code rule] :as r}]
  (when (str/blank? id) (fail! "A rule needs an id, such as R1." {:rule r}))
  (when (str/blank? rule) (fail! (str "Rule " id " needs its sentence, as a person would say it.") {:rule id}))
  (when-not (contains? codes (kw code))
    (fail! (str "Rule " id " codes to " (pr-str code) ", which is no bucket of the scenario.") {:rule id}))
  (cond-> (-> r
              (update :code kw)
              (update :when (partial mapv block)))
    (contains? r :over) (update :over vec)))

(defn world-in
  "A world layer's data, checked: dates are dates, names are named."
  [{:keys [projects trips] :as world}]
  (doseq [[id {:keys [name from to]}] projects]
    (when (str/blank? name) (fail! (str "Project " id " needs a name.") {:project id}))
    (date! (str "Project " id) from)
    (date! (str "Project " id) to))
  (doseq [{:keys [person from to]} trips]
    (when (str/blank? person) (fail! "A trip says whose it is (:person)." {:trip from}))
    (date! (str "A trip of " person) from)
    (date! (str "A trip of " person) to))
  (cond-> (update-keys world kw)
    (:cards world) (update :cards update-keys name)
    (:projects world) (update :projects update-keys name)
    (:entities world) (update :entities update-keys name)
    (:categories world) (update :categories update-keys name)))

(defn coding-in
  "A coding layer's data, checked: every rule codes to one of the buckets (those of this layer and the layers under
  it, `codes-below`), and its blocks are read as blocks."
  [codes-below {:keys [codes rules] :as coding}]
  (let [codes (update-keys (or codes {}) kw)
        all (merge codes-below codes)]
    (cond-> (assoc coding :codes codes)
      ;; A threshold is a probability, not money: 0.85 from EDN or JSON is read as the decimal it says.
      (:threshold coding) (update :threshold #(let [t (bigdec (str %))]
                                                (when-not (<= 0M t 1M)
                                                  (fail! (str "A threshold is how sure a coding must be, from 0 to 1, "
                                                              "such as 0.85; not " % ".")
                                                         {:threshold %}))
                                                t))
      rules (assoc :rules (mapv (fn [r] (if (false? (:enabled r)) (update r :code kw) (rule-in all r))) rules))
      (:buckets coding) (update :buckets (partial mapv (fn [b] (update b :codes (partial mapv (fn [[c t]] [(kw c) t])))))))))

(defn lines-in
  "A lines layer's data: lines chosen from the record ({:select {:accounts [source account ids] :from :to}}) or
  given as they are ({:cases [...]})."
  [{:keys [select cases] :as lines}]
  (when-not (or select cases)
    (fail! "A lines layer chooses lines from the record (:select) or gives them (:cases)." {}))
  (when select
    (when (empty? (:accounts select)) (fail! "Say which accounts' lines to read (:select :accounts)." {}))
    (some->> (:from select) (date! "The lines' :from"))
    (some->> (:to select) (date! "The lines' :to")))
  lines)

(defn key-in
  "A key layer's data, checked: lines a person has coded, the answer key a scenario's codings are scored against.
  {:lines [{:unit id :expect bucket :ok [other acceptable buckets] :review required|allowed :why words :at date}]},
  one entry a unit. Its buckets are checked against the stack's by the scenario (`check-key`)."
  [{:keys [lines] :as k}]
  (let [dupes (keep (fn [[u n]] (when (> n 1) u)) (frequencies (map :unit lines)))]
    (when (seq dupes) (fail! (str "The key names line " (first dupes) " twice.") {:unit (first dupes)}))
    (doseq [{:keys [unit expect review]} lines]
      (when-not (and (string? unit) (seq unit)) (fail! "Each line of a key names its unit (:unit)." {}))
      (when-not (seq (some-> expect name)) (fail! (str "Line " unit " of the key says no bucket (:expect).") {:unit unit}))
      (when (and review (not (#{"required" "allowed"} (name review))))
        (fail! (str "Line " unit ": :review is required (a person must look) or allowed (a person may).") {:unit unit})))
    (assoc k :lines (mapv #(cond-> (select-keys % [:unit :expect :ok :review :why :at :by])
                             (:expect %) (update :expect name)
                             (:ok %) (update :ok (partial mapv name))
                             (:review %) (update :review name))
                          lines))))

(defn check-key
  "Every bucket a key layer names is one of `codes` (the stack's), or needs-person."
  [codes {:keys [lines]}]
  (let [known (conj (set (map name (keys codes))) "needs-person")]
    (doseq [{:keys [unit expect ok]} lines
            c (cons expect ok)
            :when (not (known c))]
      (fail! (str "Line " unit " of the key says bucket " c ", which no coding layer of the scenario has.")
             {:unit unit :code c}))))

(defn- merge-rules
  "The rules of `below` with those of `above`: a rule of an id below is replaced where it stands, a new one comes
  last, and one turned off (enabled false) leaves."
  [below above]
  (let [by-id (into {} (map (juxt :id identity)) above)
        ids (set (map :id below))]
    (->> (concat (map #(get by-id (:id %) %) below) (remove (comp ids :id) above))
         (remove #(false? (:enabled %)))
         vec)))

(defn compose
  "The classifier's scenario from `layers` (bases' first, only those on): the worlds merged, the codings merged, and
  the lines layers' selections and cases gathered."
  [layers]
  (reduce (fn [acc {:keys [kind] :as l}]
            (case kind
              "world" (let [w (:world l)]
                        (-> (merge-with (fn [a b] (if (and (map? a) (map? b)) (merge a b) b))
                                        acc (dissoc w :trips))
                            (update :trips (fnil into []) (:trips w))))
              "coding" (let [{:keys [codes rules buckets threshold strategy]} (:coding l)]
                         (cond-> (update acc :codes merge codes)
                           rules (update :rules merge-rules rules)
                           buckets (assoc :buckets buckets)
                           threshold (assoc :threshold (bigdec threshold))
                           strategy (assoc :strategy (kw strategy))))
              "lines" (let [{:keys [select cases]} (:lines l)]
                        (cond-> acc
                          select (update :selects (fnil conj []) select)
                          cases (update :cases (fnil into []) cases)))
              acc))
          {:codes {} :rules [] :trips []}
          layers))

(defn- case-of
  "A bank line as the classifier reads it: money out is a purchase (a positive amount), in the merchant's currency
  when the line says it, on the day the card was used when it says that (where its holder was is that day's); the
  card, or else the account, as the key the world's :cards knows it by."
  [{:keys [cards]} currencies {:keys [xt/id date purchased-on counterparty description amount original-amount
                                     original-currency currency card account merchant-code channel]}]
  (let [spent (- (or original-amount amount))]
    (cond-> {:id id :date (str (or purchased-on date)) :amount spent
             :currency (or original-currency currency (currencies account))
             :merchant (or counterparty description "")
             :card (if (contains? cards card) card account)}
      merchant-code (assoc :mcc merchant-code)
      channel (assoc :channel (kw channel)))))

(defn read-lines
  "The lines `select` chooses from the record, oldest first, as the classifier reads them."
  [node world {:keys [accounts from to]}]
  (if-not (store/table-exists? node :bank-lines)
    []
    (let [currencies (into {} (map (juxt :xt/id :currency))
                           (when (store/table-exists? node :source-accounts)
                             (store/q node ["SELECT _id, currency FROM source_accounts"])))
          marks (str/join ", " (repeat (count accounts) "?"))
          rows (store/q node (cond-> (into [(str "SELECT * FROM bank_lines WHERE account IN (" marks ")"
                                                 (when from " AND date >= ?")
                                                 (when to " AND date <= ?")
                                                 " ORDER BY date, _id")]
                                           accounts)
                               from (conj (LocalDate/parse (str from)))
                               to (conj (LocalDate/parse (str to)))))]
      (mapv (partial case-of world currencies) rows))))

(def ^:private keyed
  "The maps in a world or a coding layer keyed by ids: kept in the store as lists of pairs, since the store turns a
  map's string keys into keywords (and an underscore into a dash)."
  {:world [:entities :cards :projects :categories] :coding [:codes]})

(defn stored
  "A layer as the store keeps it: its id-keyed maps as lists of [id value]."
  [{:keys [kind] :as layer}]
  (let [k (keyword kind)]
    (if-let [ks (keyed k)]
      (update layer k (fn [data] (reduce (fn [d f] (cond-> d (contains? d f) (update f #(vec (map vec %))))) data ks)))
      layer)))

(defn unstored
  "A layer as `stored` keeps it, back as it was given."
  [{:keys [kind] :as layer}]
  (let [k (keyword kind)]
    (if-let [ks (keyed k)]
      (update layer k (fn [data] (reduce (fn [d f] (cond-> d (contains? d f) (update f #(into {} %)))) data ks)))
      layer)))
