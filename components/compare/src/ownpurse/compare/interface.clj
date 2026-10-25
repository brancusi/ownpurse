(ns ownpurse.compare.interface
  "The compare of an account with the account it is evidence for: which lines each has that the other has not.
  Reads bank_lines and the links; knows nothing about any one source."
  (:require
   [ownpurse.compare.core :as core]))

(def default-days
  "How many days apart two lines of the same amount may be and still be the same money."
  core/default-days)

(defn match
  "Matches `evidence` lines with `book` lines ({:xt/id :date :amount :check-number}): first each line with a check
  number, with the book line of the same check number and amount, whatever their dates; then each other evidence
  line, in date order, with the unmatched book line of the same amount closest in date, at most `days` apart.
  Before both, the pairs a person asserted, `manual` ([[evidence-id book-id]]). Returns {:matched [{:evidence
  :book :by \"manual\"|\"check\"|\"amount\"}] :evidence-only [lines] :book-only [lines]}. Pure."
  ([evidence book days] (core/match evidence book days))
  ([evidence book days manual] (core/match evidence book days manual)))

(defn compare-accounts
  "Compares each account linked as evidence for an account of its entity's system of record (or only `:account`)
  with that account, over the lines dated from `:from` to `:to` (dates; either may be nil), pairing the lines a
  person matched first, then by check number, then by the same amount at most `:days` apart (default-days).
  Returns, for each link: {:entity :account :evidence-for :matched n :matched-by {:manual :check :amount}
  :missing-from-book [lines] :missing-from-evidence [lines] :pending [lines] :totals {:evidence :book}}, amounts
  as plain decimal strings; with `:pairs` true, also :pairs [{:by :evidence :book :note}]. A pending line is
  listed apart: it may not have reached the book yet. Throws :ownpurse/not-found when `:account` is not linked as
  evidence for an account."
  [node opts]
  (core/compare-accounts node opts))

(defn put-match!
  "Asserts that two lines are the same money: {:evidence <source row of the evidence line> :book <source row of the
  book line> :note \"...\" :from date}. The compare pairs them first. The lines must be on an evidence account and
  the account it is evidence for, and the same amount; each line is in one match at a time. Returns the match.
  Throws :ownpurse/not-found, :ownpurse/invalid-match or :ownpurse/match-conflict, each saying how to fix it."
  [node match]
  (core/put-match! node match))

(defn end-match!
  "Ends the match of the evidence line `:evidence` (a source row) from the start of `:on` (default now), keeping
  its history. Returns the match with :until."
  [node opts]
  (core/end-match! node opts))
