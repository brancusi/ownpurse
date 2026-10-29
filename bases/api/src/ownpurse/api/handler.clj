(ns ownpurse.api.handler
  "The HTTP API: routes, schemas and the Ring handler. Thin: it turns requests into interface calls and back."
  (:require
   [clojure.edn :as edn]
   [clojure.string :as str]
   [clojure.java.io :as io]
   [malli.error :as me]
   [malli.util :as mu]
   [ownpurse.api.books :as books]
   [ownpurse.api.json :as api.json]
   [ownpurse.close.interface :as close]
   [ownpurse.classify.interface :as classify]
   [ownpurse.compare.interface :as cmp]
   [ownpurse.entity.interface :as entity]
   [ownpurse.export.interface :as export]
   [ownpurse.intercompany.interface :as ic]
   [ownpurse.json.interface :as json]
   [ownpurse.mercury.interface :as mercury]
   [ownpurse.plaid.interface :as plaid]
   [ownpurse.sandbox.interface :as sandbox]
   [ownpurse.scenario.interface :as scenario]
   [ownpurse.schema.interface :as schema]
   [ownpurse.statement.interface :as statement]
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

(def Workspace
  "What GET /workspace returns: where this ownpurse runs. A workspace's books (a group's record, with the sources its
  config names), or the demo (no books: the sandbox's made-up lines only). Its name comes from the config's
  {:workspace {:name}}."
  [:map
   [:name [:maybe :string]]
   [:kind [:enum "books" "demo"]]
   [:books :int]
   [:sources [:vector :string]]])

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
  [:map [:pulled [:map [:xero Counts] [:mercury Counts] [:plaid Counts]
                  [:statements {:optional true} [:map [:files :int] [:lines :int] [:written :int]]]]]])

(def Statement
  "A statement file the record holds, and whether it ties. Amounts are decimal strings."
  [:map [:account :string] [:name :string] [:format :string] [:lines :int] [:total :string] [:tied :boolean]
   [:opening {:optional true} :string] [:closing {:optional true} :string]
   [:from {:optional true} :string] [:to {:optional true} :string]
   [:breaks [:vector [:map [:row :int] [:expected :string] [:balance :string]]]]])

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
   [:scenario {:optional true} :string]
   [:accounts [:vector [:map [:account :string] [:code {:optional true} :string] [:name :string]
                        [:xero :string] [:ownpurse :string] [:difference :string]
                        [:adjustments {:optional true} :string] [:pro-forma {:optional true} :string]]]]
   [:other [:vector [:map [:name [:maybe :string]] [:amount :string]]]]])

(defn- amount-out [^BigDecimal a]
  (.toPlainString a))

(defn- trial-balance-out
  "A trial balance check as the API returns it: dates and amounts as strings."
  [org scenario pro-forma {:keys [date year-start opening tied? accounts other]}]
  (let [amounts #(reduce (fn [m k] (cond-> m (contains? m k) (update k amount-out))) % [:xero :ownpurse :difference
                                                                                        :adjustments :pro-forma])]
    (cond-> {:org org :date (str date) :year-start (str year-start) :opening opening :tied tied?
             :accounts (mapv amounts accounts)
             :other (mapv #(update % :amount amount-out) other)}
      scenario (assoc :scenario scenario)
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
   [:parties {:optional true} [:vector [:string {:min 1}]]]
   [:assigned {:optional true} [:vector [:map {:closed true} [:entry [:string {:min 1}]] [:line {:optional true} :int]]]]
   [:narration {:optional true} [:string {:min 1}]]
   [:opening {:optional true} [:map {:closed true} [:amount (schema/schema :ownpurse/amount)]
                               [:note {:optional true} :string]]]])

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
   [:months [:vector [:map [:month :string] [:a :string] [:b :string] [:difference :string]]]]
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

(def Payees
  "Who a book paid from `from` to `to`, by party and by how. Amounts are decimal strings."
  [:map [:org :string] [:from :string] [:to :string] [:threshold :string]
   [:payees [:vector [:map [:party :string] [:name :string] [:paid [:map [:bank {:optional true} :string] [:credit-card {:optional true} :string]]] [:payments :int]
                      [:candidate :boolean] [:supplier {:optional true} :boolean]
                      [:tax-number {:optional true} :string]]]]])

(defn- xero-payees
  "A Xero org's payees as the API returns them."
  [node {:keys [org] :as query}]
  (let [result (books/xero-payees node query)]
    (-> result
        (dissoc :book)
        (assoc :org org)
        (update :payees (partial mapv #(-> %
                                           (assoc :candidate (:candidate? %))
                                           (update :paid update-keys keyword)
                                           (dissoc :candidate? :supplier?)
                                           (cond-> (contains? % :supplier?) (assoc :supplier (:supplier? %))))))
        strings)))

