(ns ownpurse.close.interface
  "The close register: the items a year's close must decide (questions whose answers may become journals), the
  journals they propose, decision groups that put items to one conversation, and packages the owner approves whole.
  An item counts as approved when it is in an approved package and has not changed since; its statuses are the close
  workflow's own labels. Nothing here posts anywhere: a journal is marked posted once a person posts it in the
  book's system of record. Knows nothing about any one source."
  (:require
   [ownpurse.close.core :as core]))

(def set-aside
  "The item statuses ownpurse reads: rejected and excluded items count nowhere."
  core/set-aside)

(defn content-hash
  "The SHA-256 of what an item says (all but its status, group, note and where its journals were posted), however
  its amounts are written: what a package seals."
  [item]
  (core/content-hash item))

(defn put-item!
  "Adds a close item or changes it: {:ref :title :status (required) :book :action :category :amount :effect
  :preconditions [...] :depends-on [refs] :evidence [...] :links [older decision ids] :reason (for an exclusion)
  :group :note :journals [{:ref :book (default the item's) :date :narration :postings [{:account :amount :commodity
  :description}] :evidence}]}. Each journal must balance in every commodity. A posted journal cannot change or go.
  Returns the item."
  [node item]
  (core/put-item! node item))

(defn item
  "The close item `ref`, or nil."
  [node ref]
  (core/item node ref))

(defn items
  "The close items, by ref: {:book (the item's, or any of its journals') :status :group}, each optional."
  [node opts]
  (core/items node opts))

(defn put-group!
  "Adds or changes a decision group: {:ref :title :prompt :note}. Items join it by their :group. Returns it, with
  its items."
  [node group]
  (core/put-group! node group))

(defn groups
  "The decision groups, each with its items."
  [node]
  (core/groups node))

(defn seal!
  "Seals a package for the owner: {:ref :items [refs] :as-of :scope :coverage {...} :note}. It keeps each item's
  content hash and one hash over them all, and is never changed. Returns it."
  [node package]
  (core/seal! node package))

(defn approve!
  "Records the owner's approval of the sealed package: {:package :hash (as the owner saw it) :by :note}. Refused when
  the hash is not the package's, or an item changed since it was sealed. Returns the package."
  [node approval]
  (core/approve! node approval))

(defn package
  "The package `ref` with its approval, if any, and :changed, the items that differ from what it sealed."
  [node ref]
  (core/package node ref))

(defn packages
  "Every package, as `package` returns it."
  [node]
  (core/packages node))

(defn approved?
  "Whether an item is approved as it stands."
  [node item]
  (core/approved? node item))

(defn mark-posted!
  "Records where an approved item's journal was posted in its book's system: {:item :journal :reference}. Returns
  the item."
  [node opts]
  (core/mark-posted! node opts))

(defn balances
  "{account amount}: the sum of book `book`'s journals not yet posted, dated from `from` to `to` (either may be
  nil). `basis` :approved (the default) counts items approved as they stand; :proposed every item not set aside."
  [node book opts]
  (core/balances node book opts))
