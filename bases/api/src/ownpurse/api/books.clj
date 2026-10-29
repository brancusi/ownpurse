(ns ownpurse.api.books
  "What the reports of a Xero org's book are made from, gathered from the bricks: its trial balance check with the
  close's adjustments, its chart, parties, links and payees. The API's export and the notebooks both start here."
  (:require
   [ownpurse.entity.interface :as entity]
   [ownpurse.payees.interface :as payees]
   [ownpurse.scenario.interface :as scenario]
   [ownpurse.xero.interface :as xero]))

(defn xero-payees
  "A Xero org's payees: its bank and credit card accounts are the payment accounts, its contacts the parties."
  [node {:keys [org from to threshold]}]
  (let [accounts (into {} (for [a (entity/source-accounts node {:source "xero"})
                                :when (and (= org (:org a)) (#{"bank" "credit-card"} (:kind a)))]
                            [(:id a) (:kind a)]))]
    (payees/payees node (cond-> {:book (xero/book-id org) :from from :to to :accounts accounts
                                 :parties (xero/parties node org)}
                          threshold (assoc :threshold threshold)))))

(defn xero-orgs
  "The Xero orgs in the record: [{:org :name}], by name."
  [node]
  (->> (entity/source-accounts node {:source "xero"})
       (filter #(= "org" (:kind %)))
       (map #(select-keys % [:org :name]))
       (sort-by :name)
       vec))

(defn xero-book
  "The opts of ownpurse.export.interface/reports (all but :dir) for Xero org `org`'s book, for the financial year
  ending `date`, read through scenario `scenario` (or `pro-forma`, the same thing: \"approved\", the default, or
  \"proposed\"); `opening` is \"balance-sheet\" (the
  default) or \"ledger\"."
  [node {:keys [org date pro-forma scenario opening]}]
  (let [book (xero/book-id org)
        basis (or scenario pro-forma "approved")
        _ (when-not (scenario/scenario node basis)
            (throw (ex-info (str "There is no scenario " basis ". GET /scenarios lists them.")
                            {:type :ownpurse/not-found :scenario basis})))
        tb (xero/check-trial-balance node org date {:opening (keyword (or opening "balance-sheet"))
                                                    :adjustments #(scenario/balances node basis book %)})]
    {:name (or (some #(when (= org (:org %)) (:name %)) (xero-orgs node)) org)
     :book book :from (:year-start tb) :to date :basis basis
     :trial-balance tb :accounts (xero/chart node org) :parties (xero/parties node org)
     :links (xero/document-links node org)
     :payees (xero-payees node {:org org :from (:year-start tb) :to date})}))
