(ns ownpurse.xero.record
  "Reading a xero-axi record and its gl output. I/O only; no mapping."
  (:require
   [clojure.java.io :as io]
   [next.jdbc :as jdbc]
   [next.jdbc.result-set :as rs]
   [ownpurse.json.interface :as json])
  (:import
   (java.sql DriverManager)
   (org.sqlite SQLiteConfig)))

(def ^:private queries
  {:orgs "SELECT tenant_id, payload FROM current WHERE entity = 'Organisation'"
   :accounts "SELECT tenant_id, xero_id, payload FROM current WHERE entity = 'Accounts'"
   :contacts "SELECT tenant_id, xero_id, payload FROM current WHERE entity = 'Contacts'"
   :documents "SELECT c.tenant_id, c.entity, c.xero_id, c.version, c.content_hash, c.status, c.observed_at,
                      c.payload, d.date
               FROM current c
               JOIN documents d ON d.tenant_id = c.tenant_id AND d.entity = c.entity AND d.xero_id = c.xero_id
               WHERE c.entity NOT IN ('Organisation', 'Accounts', 'Contacts')
               ORDER BY c.tenant_id, c.entity, c.xero_id"
   :chain-head "SELECT hash FROM chain ORDER BY seq DESC LIMIT 1"})

(defn- open ^java.sql.Connection [^java.io.File file]
  (when-not (.isFile file)
    (throw (ex-info (str "No xero-axi record at " file ". Pass the path of the record (xero-axi config shows it).")
                    {:type :ownpurse/no-record :path (str file)})))
  (DriverManager/getConnection (str "jdbc:sqlite:" file)
                               (.toProperties (doto (SQLiteConfig.) (.setReadOnly true)))))

(defn- parse-payload [row]
  (update row :payload json/parse))

(defn read-source [{:keys [record gl]}]
  (let [record-file (.getCanonicalFile (io/file record))
        gl-file (io/file gl)]
    (when-not (.isFile gl-file)
      (throw (ex-info (str "No gl file at " gl-file ". Make it with: xero-axi --offline --json --full gl all "
                           "--limit 1000000 > gl.json")
                      {:type :ownpurse/no-gl :path (str gl-file)})))
    (with-open [conn (open record-file)]
      (let [q (fn [k] (jdbc/execute! conn [(queries k)] {:builder-fn rs/as-unqualified-lower-maps}))]
        {:record (str record-file)
         :chain-head (:hash (first (q :chain-head)))
         :orgs (mapv parse-payload (q :orgs))
         :accounts (mapv parse-payload (q :accounts))
         :contacts (mapv parse-payload (q :contacts))
         :documents (mapv parse-payload (q :documents))
         :gl (:postings (json/parse gl-file))}))))
