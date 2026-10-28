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

(defonce ^:private cache
  ;; Answers by path, kept for the page's life: runs are files that do not change once written.
  (atom {}))

(defn fetch!
  "GETs `path` and returns a promise of its answer, read from Transit."
  [path]
  (if-let [hit (get @cache path)]
    (js/Promise.resolve hit)
    (-> (js/fetch path #js {:headers #js {"Accept" "application/transit+json"}})
        (.then (fn [res]
                 (-> (.text res)
                     (.then (fn [text]
                              (let [body (transit/read reader text)]
                                (if (.-ok res)
                                  (do (swap! cache assoc path body) body)
                                  (throw (ex-info (get-in body [:error :message] (str "The API answered " (.-status res)))
                                                  {:status (.-status res)}))))))))))))

(defn use-get
  "The answer at `path` as {:data} once it has come, {:error} when it failed, {:loading true} meanwhile. A nil path
  asks for nothing."
  [path]
  (let [[state set-state!] (use-state (if path {:loading true} {}))]
    (use-effect
     (fn []
       (let [live (atom true)]
         (when path
           (set-state! {:loading true})
           (-> (fetch! path)
               (.then #(when @live (set-state! {:data %})))
               (.catch #(when @live (set-state! {:error (ex-message %)})))))
         #(reset! live false)))
     [path])
    state))