(def ExportBody
  "POST /export: the org, the end of its financial year, and the folder to write into."
  [:map {:closed true}
   [:org [:string {:min 1}]]
   [:date (schema/schema :ownpurse/date)]
   [:dir [:string {:min 1}]]
   [:pro-forma {:optional true} [:enum "approved" "proposed"]]
   [:scenario {:optional true} [:string {:min 1}]]
   [:opening {:optional true} [:enum "balance-sheet" "ledger"]]])

(defn- xero-export!
  "The accountant's export of a Xero org's book for the financial year ending `date`."
  [node {:keys [dir] :as body}]
  (let [opts (books/xero-book node body)]
    (assoc (export/export! node (assoc opts :dir dir)) :tied (:tied? (:trial-balance opts)))))

(def Layer
  "One layer of a scenario: its kind, whether it is on, and what the kind reads."
  [:map {:closed true}
   [:id [:string {:min 1}]]
   [:kind (into [:enum] (keys scenario/kinds))]
   [:enabled {:optional true} :boolean]
   [:note {:optional true} :string]
   [:basis {:optional true} [:enum "approved" "proposed"]]
   [:journals {:optional true} [:vector Journal]]
   ;; Checked by the scenario component; a bare :map here would be stripped of every key by the coercion.
   [:world {:optional true} [:map-of :keyword :any]]
   [:coding {:optional true} [:map-of :keyword :any]]
   [:lines {:optional true} [:map-of :keyword :any]]
   [:key {:optional true} [:map-of :keyword :any]]])

(def ScenarioBody
  "PUT /scenarios/{ref}: a scenario, a named stack of layers, standing on another scenario if it names a base."
  [:map {:closed true}
   [:name [:string {:min 1}]]
   [:note {:optional true} :string]
   [:base {:optional true} [:string {:min 1}]]
   [:layers [:vector Layer]]])

(def Scenario
  "A scenario as the API returns it: amounts and dates as strings."
  [:map
   [:ref :string] [:name :string] [:note {:optional true} :string] [:base {:optional true} :string]
   [:built-in {:optional true} :boolean] [:duplicated-from {:optional true} :string]
   [:version {:optional true} :int] [:at {:optional true} :string]
   [:layers [:vector [:map [:id :string] [:kind :string] [:enabled :boolean] [:note {:optional true} :string]
                      [:basis {:optional true} :string]
                      [:world {:optional true} :any] [:coding {:optional true} :any] [:lines {:optional true} :any]
                      [:journals {:optional true}
                       [:vector [:map [:ref :string] [:book :string] [:date :string] [:narration :string]
                                 [:evidence {:optional true} [:vector :string]]
                                 [:postings [:vector [:map [:account :string] [:amount :string] [:commodity :string]
                                                      [:description {:optional true} :string]]]]]]]]]]])

(def Evaluation
  "What a scenario adds to a book over a period, layer by layer. Amounts are decimal strings, debits positive."
  [:map
   [:scenario :string] [:name :string] [:book :string] [:from {:optional true} :string] [:to {:optional true} :string]
   [:stack [:vector :string]]
   [:layers [:vector [:map [:scenario :string] [:layer :string] [:kind :string] [:enabled :boolean]
                      [:balances [:map-of :string :string]]]]]
   [:balances [:map-of :string :string]]])

