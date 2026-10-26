(ns ownpurse.statement.interface
  "Statement files: CSV exports of accounts no API reaches, read through a column mapping per format that the local
  config holds, and kept as source rows in the statement schema (statement.files, statement.lines). Each statement
  account is a source account, to link as evidence for a book's bank account; its lines are bank lines, for the
  compare. The only brick that reads statement files."
  (:require
   [ownpurse.statement.import :as import]
   [ownpurse.statement.source :as source]))

(def tables
  "The tables ownpurse keeps statement files in."
  source/tables)

(defn parse
  "The lines of CSV `text` read through format `fmt`: {:columns :lines [{:row :date :amount :description :reference
  :balance}]}, money in positive. See ownpurse.statement.source/parse for the format's keys."
  [fmt text]
  (source/parse fmt text))

(defn read-config
  "The :statements of a local config file ({:formats :accounts :files}), or nil."
  [path]
  (import/read-config path))

(defn import!
  "Reads every file `config` names and keeps what changed, with each account's source account and bank lines.
  opts: :now. Returns {:files :lines :written}."
  [node config opts]
  (import/import! node config opts))

(defn statements
  "Every statement file held, with its opening, closing, lines' total, running balance breaks and :tied?."
  [node]
  (import/statements node))
