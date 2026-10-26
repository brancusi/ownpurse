(ns ownpurse.api.handler
  "The HTTP API: routes, schemas and the Ring handler. Thin: it turns requests into interface calls and back."
  (:require
   [clojure.java.io :as io]
   [malli.error :as me]
   [malli.util :as mu]
   [ownpurse.api.json :as api.json]
   [ownpurse.close.interface :as close]
   [ownpurse.compare.interface :as cmp]
   [ownpurse.entity.interface :as entity]
   [ownpurse.intercompany.interface :as ic]
   [ownpurse.json.interface :as json]
   [ownpurse.mercury.interface :as mercury]
   [ownpurse.plaid.interface :as plaid]
   [ownpurse.schema.interface :as schema]
   [ownpurse.store.interface :as store]
   [ownpurse.xero.interface :as xero]
   [reitit.coercion.malli :as rcm]
   [reitit.openapi :as openapi]
   [reitit.ring :as ring]
   [reitit.ring.coercion :as coercion]
   [reitit.ring.middleware.exception :as exception]
   [reitit.ring.middleware.muuntaja :as muuntaja]
   [reitit.ring.middleware.parameters :as parameters]))

(def ApiError
  "Every error the API returns."
  [:map [:error [:map [:type :string] [:message :string]]]])

(def Snapshot
  "A snapshot of the record: where it is, when it was taken, and what it holds."
  [:map
   [:dir :string]
   [:taken-at :string]
   [:block :int]
   [:epoch :int]
   [:transactions :int]
   [:tables :int]
   [:files :int]
   [:bytes :int]])

(def Health
  "What GET /health returns: whether the record answers, and its last snapshot when it has one."
  [:map
   [:status [:enum "ok" "unavailable"]]
   [:last-snapshot {:optional true} (mu/select-keys Snapshot [:dir :taken-at :block :transactions])]])

(def BackupBody
  "POST /backup: the folder to put the snapshot in, as an absolute path."
  [:map {:closed true} [:dir [:string {:min 1}]]])

(def Counts
  "For each org, for each resource: the items the source returned and the rows written. Under \"History\", also the
  calls left for later pulls, the documents Xero refused, and the documents looked at again."
  [:map-of :string [:map-of :string [:map [:items :int] [:written :int] [:waiting {:optional true} :int]
                                     [:refused {:optional true} :int] [:rechecked {:optional true} :int]
                                     [:ended {:optional true} :int] [:unbalanced {:optional true} :int]]]])

(def Pulled
  "What POST /pull returns: the counts for each source's orgs."
  [:map [:pulled [:map [:xero Counts] [:mercury Counts] [:plaid Counts]]]])

(def Entity
  "An entity: one business or household."
  [:map [:id :string] [:name :string] [:kind :string]])

(def Link
  "A link of a source account to an entity, from a day (and, once ended, until one)."
  [:map
   [:entity :string]
   [:account {:optional true} :string]
   [:role :string]
   [:evidence-for {:optional true} :string]
   [:from {:optional true} :string]
   [:until {:optional true} :string]])

(def Entities
  "What GET /entities returns: every entity with the links it has now."
  [:map [:entities [:vector (conj Entity [:links [:vector Link]])]]])

(def SourceAccount
  "Something a source holds that can belong to an entity, with its link when it has one."
  [:map
   [:id :string] [:source :string] [:source-table :string] [:org :string] [:kind :string] [:name :string]
   [:code {:optional true} :string] [:currency {:optional true} :string] [:status {:optional true} :string]
   [:link {:optional true} Link]])

(def Line
  "A line on an account, as the compare lists it. Amounts are decimal strings."
  [:map
   [:id :string] [:date :string] [:amount :string] [:status :string] [:source-row :string]
   [:counterparty {:optional true} :string] [:description {:optional true} :string]
   [:reference {:optional true} :string] [:check-number {:optional true} :string]])

