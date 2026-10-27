;; # A book's reports
;;
;; The reports of one Xero org's book for a financial year, from the record the REPL said to read: the same tables
;; the accountant's export writes, with the close's adjustments beside the system's figures. Show it from the REPL:
;;
;; ```clojure
;; (ownpurse.notebook/show! :profit-and-loss {:org "Tallowmere" :date "2026-03-31"})
;; (ownpurse.notebook/show! :book {:org "Tallowmere" :date "2026-03-31" :reports [:trial-balance :balance-sheet]})
;; ```
(ns ownpurse.notebooks.book
  {:nextjournal.clerk/visibility {:code :hide :result :show}
   :nextjournal.clerk/no-cache true}
  (:require
   [clojure.string :as str]
   [nextjournal.clerk :as clerk]
   [ownpurse.api.books :as books]
   [ownpurse.export.interface :as export]
   [ownpurse.notebook :as notebook])
  (:import
   (java.time LocalDate)))

{:nextjournal.clerk/visibility {:result :hide}}

(def params (notebook/params))

(def node (notebook/record))

(def orgs (books/xero-orgs node))

(def org
  (let [wanted (:org params)
        starts? #(str/starts-with? (str/lower-case (str (:name %))) (str/lower-case (str wanted)))]
    (cond
      (nil? wanted) (first orgs)
      :else (or (some #(when (= wanted (:org %)) %) orgs) (some #(when (starts? %) %) orgs)))))

(def date
  (LocalDate/parse (or (:date params) (str (dec (.getYear (LocalDate/now))) "-12-31"))))

(def opts
  (when org
    (books/xero-book node {:org (:org org) :date date :pro-forma (:pro-forma params)})))

(def shown
  (let [wanted (set (:reports params))]
    (filterv #(or (empty? wanted) (wanted (key %))) notebook/reports)))

{:nextjournal.clerk/visibility {:result :show}}

(if-not opts
  (clerk/md (str "No Xero org in the record matches `" (pr-str (:org params)) "`. The orgs it holds: "
                 (str/join ", " (map #(str (:name %) " (`" (:org %) "`)") orgs)) "."))
  (let [{:keys [trial-balance basis from to]} opts]
    (clerk/md (str "## " (:name opts) "\n\n"
                   from " to " to ". "
                   (if (:tied? trial-balance)
                     "The book ties to Xero's trial balance, account by account."
                     "**The book does not tie to Xero's trial balance:** see the differences in the trial balance.")
                   " Adjustments counted: the close's journals "
                   (if (= "proposed" basis) "not set aside (a draft)." "approved as they stand.")))))

(when opts
  (clerk/fragment
   (for [[_ sheet] shown
         :let [table (export/report node opts sheet)]
         :when table]
     (clerk/fragment (clerk/md (str "### " sheet)) (notebook/table table)))))

(let [others (map :name (remove #(= (:org %) (:org org)) orgs))]
  (clerk/md (str (when (seq others) (str "Other books: " (str/join ", " others) ". "))
                 "Reports: " (str/join ", " (map #(str "`" (key %) "`") notebook/reports)) ".")))
