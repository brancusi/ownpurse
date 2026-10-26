(ns ownpurse.store.backup
  "Snapshots of a running record, and restoring one. XTDB writes its storage files once and never changes them,
  aligned with the blocks it flushes from the log; a snapshot asks the node to flush a block, copies every storage
  file up to that block and checks that a fresh node opens the copy. A restored record starts a new log, in the
  next log epoch, as XTDB requires of a node whose storage is ahead of its log. The functions here reach into XTDB's
  internals (the flush message, the block index); keep them in this namespace."
  (:require
   [clojure.edn :as edn]
   [clojure.java.io :as io]
   [xtdb.api :as xt]
   [xtdb.db-catalog :as db]
   [xtdb.log :as xt-log]
   [xtdb.node :as xtn])
  (:import
   (java.nio.file CopyOption FileVisitOption Files LinkOption NoSuchFileException Path StandardCopyOption)
   (java.nio.file.attribute FileAttribute)
   (java.time Duration Instant ZoneOffset)
   (java.time.format DateTimeFormatter)
   (xtdb.api.log SourceMessage$FlushBlock)
   (xtdb.api.storage Storage)
   (xtdb.database Database)))

(def ^:private manifest-name "snapshot.edn")

(defn- path ^Path [f & more]
  (.toPath ^java.io.File (apply io/file f more)))

(defn- storage-root
  "Where XTDB keeps its files under the storage directory (\"v06\" in XTDB 2.2)."
  ^Path []
  (Storage/storageRoot Storage/VERSION 0))

(defn- epoch-file [dir]
  (io/file dir "xtdb" "epoch.edn"))

(defn epoch
  "The log epoch of the record in `dir`: 0, unless it was restored from a snapshot."
  [dir]
  (let [f (epoch-file dir)]
    (if (.exists f) (edn/read-string (slurp f)) 0)))

(defn sync!
  "Waits until the node has indexed every message on its log, at most `timeout`: a node just opened answers
  queries before it has caught up."
  [node ^Duration timeout]
  (xt-log/sync-node node timeout))

(defn- block-index [^Database db]
  (.getCurrentBlockIndex (.getTableCatalog db)))

(defn- flush-block!
  "Asks the node to finish its current block, and waits until it has: every transaction before the request is then
  in storage files. Returns the new block's index."
  [node ^Duration timeout]
  (let [^Database db (db/primary-db node)
        before (block-index db)
        deadline (.plus (Instant/now) timeout)]
    (.appendMessageBlocking (.getSourceLog db) (SourceMessage$FlushBlock. (or before -1)) 0)
    (loop []
      (let [now (block-index db)]
        (cond
          (and (some? now) (not= before now)) now
          (.isAfter (Instant/now) deadline)
          (throw (ex-info (str "The record did not finish a block within " (.toSeconds timeout) " seconds. Try "
                               "again; if it keeps failing, stop ownpurse and copy the data directory instead.")
                          {:type :ownpurse/backup-timeout}))
          :else (do (Thread/sleep 25) (recur)))))))

(defn- lex-hex
  "Reads XTDB's lexically ordered hex: the first digit says how many more follow, less one."
  [^String s]
  (Long/parseLong (subs s 1) 16))

