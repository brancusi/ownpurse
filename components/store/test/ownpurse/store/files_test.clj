(ns ownpurse.store.files-test
  (:require
   [clojure.java.io :as io]
   [clojure.test :refer [deftest is testing]]
   [ownpurse.store.interface :as store])
  (:import
   (java.io OutputStream)
   (java.nio.file Files)
   (java.nio.file.attribute FileAttribute)))

(defn- temp-dir []
  (.toFile (Files/createTempDirectory "ownpurse files test" (make-array FileAttribute 0))))

(defn- delete-tree! [^java.io.File f]
  (when (.isDirectory f) (run! delete-tree! (.listFiles f)))
  (io/delete-file f true))

(defn- writing [^bytes b]
  (fn [^OutputStream out] (.write out b)))

(deftest keep-file-test
  (let [tmp (temp-dir)
        record {:mode :disk :dir (str tmp)}
        pdf (byte-array [37 80 68 70 0 -1 10])]
    (try
      (testing "a file is kept under its SHA-256, and its path is relative to the data directory"
        (let [{:keys [sha256 bytes path]} (store/keep-file! record (writing pdf))]
          (is (= 64 (count sha256)))
          (is (= 7 bytes))
          (is (= (str "files/sha256/" (subs sha256 0 2) "/" sha256) path))
          (is (= (seq pdf) (seq (Files/readAllBytes (.toPath (io/file tmp path))))))
          (testing "the same bytes kept twice are one file"
            (is (= path (:path (store/keep-file! record (writing pdf))))))))
      (testing "when writing fails, nothing is kept"
        (is (thrown? clojure.lang.ExceptionInfo
                     (store/keep-file! record (fn [^OutputStream out] (.write out (byte-array 3))
                                                (throw (ex-info "the tool failed" {}))))))
        (is (= 1 (count (filter #(.isFile ^java.io.File %) (file-seq (io/file tmp "files")))))))
      (testing "a record in memory keeps no files"
        (is (= :ownpurse/no-disk
               (:type (try (store/keep-file! {:mode :in-memory} (writing pdf)) nil
                           (catch clojure.lang.ExceptionInfo e (ex-data e)))))))
      (finally (delete-tree! tmp)))))
