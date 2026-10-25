(ns ^{:clojure.tools.namespace.repl/load false} dev.watch
  "Tests on save. (start) watches every brick's src and test folders; when a .clj file is saved it reloads that
  namespace and runs the tests that cover it: the namespace itself if it is a test, otherwise every test in its
  brick. A one-line result goes to the REPL's output, and the whole run is kept for (last-run). A change to the
  running system (a new route, a component's config) still needs (dev/reset)."
  (:require
   [clojure.java.io :as io]
   [clojure.string :as str]
   [clojure.test :as test]
   [clojure.tools.namespace.file :as ns-file]
   [clojure.tools.namespace.find :as find]
   [nextjournal.beholder :as beholder]))

(defonce ^:private state (atom {:watcher nil :last-run nil}))

(defn- brick-dirs
  "Every src and test folder of every brick, as paths."
  []
  (for [kind ["components" "bases"]
        brick (.listFiles (io/file kind))
        :when (.isDirectory brick)
        sub ["src" "test"]
        :let [dir (io/file brick sub)]
        :when (.isDirectory dir)]
    (str dir)))

(defn- brick-of
  "The brick folder (components/xero) a file belongs to, or nil."
  [path]
  (second (re-find #"((?:components|bases)/[^/]+)/(?:src|test)/" (str path))))

(defn- test-namespaces
  "The tests to run after `ns-sym` in `brick` changed."
  [brick ns-sym]
  (if (str/ends-with? (name ns-sym) "-test")
    [ns-sym]
    (->> (find/find-namespaces-in-dir (io/file brick "test"))
         (filter #(str/ends-with? (name %) "-test"))
         sort)))

(defn- check-file
  "Reloads the namespace in `path` and runs its tests; returns what happened."
  [path]
  (let [ns-sym (second (ns-file/read-file-ns-decl (io/file path)))
        brick (brick-of path)]
    (when (and ns-sym brick)
      (try
        (require ns-sym :reload)
        (let [tests (test-namespaces brick ns-sym)
              _ (apply require tests)
              out (java.io.StringWriter.)
              summary (binding [test/*test-out* out] (apply test/run-tests tests))]
          {:file (str path) :ns ns-sym :tests tests
           :summary (select-keys summary [:test :pass :fail :error])
           :out (str out)})
        (catch Throwable e
          {:file (str path) :ns ns-sym :error (ex-message e) :exception e})))))

(defn- line
  "One line for the REPL's output."
  [{:keys [ns summary error]}]
  (if error
    (str "watch: " ns " did not load: " error)
    (let [{:keys [test pass fail error]} summary]
      (str "watch: " ns ": " test " tests, " pass " passed"
           (when (pos? (+ fail error)) (str ", " fail " failed, " error " errors: (dev/last-run) shows them"))))))

(defn stop
  "Stops watching."
  []
  (when-let [w (:watcher @state)] (beholder/stop w))
  (swap! state assoc :watcher nil)
  :stopped)

(defn start
  "Watches the bricks and runs the covering tests on every save. Calling it again restarts the watcher."
  []
  (stop)
  (let [out *out*
        on-change (fn [{:keys [type path]}]
                    (when (and (#{:create :modify} type) (str/ends-with? (str path) ".clj"))
                      (locking state
                        (when-let [run (try (check-file (str path))
                                            (catch Throwable e {:file (str path) :error (ex-message e) :exception e}))]
                          (swap! state assoc :last-run run)
                          (binding [*out* out] (println (line run)))))))]
    (swap! state assoc :watcher (apply beholder/watch on-change (brick-dirs)))
    :watching))

(defn last-run
  "What the last save ran: the file, its namespace, the tests, the summary and their output (or the load error)."
  []
  (:last-run @state))
