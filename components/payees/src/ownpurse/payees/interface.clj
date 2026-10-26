(ns ownpurse.payees.interface
  "Who a book paid, and how: money out of its bank accounts and its credit cards, by party, and which payees may need
  a 1099 (paid by bank above the year's threshold; a card payment is reported by the card's processor). A list
  for the accountant, not a filing. Knows nothing about any one source."
  (:require
   [ownpurse.payees.core :as core]))

(defn default-threshold
  "The year's default 1099 threshold: 600.00 for payments up to 2025, 2000.00 from 2026."
  [year]
  (core/default-threshold year))

(defn payees
  "Book `book`'s payees from `from` to `to`: {:book :from :to :accounts {account kind} (the book's payment accounts,
  kind \"bank\" or \"credit-card\") :parties {party {:name :tax-number :supplier?}} :threshold (default the year's)}.
  Returns {:book :from :to :threshold :payees [{:party :name :paid {kind amount} :payments :candidate? :supplier?
  :tax-number}]}, paid net of refunds, the largest paid by bank first. A payment with no party is not counted."
  [node opts]
  (core/payees node opts))