(defn- scenario-out [s]
  (update s :layers (partial mapv (fn [l]
                                    (cond-> l
                                      (:journals l)
                                      (update :journals (partial mapv (fn [j] (-> j
                                                                                  (update :date str)
                                                                                  (update :postings (partial mapv #(update % :amount amount-out))))))))))))

(defn- evaluation-out [{:keys [from to] :as e}]
  (let [amounts #(update-vals % amount-out)]
    (cond-> (-> (dissoc e :from :to)
                (update :balances amounts)
                (update :layers (partial mapv #(update % :balances amounts))))
      from (assoc :from (str from))
      to (assoc :to (str to)))))

(defn- adjustments
  "The trial balance's :adjustments for `book`: what scenario `ref` adds, or nil for none."
  [node book ref]
  (when ref #(scenario/balances node ref book %)))

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

(defn- workspace
  "Where this ownpurse runs, from the local config (`local`, a file that may not exist) and the record's entities."
  [node {:keys [local]}]
  (let [f (some-> local str io/file)
        cfg (when (and f (.exists f)) (edn/read-string {:default tagged-literal} (slurp f)))
        sources (cond-> []
                  (seq (get-in cfg [:xero :orgs])) (conj "Xero")
                  (seq (get-in cfg [:mercury :orgs])) (conj "Mercury")
                  (seq (get-in cfg [:plaid :items])) (conj "Plaid"))
        books (count (entity/entities node))]
    {:name (get-in cfg [:workspace :name])
     :kind (if (or (seq sources) (pos? books)) "books" "demo")
     :books books
     :sources sources}))

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
  "Pulls every org and item the local config names: Xero's, then Mercury's, then Plaid's; then imports its statement
  files."
  [node {:keys [local xero mercury plaid store]}]
  (let [xero-orgs (xero/read-orgs local)
        mercury-orgs (mercury/read-orgs local)
        plaid-items (plaid/read-items local)
        statements (statement/read-config local)]
    (when (and (empty? xero-orgs) (empty? mercury-orgs) (empty? plaid-items) (empty? (:files statements)))
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
     :plaid (if (seq plaid-items) (plaid/pull-items! node {:cli (:cli plaid) :items plaid-items}) {})
     :statements (if (seq (:files statements)) (statement/import! node statements {}) {:files 0 :lines 0 :written 0})}))

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
   :ownpurse/link-conflict 409 :ownpurse/match-conflict 409 :ownpurse/close-conflict 409 :ownpurse/scenario-conflict 409 :ownpurse/export-exists 409 :ownpurse/no-disk 409 :ownpurse/backup-exists 409
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

(defn- over-of
  "Set of lines `id` as a run of a workspace's scenario reads it: a sample's lines, a scenario's lines layer
  (<scenario>~<layer>) or a saved set; nil for none. A set there is not is an error."
  [node sb id]
  (when id
    (or (sandbox/over sb id)
        (and node (scenario/set-over node id))
        (throw (ex-info (str "No set of lines " id ".") {:type :ownpurse/not-found})))))

(defn- sandbox-routes
  "The sandbox: scenarios of bank lines, the world they happened in, and the runs that classified them, for the admin.
  Answers are data as the components hold them; ask for application/transit+json to keep keywords, sets and decimals."
  [{:keys [sandbox node]}]
  (let [sb (sandbox/open sandbox)
        ;; A scenario of the record that codes lines is shown as the invented ones are: its coding is one run.
        record? #(and node (scenario/coding-scenario? node %))
        Scenario [:map [:scenario [:string {:min 1}]]]
        Run (mu/merge Scenario [:map [:run [:re #"^[A-Za-z0-9._~-]+$"]]])
        ;; A record scenario's run over its own lines is "latest"; over another set, that set's id.
        run-over #(when-not (= "latest" %) (over-of node sb %))]
    [["/sandbox/lines"
      {:get {:summary (str "The sets of lines the sandbox picks from: this workspace's books' (:kind \"books\", one a "
                           "scenario's lines layer) and the made-up samples (:kind \"sample\"), each with the scenarios "
                           "that read it.")
             :responses {200 {:body :any}}
             :handler (fn [_] (answer #(hash-map :lines (into (if node (scenario/line-sets node) [])
                                                              (sandbox/line-sets sb)))))}}]
     ["/sandbox/lines/{*set}"
      {:get {:summary "One set of lines, with a preview: its first two hundred lines, the days they span, their currencies."
             :parameters {:path [:map [:set [:string {:min 1}]]]}
             :responses {200 {:body :any} 404 {:body ApiError}}
             :handler (fn [{{:keys [path]} :parameters}]
                        (answer #(hash-map :lines (if (str/starts-with? (:set path) "sample~")
                                                    (sandbox/line-set sb (:set path))
                                                    (if node
                                                      (scenario/line-set node (:set path))
                                                      (throw (ex-info (str "No set of lines " (:set path) ".")
                                                                      {:type :ownpurse/not-found})))))))}}]
     ["/sandbox/scenarios"
      {:get {:summary (str "Every scenario of bank lines, its size and its runs, newest first: the record's own "
                           "(:record true, real lines with no answer key), then the invented ones.")
             :responses {200 {:body :any}}
             :handler (fn [_] (answer #(hash-map :scenarios (into (if node (scenario/sandbox-scenarios node) [])
                                                                  (sandbox/scenarios sb)))))}}]
     ["/sandbox/scenarios/{scenario}"
      {:get {:summary "A scenario's world: its people, cards, trips, projects, buckets, categories and rules."
             :parameters {:path Scenario}
             :responses {200 {:body :any} 404 {:body ApiError}}
             :handler (fn [{{:keys [path]} :parameters}]
                        (answer #(hash-map :world (if (record? (:scenario path))
                                                    (scenario/sandbox-world node (:scenario path))
                                                    (sandbox/world sb (:scenario path))))))}}]
     ["/sandbox/scenarios/{scenario}/runs/{run}"
      {:get {:summary "A run of a scenario: one row a unit, with each strategy's coding and confidence."
             :parameters {:path Run}
             :responses {200 {:body :any} 404 {:body ApiError}}
             :handler (fn [{{:keys [path]} :parameters}]
                        (answer #(if (record? (:scenario path))
                                   (scenario/sandbox-run node (:scenario path) (run-over (:run path)))
                                   (sandbox/run-lines sb (:scenario path) (:run path)))))}}]
     ["/sandbox/scenarios/{scenario}/trace/{*line}"
      {:get {:summary (str "One line of a record scenario through the pipeline, step by step: each step's function "
                           "called again on the record's data, its input and output, and every query it made, with "
                           "parameters and the row that came back. Nothing is asked of Jev.")
             :parameters {:path (mu/merge Scenario [:map [:line [:string {:min 1}]]])
                          :query [:map [:run {:optional true} [:re #"^[A-Za-z0-9._~-]+$"]]]}
             :responses {200 {:body :any} 404 {:body ApiError}}
             :handler (fn [{{:keys [path query]} :parameters}]
                        (answer #(if (record? (:scenario path))
                                   (hash-map :trace (scenario/sandbox-trace node (:scenario path)
                                                                            (some-> (:run query) run-over) (:line path)))
                                   (throw (ex-info (str "A trace follows a line of the record; " (:scenario path)
                                                        " is an invented sample, read from its file.")
                                                   {:type :ownpurse/not-found})))))}}]
     ;; A catch-all: one item of a receipt is a unit whose id has a slash (d88/1).
     ["/sandbox/scenarios/{scenario}/runs/{run}/lines/{*line}"
      {:get {:summary (str "One unit of a run all the way through: its facts, the requests exactly as sent, what came "
                           "back, how each rule held, each strategy's decision, and the commentary on it. An item of "
                           "a receipt is addressed as its line and number: lines/d88/1.")
             :parameters {:path (mu/merge Run [:map [:line [:string {:min 1}]]])}
             :responses {200 {:body :any} 404 {:body ApiError}}
             :handler (fn [{{:keys [path]} :parameters}]
                        (answer #(if (record? (:scenario path))
                                   (scenario/sandbox-line node (:scenario path) (run-over (:run path)) (:line path))
                                   (sandbox/line sb (:scenario path) (:run path) (:line path)))))}}]]))

(defn- coding-routes
  "Coding a scenario's bank lines: a run in the background, its progress, each unit's coding, and two scenarios'
  codings side by side. Answers are data as the component holds them; ask for application/transit+json to keep
  keywords and decimals."
  [{:keys [node jev sandbox]}]
  (let [running (atom {})
        sb (sandbox/open sandbox)
        Ref [:map [:ref [:string {:min 1}]]]
        Over [:map [:over {:optional true} [:string {:min 1}]]]]
    [["/scenarios/{ref}/coding"
      {:get {:summary (str "Coding a scenario's lines: what a run would take (:plan), the run going on, if any "
                           "(:running, with units done of all), and the runs so far, newest first. `over` a set of "
                           "lines (a sample, <scenario>~<layer> or a saved set's name) for a run over it instead.")
             :parameters {:path Ref :query Over}
             :responses {200 {:body :any} 404 {:body ApiError}}
             :handler (fn [{{:keys [path query]} :parameters}]
                        (answer #(let [over (over-of node sb (:over query))]
                                   {:plan (scenario/plan node (:ref path) over)
                                    :running (get @running [(:ref path) (:over query)])
                                    :runs (scenario/coding-runs node (:ref path) over)})))}
       :post {:summary (str "Codes a scenario's lines in the background, through Jev, keeping every answer by its "
                            "exact request: coding again after an edit asks only what changed. `limit` codes at most "
                            "that many units, for a trial. GET the same path for progress. Refused while a run of "
                            "the scenario is going on, or when no TypeSafe key is set. Nothing posts anywhere.")
              :parameters {:path Ref :body [:map {:closed true}
                                            [:limit {:optional true} [:int {:min 1}]]
                                            [:over {:optional true} [:string {:min 1}]]]}
              :responses {200 {:body :any} 404 {:body ApiError} 409 {:body ApiError}}
              :handler
              (fn [{{:keys [path body]} :parameters}]
                (let [ref (:ref path)
                      k [ref (:over body)]]
                  (answer
                   #(let [over (over-of node sb (:over body))
                          plan (scenario/plan node ref over)
                          units (cond-> (:units plan) (:limit body) (min (:limit body)))]
                      (when-not (:api-key jev)
                        (throw (ex-info (str "No TypeSafe key: set TYPESAFE_API_KEY, or the key in the config's "
                                             "secrets, then restart ownpurse.")
                                        {:type :ownpurse/scenario-conflict})))
                      ;; A run that failed is over: its entry says why until the next run replaces it.
                      (when-not (let [now @running]
                                  (and (or (nil? (get now k)) (:failed (get now k)))
                                       (compare-and-set! running now
                                                         (assoc now k {:done 0 :of units
                                                                         :started (str (java.time.Instant/now))}))))
                        (throw (ex-info (str "Scenario " ref " is being coded now. GET /scenarios/" ref "/coding for "
                                             "its progress.")
                                        {:type :ownpurse/scenario-conflict})))
                      (future
                        (try
                          (scenario/code! node ref (classify/jev-ask jev)
                                          {:model (:model jev) :limit (:limit body) :over over
                                           :on-unit (fn [_] (swap! running update-in [k :done] inc))})
                          (swap! running dissoc k)
                          (catch Throwable e
                            (swap! running assoc-in [k :failed] (ex-message e)))))
                      {:started true :plan plan :units units}))))}}]
     ["/scenarios/{ref}/coding/estimate"
      {:get {:summary (str "What coding a scenario's lines would take now, asking nothing: the requests Jev would be "
                           "asked and those the record already answers, the input tokens, the cost at Jev's published "
                           "price (the API reports tokens, not prices) and about how many seconds. `limit` as for the "
                           "run.")
             :parameters {:path Ref :query (mu/merge Over [:map [:limit {:optional true} [:int {:min 1}]]])}
             :responses {200 {:body :any} 404 {:body ApiError}}
             :handler (fn [{{:keys [path query]} :parameters}]
                        (answer #(hash-map :estimate (scenario/estimate node (:ref path) (:model jev)
                                                                        (assoc (select-keys query [:limit])
                                                                               :over (over-of node sb (:over query)))))))}}]
     ["/lines/{name}"
      {:put {:summary (str "Saves a set of lines by name: a title, a note and a selection over the books (accounts, or "
                           "all: every book's lines; from and to; lines, by id, for one line or a few). Any scenario "
                           "runs over it. Its key is kept. Nothing posts anywhere.")
             :parameters {:path [:map [:name [:string {:min 1}]]]
                          :body [:map {:closed true}
                                 [:title [:string {:min 1}]]
                                 [:note {:optional true} :string]
                                 [:select [:map {:closed true}
                                           [:accounts {:optional true} [:vector [:string {:min 1}]]]
                                           [:all {:optional true} :boolean]
                                           [:lines {:optional true} [:vector [:string {:min 1}]]]
                                           [:from {:optional true} :string]
                                           [:to {:optional true} :string]]]]}
             :responses {200 {:body :any} 400 {:body ApiError}}
             :handler (fn [{{:keys [path body]} :parameters}]
                        (answer #(hash-map :lines (scenario/line-set-put! node (assoc body :name (:name path))))))}}]
     ["/lines/{name}/key"
      {:post {:summary "Keys one line in a saved set: the bucket a person says it belongs in; `remove` takes it out."
              :parameters {:path [:map [:name [:string {:min 1}]]]
                           :body [:map {:closed true}
                                  [:unit [:string {:min 1}]]
                                  [:expect {:optional true} [:string {:min 1}]]
                                  [:ok {:optional true} [:vector [:string {:min 1}]]]
                                  [:review {:optional true} [:enum "required" "allowed"]]
                                  [:why {:optional true} :string]
                                  [:remove {:optional true} :boolean]]}
              :responses {200 {:body :any} 400 {:body ApiError} 404 {:body ApiError}}
              :handler (fn [{{:keys [path body]} :parameters}]
                         (answer #(do (when-not (or (:remove body) (:expect body))
                                        (throw (ex-info "Say the bucket the line belongs in (expect), or remove it."
                                                        {:type :ownpurse/invalid})))
                                      {:lines (scenario/line-set-confirm! node (:name path) body)})))}}]
     ["/scenarios/{ref}/key"
      {:get {:summary (str "A scenario's answer key, its bases' first: each line a person coded, the bucket it belongs "
                           "in (:expect), others that are fine (:ok), whether a person must or may look (:review), "
                           "why, and the scenario whose key layer says it (:in).")
             :parameters {:path Ref}
             :responses {200 {:body :any} 404 {:body ApiError}}
             :handler (fn [{{:keys [path]} :parameters}]
                        (answer #(hash-map :key (vec (vals (scenario/answer-key node (:ref path)))))))}
       :post {:summary (str "Keys one line in this scenario's own key layer (made when it has none): the bucket a "
                            "person says it belongs in. Its codings are then scored right or wrong. `remove` takes "
                            "the line out of this scenario's key; a base's key of it stands. Never sent to Jev.")
              :parameters {:path Ref
                           :body [:map {:closed true}
                                  [:unit [:string {:min 1}]]
                                  [:expect {:optional true} [:string {:min 1}]]
                                  [:ok {:optional true} [:vector [:string {:min 1}]]]
                                  [:review {:optional true} [:enum "required" "allowed"]]
                                  [:why {:optional true} :string]
                                  [:remove {:optional true} :boolean]]}
              :responses {200 {:body :any} 400 {:body ApiError} 404 {:body ApiError} 409 {:body ApiError}}
              :handler (fn [{{:keys [path body]} :parameters}]
                         (answer #(do (when-not (or (:remove body) (:expect body))
                                        (throw (ex-info "Say the bucket the line belongs in (expect), or remove it."
                                                        {:type :ownpurse/invalid})))
                                      (scenario/confirm! node (:ref path) body)
                                      {:key (get (scenario/answer-key node (:ref path)) (:unit body))})))}}]
     ["/scenarios/{ref}/composed"
      {:get {:summary (str "A scenario's world and coding as its layers that are on compose them, its bases' first: "
                           "the rules, codes, trips and projects that stand, the threshold and strategy, and which "
                           "lines it reads (:selects). The lines themselves are not read.")
             :parameters {:path Ref}
             :responses {200 {:body :any} 404 {:body ApiError}}
             :handler (fn [{{:keys [path]} :parameters}]
                        (answer #(hash-map :composed (scenario/composed node (:ref path))
                                           :rules (scenario/rule-ledger node (:ref path)))))}}]
     ["/scenarios/{ref}/codings"
      {:get {:summary (str "A scenario's coding of each unit, as its last run left it: the bucket that stands, the "
                           "one proposed, how sure, and whether it stands without a person.")
             :parameters {:path Ref}
             :responses {200 {:body :any}}
             :handler (fn [{{:keys [path]} :parameters}]
                        (answer #(hash-map :codings (scenario/codings node (:ref path)))))}}]
     ;; A catch-all: a unit's id holds colons and, for an item of a receipt, a slash.
     ["/scenarios/{ref}/codings/{*unit}"
      {:get {:summary "One unit's coding all the way through: its facts and each strategy's decision."
             :parameters {:path (mu/merge Ref [:map [:unit [:string {:min 1}]]])}
             :responses {200 {:body :any} 404 {:body ApiError}}
             :handler (fn [{{:keys [path]} :parameters}]
                        (answer #(hash-map :coding (or (scenario/coding node (:ref path) (:unit path))
                                                       (throw (ex-info (str "Scenario " (:ref path) " has not coded "
                                                                            (:unit path) ".")
                                                                       {:type :ownpurse/not-found}))))))}}]
     ["/scenarios/{ref}/compare/{other}"
      {:get {:summary "Each unit two scenarios code differently, with both codings."
             :parameters {:path (mu/merge Ref [:map [:other [:string {:min 1}]]])}
             :responses {200 {:body :any}}
             :handler (fn [{{:keys [path]} :parameters}]
                        (answer #(hash-map :differences (scenario/compare-codings node (:ref path) (:other path)))))}}]]))

(defn- routes [{:keys [node] :as config}]
  [["/admin" {:no-doc true :get {:handler (fn [_] {:status 302 :headers {"Location" "/admin/"} :body ""})}}]
   (sandbox-routes config)
   (coding-routes config)
   ["/openapi.json"
    {:get {:no-doc true
           :openapi {:info {:title "ownpurse" :version "0.0.0"
                            :description "The local ledger. Every endpoint, its parameters and its responses."}}
           :handler (openapi/create-openapi-handler)}}]
   ["/workspace"
    {:get {:summary "Where this ownpurse runs: a workspace's books (its name, how many, from which sources) or the demo."
           :responses {200 {:body Workspace}}
           :handler (fn [_] (answer #(workspace node config)))}}]
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
                                [:pro-forma {:optional true} [:enum "approved" "proposed"]]
                                [:scenario {:optional true} [:string {:min 1}]]]}
           :responses {200 {:body [:map [:trial-balance TrialBalance]]} 400 {:body ApiError} 404 {:body ApiError}}
           :handler (fn [{{{:keys [org date opening pro-forma] :as query} :query} :parameters}]
                      (answer #(let [ref (or (:scenario query) pro-forma)
                                     adjust (adjustments node (xero/book-id org) ref)]
                                 (when (and (:scenario query) pro-forma)
                                   (throw (ex-info "Say scenario or pro-forma, not both: pro-forma=x is scenario=x."
                                                   {:type :ownpurse/invalid})))
                                 (when (and ref (not (scenario/scenario node ref)))
                                   (throw (ex-info (str "There is no scenario " ref ". GET /scenarios lists them.")
                                                   {:type :ownpurse/not-found :scenario ref})))
                                 {:trial-balance
                                  (trial-balance-out org (:scenario query) pro-forma
                                                     (xero/check-trial-balance
                                                      node org date
                                                      (cond-> {:opening (keyword (or opening "balance-sheet"))}
                                                        adjust (assoc :adjustments adjust))))})))}}]
   ["/export"
    {:post {:summary (str "Writes the accountant's export of a Xero org's book for the financial year ending `date` into "
                          "a new folder under `dir` (an absolute path): a workbook with a README (the record's "
                          "transaction and each source's last fetch), the trial balance, balance sheet and profit and "
                          "loss (the system's figures beside the pro-forma, with the close items' journals counted as "
                          "`pro-forma` says), the general ledger with each line's document and a link to it in Xero, "
                          "the adjustments, the exclusions, the bank reconciliation and its unmatched lines, the "
                          "intercompany pairs and the payees; and the same sheets as CSV files beside it. Amounts are "
                          "exact decimals, debits positive. Reads what the record holds; it calls no source. `tied` "
                          "says whether the book ties with the system's trial balance.")
            :parameters {:body ExportBody}
            :responses {200 {:body [:map [:export [:map [:dir :string] [:workbook :string] [:tied :boolean]
                                                   [:sheets [:vector [:map [:name :string] [:rows :int]]]]]]]}
                        400 {:body ApiError} 404 {:body ApiError} 409 {:body ApiError}}
            :handler (fn [{{:keys [body]} :parameters}]
                       (answer #(hash-map :export (xero-export! node body))))}}]
   ["/statements"
    {:get {:summary (str "Every statement file the record holds (the local config's :statements :files, imported on "
                         "each pull), and whether it ties: its opening and its lines' total make its closing, and its "
                         "running balance, when it has one, follows from its lines (`breaks` lists where not). Link "
                         "a statement account (GET /source-accounts?source=statement) as evidence for a book's bank "
                         "account, and GET /compare compares them.")
           :responses {200 {:body [:map [:statements [:vector Statement]]]}}
           :handler (fn [_] (answer #(hash-map :statements
                                               (mapv (fn [f] (-> f (assoc :tied (:tied? f)) (dissoc :tied?) strings))
                                                     (statement/statements node)))))}}]
   ["/payees"
    {:get {:summary (str "Who a Xero org paid from `from` to `to`, by contact: money out of its bank accounts and its "
                         "credit cards, net of refunds, the largest paid by bank first. `candidate` marks a payee "
                         "paid by bank at or above `threshold` (default 600.00 for payments up to 2025, 2000.00 from "
                         "2026), who may need a 1099; card payments are shown apart, since the card's processor "
                         "reports them. A list for the accountant, not a filing: whether a payee is exempt (a "
                         "corporation, goods rather than services) is theirs to decide.")
           :parameters {:query [:map
                                [:org [:string {:min 1}]]
                                [:from (schema/schema :ownpurse/date)]
                                [:to (schema/schema :ownpurse/date)]
                                [:threshold {:optional true} (schema/schema :ownpurse/amount)]]}
           :responses {200 {:body [:map [:payees Payees]]}}
           :handler (fn [{{:keys [query]} :parameters}]
                      (answer #(hash-map :payees (xero-payees node query))))}}]
   ["/intercompany/pairs"
    {:get {:summary "Every intercompany pair: accounts in one book that mirror accounts in another."
           :responses {200 {:body [:map [:pairs [:vector :map]]]}}
           :handler (fn [_] (answer #(hash-map :pairs (strings (ic/pairs node)))))}}]
   ["/intercompany/pairs/{ref}"
    {:put {:summary (str "Adds or changes an intercompany pair: side a's accounts in one book mirror side b's in "
                         "another (a loan and its receivable, a due-to and its due-from, a grant paid and received). "
                         "A side may hold several accounts; `parties` keeps only those parties' postings, for one "
                         "account that holds several counterparties', with the lines that name no party but are "
                         "`assigned` to them by entry (and line), or whose narration matches the `narration` pattern. "
                         "`opening` is a balance a side holds from before its postings. `from` and `to` bound when "
                         "the pair holds. "
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
                         "of the opposite amount at most `days` apart (default 5), else its entry's total does; "
                         "`months` gives each side's movement by month and their difference. A "
                         "side with parties lists its lines that name no party as unattributed.")
           :parameters {:query [:map
                                [:from {:optional true} (schema/schema :ownpurse/date)]
                                [:to (schema/schema :ownpurse/date)]
                                [:days {:optional true} [:int {:min 0 :max 60}]]]}
           :responses {200 {:body [:map [:pairs [:vector PairReport]]]}}
           :handler (fn [{{:keys [query]} :parameters}]
                      (answer #(hash-map :pairs (mapv pair-report-out (ic/report node query)))))}}]
   ["/scenarios"
    {:get {:summary (str "Every scenario: a named stack of layers (journals of its own, the close register's) that "
                         "the reports read the books through. \"approved\" and \"proposed\" are built in: the close "
                         "register's journals as approved and as proposed.")
           :responses {200 {:body [:map [:scenarios [:vector Scenario]] [:kinds [:map-of :string :string]]]}}
           :handler (fn [_] (answer #(hash-map :scenarios (mapv scenario-out (scenario/scenarios node))
                                               :kinds scenario/kinds)))}
    :post {:summary (str "Adds a new scenario, as PUT /scenarios/{ref} with its ref in the body, but refused when "
                         "there is one of that ref already: a new scenario never replaces one.")
           :parameters {:body (mu/merge ScenarioBody [:map {:closed true} [:ref [:string {:min 1}]]])}
           :responses {200 {:body [:map [:scenario Scenario]]} 400 {:body ApiError} 404 {:body ApiError}
                       409 {:body ApiError}}
           :handler (fn [{{:keys [body]} :parameters}]
                      (answer #(hash-map :scenario (scenario-out (scenario/create! node body)))))}}]
   ["/scenarios/{ref}"
    {:get {:summary "One scenario as it stands."
           :parameters {:path [:map [:ref [:string {:min 1}]]]}
           :responses {200 {:body [:map [:scenario Scenario]]} 404 {:body ApiError}}
           :handler (fn [{{:keys [path]} :parameters}]
                      (answer #(hash-map :scenario
                                         (scenario-out (or (scenario/scenario node (:ref path))
                                                           (throw (ex-info (str "There is no scenario " (:ref path) ".")
                                                                           {:type :ownpurse/not-found})))))))}
     :put {:summary (str "Adds a scenario or changes it. Its layers apply in order, after those of its base, if it "
                         "names one. A layer of kind journals holds journals in any book, each balancing in every "
                         "commodity, accounts by the book's account id or code; a layer of kind close reads the close "
                         "register (basis approved or proposed). `enabled` false keeps a layer but counts it nowhere. "
                         "Every version is kept. Nothing posts anywhere.")
           :parameters {:path [:map [:ref [:string {:min 1}]]] :body ScenarioBody}
           :responses {200 {:body [:map [:scenario Scenario]]} 400 {:body ApiError} 404 {:body ApiError}
                       409 {:body ApiError}}
           :handler (fn [{{:keys [path body]} :parameters}]
                      (answer #(hash-map :scenario (scenario-out (scenario/put! node (assoc body :ref (:ref path)))))))}
     :delete {:summary (str "Drops a scenario: it counts nowhere from now on, and its versions stay. Refused while "
                            "another scenario stands on it.")
              :parameters {:path [:map [:ref [:string {:min 1}]]]}
              :responses {200 {:body [:map [:ref :string] [:dropped :boolean]]} 404 {:body ApiError} 409 {:body ApiError}}
              :handler (fn [{{:keys [path]} :parameters}] (answer #(scenario/drop! node (:ref path))))}}]
   ["/scenarios/{ref}/layers/{layer}"
    {:put {:summary "Adds one layer to a scenario, or replaces the layer of that id: to change it, or turn it on or off."
           :parameters {:path [:map [:ref [:string {:min 1}]] [:layer [:string {:min 1}]]]
                        :body (mu/dissoc Layer :id)}
           :responses {200 {:body [:map [:scenario Scenario]]} 400 {:body ApiError} 404 {:body ApiError}
                       409 {:body ApiError}}
           :handler (fn [{{:keys [path body]} :parameters}]
                      (answer #(hash-map :scenario (scenario-out (scenario/put-layer! node (:ref path)
                                                                                     (assoc body :id (:layer path)))))))}}]
   ["/scenarios/{ref}/duplicate"
    {:post {:summary (str "A new scenario with this one's base and layers as they stand. The two change apart from "
                          "then on; the new one says which it was duplicated from.")
            :parameters {:path [:map [:ref [:string {:min 1}]]]
                         :body [:map {:closed true} [:ref [:string {:min 1}]] [:name {:optional true} [:string {:min 1}]]]}
            :responses {200 {:body [:map [:scenario Scenario]]} 400 {:body ApiError} 404 {:body ApiError}
                        409 {:body ApiError}}
            :handler (fn [{{:keys [path body]} :parameters}]
                       (answer #(hash-map :scenario (scenario-out (scenario/duplicate! node (:ref path) body)))))}}]
   ["/scenarios/{ref}/versions"
    {:get {:summary "Every version of a scenario, oldest first, with when it was kept."
           :parameters {:path [:map [:ref [:string {:min 1}]]]}
           :responses {200 {:body [:map [:versions [:vector Scenario]]]}}
           :handler (fn [{{:keys [path]} :parameters}]
                      (answer #(hash-map :versions (mapv scenario-out (scenario/versions node (:ref path))))))}}]
   ["/scenarios/{ref}/evaluation"
    {:get {:summary (str "What a scenario adds to a book over a period, account by account, and what each layer of "
                         "it (its bases' first) adds. GET /trial-balance?scenario= shows the book with it.")
           :parameters {:path [:map [:ref [:string {:min 1}]]]
                        :query [:map [:book [:string {:min 1}]] [:from {:optional true} (schema/schema :ownpurse/date)]
                                [:to {:optional true} (schema/schema :ownpurse/date)]]}
           :responses {200 {:body [:map [:evaluation Evaluation]]} 400 {:body ApiError} 404 {:body ApiError}}
           :handler (fn [{{:keys [path query]} :parameters}]
                      (answer #(hash-map :evaluation
                                         (evaluation-out (scenario/evaluate node (:ref path) (:book query) query)))))}}]
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
  "The Ring handler over `node`, and the admin's built files at /admin. `local` is the config file naming the orgs to pull; `xero`, `mercury` and `plaid` are {:cli}:
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
   (ring/routes
    (ring/create-resource-handler {:path "/admin" :root "ownpurse/admin"})
    (ring/create-default-handler
     {:not-found (constantly (json-response (error 404 "not-found" "No such endpoint. GET /openapi.json lists them.")))
     :method-not-allowed (constantly (json-response (error 405 "method-not-allowed" "That method is not allowed here. GET /openapi.json lists the methods.")))
     :not-acceptable (constantly (json-response (error 406 "not-acceptable" "ownpurse answers in JSON, or in Transit for the admin.")))}))))
