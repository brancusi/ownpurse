(ns ownpurse.store.backup-test
  (:require
   [clojure.java.io :as io]
   [clojure.test :refer [deftest is testing]]
   [ownpurse.store.interface :as store])
  (:import
   (java.nio.file Files)
   (java.nio.file.attribute FileAttribute)))

(defn- temp-dir []
  (.toFile (Files/createTempDirectory "ownpurse-backup-test" (make-array FileAttribute 0))))

(defn- delete-tree! [^java.io.File f]
  (when (.isDirectory f) (run! delete-tree! (.listFiles f)))
  (io/delete-file f true))

(defn- error [f]
  (try (f) nil (catch clojure.lang.ExceptionInfo e (ex-data e))))

(def ^:private metadata {:source "test" :why "fixtures"})

(defn- history [node]
  (store/q node (str "SELECT _id, amount, _valid_from, _valid_to, _system_from, _system_to FROM probes "
                     "FOR ALL SYSTEM_TIME FOR ALL VALID_TIME ORDER BY _id, _system_from, _valid_from")))

(defn- txs [node]
  (store/q node "SELECT _id, system_time, committed, user_metadata FROM xt.txs ORDER BY _id"))

(deftest snapshot-and-restore-test
  ;; Every folder has a space in its name, as synced folders on macOS do (CloudStorage/Dropbox-Team/Full Name).
  (let [tmp (temp-dir)
        record {:mode :disk :dir (str (io/file tmp "the record"))}
        restored {:mode :disk :dir (str (io/file tmp "restored record"))}
        backups (io/file tmp "Some Person" "backups")
        receipt (store/keep-file! record (fn [^java.io.OutputStream out] (.write out (.getBytes "a receipt"))))]
    (try
      (let [node (store/start record)
            _ (store/put! node {:probes [{:xt/id "p1" :amount 10.00M :xt/valid-from #inst "2026-01-01"}]} metadata)
            _ (store/put! node {:probes [{:xt/id "p1" :amount 12.50M :xt/valid-from #inst "2026-02-01"}
                                         {:xt/id "p2" :amount 3.10M}]}
                          metadata)
            _ (store/delete! node :probes ["p2"] {} metadata)
            manifest (store/snapshot! node record (str backups))]
        (store/put! node {:probes [{:xt/id "p3" :amount 1.00M}]} metadata)
        (testing "a snapshot of a running record lands in its own folder, with a manifest"
          (is (= #{"ownpurse-"} (set (map #(subs % 0 9) (.list backups)))))
          (is (= (str (io/file backups (.getName (io/file (:dir manifest))))) (:dir manifest)))
          (is (.exists (io/file (:dir manifest) "snapshot.edn")))
          (is (= {:epoch 0 :transactions 3 :kept-files 1 :kept-bytes 9}
                 (select-keys manifest [:epoch :transactions :kept-files :kept-bytes])))
          (is (.exists (io/file (:dir manifest) (:path receipt))))
          (is (pos? (:files manifest))))
        (testing "the record remembers its last snapshot"
          (is (= (:dir manifest) (:dir (store/last-snapshot node)))))
        (store/stop node)
        (testing "a restore refuses a folder that is not a snapshot"
          (is (= :ownpurse/not-a-snapshot (:type (error #(store/restore! {:from (str tmp) :dir (:dir restored)})))))))
      (let [[snapshot] (map #(io/file backups %) (.list backups))
            node (store/start record)
            full (history node)
            full-txs (txs node)]
        (store/stop node)
        (is (= {:epoch 1 :transactions 3} (select-keys (store/restore! {:from (str snapshot) :dir (:dir restored)})
                                                       [:epoch :transactions])))
        (testing "a restored record holds every version with its valid and system times, and every transaction"
          (let [node (store/start restored)]
            (try
              (is (= (remove #(= "p3" (:xt/id %)) full) (history node)))
              (is (= (take 3 full-txs) (txs node)))
              (is (= "a receipt" (slurp (io/file (:dir restored) (:path receipt)))))
              (store/put! node {:probes [{:xt/id "p4" :amount 4.00M}]} metadata)
              (finally (store/stop node)))))
        (testing "and goes on in a new epoch: what it writes after the restore survives a restart"
          (let [node (store/start restored)]
            (try
              (is (= ["p1" "p4"] (map :xt/id (store/q node "SELECT _id FROM probes ORDER BY _id"))))
              (is (= 1 (:epoch (store/snapshot! node restored (str (io/file tmp "Some Person" "backups 2"))))))
              (finally (store/stop node)))))
        (testing "a restore never writes over a record"
          (is (= :ownpurse/record-exists
                 (:type (error #(store/restore! {:from (str snapshot) :dir (:dir restored)})))))))
      (finally (delete-tree! tmp)))))

(deftest in-memory-test
  (let [node (store/start {:mode :in-memory})]
    (try
      (is (= :ownpurse/no-disk (:type (error #(store/snapshot! node {:mode :in-memory} "/nowhere")))))
      (is (nil? (store/last-snapshot node)))
      (finally (store/stop node)))))
