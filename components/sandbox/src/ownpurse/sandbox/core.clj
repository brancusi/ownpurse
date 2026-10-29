(ns ownpurse.sandbox.core
  "A sandbox: bank lines, the world they happened in (people, cards, trips, projects), a rule set, and the runs that
  classified them. Today its lines are a classify scenario's invented ones; the same shapes will carry a book's real
  lines. Everything here is data for a client to show: no markup."
  (:require
   [clojure.edn :as edn]
   [clojure.java.io :as io]
   [clojure.string :as str]
   [ownpurse.classify.interface :as classify]))

(defn open
  "A sandbox over the runs kept in folder `dir`, and the scenarios in folder `scenarios` beside ownpurse's own: a
  workspace keeps its scenarios and runs in its own folders, never in this repository. Runs are read once and kept
  until their file changes."
  [{:keys [dir scenarios]}]
  {:dir dir :scenarios scenarios :cache (atom {})})

(defn- edn-names [dir]
  (into {} (keep #(when-let [n (second (re-matches #"(.+)\.edn" (.getName ^java.io.File %)))] [n %]))
        (some-> dir io/file .listFiles)))

(defn- sources
  "Every scenario by name: ownpurse's own (by name, on the classpath), then the workspace's (as files), which win."
  [{:keys [scenarios]}]
  (merge (when-let [u (io/resource "classify/eval")]
           (when (= "file" (.getProtocol u)) (update-vals (edn-names (io/file u)) (constantly nil))))
         (edn-names scenarios)))

(defn- source [sandbox name]
  (let [all (sources sandbox)]
    (if (contains? all name)
      (or (get all name) name)
      (throw (ex-info (str "No scenario " name ".") {:type :ownpurse/not-found :scenario name})))))

(defn- run-files
  "The scenario's saved runs, oldest first: not their progress or their notes."
  [{:keys [dir]} scenario]
  (->> (some-> (io/file dir) .listFiles)
       (filter #(let [n (.getName ^java.io.File %)]
                  (and (str/starts-with? n (str scenario "-")) (str/ends-with? n ".edn")
                       (not (re-find #"\.(notes|partial)\.edn$" n)))))
       (sort-by #(.getName ^java.io.File %))))

(defn- run-id [^java.io.File f] (str/replace (.getName f) #"\.edn$" ""))

(defn- run-file
  "The file of run `id` of `scenario`, or an error when there is none: ids are names in the runs folder, never paths."
  [sandbox scenario id]
  (or (some #(when (= id (run-id %)) %) (run-files sandbox scenario))
      (throw (ex-info (str "No run " id " of the scenario " scenario ".")
                      {:type :ownpurse/not-found :scenario scenario :run id}))))

(defn- cached [{:keys [cache]} ^java.io.File f read]
  (let [k [(.getPath f) (.lastModified f)]]
    (or (get @cache k)
        (get (swap! cache (fn [c] (assoc (into {} (remove #(= (first (key %)) (.getPath f))) c) k (read f)))) k))))

(defn- load-run [sandbox f] (cached sandbox f #(classify/load-run %)))

(defn- notes
  "Commentary on a run's failing lines, from the file beside it (its name with .notes.edn), when there is one."
  [^java.io.File f]
  (let [n (io/file (.getParentFile f) (str/replace (.getName f) #"\.edn$" ".notes.edn"))]
    (when (.exists n) (edn/read-string (slurp n)))))

(defn- scenario [sandbox name] (classify/scenario (source sandbox name)))

(defn- run-summary [run id]
  (-> (select-keys run [:at :model :source :usage :scenario-hash])
      (assoc :id id :results (count (:results run)))))

(defn scenarios
  "Every scenario, its size, and its runs, newest first."
  [sandbox]
  (vec (for [name (sort (keys (sources sandbox)))
             :let [s (scenario sandbox name)]]
         {:name name
          :title (str (str/capitalize name) "'s own rules")
          :buckets (if (:buckets s)
                     (vec (for [{:keys [codes]} (:buckets s) [_ label] codes] label))
                     (mapv #(str/capitalize (str/replace (clojure.core/name %) "-" " ")) (keys (:codes s))))
          :lines (count (:cases s))
          :units (count (mapcat classify/units (:cases s)))
          :rules (count (:rules s))
          :runs (vec (for [f (reverse (run-files sandbox name))]
                       {:id (run-id f) :at (second (re-find #"-(\d{4}-.+)$" (run-id f)))}))})))

(defn- glance
  "A line as a preview shows it: when, where, how much; and how many items its receipt lists."
  [{:keys [id date merchant amount currency items]}]
  (cond-> {:id id :date date :merchant merchant :amount amount :currency currency}
    (seq items) (assoc :items (count items))))

(defn line-sets
  "Every scenario's made-up lines as a set of lines to pick: {:id \"sample~<name>\" :kind \"sample\" :title :note
  :where :lines :units :scenarios :peek}, :scenarios its own rules (a file the eval runs); any scenario of the
  workspace runs over them too."
  [sandbox]
  (vec (for [name (sort (keys (sources sandbox)))
             :let [{:keys [cases business entities]} (scenario sandbox name)]]
         (cond-> {:id (str "sample~" name) :kind "sample" :title (str/capitalize name)
                  :where [["made up" (str "the " name " set, each line with the coding a bookkeeper would give it")]]
                  :lines (count cases) :units (count (mapcat classify/units cases)) :scenarios [name]}
           (get-in entities [business :name]) (assoc :note (str (get-in entities [business :name]) " and its people, invented"))
           true (assoc :peek (mapv glance (take 3 (sort-by :date cases))))))))

(defn over
  "Sample set `id` as a workspace's scenario runs over it: {:id :title :cases}, each line keyed as it says (its
  :expect); nil when there is no such sample."
  [sandbox id]
  (when-let [name (second (re-matches #"sample~(.+)" id))]
    (when (contains? (sources sandbox) name)
      {:id id :title (str/capitalize name) :cases (vec (:cases (scenario sandbox name)))})))

(defn line-set
  "Sample set `id` with a preview: its first two hundred lines, the days they span, and their currencies."
  [sandbox id]
  (let [name (second (re-matches #"sample~(.+)" id))
        s (some #(when (= id (:id %)) %) (line-sets sandbox))]
    (when-not s (throw (ex-info (str "No set of lines " id ".") {:type :ownpurse/not-found})))
    (let [cases (sort-by :date (:cases (scenario sandbox name)))]
      (assoc s
             :first (mapv glance (take 200 cases))
             :from (:date (first cases)) :to (:date (last cases))
             :currencies (into (sorted-map) (frequencies (keep :currency cases)))))))

(defn world
  "What a scenario's lines happened in: its people, cards, trips and projects, the buckets and their words, the
  categories and the rules. Everything but the lines."
  [sandbox scenario-name]
  (let [s (scenario sandbox scenario-name)]
    (-> (dissoc s :cases)
        (assoc :name scenario-name
               :buckets (or (:buckets s)
                            [{:title "Codes" :codes (vec (for [[c _] (:codes s)]
                                                           [c (str/capitalize (str/replace (name c) "-" " "))]))}])))))

(defn run-lines
  "A run of a scenario, one row a unit: the unit (its line, or one item of it, with the answer key), and each
  strategy's coding. Light: what a list needs, scored by the client at any threshold."
  [sandbox name id]
  (let [s (scenario sandbox name)
        f (run-file sandbox name id)
        run (load-run sandbox f)
        units (into {} (map (juxt :id identity)) (mapcat classify/units (:cases s)))
        noted (:lines (notes f))]
    {:run (run-summary run id)
     :stale (when (not= (:scenario-hash run) (classify/scenario-hash (source sandbox name)))
              "The scenario changed since this run: its answer key may differ from what the run was scored against.")
     :lines (vec (for [{:keys [id decisions]} (:results run)]
                   {:id id
                    :unit (dissoc (units id) :items)
                    :decisions (update-vals decisions #(select-keys % [:code :confidence :reason]))
                    :noted (contains? noted id)}))}))

(defn line
  "One unit of a run, all the way through: the bank line as sent, what code worked out, the requests exactly as sent
  (rebuilt from today's code when the run did not keep them), what came back, how each rule held, each strategy's
  decision, and the commentary on it."
  [sandbox name id line-id]
  (let [s (scenario sandbox name)
        f (run-file sandbox name id)
        run (load-run sandbox f)
        result (or (some #(when (= line-id (:id %)) %) (:results run))
                   (throw (ex-info (str "No line " line-id " in run " id ".")
                                   {:type :ownpurse/not-found :line line-id})))
        unit (some #(when (= line-id (:id %)) %) (mapcat classify/units (:cases s)))
        line (some #(when (= (or (:line unit) line-id) (:id %)) %) (:cases s))
        kept (classify/sent run result)
        facts (:facts result)]
    {:id line-id
     :run (run-summary run id)
     :unit unit
     :line line
     :facts facts
     :requests (or kept (classify/requests s facts))
     :enrich (if kept (:enrich kept) (classify/enrichment s unit))
     :rebuilt (nil? kept)
     :responses (:responses result)
     :decisions (:decisions result)
     :explained (classify/explain s result)
     :note (get-in (notes f) [:lines line-id])}))
