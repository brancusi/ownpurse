(ns ownpurse.api.handler
  "The HTTP API: routes, schemas and the Ring handler. Thin: it turns requests into interface calls and back."
  (:require
   [malli.error :as me]
   [ownpurse.api.json :as api.json]
   [ownpurse.compare.interface :as cmp]
   [ownpurse.entity.interface :as entity]
   [ownpurse.json.interface :as json]
   [ownpurse.mercury.interface :as mercury]
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

(def Health
  "What GET /health returns."
  [:map [:status [:enum "ok" "unavailable"]]])

(def Counts
  "For each org, for each resource: the items the source returned and the rows written."
  [:map-of :string [:map-of :string [:map [:items :int] [:written :int]]]])

(def Pulled
  "What POST /pull returns: the counts for each source's orgs."
  [:map [:pulled [:map [:xero Counts] [:mercury Counts]]]])

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
      {:status 200 :body {:status "ok"}}
      {:status 503 :body {:status "unavailable"}})))

(defn- error-type [t]
  (if (namespace t) (str (namespace t) "/" (name t)) (name t)))

(defn- pull-all!
  "Pulls every org the local config names, Xero's then Mercury's."
  [node {:keys [local xero mercury]}]
  (let [xero-orgs (xero/read-orgs local)
        mercury-orgs (mercury/read-orgs local)]
    (when (and (empty? xero-orgs) (empty? mercury-orgs))
      (throw (ex-info (str local " names no orgs. Add {:xero {:orgs [{:tenant \"<tenant id>\" :name \"<a name>\"}]}} "
                           "or {:mercury {:orgs [{:alias \"<mercury-cli alias>\" :name \"<a name>\"}]}}.")
                      {:type :ownpurse/no-local-config :path (str local)})))
    {:xero (if (seq xero-orgs) (xero/pull-orgs! node {:cli (:cli xero) :orgs xero-orgs}) {})
     :mercury (if (seq mercury-orgs) (mercury/pull-orgs! node {:cli (:cli mercury) :orgs mercury-orgs}) {})}))

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
                (#{"xero" "mercury"} (some-> t namespace)) (error 502 (error-type t) (ex-message e))
                :else (throw e))))
          (finally (reset! running false)))))))

(def ^:private entity-errors
  {:ownpurse/invalid 400 :ownpurse/invalid-link 400 :ownpurse/invalid-match 400 :ownpurse/not-found 404
   :ownpurse/link-conflict 409 :ownpurse/match-conflict 409})

(defn- answer
  "Calls `f` and answers 200 with what it returns, or the error the entity component threw, as structured JSON."
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
   ["/pull"
    {:post {:summary (str "Fetches every org in the local config through its source's tool, and keeps what changed. "
                          "Costs the source's API calls; later pulls ask only for what changed.")
            :responses {200 {:body Pulled} 409 {:body ApiError} 502 {:body ApiError}}
            :handler (pull node (select-keys config [:local :xero :mercury]))}}]
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
  "The Ring handler over `node`. `local` is the config file naming the orgs to pull; `xero` and `mercury` are {:cli}:
  the program that reads each source."
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
