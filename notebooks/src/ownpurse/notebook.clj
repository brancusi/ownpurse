(ns ownpurse.notebook
  "Clerk notebooks over a record. The REPL that holds a record says which one the notebooks read (use-record!),
  starts Clerk (serve!) and shows a notebook with its params (show!). A notebook reads the record only through
  here, so the same notebook shows the synthetic samples in this repository's REPL and a private record in a
  workspace's REPL. Clerk's cache is off: nothing a notebook shows is written to disk, and a notebook shown again
  reads the record again."
  (:require
   [clojure.string :as str]
   [nextjournal.beholder :as beholder]
   [nextjournal.clerk :as clerk]
   [nextjournal.clerk.config :as clerk.config]
   [nextjournal.clerk.viewer :as viewer])
  (:import
   (xtdb.api Xtdb)))

;; Clerk's cache keeps each result on disk under .clerk/ and reuses it while the code is unchanged: a record's
;; figures would be written beside the code, and a notebook would show them stale after a pull.
(alter-var-root #'clerk.config/cache-disabled? (constantly true))

;; {:record (fn [] node) :params {...} :shown notebook :watcher w}
(defonce ^:private state (atom {:params {}}))

(defn- not-a-node [x]
  (ex-info (str "The notebooks need an XTDB node in this JVM, and were given " (if x (.getName (class x)) "nothing")
                ". A client of a server running elsewhere will not do: the reports call ownpurse's components "
                "on the node. Stop that server and open the record in this REPL, then call use-record! again.")
           {:type :ownpurse/no-record :given (some-> x class .getName)}))

(defn use-record!
  "Says which record the notebooks read: `node-fn`, a function of no arguments that returns an XTDB node (called on
  each read, so it follows a restarted system), and `label`, what every notebook names it by at its top (where the
  record is: \"data/ in ownpurse, the dev system\"). Throws when `node-fn` returns something else, such as a client
  of a server running elsewhere; a record not opened yet is let through until a notebook reads it."
  ([node-fn] (use-record! node-fn "a record the REPL did not name"))
  ([node-fn label]
   (let [x (try (node-fn) (catch Exception _ nil))]
     (when (and x (not (instance? Xtdb x)))
       (throw (not-a-node x))))
   (swap! state assoc :record node-fn :label label)
   :ok))

(defn label
  "What the REPL called the record the notebooks read."
  []
  (:label @state))

(defn record
  "The XTDB node the notebooks read. Throws, saying what to do, when the REPL has not said which or it is not a
  node."
  []
  (if-let [f (:record @state)]
    (let [x (f)]
      (if (instance? Xtdb x) x (throw (not-a-node x))))
    (throw (ex-info (str "No record for the notebooks. In the REPL that holds one, call "
                         "(ownpurse.notebook/use-record! (fn [] node)).")
                    {:type :ownpurse/no-record}))))

(defn params
  "The params the notebook being shown was given: {:org :date :reports ...}."
  []
  (:params @state))

(def ^:private notebooks
  "Short names for the notebooks here."
  {:tour 'ownpurse.notebooks.tour
   :book 'ownpurse.notebooks.book})

(def reports
  "The reports the book notebook shows, by keyword, as the export names its sheets."
  (array-map :readme "README" :trial-balance "Trial balance" :balance-sheet "Balance sheet"
             :profit-and-loss "Profit and loss" :general-ledger "General ledger" :adjustments "Adjustments"
             :exclusions "Exclusions" :bank-reconciliation "Bank reconciliation"
             :unreconciled-lines "Unreconciled lines" :intercompany "Intercompany" :payees "Payees"))

(defn show!
  "Shows a notebook: a keyword from `reports` shows that report of a book ((show! :profit-and-loss {:org \"Tallowmere\"
  :date \"2026-02-28\"})); :tour or :book a notebook here; any other symbol or file path that notebook. `params` are
  what the notebook reads with (params): for a book, :org (its id or the start of its name), :date (the end of the
  financial year), :reports (keywords; all when absent), :pro-forma (\"approved\" or \"proposed\")."
  ([notebook] (show! notebook {}))
  ([notebook params]
   (let [[notebook params] (if (contains? reports notebook)
                             [:book (update params :reports #(or % [notebook]))]
                             [notebook params])
         target (get notebooks notebook notebook)]
     (swap! state assoc :params params :shown target)
     (clerk/show! target)
     :shown)))

(defn reshow!
  "Shows the last notebook again with the same params: after a pull, or after reloading changed code."
  []
  (when-let [target (:shown @state)]
    (clerk/show! target)
    :shown))

(defn- reload-and-reshow!
  "On a save of a source file under the watched code (not a test, not a notebook): loads it, then shows the last
  notebook again."
  [{:keys [path]}]
  (let [file (str path)]
    (when (and (str/ends-with? file ".clj") (str/includes? file "/src/") (not (str/includes? file "/notebooks/")))
      (try (load-file file)
           (reshow!)
           (catch Throwable t
             (println (str "Notebook not shown again: " file " did not load: " (ex-message t))))))))

(defn serve!
  "Starts Clerk on 127.0.0.1. opts: :port (7777), :browse (true opens a browser), :watch-paths (folders of notebooks:
  a saved notebook is shown), :code-paths (folders of code the notebooks call: a saved file is loaded and the last
  notebook shown again)."
  ([] (serve! {}))
  ([{:keys [port browse watch-paths code-paths]}]
   (clerk/serve! (cond-> {:host "127.0.0.1" :port (or port 7777) :browse (boolean browse)}
                   (seq watch-paths) (assoc :watch-paths watch-paths)))
   (some-> (:watcher @state) beholder/stop)
   (swap! state assoc :watcher (when (seq code-paths)
                                 (apply beholder/watch #(future (reload-and-reshow! %)) code-paths)))
   {:url (str "http://127.0.0.1:" (or port 7777))}))

(defn halt!
  "Stops Clerk and the code watcher."
  []
  (some-> (:watcher @state) beholder/stop)
  (swap! state dissoc :watcher)
  (clerk/halt!)
  :halted)

;; Viewers

(defn- amount
  "An amount as the reports print it: its exact decimal, right-aligned."
  [^BigDecimal a]
  (clerk/html [:span.tabular-nums {:style {:display "block" :text-align "right"}} (.toPlainString a)]))

(defn- id-column? [column]
  (or (str/ends-with? column " id") (= "entry" column)))

(defn table
  "A report as ownpurse.export.interface/reports returns it, {:columns :rows}, as a Clerk table: amounts exact and
  right-aligned, text as it is. The columns of ids (\"account id\", \"entry\") are left out unless :ids is true."
  ([report] (table report {}))
  ([{:keys [columns rows]} {:keys [ids]}]
   (let [keep (if ids
                (range (count columns))
                (remove #(id-column? (nth columns %)) (range (count columns))))
         cell #(if (decimal? %) (amount %) (str %))]
     (clerk/with-viewer viewer/table-viewer {:nextjournal.clerk/width :wide}
       {:head (mapv columns keep)
        :rows (mapv (fn [r] (mapv #(cell (nth r %)) keep)) rows)}))))