(def Compare
  "The compare of one evidence account with the account it is evidence for."
  [:map
   [:entity :string] [:account :string] [:account-name {:optional true} [:maybe :string]]
   [:evidence-for :string] [:evidence-for-name {:optional true} [:maybe :string]]
   [:from {:optional true} :string] [:to {:optional true} :string] [:days :int]
   [:matched :int]
   [:matched-by [:map [:manual :int] [:check :int] [:amount :int]]]
   [:pairs {:optional true} [:vector [:map [:by :string] [:evidence Line] [:book Line] [:note {:optional true} :string]]]]
   [:missing-from-book [:vector Line]]
   [:missing-from-evidence [:vector Line]]
   [:pending [:vector Line]]
   [:totals [:map [:evidence :string] [:book :string]]]])

(def TrialBalance
  "An org's book against Xero's trial balance at a date. Amounts are decimal strings, debits positive."
  [:map
   [:org :string] [:date :string] [:year-start :string] [:opening :string] [:tied :boolean]
   [:pro-forma {:optional true} [:enum "approved" "proposed"]]
   [:accounts [:vector [:map [:account :string] [:code {:optional true} :string] [:name :string]
                        [:xero :string] [:ownpurse :string] [:difference :string]
                        [:adjustments {:optional true} :string] [:pro-forma {:optional true} :string]]]]
   [:other [:vector [:map [:name [:maybe :string]] [:amount :string]]]]])

(defn- amount-out [^BigDecimal a]
  (.toPlainString a))

(defn- trial-balance-out
  "A trial balance check as the API returns it: dates and amounts as strings."
  [org pro-forma {:keys [date year-start opening tied? accounts other]}]
  (let [amounts #(reduce (fn [m k] (cond-> m (contains? m k) (update k amount-out))) % [:xero :ownpurse :difference
                                                                                        :adjustments :pro-forma])]
    (cond-> {:org org :date (str date) :year-start (str year-start) :opening opening :tied tied?
             :accounts (mapv amounts accounts)
             :other (mapv #(update % :amount amount-out) other)}
      pro-forma (assoc :pro-forma pro-forma))))

(def Posting
  "A journal's posting: an amount as a decimal string, debits positive."
  [:map {:closed true}
   [:account [:string {:min 1}]]
   [:amount (schema/schema :ownpurse/amount)]
   [:commodity (schema/schema :ownpurse/commodity)]
   [:description {:optional true} :string]])

(def Journal
  "A journal a close item proposes, in one book. It must balance in every commodity."
  [:map {:closed true}
   [:ref [:string {:min 1}]]
   [:book {:optional true} [:string {:min 1}]]
   [:date (schema/schema :ownpurse/date)]
   [:narration [:string {:min 1}]]
   [:postings [:vector {:min 2} Posting]]
   [:evidence {:optional true} [:vector :string]]])

(def CloseItemBody
  "PUT /close/items/{ref}: a close item, a question whose answer may become journals."
  [:map {:closed true}
   [:title [:string {:min 1}]]
   [:status [:string {:min 1}]]
   [:book {:optional true} [:string {:min 1}]]
   [:category {:optional true} :string]
   [:action {:optional true} :string]
   [:amount {:optional true} (schema/schema :ownpurse/amount)]
   [:effect {:optional true} :string]
   [:preconditions {:optional true} [:vector :string]]
   [:depends-on {:optional true} [:vector :string]]
   [:evidence {:optional true} [:vector :string]]
   [:links {:optional true} [:vector :string]]
   [:reason {:optional true} :string]
   [:group {:optional true} :string]
   [:note {:optional true} :string]
   [:journals {:optional true} [:vector Journal]]])

(def CloseItem
  "A close item as the API returns it: amounts and dates as strings, and whether it is approved as it stands."
  [:map
   [:ref :string] [:title :string] [:status :string] [:approved :boolean]
   [:amount {:optional true} :string]
   [:journals [:vector [:map [:ref :string] [:book :string] [:date :string] [:narration :string]
                        [:postings [:vector [:map [:account :string] [:amount :string] [:commodity :string]]]]
                        [:posted-reference {:optional true} :string]]]]])

(def CloseGroup
  "A decision group: items put to one conversation."
  [:map [:ref :string] [:title :string] [:prompt {:optional true} :string] [:items [:vector :string]]])

