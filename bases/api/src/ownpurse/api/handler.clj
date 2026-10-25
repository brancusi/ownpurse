(ns ownpurse.api.handler
  "The HTTP API: routes, schemas and the Ring handler. Thin: it turns requests into interface calls and back."
  (:require
   [malli.error :as me]
   [ownpurse.api.json :as api.json]
   [ownpurse.json.interface :as json]
   [ownpurse.mercury.interface :as mercury]
   [ownpurse.store.interface :as store]
   [ownpurse.xero.interface :as xero]
   [reitit.coercion.malli :as rcm]
   [reitit.openapi :as openapi]
   [reitit.ring :as ring]
   [reitit.ring.coercion :as coercion]
   [reitit.ring.middleware.exception :as exception]
   [reitit.ring.middleware.muuntaja :as muuntaja]))

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
            :handler (pull node (select-keys config [:local :xero :mercury]))}}]])

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
