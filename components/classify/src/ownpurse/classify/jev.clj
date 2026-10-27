(ns ownpurse.classify.jev
  "TypeSafe's System One API (Jev): state and typed questions in, typed answers out. https://docs.typesafe.ai/api"
  (:require
   [ownpurse.json.interface :as json])
  (:import
   (java.net URI)
   (java.net.http HttpClient HttpRequest HttpRequest$BodyPublishers HttpResponse$BodyHandlers)
   (java.time Duration)))

(def ^:private http (delay (HttpClient/newHttpClient)))

(def ^:private retryable #{429 529})

(defn- post [{:keys [url api-key]} body]
  (let [request (-> (HttpRequest/newBuilder (URI/create (str url "/v1/systemone")))
                    (.timeout (Duration/ofSeconds 60))
                    (.header "Authorization" (str "Bearer " api-key))
                    (.header "Content-Type" "application/json")
                    (.POST (HttpRequest$BodyPublishers/ofString (json/write-str body)))
                    (.build))
        response (.send ^HttpClient @http request (HttpResponse$BodyHandlers/ofString))]
    {:status (.statusCode response) :body (.body response)}))

(defn ask
  "Asks Jev `questions` (a map of id to question) about `state`, and returns the response: :model, :answers by the
  same ids, and :usage. Retries a rate limit or an overload with backoff."
  [{:keys [api-key model] :as jev} state questions]
  (when-not api-key
    (throw (ex-info (str "No TypeSafe API key. Set TYPESAFE_API_KEY, or put {:typesafe {:api-key \"...\"}} in "
                         "~/.config/ownpurse/secrets.edn (or the file OWNPURSE_SECRETS names).")
                    {:type :ownpurse/no-api-key})))
  (let [body {:model model :state state :questions questions}]
    (loop [attempt 1]
      (let [{:keys [status] :as response} (post jev body)]
        (cond
          (= 200 status) (json/parse (:body response))
          (and (retryable status) (< attempt 5)) (do (Thread/sleep (* 500 (bit-shift-left 1 attempt)))
                                                     (recur (inc attempt)))
          :else (throw (ex-info (str "TypeSafe answered " status ": " (:body response))
                                {:type :ownpurse/jev-error :status status})))))))