(defn- file-block
  "The block a storage file belongs to, from its name (b1e0.binpb, l01-rc-b1e0.arrow), or nil."
  [^Path p]
  (some->> (str (.getFileName p)) (re-find #"(?:^|-)b([0-9a-f]+)\.(?:binpb|arrow)$") second lex-hex))

(defn- regular-files [^Path root]
  (with-open [s (Files/walk root (make-array FileVisitOption 0))]
    (->> (iterator-seq (.iterator s))
         (filter #(Files/isRegularFile % (make-array LinkOption 0)))
         (mapv #(.relativize root %)))))

(defn- snapshot-files
  "The storage files a snapshot at `block` needs: every file of a block up to it. Files of a later block, and
  XTDB's own scratch (.tmp, exports), are left out."
  [^Path root block]
  (->> (regular-files root)
       (remove #(re-find #"^(?:\.tmp|exports)(?:/|$)" (str %)))
       (filter #(let [b (file-block %)] (or (nil? b) (<= b block))))))

(defn- copy-files!
  "Copies `files` from `from` to `to`, both storage roots. A file gone before it was copied was removed by XTDB's
  garbage collector, which removes only files no block refers to any longer; it is left out. Returns
  {:files n :bytes n}."
  [^Path from ^Path to files]
  (reduce (fn [acc ^Path p]
            (let [dst (.resolve to p)]
              (Files/createDirectories (.getParent dst) (make-array FileAttribute 0))
              (try
                (Files/copy (.resolve from p) dst ^"[Ljava.nio.file.CopyOption;" (into-array CopyOption [StandardCopyOption/COPY_ATTRIBUTES]))
                (-> acc (update :files inc) (update :bytes + (Files/size dst)))
                (catch NoSuchFileException _ acc))))
          {:files 0 :bytes 0}
          files))

(defn- node-on
  "Opens a node on the storage in `dir` (a data directory) with the log epoch `log-epoch`. `opts`: :log-in-memory?
  (a check that writes nothing to a log on disk)."
  [dir log-epoch {:keys [log-in-memory?]}]
  (let [root (io/file dir "xtdb")]
    (.open (doto (xtn/->config {:log (if log-in-memory?
                                       [:in-memory {:epoch log-epoch}]
                                       [:local {:path (.toPath (io/file root "log")) :epoch log-epoch}])
                                :storage [:local {:path (.toPath (io/file root "objects"))}]})
             (-> (.getCompactor) (.threads 0))
             (.setServer nil)
             (.setFlightSql nil)))))

(defn- tables
  "Every table in the record, as SQL names, XTDB's own included."
  [node]
  (->> (xt/q node (str "SELECT table_schema, table_name FROM information_schema.tables "
                       "WHERE table_schema NOT IN ('information_schema', 'pg_catalog')"))
       (map (fn [{:keys [table-schema table-name]}] (str table-schema "." table-name)))
       sort))

(defn- count-txs [node latest-tx-id]
  (:n (first (xt/q node ["SELECT COUNT(*) AS n FROM xt.txs WHERE _id <= ?" latest-tx-id]))))

(defn- check-copy!
  "Opens the copy in `dir` as a restore would, in the next epoch but with a log in memory, reads every version of every table, and checks that it holds the
  transactions the record held up to `latest-tx-id`. Returns the tables' names. Throws :ownpurse/backup-invalid."
  [dir storage-epoch latest-tx-id expected-txs]
  (with-open [^java.lang.AutoCloseable copy (node-on dir (inc storage-epoch) {:log-in-memory? true})]
    (let [names (tables copy)]
      (doseq [t names]
        (xt/q copy (str "SELECT COUNT(*) AS n FROM " t " FOR ALL SYSTEM_TIME FOR ALL VALID_TIME")))
      (let [held (count-txs copy latest-tx-id)]
        (when (not= expected-txs held)
          (throw (ex-info (str "The snapshot holds " held " transactions, not " expected-txs ". It was not kept; "
                               "try again.")
                          {:type :ownpurse/backup-invalid :expected expected-txs :held held}))))
      names)))

(defn- delete-tree! [^java.io.File f]
  (when (.isDirectory f)
    (run! delete-tree! (.listFiles f)))
  (io/delete-file f true))

(def ^:private stamp (.withZone (DateTimeFormatter/ofPattern "yyyyMMdd'T'HHmmss'Z'") ZoneOffset/UTC))

(defn snapshot!
  "Snapshots the running record of `node`, kept on disk in `dir`, into a new folder under `to`. Returns the
  manifest, also written to the folder as snapshot.edn."
  [node {:keys [dir to timeout] :or {timeout (Duration/ofSeconds 60)}}]
  (let [taken-at (Instant/now)
        _ (sync! node timeout)
        block (flush-block! node timeout)
        latest-tx-id (or (some-> (xt/q node "SELECT MAX(_id) AS id FROM xt.txs") first :id) -1)
        expected-txs (count-txs node latest-tx-id)
        folder (str "ownpurse-" (.format stamp taken-at))
        final (io/file to folder)
        partial (io/file to (str "." folder ".partial"))
        log-epoch (epoch dir)]
    (when (.exists final)
      (throw (ex-info (str final " already exists. Take the snapshot again in a second.")
                      {:type :ownpurse/backup-exists :dir (str final)})))
    (try
      (let [from (.resolve (path dir "xtdb" "objects") (storage-root))
            copied (copy-files! from (.resolve (path partial "xtdb" "objects") (storage-root))
                                (snapshot-files from block))
            names (check-copy! partial log-epoch latest-tx-id expected-txs)
            manifest (merge {:format 1 :taken-at (str taken-at) :block block :epoch log-epoch :latest-tx-id latest-tx-id
                             :transactions expected-txs :tables (count names)}
                            copied)]
        (spit (io/file partial manifest-name) (pr-str manifest))
        (Files/move (.toPath partial) (.toPath final) (into-array CopyOption [StandardCopyOption/ATOMIC_MOVE]))
        (assoc manifest :dir (str final)))
      (catch Throwable t
        (delete-tree! partial)
        (throw t)))))

(defn read-manifest
  "The manifest of the snapshot in `from`. Throws :ownpurse/not-a-snapshot when there is none."
  [from]
  (let [f (io/file from manifest-name)]
    (when-not (.exists f)
      (throw (ex-info (str from " is not an ownpurse snapshot: it has no " manifest-name ".")
                      {:type :ownpurse/not-a-snapshot :dir (str from)})))
    (edn/read-string (slurp f))))

(defn restore!
  "Restores the snapshot in `from` as a new record in the data directory `dir`, which must not hold one. The record
  starts a new log in the next epoch. Returns {:dir :epoch :block :transactions}."
  [{:keys [from dir]}]
  (let [{:keys [block transactions] storage-epoch :epoch} (read-manifest from)
        target (io/file dir "xtdb")]
    (when (.exists target)
      (throw (ex-info (str target " already exists: restore into an empty data directory, never over a record.")
                      {:type :ownpurse/record-exists :dir (str target)})))
    (let [src (.resolve (path from "xtdb" "objects") (storage-root))]
      (copy-files! src (.resolve (path dir "xtdb" "objects") (storage-root)) (regular-files src)))
    (spit (epoch-file dir) (pr-str (inc storage-epoch)))
    {:dir (str dir) :epoch (inc storage-epoch) :block block :transactions transactions}))
