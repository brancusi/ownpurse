(ns ownpurse.export.interface
  "The accountant's export of one book: a workbook of reports (README, trial balance, balance sheet, profit and
  loss, general ledger, adjustments, exclusions, bank reconciliation, unreconciled lines, intercompany, payees),
  and the same sheets as CSV files beside it. Amounts are exact decimals, debits positive; never a float, never a
  formula. Knows nothing about any one source: the caller gives what is the source's."
  (:require
   [ownpurse.export.core :as core]))

(defn reports
  "The export's reports, in order: [[sheet-name {:columns [...] :rows [[cell ...] ...]}] ...]. opts as `export!`."
  [node opts]
  (core/reports node opts))

(defn export!
  "Writes the export into a new folder under `dir` (an absolute path), named after the book and the period's end:
  the workbook, and a folder of CSV files. opts: {:dir :name (the entity's) :book :from :to :basis (approved or
  proposed) :trial-balance (the trial balance check at `to`, with adjustments) :accounts {account {:code :name
  :class}} :parties {party {:name}} :links {document url} :payees (as ownpurse.payees.interface/payees returns, or
  nil)}. Refuses a folder that exists. Returns {:dir :workbook :sheets [{:name :rows}]}."
  [node opts]
  (core/export! node opts))