(def ClosePackageBody
  "POST /close/packages: the items a package puts to the owner, sealed as they stand."
  [:map {:closed true}
   [:ref [:string {:min 1}]]
   [:items [:vector {:min 1} :string]]
   [:as-of {:optional true} :string]
   [:scope {:optional true} :string]
   [:coverage {:optional true} [:map-of :keyword :int]]
   [:note {:optional true} :string]])

(def ClosePackage
  "A sealed package, its approval if any, and the items changed since it was sealed."
  [:map
   [:ref :string] [:items [:vector :string]] [:hashes [:vector :string]] [:hash :string] [:sealed-at :string]
   [:changed [:vector :string]]
   [:approval {:optional true} [:map [:by :string] [:at :string] [:hash :string] [:note {:optional true} :string]]]])

(def PairSide
  "One side of an intercompany pair: a book, its accounts, and optionally the parties whose postings count."
  [:map {:closed true}
   [:book [:string {:min 1}]]
   [:accounts [:vector {:min 1} [:string {:min 1}]]]
   [:parties {:optional true} [:vector [:string {:min 1}]]]])

(def PairBody
  "PUT /intercompany/pairs/{ref}: accounts in one book that mirror accounts in another."
  [:map {:closed true}
   [:name {:optional true} :string]
   [:a PairSide]
   [:b PairSide]
   [:from {:optional true} (schema/schema :ownpurse/date)]
   [:to {:optional true} (schema/schema :ownpurse/date)]
   [:note {:optional true} :string]
   [:explained {:optional true} [:vector [:map {:closed true} [:ref [:string {:min 1}]]
                                          [:amount (schema/schema :ownpurse/amount)]
                                          [:note {:optional true} :string]]]]])

(def PairLine
  "A posting the reconciliation lists."
  [:map [:entry :string] [:account :string] [:date :string] [:amount :string]])

(def PairSideReport
  "One side of a pair, reconciled."
  [:map [:book :string] [:accounts [:vector :string]] [:balance :string] [:movement :string]
   [:unattributed {:optional true} [:vector PairLine]]])

(def PairReport
  "A pair reconciled over a period. Amounts are decimal strings, debits positive."
  [:map [:ref :string] [:a PairSideReport] [:b PairSideReport] [:difference :string] [:unexplained :string]
   [:movement-difference :string] [:matched :int] [:reconciled :boolean]
   [:unmatched [:map [:a [:vector PairLine]] [:b [:vector PairLine]]]]])

(defn- strings
  "`x` with every amount (a BigDecimal) and date as a string, all the way down."
  [x]
  (cond
    (decimal? x) (amount-out x)
    (instance? java.time.LocalDate x) (str x)
    (map? x) (update-vals x strings)
    (sequential? x) (mapv strings x)
    :else x))

(defn- pair-report-out [r]
  (-> r (assoc :reconciled (:reconciled? r)) (dissoc :reconciled?) strings))

