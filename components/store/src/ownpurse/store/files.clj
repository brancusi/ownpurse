(ns ownpurse.store.files
  "Files the record keeps beside XTDB, such as a source's attachments: in the data directory under files/sha256, each
  named for the SHA-256 of its bytes. A file is written once, to a scratch name first and then moved into place, and
  never changed or removed; the same bytes kept twice are one file. A row in the record names the file by its path,
  relative to the data directory."
  (:require
   [clojure.java.io :as io])
  (:import
   (java.io OutputStream)
   (java.nio.file CopyOption Files StandardCopyOption)
   (java.security DigestOutputStream MessageDigest)
   (java.util HexFormat UUID)))

(def root
  "The folder in the data directory that holds the files."
  "files")

(defn- hex [^bytes b]
  (.formatHex (HexFormat/of) b))

(defn keep-file!
  "Keeps the bytes `write!` writes (a function of an OutputStream) as a file of the record in the data directory
  `dir`. Returns {:sha256 :bytes :path}, the path relative to `dir`. When `write!` throws, nothing is kept."
  [dir write!]
  (let [scratch (io/file dir root ".tmp" (str (UUID/randomUUID)))
        digest (MessageDigest/getInstance "SHA-256")]
    (io/make-parents scratch)
    (try
      (with-open [out (DigestOutputStream. (io/output-stream scratch) digest)]
        (write! ^OutputStream out))
      (let [sha256 (hex (.digest digest))
            path (str root "/sha256/" (subs sha256 0 2) "/" sha256)
            final (io/file dir path)
            size (.length scratch)]
        (if (.exists final)
          (io/delete-file scratch)
          (do (io/make-parents final)
              (Files/move (.toPath scratch) (.toPath final)
                          (into-array CopyOption [StandardCopyOption/ATOMIC_MOVE]))))
        {:sha256 sha256 :bytes size :path path})
      (finally
        (io/delete-file scratch true)))))
