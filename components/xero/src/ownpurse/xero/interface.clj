(ns ownpurse.xero.interface
  "Xero. The only brick that knows about Xero. It runs xero-cli, which talks to Xero, and keeps what Xero said as
  source rows in the xero schema (xero.invoices, xero.contacts, ...). It never calls Xero itself."
  (:require
   [ownpurse.xero.cli :as cli]
   [ownpurse.xero.pull :as pull]
   [ownpurse.xero.source :as source]))

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
  "Fetches and keeps each resource in `resources` for one org: {:cli :tenant :resources}. After the first fetch, a
  resource with UpdatedDateUTC asks only for what changed. Returns {resource {:items :written}}."
  [node opts]
  (pull/pull! node opts))

(defn read-orgs
  "The orgs named in a local config file, {:xero {:orgs [{:tenant :name}]}}. Throws :ownpurse/no-local-config,
  saying how to make the file, when it is missing or names none."
  [path]
  (pull/read-orgs path))

(defn pull-orgs!
  "Pulls every resource in `pull-order` for each org: {:cli :orgs}. Returns {name {resource {:items :written}}}."
  [node opts]
  (pull/pull-orgs! node opts))

(defn read-answers
  "Reads a folder of Xero answers saved one per resource (Invoices.json, Organisations.json), as the synthetic
  samples are. Returns {resource answer}."
  [dir]
  (pull/read-answers dir))
