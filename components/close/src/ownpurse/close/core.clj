(ns ownpurse.close.core
  (:require
   [clojure.string :as str]
   [ownpurse.money.interface :as money]
   [ownpurse.schema.interface :as schema]
   [ownpurse.store.interface :as store])
  (:import
   (java.nio.charset StandardCharsets)
   (java.security MessageDigest)
   (java.time Instant LocalDate)))

(def set-aside
  "The statuses ownpurse reads: an item rejected or excluded counts nowhere. Every other status is the close
  workflow's own (accountant decision, evidence needed, ...), kept as it is given."
  #{"rejected" "excluded"})

(def ^:private metadata {:source "ownpurse" :why "close register"})

(defn- fail! [type message data]
  (throw (ex-info message (assoc data :type type))))

(def ^:private ref-pattern #"[A-Za-z0-9][A-Za-z0-9._-]{0,63}")

(defn- check-ref! [what ref]
  (when-not (and (string? ref) (re-matches ref-pattern ref))
    (fail! :ownpurse/invalid (str "A " what " needs a reference of letters, digits, dots, dashes and underscores, "
                                  "such as C-03 (its id in your close register).")
           {:ref ref})))

(defn item-id [ref] (str "close-item:" ref))
(defn- group-id [ref] (str "close-group:" ref))
(defn- package-id [ref] (str "close-package:" ref))
(defn- approval-id [ref] (str "close-approval:" ref))

(defn- rows [node table sql-and-params]
  (if (store/table-exists? node table) (store/q node sql-and-params) []))

(defn- sha-256 [s]
  (let [digest (.digest (MessageDigest/getInstance "SHA-256") (.getBytes ^String s StandardCharsets/UTF_8))]
    (apply str (map #(format "%02x" %) digest))))

(defn- plain [a]
  (when a (.toPlainString (.stripTrailingZeros (bigdec a)))))

(defn content-hash
  "The SHA-256 of what an item says: everything but its status, group, note and where its journals were posted,
  the same however its amounts are written. What a package seals."
  [{:keys [ref book title action category amount effect preconditions depends-on evidence links reason journals]}]
  (sha-256 (pr-str [ref book title action category (plain amount) effect (vec preconditions) (vec (sort depends-on))
                    (vec (sort evidence)) (vec (sort links)) reason
                    (mapv (fn [{:keys [ref book date narration postings evidence]}]
                            [ref book (str date) narration
                             (mapv (fn [{:keys [account amount commodity description]}]
                                     [account (plain amount) commodity description])
                                   postings)
                             (vec (sort evidence))])
                          journals)])))

(defn- check-journal! [item-ref {:keys [ref book date narration postings]}]
  (check-ref! "journal" ref)
  (when (or (str/blank? book) (str/blank? narration) (not (instance? LocalDate date)))
    (fail! :ownpurse/invalid (str "Journal " ref " of item " item-ref " needs a book, a date and a narration.")
           {:item item-ref :journal ref}))
  (schema/check! :ownpurse/entry
                 {:entry/id (str (item-id item-ref) ":" ref)
                  :entry/book book
                  :entry/date date
                  :entry/postings (vec (map-indexed (fn [i {:keys [account amount commodity description]}]
                                                      (cond-> {:posting/line i :posting/account account
                                                               :posting/amount amount :posting/commodity commodity}
                                                        description (assoc :posting/description description)))
                                                    postings))}))

(defn- out
  "A row as it is returned: without XTDB's columns, its amounts as BigDecimals."
  [row]
  (cond-> (dissoc row :xt/id :xt/valid-from :xt/valid-to)
    (:amount row) (update :amount bigdec)
    (:journals row) (update :journals (partial mapv (fn [j] (update j :postings (partial mapv #(update % :amount bigdec))))))))

(defn item [node ref]
  (some-> (first (rows node :close-items ["SELECT * FROM close_items WHERE _id = ?" (item-id ref)])) out))

(def ^:private item-keys
  [:ref :book :title :action :category :amount :status :effect :preconditions :depends-on :evidence :links :reason
   :group :note])

(defn put-item!
  "Adds an item or changes it. A journal posted in the book's system keeps where it was posted, and cannot change
  or go."
  [node {:keys [ref title status journals] :as given}]
  (check-ref! "close item" ref)
  (when (or (str/blank? title) (str/blank? status))
    (fail! :ownpurse/invalid "A close item needs a title and a status." {:ref ref}))
  (let [given-journals (mapv #(update % :book (fn [b] (or b (:book given)))) journals)
        _ (when-let [dupes (seq (keep (fn [[r n]] (when (> n 1) r)) (frequencies (map :ref given-journals))))]
            (fail! :ownpurse/invalid (str "Item " ref " names journal " (first dupes) " twice.") {:ref ref}))
        _ (run! (partial check-journal! ref) given-journals)
        before (into {} (map (juxt :ref identity)) (:journals (item node ref)))
        journals (mapv (fn [j]
                         (let [j (cond-> (select-keys j [:ref :book :date :narration :evidence])
                                   true (assoc :postings (mapv #(select-keys % [:account :amount :commodity :description])
                                                               (:postings j))
                                               :evidence (vec (distinct (:evidence j)))))
                               held (before (:ref j))]
                           (if-let [reference (:posted-reference held)]
                             (if (= (content-hash {:journals [j]}) (content-hash {:journals [(dissoc held :posted-reference)]}))
                               (assoc j :posted-reference reference)
                               (fail! :ownpurse/close-conflict
                                      (str "Journal " (:ref j) " of item " ref " is posted (" reference "). A correction "
                                           "is a new journal.")
                                      {:ref ref :journal (:ref j)}))
                             j)))
                       given-journals)
        gone (remove (set (map :ref journals)) (keep (fn [[r j]] (when (:posted-reference j) r)) before))]
    (when (seq gone)
      (fail! :ownpurse/close-conflict (str "Item " ref " leaves out journal " (first gone) ", which is posted.")
             {:ref ref :journal (first gone)}))
    (store/put-changed! node {:close-items [(into {:xt/id (item-id ref) :journals journals}
                                                  (remove (comp nil? val))
                                                  (update (select-keys given item-keys) :amount #(some-> % money/amount)))]}
                        metadata)
    (item node ref)))

(defn items [node {:keys [book status group]}]
  (->> (rows node :close-items ["SELECT * FROM close_items ORDER BY ref"])
       (map out)
       (filter #(and (or (nil? book) (= book (:book %)) (some (comp #{book} :book) (:journals %)))
                     (or (nil? status) (= status (:status %)))
                     (or (nil? group) (= group (:group %)))))
       vec))

(defn put-group! [node {:keys [ref title prompt note]}]
  (check-ref! "decision group" ref)
  (when (str/blank? title)
    (fail! :ownpurse/invalid "A decision group needs a title." {:ref ref}))
  (store/put-changed! node {:close-groups [(cond-> {:xt/id (group-id ref) :ref ref :title title}
                                             prompt (assoc :prompt prompt)
                                             note (assoc :note note))]}
                      metadata)
  (let [members (items node {})]
    (->> (rows node :close-groups ["SELECT * FROM close_groups WHERE _id = ?" (group-id ref)])
         first out
         (#(assoc % :items (vec (keep (fn [i] (when (= ref (:group i)) (:ref i))) members)))))))

(defn groups [node]
  (let [members (group-by :group (items node {}))]
    (mapv #(assoc (out %) :items (mapv :ref (members (:ref %))))
          (rows node :close-groups ["SELECT * FROM close_groups ORDER BY ref"]))))

(defn- all-items [node] (items node {}))

(defn- package-hash
  "One hash for a package: of its items' references and content hashes, in order."
  [hashes]
  (sha-256 (pr-str (vec (sort hashes)))))

(defn- package-row [node ref]
  (first (rows node :close-packages ["SELECT * FROM close_packages WHERE _id = ?" (package-id ref)])))

(defn- approval-row [node ref]
  (first (rows node :close-approvals ["SELECT * FROM close_approvals WHERE _id = ?" (approval-id ref)])))

(defn- changed-since
  "The refs of a package's items whose content differs from what it sealed."
  [node {:keys [items hashes]}]
  (let [now (into {} (map (juxt :ref content-hash)) (filter (comp (set items) :ref) (all-items node)))]
    (vec (keep (fn [[r h]] (when (not= h (now r)) r)) (map vector items hashes)))))

(defn- package-out [node row]
  (when row
    (let [approval (approval-row node (:ref row))]
      (cond-> (-> (out row) (assoc :changed (changed-since node row)))
        approval (assoc :approval (out approval))))))

(defn package [node ref]
  (package-out node (package-row node ref)))

(defn packages [node]
  (mapv (partial package-out node) (rows node :close-packages ["SELECT * FROM close_packages ORDER BY ref"])))

(defn seal!
  "Seals a package: the items it puts to the owner, each item's content hash, and one hash over them all."
  [node {:keys [ref as-of scope items coverage note]}]
  (check-ref! "package" ref)
  (when (empty? items)
    (fail! :ownpurse/invalid "A package names the items it puts to the owner." {:ref ref}))
  (when (package-row node ref)
    (fail! :ownpurse/close-conflict (str "There is already a package " ref ". A sealed package is never changed; seal "
                                         "a new one.")
           {:ref ref}))
  (let [held (mapv #(or (item node %) (fail! :ownpurse/not-found (str "There is no close item " % ".") {:item %})) items)
        hashes (mapv content-hash held)]
    (store/put-changed! node {:close-packages [(cond-> {:xt/id (package-id ref) :ref ref :items (vec items)
                                                        :hashes hashes :hash (package-hash hashes)
                                                        :sealed-at (Instant/now)}
                                                 as-of (assoc :as-of as-of)
                                                 scope (assoc :scope scope)
                                                 coverage (assoc :coverage coverage)
                                                 note (assoc :note note))]}
                        (assoc metadata :why "close register: package sealed"))
    (package node ref)))

(defn approve!
  "Records the owner's approval of a sealed package, given its hash as the owner saw it. Refused if any of its
  items changed since it was sealed."
  [node {:keys [package hash by note]}]
  (when (or (str/blank? by) (str/blank? hash))
    (fail! :ownpurse/invalid "An approval says who approves (by) and the package's hash, as they saw it." {:package package}))
  (let [row (or (package-row node package)
                (fail! :ownpurse/not-found (str "There is no package " package ".") {:package package}))
        changed (changed-since node row)]
    (when-not (= hash (:hash row))
      (fail! :ownpurse/close-conflict (str "The hash given is not package " package "'s. Read the package again and "
                                           "approve what it holds.")
             {:package package}))
    (when (seq changed)
      (fail! :ownpurse/close-conflict (str "Items changed since package " package " was sealed: "
                                           (str/join ", " changed) ". Seal a new package.")
             {:package package :changed changed}))
    (when (approval-row node package)
      (fail! :ownpurse/close-conflict (str "Package " package " is already approved.") {:package package}))
    (store/put-changed! node {:close-approvals [(cond-> {:xt/id (approval-id package) :package package :hash hash
                                                         :by by :at (Instant/now)}
                                                  note (assoc :note note))]}
                        (assoc metadata :why "close register: package approved" :by by))
    (package-out node (package-row node package))))

(defn- approved-hashes
  "{item-ref content-hash} for each item in an approved package, as approved."
  [node]
  (into {}
        (for [row (rows node :close-packages ["SELECT * FROM close_packages"])
              :when (approval-row node (:ref row))
              [r h] (map vector (:items row) (:hashes row))]
          [r h])))

(defn approved?
  "Whether an item is approved as it stands: in an approved package, unchanged since."
  ([node it] (approved? (approved-hashes node) node it))
  ([approved _node it] (= (approved (:ref it)) (content-hash it))))

(defn mark-posted!
  "Records where an approved item's journal was posted in its book's system: {:item :journal :reference}."
  [node {item-ref :item journal :journal reference :reference}]
  (when (str/blank? reference)
    (fail! :ownpurse/invalid "Say where it was posted: the journal's number or reference in the book's system."
           {:item item-ref}))
  (let [it (or (item node item-ref) (fail! :ownpurse/not-found (str "There is no close item " item-ref ".") {:item item-ref}))]
    (when-not (approved? node it)
      (fail! :ownpurse/close-conflict (str "Item " item-ref " is not approved as it stands; only an approved item's "
                                           "journals are posted.")
             {:item item-ref}))
    (when-not (some #(= journal (:ref %)) (:journals it))
      (fail! :ownpurse/not-found (str "Item " item-ref " has no journal " journal ".") {:item item-ref :journal journal}))
    (store/put-changed! node {:close-items [(-> it
                                                (assoc :xt/id (item-id item-ref))
                                                (update :journals (partial mapv #(cond-> %
                                                                                   (= journal (:ref %))
                                                                                   (assoc :posted-reference reference)))))]}
                        (assoc metadata :why "close register: journal posted"))
    (item node item-ref)))

(defn balances
  "The sum of a book's journals not yet posted, by account. `basis` :approved counts the journals of items approved
  as they stand; :proposed counts those of every item not rejected or excluded."
  [node book {:keys [from to basis] :or {basis :approved}}]
  (let [approved (approved-hashes node)]
    (reduce (fn [acc {:keys [postings]}]
              (reduce (fn [acc {:keys [account amount]}] (update acc account (fnil + 0M) amount)) acc postings))
            {}
            (for [it (items node {})
                  :when (case basis
                          :approved (approved? approved node it)
                          :proposed (not (set-aside (:status it))))
                  j (:journals it)
                  :when (and (= book (:book j))
                             (not (:posted-reference j))
                             (or (nil? from) (not (.isBefore ^LocalDate (:date j) from)))
                             (or (nil? to) (not (.isAfter ^LocalDate (:date j) to))))]
              j))))
