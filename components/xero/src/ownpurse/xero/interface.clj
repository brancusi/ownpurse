(ns ownpurse.xero.interface
  "Xero. The only brick that knows about Xero. It runs xero-cli, which talks to Xero, and keeps what Xero said as
  source rows in the xero schema (xero.invoices, xero.contacts, ...). It never calls Xero itself."
  (:require
   [ownpurse.xero.cli :as cli]
   [ownpurse.xero.ledger :as ledger]
   [ownpurse.xero.pull :as pull]
   [ownpurse.xero.source :as source]
   [ownpurse.xero.trial-balance :as trial-balance]))

(def resources
  "The Xero resources ownpurse keeps, by Xero's name, each with its table (`:xero/invoices` is xero.invoices)."
  source/resources)

(def pull-order
  "The resources a pull asks for, in order: the org and its lists first, then the documents."
  source/pull-order)

(def exit-types
  "xero-cli's exit codes, as the :type of the error a fetch throws: 1 :xero/usage, 2 :xero/login-needed,
  3 :xero/daily-limit, 4 :xero/refused, 5 :xero/failed."
  cli/exit-types)

(defn rows
  "Xero's answer to `resource` for org `tenant` as rows for its table, {table [rows]}: each item as Xero sent it,
  in kebab case, with Xero's dates as dates, its row id derived from Xero's id, the org as `org`, and valid from
  its UpdatedDateUTC (never later than `fetched-at`). Pure."
  [tenant resource answer fetched-at]
  (source/rows tenant resource answer fetched-at))

(defn fetch
  "Runs `xero-cli get <resource> --org <tenant> --all` and returns Xero's answer, every number exact. `cli` is the
  program (a path or a vector); `opts` may hold :modified-since (an Instant). Throws one of `exit-types`."
  [cli tenant resource opts]
  (cli/get-resource cli tenant resource opts))

(defn keep!
  "Keeps Xero's answer to `resource` for org `tenant`, writing only the rows that changed, in one transaction
  whose metadata names the tool, the org, the resource and when Xero answered. `opts` may hold :tool-version,
  :modified-since and :now (used when the answer has no DateTimeUTC). Returns {:items n :written n}."
  [node tenant resource answer opts]
  (pull/keep! node tenant resource answer opts))

(defn pull!
  "Fetches and keeps each resource in `resources` for one org, then its reports, then its documents' history and
  attachments, then (when :history asks for :files) the attachments' files: {:cli :tenant :resources :reports
  :history :today :keep-file}. `keep-file` is store/keep-file! over the store's config. After the first fetch, a
  resource with UpdatedDateUTC asks only for what changed. Returns {resource {:items :written}}, with {:items
  :written :waiting} under \"History\" and \"Files\"."
  [node opts]
  (pull/pull! node opts))

(defn read-orgs
  "The orgs named in a local config file, {:xero {:orgs [{:tenant :name}]}}, or [] when it names none. Throws
  :ownpurse/no-local-config, saying how to make the file, when it is missing."
  [path]
  (pull/read-orgs path))

(def default-reports
  "The reports a pull keeps unless the local config says otherwise: {:months 13 :refresh 2}, the months ending with
  this one, the latest of them fetched again on every pull."
  pull/default-reports)

(defn report-requests
  "The reports a pull asks for over `months` months ending with the month of `today`, newest first: for each, the
  TrialBalance and BalanceSheet at its last day, the ProfitAndLoss on an accrual and on a cash basis, and the
  BankSummary. Each is {:report :params :month-end}. Pure."
  [today months]
  (source/report-requests today months))

(defn read-reports
  "Which reports a local config file asks for, {:xero {:reports {:months n :refresh n}}}, over `default-reports`.
  {:months 0} asks for none."
  [path]
  (pull/read-reports path))

(def default-history
  "What a pull spends on documents' history and attachment details unless the local config says otherwise: at most
  {:calls 100} per org, and {:files 0} downloads of attachments' files, on {:resources [\"BankTransactions\"
  \"ManualJournals\" \"Invoices\"]} in that order."
  pull/default-history)

(defn read-history
  "How a local config file limits the documents' history and attachments, {:xero {:history {:calls n :files n
  :resources [...] :first {:from \"2025-01-01\" :to \"2025-12-31\"}}}}, over `default-history`. {:calls 0} fetches
  none; {:files n} downloads up to n files a pull; :first fetches the documents dated in that range before the rest."
  [path]
  (pull/read-history path))

(defn pull-orgs!
  "Pulls every resource in `pull-order`, then the reports, then the documents' history and attachments, for each
  org, and their files when asked for: {:cli :orgs :reports :history :today :keep-file}. Reports are rows of xero.reports, one per report, org and parameters,
  and a new version when Xero's figures change. A document's history and its attachments' details are rows of
  xero.history and xero.attachments, fetched for each version of it the store holds, within a cap per pull; a file
  is kept in the record and named by a row of xero.files. Returns {name {resource {:items :written}}}, with the
  reports under \"Reports\", the history under \"History\" and the files under \"Files\"."
  [node opts]
  (pull/pull-orgs! node opts))

(defn read-answers
  "Reads a folder of Xero answers saved one per resource (Invoices.json, Organisations.json), as the synthetic
  samples are. Returns {resource answer}."
  [dir]
  (pull/read-answers dir))

(defn book-id
  "The id of org `tenant`'s book, as its entries and postings name it."
  [tenant]
  (ledger/book-id tenant))

(defn entries
  "The entries of org `tenant`'s documents by Xero's own rules (see ownpurse.xero.ledger), from its rows by table
  ({:accounts :bank-transactions :bank-transfers :manual-journals :invoices :credit-notes :payments}), in its
  currency `base`. Returns {:entries :unbalanced :skipped}. Pure."
  [tenant base docs]
  (ledger/entries tenant base docs))

(defn post!
  "Makes org `tenant`'s entries those of the documents the store holds now. A pull does this after it keeps the
  documents. Returns {:items :written :ended}, with :unbalanced when some documents did not balance."
  [node tenant]
  (pull/post! node tenant))

(defn check-trial-balance
  "Org `tenant`'s book against Xero's trial balance at `date`, account by account, to the cent. opts: :opening,
  :balance-sheet (the default: the balance sheet accounts at the start of the year from Xero's balance sheet at the
  day before) or :ledger (from the book's postings). Returns {:date :year-start :opening :tied? :accounts :other}."
  [node tenant date opts]
  (trial-balance/check node tenant date opts))
