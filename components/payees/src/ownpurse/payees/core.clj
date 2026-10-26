(ns ownpurse.payees.core
  (:require
   [ownpurse.ledger.interface :as ledger])
  (:import
   (java.time LocalDate)))

(defn default-threshold
  "The amount paid in a year above which a payee may need a 1099-NEC or 1099-MISC: 600.00 for payments made up to
  2025, 2000.00 from 2026. A default for the list only; the accountant decides."
  [year]
  (if (<= year 2025) 600.00M 2000.00M))

(defn payees [node {:keys [book ^LocalDate from to accounts parties threshold]}]
  (let [threshold (or threshold (default-threshold (.getYear ^LocalDate (or to from))))
        lines (for [[account kind] accounts
                    p (ledger/postings node book account {:from from :to to})
                    :when (:party p)]
                (assoc p :kind kind))
        rows (for [[party ls] (group-by :party lines)
                   :let [paid (reduce (fn [acc {:keys [kind amount]}] (update acc kind (fnil - 0M) amount)) {} ls)
                         by-bank (get paid "bank" 0M)
                         info (get parties party)]
                   :when (some pos? (vals paid))]
               (cond-> {:party party
                        :name (or (:name info) party)
                        :paid paid
                        :payments (count (filter (comp neg? :amount) ls))
                        :candidate? (>= by-bank threshold)}
                 (contains? info :supplier?) (assoc :supplier? (:supplier? info))
                 (:tax-number info) (assoc :tax-number (:tax-number info))))]
    {:book book :from from :to to :threshold threshold
     :payees (vec (sort-by (juxt (comp - #(get-in % [:paid "bank"] 0M)) :name) rows))}))
