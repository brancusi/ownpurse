(ns ownpurse.statement.import
  "Reading the statement files a local config names, and keeping their lines, as source rows of the statement
  schema, with bank lines on each statement account."
  (:require
   [clojure.edn :as edn]
   [clojure.java.io :as io]
   [ownpurse.money.interface :as money]
   [ownpurse.schema.interface :as schema]
   [ownpurse.statement.source :as source]
   [ownpurse.store.interface :as store])
  (:import
   (java.time Instant LocalDate)))

(defn- fail! [message data]
  (throw (ex-info message (assoc data :type :ownpurse/invalid))))

(defn read-config
  "The statements a local config names: {:statements {:formats {name format} :accounts [{:account :name :kind
  :currency}] :files [{:path :account :format :name :opening :closing :from :to}]}}, or nil when it names none."
  [path]
  (let [f (io/file path)]
    (when (.isFile f)
      (:statements (edn/read-string (slurp f))))))

(defn- check-config! [{:keys [formats accounts files]}]
  (let [names (set (map :account accounts))]
    (doseq [{:keys [account kind]} accounts]
      (when-not (and (string? account) (re-matches #"[a-z0-9][a-z0-9-]*" account))
        (fail! (str "A statement account needs a name of lowercase letters, digits and hyphens, such as "
                    "operating-checking; got " (pr-str account) ".")
               {:account account}))
      (when-not (contains? #{nil "bank" "credit-card"} kind)
        (fail! (str "Statement account " account "'s kind is bank or credit-card.") {:account account})))
    (doseq [{:keys [path account format]} files]
      (when-not (names account)
        (fail! (str "A statement file names the account " (pr-str account) ", which :statements :accounts does not "
                    "list.")
               {:account account}))
      (when-not (get formats format)
        (fail! (str "A statement file names the format " (pr-str format) ", which :statements :formats does not "
                    "define.")
               {:format format}))
      (when-not (.isFile (io/file (str path)))
        (fail! (str "There is no statement file at " path ".") {:account account})))))

(defn- day [d] (cond (nil? d) nil (instance? LocalDate d) d :else (LocalDate/parse (str d))))

(defn read-file
  "One file of the config, read and parsed: {:file row :lines rows}. Its row keeps the file's name, never its path."
  [formats {:keys [path account format opening closing from to] :as file}]
  (let [text (slurp (io/file path))
        file-name (or (:name file) (.getName (io/file path)))
        file-key (source/file-id account file-name)
        {:keys [columns lines]} (try (source/parse (get formats format) text)
                                     (catch clojure.lang.ExceptionInfo e
                                       (throw (ex-info (str file-name ": " (ex-message e))
                                                       (assoc (ex-data e) :file file-name)))))
        opening (some-> opening money/amount)
        rows (source/line-rows account lines)]
    {:file (cond-> {:xt/id file-key :account account :name file-name :format format :sha256 (source/sha-256 text)
                    :columns columns :lines (mapv :xt/id rows)
                    :total (reduce + 0M (map :amount rows))
                    :breaks (source/balance-breaks opening lines)}
             opening (assoc :opening opening)
             closing (assoc :closing (money/amount closing))
             from (assoc :from (day from))
             to (assoc :to (day to)))
     :lines rows}))

(defn- held [node table account]
  (if (store/table-exists? node table)
    (store/q node [(str "SELECT * FROM " (source/sql-table table) " WHERE account = ?") account])
    []))

(defn import!
  "Reads every file the statements config names and keeps what changed, in one transaction: the files, their lines,
  each account's source account, and its bank lines (the lines its files hold now). A bank line no file holds any
  more ends, by DELETE; its line stays. Returns {:files :lines :written}, :written counting lines."
  [node {:keys [formats accounts files] :as config} {:keys [now]}]
  (check-config! config)
  (let [now (or now (Instant/now))
        read (mapv (partial read-file formats) files)
        new-files (into {} (map (juxt (comp :xt/id :file) :file)) read)
        new-lines (into {} (map (juxt :xt/id identity)) (mapcat :lines read))
        per-account (for [{:keys [account currency]} accounts]
                      (let [files (vals (merge (into {} (map (juxt :xt/id identity)) (held node (:files source/tables) account))
                                               (into {} (filter #(= account (:account (val %)))) new-files)))
                            current (set (mapcat :lines files))
                            lines (vals (merge (into {} (map (juxt :xt/id identity)) (held node (:lines source/tables) account))
                                               new-lines))]
                        {:account account
                         :bank-lines (mapv #(schema/check! :ownpurse/bank-line %)
                                           (source/bank-lines currency (filter (comp current :xt/id) lines)))}))
        metadata {:source "statement-files" :why "import" :imported-at (str now)}
        {:keys [written]} (store/put-changed!
                           node
                           {(:files source/tables) (vec (vals new-files))
                            (:lines source/tables) (vec (vals new-lines))
                            :source-accounts (mapv #(schema/check! :ownpurse/source-account (source/source-account %))
                                                   accounts)
                            :bank-lines (vec (mapcat :bank-lines per-account))}
                           metadata)]
    (when (store/table-exists? node :bank-lines)
      (doseq [{:keys [account bank-lines]} per-account
              :let [stale (into [] (comp (map :xt/id) (remove (set (map :xt/id bank-lines))))
                                (store/q node ["SELECT _id FROM bank_lines WHERE source_table = ? AND account = ?"
                                               (source/sql-table (:lines source/tables)) (source/account-id account)]))]
              :when (seq stale)]
        (store/delete! node :bank-lines stale {} (assoc metadata :why "import: lines no statement holds"))))
    {:files (count read) :lines (count new-lines) :written (get written (:lines source/tables) 0)}))

(defn statements
  "Every statement file the record holds, by account and name, with whether it ties: its opening and its lines'
  total make its closing, and its running balance follows from its lines."
  [node]
  (if-not (store/table-exists? node (:files source/tables))
    []
    (vec (for [f (store/q node [(str "SELECT * FROM " (source/sql-table (:files source/tables))
                                     " ORDER BY account, name")])
               :let [total (money/amount (:total f))
                     opening (some-> (:opening f) money/amount)
                     closing (some-> (:closing f) money/amount)]]
           (cond-> {:account (:account f) :name (:name f) :format (:format f) :lines (count (:lines f))
                    :total total :breaks (vec (:breaks f))
                    :tied? (and (some? opening) (some? closing) (empty? (:breaks f))
                                (zero? (.compareTo ^BigDecimal (+ opening total) ^BigDecimal closing)))}
             opening (assoc :opening opening)
             closing (assoc :closing closing)
             (:from f) (assoc :from (:from f))
             (:to f) (assoc :to (:to f)))))))
