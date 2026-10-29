(ns ownpurse.admin.api
  "Reading the ownpurse API in Transit, so the components' data arrive as they hold it: keywords, sets, maps. Decimals
  arrive as numbers: the admin shows them, it never adds them up."
  (:require
   [cognitect.transit :as transit]
   [uix.core :refer [use-effect use-state]]))

(def ^:private reader
  (transit/reader :json {:handlers {"f" (fn [s] (js/Number s))
                                    "instant" identity
                                    "date" identity}}))

(goog-define ^{:doc "Where the answers come from. Empty: the ownpurse server this page came from. On ownpurse.com the
  sandbox is a static copy: the answers written as files under this path (bin/admin site)."}
  base "")

(goog-define ^{:doc "Appended to each path in the static copy: an answer is a file, sandbox/scenarios.json."}
  suffix "")

(def static?
  "Whether this is the static copy on ownpurse.com, with invented data and no server."
  (not= "" suffix))

(defonce ^:private cache
  ;; Answers by path, kept until a write: a sandbox run is a file that does not change once written, and a scenario
  ;; changes only through this page's writes, each of which empties the cache (invalidate!).
  (atom {}))

(defonce ^:private generation
  ;; Counts the writes: every use-get asks again when it moves.
  (atom 0))

(defn invalidate!
  "Forgets every answer, and has every open view ask again."
  []
  (reset! cache {})
  (swap! generation inc))

(defn- answered
  "A promise of the body of fetch response `res`, read from Transit; rejected with the API's message when it failed."
  [res]
  (-> (.text res)
      (.then (fn [text]
               (let [body (try (transit/read reader text) (catch :default _ nil))]
                 (if (.-ok res)
                   body
                   (throw (ex-info (or (get-in body [:error :message]) (str "The API answered " (.-status res)))
                                   {:status (.-status res)}))))))))

(defn fetch!
  "GETs `path` and returns a promise of its answer, read from Transit. `fresh` asks the server even when the answer
  is kept."
  ([path] (fetch! path false))
  ([path fresh]
   (if-let [hit (and (not fresh) (get @cache path))]
     (js/Promise.resolve hit)
     (-> (js/fetch (str base path suffix) #js {:headers #js {"Accept" "application/transit+json"}})
         (.then answered)
         (.then (fn [body] (swap! cache assoc path body) body))))))

(defn send!
  "Sends `method` (\"PUT\", \"POST\", \"DELETE\") to `path` with `body` as JSON (keywords as strings, sets as arrays,
  as a JSON client would), and returns a promise of the answer. Every write forgets the kept answers."
  ([method path] (send! method path nil))
  ([method path body]
   (-> (js/fetch (str base path)
                 (clj->js (cond-> {:method method :headers {"Accept" "application/transit+json"
                                                            "Content-Type" "application/json"}}
                            (some? body) (assoc :body (js/JSON.stringify (clj->js body))))))
       (.then answered)
       (.then (fn [body] (invalidate!) body)))))

(defn use-generation
  "The count of writes so far, kept up to date: a view that reads it asks again after every write."
  []
  (let [[g set-g!] (use-state @generation)]
    (use-effect
     (fn []
       (let [k (gensym "generation")]
         (add-watch generation k (fn [_ _ _ n] (set-g! n)))
         #(remove-watch generation k)))
     [])
    g))

(defn use-get
  "The answer at `path` as {:data} once it has come, {:error} when it failed, {:loading true} meanwhile. A nil path
  asks for nothing. It asks again after any write; with `every` (milliseconds), it also asks the server again that
  often, keeping the answer it has meanwhile (for progress)."
  ([path] (use-get path nil))
  ([path {:keys [every]}]
   (let [[state set-state!] (use-state (if path {:loading true} {}))
         g (use-generation)]
     (use-effect
      (fn []
        (let [live (atom true)
              get! (fn [fresh]
                     (-> (fetch! path fresh)
                         (.then #(when @live (set-state! {:data % :path path})))
                         (.catch #(when @live (set-state! {:error (ex-message %) :path path})))))
              timer (when (and path every) (js/setInterval #(get! true) every))]
          (when path
            (set-state! (fn [s] (if (and (:data s) (= path (:path s))) s {:loading true :path path})))
            (get! false))
          #(do (reset! live false) (when timer (js/clearInterval timer)))))
      [path g every])
     state)))

(defn use-send
  "A write and its state, for a button: {:send! (fn [method path body] → promise of the answer, nil when it failed), :busy, :error,
  :clear!}. The error is the API's message, kept until the next write or clear!."
  []
  (let [[busy set-busy!] (use-state false)
        [error set-error!] (use-state nil)]
    {:busy busy
     :error error
     :clear! #(set-error! nil)
     :send! (fn [method path body]
              (set-busy! true)
              (set-error! nil)
              (-> (send! method path body)
                  (.then (fn [b] (set-busy! false) b))
                  (.catch (fn [e] (set-busy! false) (set-error! (ex-message e)) nil))))}))
