(ns ownpurse.entity.interface
  "The entities you own, and which source accounts belong to each (docs/MODEL.md, \"Entities and their sources\").
  Knows nothing about any one source: each source writes its own accounts into source_accounts when it pulls, and
  links name them by id."
  (:require
   [ownpurse.entity.core :as core]))

(defn put-entity!
  "Makes or renames an entity: {:id \"tallowmere\" :name \"...\" :kind \"business\"|\"household\"}. Returns it.
  Throws :ownpurse/invalid when it does not match :ownpurse/entity."
  [node entity]
  (core/put-entity! node entity))

(defn entities
  "Every entity, each with the links it has now: [{:id :name :kind :links [{:account :role :evidence-for :from}]}]."
  [node]
  (core/entities node))

(defn links
  "Every link there is now: [{:entity :account :role :evidence-for :from}], by entity and account."
  [node]
  (core/links node))

(defn source-accounts
  "Every source account, each with its link when it has one. opts: :source (\"mercury\"), :unlinked (true: only
  those not linked)."
  [node opts]
  (core/source-accounts node opts))

(defn source-account
  "The source account `id` as the store holds it now, or nil."
  [node id]
  (core/source-account node id))

(defn link!
  "Links a source account to an entity: {:entity slug :account id :role \"system-of-record\"|\"evidence\"
  :evidence-for id :from date}, valid from `:from` (default now). Linking it again to the same entity changes the
  link. Returns the link. Throws :ownpurse/not-found (no such entity or account), :ownpurse/invalid-link (a rule of
  the model broken) or :ownpurse/link-conflict (the account belongs to another entity, or the entity has another
  system of record), each saying how to fix it."
  [node link]
  (core/link! node link))

(defn unlink!
  "Ends the link of `:account` from the start of `:on` (a date; default now), keeping its history. Returns the link
  with :until. Throws :ownpurse/not-found when it is not linked."
  [node opts]
  (core/unlink! node opts))
