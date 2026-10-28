(ns sandbox-site
  "The sandbox's static copy for ownpurse.com: the API's own answers, in Transit, written as files the admin's :site
  build reads (bin/admin site). Only ownpurse's invented scenarios and the runs committed here, never a workspace's:
  the sandbox is opened with no scenarios folder of its own. Each scenario keeps only its newest run, every line of
  which is written whole.

    clojure -M:dev -m sandbox-site <out dir>"
  (:require
   [clojure.java.io :as io]
   [clojure.string :as str]
   [cognitect.transit :as transit]
   [muuntaja.core :as m]
   [ownpurse.api.handler :as handler]
   [ownpurse.api.json :as json]
   [ownpurse.store.interface :as store]))

(def ^:private transit-type "application/transit+json")

(defn- bytes-of
  "A body as bytes, whether it came as bytes or as a stream."
  ^bytes [body]
  (if (bytes? body)
    body
    (with-open [in (io/input-stream body) out (java.io.ByteArrayOutputStream.)]
      (io/copy in out)
      (.toByteArray out))))

(defn- answer
  "The API's answer at `path`, as data and as the bytes it sent."
  [app path]
  (let [{:keys [status body]} (app {:request-method :get :uri path :headers {"accept" transit-type}})
        bytes (bytes-of body)]
    (when (not= 200 status)
      (throw (ex-info (str "The API answered " status " at " path) {:path path :status status})))
    {:bytes bytes
     :data (transit/read (transit/reader (java.io.ByteArrayInputStream. bytes) :json
                                         {:handlers {"f" (transit/read-handler bigdec)}}))}))

(defn- write! [out path bytes]
  (let [f (io/file out (str (subs path 1) ".json"))]
    (io/make-parents f)
    (with-open [o (io/output-stream f)] (.write o ^bytes bytes))
    (alength ^bytes bytes)))

(defn export!
  "Writes every answer the sandbox and the docs ask for under folder `out`. Returns what it wrote."
  [out]
  (let [node (store/start {:mode :in-memory})]
    (try
      (let [app (handler/handler {:node node :sandbox {:dir "components/classify/runs" :scenarios nil}})
            scenarios (->> (:scenarios (:data (answer app "/sandbox/scenarios")))
                           (filter (comp seq :runs))
                           (mapv #(update % :runs (comp vec (partial take 1)))))
            sizes (atom {})
            put! (fn [path bytes] (swap! sizes assoc path (write! out path bytes)))]
        (put! "/sandbox/scenarios" (bytes-of (m/encode json/muuntaja transit-type {:scenarios scenarios})))
        (doseq [{:keys [name runs]} scenarios
                :let [world (str "/sandbox/scenarios/" name)
                      run (str world "/runs/" (:id (first runs)))
                      {:keys [bytes data]} (answer app run)]]
          (put! world (:bytes (answer app world)))
          (put! run bytes)
          (doseq [{:keys [id]} (:lines data)]
            (put! (str run "/lines/" id) (:bytes (answer app (str run "/lines/" id))))))
        {:scenarios (mapv :name scenarios)
         :files (count @sizes)
         :bytes (reduce + (vals @sizes))})
      (finally (store/stop node)))))

(defn -main [out]
  (let [{:keys [scenarios files bytes]} (export! out)]
    (println (str "Wrote " files " answers (" (quot bytes 1024) " KB) for " (str/join ", " scenarios)
                  " into " out))
    (shutdown-agents)))