(defn- close-item-out [node item]
  (cond-> (-> item
              (assoc :approved (close/approved? node item))
              (update :journals (partial mapv (fn [j] (-> j
                                                          (update :date str)
                                                          (update :postings (partial mapv #(update % :amount amount-out))))))))
    (:amount item) (update :amount amount-out)))

(defn- close-package-out [package]
  (cond-> (update package :sealed-at str)
    (:approval package) (update :approval #(-> % (dissoc :package) (update :at str)))))

(def Match
  "Two lines a person says are the same money, by their source rows."
  [:map [:evidence :string] [:book :string] [:note {:optional true} :string] [:from {:optional true} :string]
   [:until {:optional true} :string]])

(def MatchBody
  "PUT /matches: the source rows of an evidence line and a book line of the same amount."
  [:map {:closed true}
   [:evidence [:string {:min 1}]]
   [:book [:string {:min 1}]]
   [:note {:optional true} :string]
   [:from {:optional true} (schema/schema :ownpurse/date)]])

(def EntityBody
  "PUT /entities/{id}: the entity's name and kind."
  [:map {:closed true} [:name [:string {:min 1}]] [:kind [:enum "business" "household"]]])

(def LinkBody
  "PUT /links: which entity a source account belongs to, and how."
  [:map {:closed true}
   [:entity (schema/schema :ownpurse/slug)]
   [:account [:string {:min 1}]]
   [:role (schema/schema :link/role)]
   [:evidence-for {:optional true} [:string {:min 1}]]
   [:from {:optional true} (schema/schema :ownpurse/date)]])

(defn- error [status type message]
  {:status status :body {:error {:type type :message message}}})

(defn- health [node]
  (fn [_]
    (if (:ok? (store/health node))
      {:status 200 :body (if-let [{:keys [dir taken-at block transactions]} (store/last-snapshot node)]
                           {:status "ok"
                            :last-snapshot {:dir dir :taken-at (str (.toInstant ^java.time.ZonedDateTime taken-at))
                                            :block block :transactions transactions}}
                           {:status "ok"})}
      {:status 503 :body {:status "unavailable"}})))

(defn- backup!
  "Snapshots the record into the folder `dir`, which must be an absolute path."
  [node store-config {:keys [dir]}]
  (when-not (.isAbsolute (io/file dir))
    (throw (ex-info (str dir " is not an absolute path. Give the folder for snapshots in full, for example "
                         "/Volumes/backup/ownpurse.")
                    {:type :ownpurse/invalid :dir dir})))
  {:snapshot (dissoc (store/snapshot! node store-config dir) :format :latest-tx-id)})

(defn- error-type [t]
  (if (namespace t) (str (namespace t) "/" (name t)) (name t)))

(defn- pull-all!
  "Pulls every org and item the local config names: Xero's, then Mercury's, then Plaid's."
  [node {:keys [local xero mercury plaid store]}]
  (let [xero-orgs (xero/read-orgs local)
        mercury-orgs (mercury/read-orgs local)
        plaid-items (plaid/read-items local)]
    (when (and (empty? xero-orgs) (empty? mercury-orgs) (empty? plaid-items))
      (throw (ex-info (str local " names no orgs. Add {:xero {:orgs [{:tenant \"<tenant id>\" :name \"<a name>\"}]}}, "
                           "{:mercury {:orgs [{:alias \"<mercury-cli alias>\" :name \"<a name>\"}]}} "
                           "or {:plaid {:items [{:alias \"<plaid-reader alias>\" :name \"<a name>\"}]}}.")
                      {:type :ownpurse/no-local-config :path (str local)})))
    {:xero (if (seq xero-orgs)
             (xero/pull-orgs! node {:cli (:cli xero) :orgs xero-orgs :reports (xero/read-reports local)
                                    :history (xero/read-history local)
                                    :keep-file (when (= :disk (:mode store)) #(store/keep-file! store %))})
             {})
     :mercury (if (seq mercury-orgs) (mercury/pull-orgs! node {:cli (:cli mercury) :orgs mercury-orgs}) {})
     :plaid (if (seq plaid-items) (plaid/pull-items! node {:cli (:cli plaid) :items plaid-items}) {})}))

(defn- pull
  "Pulls every org in the local config through its source's tool, one pull at a time."
  [node sources]
  (let [running (atom false)]
    (fn [_]
      (if-not (compare-and-set! running false true)
        (error 409 "pull-running" "A pull is already running. Wait for it to finish, then ask again.")
        (try
          {:status 200 :body {:pulled (pull-all! node sources)}}
          (catch clojure.lang.ExceptionInfo e
            (let [t (:type (ex-data e))]
              (cond
                (= :ownpurse/no-local-config t) (error 409 (error-type t) (ex-message e))
                (#{"xero" "mercury" "plaid"} (some-> t namespace)) (error 502 (error-type t) (ex-message e))
                :else (throw e))))
          (finally (reset! running false)))))))

(def ^:private entity-errors
  {:ownpurse/invalid 400 :ownpurse/invalid-link 400 :ownpurse/invalid-match 400 :ownpurse/not-found 404
   :ownpurse/link-conflict 409 :ownpurse/match-conflict 409 :ownpurse/close-conflict 409 :ownpurse/no-disk 409 :ownpurse/backup-exists 409
   :ownpurse/backup-timeout 503 :ownpurse/backup-invalid 500})

(defn- answer
  "Calls `f` and answers 200 with what it returns, or the error a component threw, as structured JSON."
  [f]
  (try
    {:status 200 :body (f)}
    (catch clojure.lang.ExceptionInfo e
      (let [t (:type (ex-data e))]
        (if-let [status (entity-errors t)]
          (error status (error-type t) (ex-message e))
          (throw e))))))

(defn- routes [{:keys [node] :as config}]
  [["/openapi.json"
    {:get {:no-doc true
           :openapi {:info {:title "ownpurse" :version "0.0.0"
                            :description "The local ledger. Every endpoint, its parameters and its responses."}}
           :handler (openapi/create-openapi-handler)}}]
   ["/health"
    {:get {:summary "Whether ownpurse and its record are up."
           :responses {200 {:body Health} 503 {:body Health}}
           :handler (health node)}}]
   ["/backup"
    {:post {:summary (str "Snapshots the running record into a new folder under dir (ownpurse-<time>), and checks that "
                          "a fresh node opens it. Restore it with ownpurse stopped: see docs/BACKUP.md.")
            :parameters {:body BackupBody}
            :responses {200 {:body [:map [:snapshot Snapshot]]} 400 {:body ApiError} 409 {:body ApiError}
                        500 {:body ApiError} 503 {:body ApiError}}
            :handler (fn [{{:keys [body]} :parameters}]
                       (answer #(backup! node (:store config) body)))}}]
   ["/pull"
    {:post {:summary (str "Fetches every org in the local config through its source's tool, and keeps what changed. "
                          "Costs the source's API calls; later pulls ask only for what changed.")
            :responses {200 {:body Pulled} 409 {:body ApiError} 502 {:body ApiError}}
            :handler (pull node (select-keys config [:local :xero :mercury :plaid :store]))}}]
   ["/entities"
    {:get {:summary "Every entity, with the source accounts linked to it now."
           :responses {200 {:body Entities}}
           :handler (fn [_] (answer #(hash-map :entities (entity/entities node))))}}]
   ["/entities/{id}"
    {:put {:summary "Makes an entity, or renames it. The id is a slug: lowercase letters, digits and hyphens."
           :parameters {:path [:map [:id (schema/schema :ownpurse/slug)]] :body EntityBody}
           :responses {200 {:body [:map [:entity Entity]]} 400 {:body ApiError}}
           :handler (fn [{{:keys [path body]} :parameters}]
                      (answer #(hash-map :entity (entity/put-entity! node (assoc body :id (:id path))))))}}]
   ["/source-accounts"
    {:get {:summary (str "Every source account: what each source holds that can belong to an entity (an org, a bank "
                         "account, a credit card), with its link. A source's accounts appear after its first pull.")
           :parameters {:query [:map [:source {:optional true} :string] [:unlinked {:optional true} :boolean]]}
           :responses {200 {:body [:map [:source-accounts [:vector SourceAccount]]]}}
           :handler (fn [{{:keys [query]} :parameters}]
                      (answer #(hash-map :source-accounts (entity/source-accounts node query))))}}]
   ["/compare"
    {:get {:summary (str "Compares each account linked as evidence for an account (or only `account`) with that "
                         "account: the lines each has that the other has not, over the days from `from` to `to`. "
                         "Lines a person matched (PUT /matches) pair first, then lines of the same check number and "
                         "amount, then lines of the same amount at most `days` apart (default 5); `matched-by` "
                         "counts each kind, and `pairs=true` lists the pairs. Pending lines are listed apart. Reads "
                         "what the last pull kept; it calls no source.")
           :parameters {:query [:map
                                [:account {:optional true} [:string {:min 1}]]
                                [:from {:optional true} (schema/schema :ownpurse/date)]
                                [:to {:optional true} (schema/schema :ownpurse/date)]
                                [:days {:optional true} [:int {:min 0 :max 60}]]
                                [:pairs {:optional true} :boolean]]}
           :responses {200 {:body [:map [:compares [:vector Compare]]]} 404 {:body ApiError}}
           :handler (fn [{{:keys [query]} :parameters}]
                      (answer #(hash-map :compares (cmp/compare-accounts node query))))}}]
   ["/trial-balance"
    {:get {:summary (str "Checks a Xero org's book (its entries, posted from its documents on every pull) against "
                         "Xero's trial balance at `date`, account by account, to the cent. `date` must be a month end "
                         "a pull kept the report for. Balance sheet accounts start the financial year at Xero's "
                         "balance sheet the day before (opening=balance-sheet, the default) or at the book's own "
                         "postings (opening=ledger). `tied` is true when every account equals Xero's. Reads what the "
                         "last pull kept; it calls no source.")
           :parameters {:query [:map
                                [:org [:string {:min 1}]]
                                [:date (schema/schema :ownpurse/date)]
                                [:opening {:optional true} [:enum "balance-sheet" "ledger"]]
                                [:pro-forma {:optional true} [:enum "approved" "proposed"]]]}
           :responses {200 {:body [:map [:trial-balance TrialBalance]]} 404 {:body ApiError}}
           :handler (fn [{{{:keys [org date opening pro-forma]} :query} :parameters}]
                      (let [book (xero/book-id org)
                            adjustments (when pro-forma
                                          #(close/balances node book (assoc % :basis (keyword pro-forma))))]
                        (answer #(hash-map :trial-balance
                                           (trial-balance-out org pro-forma
                                                              (xero/check-trial-balance
                                                               node org date
                                                               (cond-> {:opening (keyword (or opening "balance-sheet"))}
                                                                 adjustments (assoc :adjustments adjustments))))))))}}]
   ["/intercompany/pairs"
    {:get {:summary "Every intercompany pair: accounts in one book that mirror accounts in another."
           :responses {200 {:body [:map [:pairs [:vector :map]]]}}
           :handler (fn [_] (answer #(hash-map :pairs (strings (ic/pairs node)))))}}]
   ["/intercompany/pairs/{ref}"
    {:put {:summary (str "Adds or changes an intercompany pair: side a's accounts in one book mirror side b's in "
                         "another (a loan and its receivable, a due-to and its due-from, a grant paid and received). "
                         "A side may hold several accounts; `parties` keeps only those parties' postings, for one "
                         "account that holds several counterparties'. `from` and `to` bound when the pair holds. "
                         "`explained` pins known parts of the difference, such as a prior-period amount.")
           :parameters {:path [:map [:ref [:string {:min 1}]]] :body PairBody}
           :responses {200 {:body [:map [:pair :map]]} 400 {:body ApiError}}
           :handler (fn [{{:keys [path body]} :parameters}]
                      (answer #(hash-map :pair (strings (ic/put-pair! node (assoc body :ref (:ref path)))))))}}]
   ["/intercompany"
    {:get {:summary (str "Reconciles every intercompany pair in force from `from` to `to`, from the books' postings. "
                         "Per pair: each side's balance at `to` and movement in the period, the difference (the "
                         "balances summed, zero when they mirror), what is unexplained after the pinned "
                         "explanations, and the period's lines the other side has no mirror for. A line matches one "
                         "of the opposite amount at most `days` apart (default 5), else its entry's total does. A "
                         "side with parties lists its lines that name no party as unattributed.")
           :parameters {:query [:map
                                [:from {:optional true} (schema/schema :ownpurse/date)]
                                [:to (schema/schema :ownpurse/date)]
                                [:days {:optional true} [:int {:min 0 :max 60}]]]}
           :responses {200 {:body [:map [:pairs [:vector PairReport]]]}}
           :handler (fn [{{:keys [query]} :parameters}]
                      (answer #(hash-map :pairs (mapv pair-report-out (ic/report node query)))))}}]
   ["/close/items"
    {:get {:summary (str "The close register's items: the questions a close must decide, each with its workflow "
                         "status, evidence and the journals it proposes. `book` finds an item by its book or any of "
                         "its journals'. `approved` is true when the item is in an approved package and unchanged since.")
           :parameters {:query [:map [:book {:optional true} :string] [:status {:optional true} :string]
                                [:group {:optional true} :string]]}
           :responses {200 {:body [:map [:items [:vector CloseItem]]]}}
           :handler (fn [{{:keys [query]} :parameters}]
                      (answer #(hash-map :items (mapv (partial close-item-out node) (close/items node query)))))}}]
   ["/close/items/{ref}"
    {:put {:summary (str "Adds a close item or changes it. Its status is your close workflow's own label; ownpurse "
                         "reads only rejected and excluded, which count nowhere. Each journal names its book (default "
                         "the item's), and its postings name accounts by the book's account id or code, and must "
                         "balance. A posted journal cannot change. Nothing posts anywhere.")
           :parameters {:path [:map [:ref [:string {:min 1}]]] :body CloseItemBody}
           :responses {200 {:body [:map [:item CloseItem]]} 400 {:body ApiError} 409 {:body ApiError}}
           :handler (fn [{{:keys [path body]} :parameters}]
                      (answer #(hash-map :item (close-item-out node (close/put-item! node (assoc body :ref (:ref path)))))))}}]
   ["/close/items/{ref}/journals/{journal}/posted"
    {:post {:summary (str "Records where an approved item's journal was posted in its book's system of record, once a "
                          "person has posted it. From then on it leaves the pro-forma: the book's own figures hold it.")
            :parameters {:path [:map [:ref [:string {:min 1}]] [:journal [:string {:min 1}]]]
                         :body [:map {:closed true} [:reference [:string {:min 1}]]]}
            :responses {200 {:body [:map [:item CloseItem]]} 400 {:body ApiError} 404 {:body ApiError}
                        409 {:body ApiError}}
            :handler (fn [{{:keys [path body]} :parameters}]
                       (answer #(hash-map :item (close-item-out node (close/mark-posted!
                                                                      node {:item (:ref path) :journal (:journal path)
                                                                            :reference (:reference body)})))))}}]
   ["/close/groups"
    {:get {:summary "The decision groups, each with the items that name it as their group."
           :responses {200 {:body [:map [:groups [:vector CloseGroup]]]}}
           :handler (fn [_] (answer #(hash-map :groups (close/groups node))))}}]
   ["/close/groups/{ref}"
    {:put {:summary "Adds or changes a decision group: items put to the owner or the accountant in one conversation."
           :parameters {:path [:map [:ref [:string {:min 1}]]]
                        :body [:map {:closed true} [:title [:string {:min 1}]] [:prompt {:optional true} :string]
                               [:note {:optional true} :string]]}
           :responses {200 {:body [:map [:group CloseGroup]]} 400 {:body ApiError}}
           :handler (fn [{{:keys [path body]} :parameters}]
                      (answer #(hash-map :group (close/put-group! node (assoc body :ref (:ref path))))))}}]
   ["/close/packages"
    {:get {:summary (str "Every sealed package, with its approval and the items changed since it was sealed (an "
                         "approval covers an item only as it was sealed).")
           :responses {200 {:body [:map [:packages [:vector ClosePackage]]]}}
           :handler (fn [_] (answer #(hash-map :packages (mapv close-package-out (close/packages node)))))}
     :post {:summary (str "Seals a package for the owner: the items as they stand now, each item's content hash, and "
                          "one hash over them all. A package is never changed; seal a new one.")
            :parameters {:body ClosePackageBody}
            :responses {200 {:body [:map [:package ClosePackage]]} 400 {:body ApiError} 404 {:body ApiError}
                        409 {:body ApiError}}
            :handler (fn [{{:keys [body]} :parameters}]
                       (answer #(hash-map :package (close-package-out (close/seal! node body)))))}}]
   ["/close/packages/{ref}/approval"
    {:post {:summary (str "Records the owner's approval of a sealed package, with the package's hash as they saw it. "
                          "Refused when the hash differs or an item changed since sealing. Its items' journals then "
                          "count in the pro-forma trial balance (GET /trial-balance?pro-forma=approved).")
            :parameters {:path [:map [:ref [:string {:min 1}]]]
                         :body [:map {:closed true} [:hash [:string {:min 1}]] [:by [:string {:min 1}]]
                                [:note {:optional true} :string]]}
            :responses {200 {:body [:map [:package ClosePackage]]} 400 {:body ApiError} 404 {:body ApiError}
                        409 {:body ApiError}}
            :handler (fn [{{:keys [path body]} :parameters}]
                       (answer #(hash-map :package (close-package-out (close/approve! node (assoc body :package (:ref path)))))))}}]
   ["/matches"
    {:put {:summary (str "Says two lines are the same money: an evidence line and a book line of the same amount, on "
                         "an evidence account and the account it is evidence for, by their source rows (the compare "
                         "lists each line's source-row). The compare pairs them first. Each line is in one match at "
                         "a time.")
           :parameters {:body MatchBody}
           :responses {200 {:body [:map [:match Match]]} 400 {:body ApiError} 404 {:body ApiError}
                       409 {:body ApiError}}
           :handler (fn [{{:keys [body]} :parameters}]
                      (answer #(hash-map :match (cmp/put-match! node body))))}
     :delete {:summary "Ends the match of an evidence line, by its source row, from a day (default now). Its history is kept."
              :parameters {:query [:map [:evidence [:string {:min 1}]]
                                   [:on {:optional true} (schema/schema :ownpurse/date)]]}
              :responses {200 {:body [:map [:ended Match]]} 400 {:body ApiError} 404 {:body ApiError}}
              :handler (fn [{{:keys [query]} :parameters}]
                         (answer #(hash-map :ended (cmp/end-match! node query))))}}]
   ["/links"
    {:put {:summary (str "Links a source account to an entity, from a day (default today): as its system of record "
                         "(an org, one per entity), or as evidence, optionally for an account of that system of "
                         "record. Linking it again to the same entity changes the link.")
           :parameters {:body LinkBody}
           :responses {200 {:body [:map [:link Link]]} 400 {:body ApiError} 404 {:body ApiError}
                       409 {:body ApiError}}
           :handler (fn [{{:keys [body]} :parameters}]
                      (answer #(hash-map :link (entity/link! node body))))}
     :delete {:summary "Ends a source account's link from a day (default now). Its history is kept."
              :parameters {:query [:map [:account [:string {:min 1}]] [:on {:optional true} (schema/schema :ownpurse/date)]]}
              :responses {200 {:body [:map [:ended Link]]} 400 {:body ApiError} 404 {:body ApiError}}
              :handler (fn [{{:keys [query]} :parameters}]
                         (answer #(hash-map :ended (entity/unlink! node query))))}}]])

(def ^:private exception-middleware
  (exception/create-exception-middleware
   (merge exception/default-handlers
          {::exception/default
           (fn [e _] (error 500 "internal" (str "Something went wrong inside ownpurse: " (ex-message e))))
           :reitit.coercion/request-coercion
           (fn [e _] (error 400 "invalid-request"
                            (str "The request does not match the schema: " (pr-str (me/humanize (ex-data e))))))})))

(defn- json-response [{:keys [status body]}]
  {:status status
   :headers {"Content-Type" "application/json; charset=utf-8"}
   :body (json/write-str body)})

(defn handler
  "The Ring handler over `node`. `local` is the config file naming the orgs to pull; `xero`, `mercury` and `plaid` are {:cli}:
  the program that reads each source; `store` is the store's config, which says where the record is kept."
  [config]
  (ring/ring-handler
   (ring/router
    (routes config)
    {:data {:coercion rcm/coercion
            :responses {500 {:body ApiError}}
            :muuntaja api.json/muuntaja
            :middleware [openapi/openapi-feature
                         parameters/parameters-middleware
                         muuntaja/format-negotiate-middleware
                         muuntaja/format-response-middleware
                         exception-middleware
                         muuntaja/format-request-middleware
                         coercion/coerce-response-middleware
                         coercion/coerce-request-middleware]}})
   (ring/create-default-handler
    {:not-found (constantly (json-response (error 404 "not-found" "No such endpoint. GET /openapi.json lists them.")))
     :method-not-allowed (constantly (json-response (error 405 "method-not-allowed" "That method is not allowed here. GET /openapi.json lists the methods.")))
     :not-acceptable (constantly (json-response (error 406 "not-acceptable" "ownpurse answers in JSON only.")))})))
