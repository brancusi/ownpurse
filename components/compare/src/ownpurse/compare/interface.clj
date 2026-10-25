(ns ownpurse.compare.interface
  "The compare of an account with the account it is evidence for: which lines each has that the other has not.
  Reads bank_lines and the links; knows nothing about any one source."
  (:require
   [ownpurse.compare.core :as core]))

(def default-days
  "How many days apart two lines of the same amount may be and still be the same money."
  core/default-days)

(defn match
  "Matches `evidence` lines with `book` lines ({:xt/id :date :amount}): each evidence line, in date order, with the
  unmatched book line of the same amount closest in date, at most `days` apart. Returns {:matched [[evidence
  book]] :evidence-only [lines] :book-only [lines]}. Pure."
  [evidence book days]
  (core/match evidence book days))

(defn compare-accounts
  "Compares each account linked as evidence for an account of its entity's system of record (or only `:account`)
  with that account, over the lines dated from `:from` to `:to` (dates; either may be nil), matching lines of the
  same amount at most `:days` apart (default-days). Returns, for each link: {:entity :account :evidence-for
  :matched n :missing-from-book [lines] :missing-from-evidence [lines] :pending [lines] :totals {:evidence :book}},
  amounts as plain decimal strings. A pending line is listed apart: it may not have reached the book yet. Throws
  :ownpurse/not-found when `:account` is not linked as evidence for an account."
  [node opts]
  (core/compare-accounts node opts))
