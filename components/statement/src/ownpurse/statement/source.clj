(ns ownpurse.statement.source
  "Statement files as source rows: a CSV export read through its format (which column holds what), each line a row
  of statement.lines, each file a row of statement.files with its opening and closing balances; and from them,
  source accounts and bank lines. Pure."
  (:require
   [clojure.data.csv :as csv]
   [clojure.string :as str])
  (:import
   (java.nio.charset StandardCharsets)
   (java.security MessageDigest)
   (java.time LocalDate)
   (java.time.format DateTimeFormatter DateTimeParseException)))

(def tables
  "The tables ownpurse keeps statement files in."
  {:files :statement/files
   :lines :statement/lines})

(defn sql-table [table]
  (str (namespace table) "." (str/replace (name table) "-" "_")))

(defn account-id
  "The source account of statement account `account` (its name in the local config)."
  [account]
  (str "statement:" account ":account"))

(defn file-id [account file-name]
  (str "statement:" account ":file:" file-name))

(defn- fail! [message data]
  (throw (ex-info message (assoc data :type :ownpurse/invalid))))

(defn sha-256 [^String s]
  (let [digest (.digest (MessageDigest/getInstance "SHA-256") (.getBytes s StandardCharsets/UTF_8))]
    (apply str (map #(format "%02x" %) digest))))

(defn amount
  "A statement's figure as a decimal: currency signs and spaces dropped, (12.00) negative, and with `decimal` \",\"
  the comma as the decimal point (1.234,56). Blank is nil."
  [s decimal]
  (let [s (str/replace (str s) #"[\s$£€]" "")]
    (when-not (str/blank? s)
      (let [negative? (re-matches #"\(.*\)" s)
            s (str/replace s #"[()]" "")
            s (if (= "," decimal) (-> s (str/replace "." "") (str/replace "," ".")) (str/replace s "," ""))
            d (try (bigdec s) (catch NumberFormatException _ nil))]
        (when d
          (let [d (if negative? (.negate ^BigDecimal d) d)]
            (if (< (.scale ^BigDecimal d) 2) (.setScale ^BigDecimal d 2) d)))))))

(defn- column-index [header column what]
  (let [i (.indexOf ^java.util.List header column)]
    (when (neg? i)
      (fail! (str "The file has no column \"" column "\" for the " what ". Its columns are: "
                  (str/join ", " (map #(str "\"" % "\"") header)) ". Fix the format in your local config.")
             {:column column}))
    i))

(defn parse
  "The lines of a CSV `text` read through `fmt`: {:skip (rows before the header) :delimiter (default \",\")
  :decimal (\".\" or \",\") :date :date-format (default yyyy-MM-dd) :amount, or :debit and :credit, :sign
  (\"credit-positive\", the default: money in is positive; or \"debit-positive\") :description :id :balance}.
  Returns {:columns header :lines [{:row :date :amount :description :reference :balance}]}, `row` the line's
  number in the file. A line with no date and no amount is skipped (a total, a blank)."
  [fmt text]
  (let [{:keys [skip delimiter decimal date-format sign] :or {skip 0}} fmt
        rows (vec (csv/read-csv text :separator (first (or delimiter ","))))
        header (mapv str/trim (get rows skip))
        _ (when (empty? header) (fail! "The file has no header row where its format says." {:skip skip}))
        col (fn [k what] (when-let [c (get fmt k)] (column-index header c what)))
        _ (when-not (or (:amount fmt) (and (:debit fmt) (:credit fmt)))
            (fail! "A format names an amount column, or a debit and a credit column." {}))
        [di ai dbi cri desci idi bi] [(col :date "date") (col :amount "amount") (col :debit "debit")
                                      (col :credit "credit") (col :description "description") (col :id "id")
                                      (col :balance "balance")]
        _ (when-not di (fail! "A format names its date column." {}))
        formatter (DateTimeFormatter/ofPattern (or date-format "yyyy-MM-dd"))
        cell (fn [r i] (when i (str/trim (get r i ""))))
        money (fn [r i n what]
                (let [s (cell r i)]
                  (when-not (str/blank? s)
                    (or (amount s decimal)
                        (fail! (str "Line " n ": \"" s "\" is not an amount (" what ").") {:row n})))))]
    {:columns header
     :lines (vec (for [[k r] (map-indexed vector (subvec rows (inc skip)))
                       :let [n (+ skip k 2)
                             date-text (cell r di)
                             raw (if ai
                                   (money r ai n "amount")
                                   (let [d (money r dbi n "debit") c (money r cri n "credit")]
                                     (when (or d c) (- (or c 0M) (or d 0M)))))]
                       :when (not (and (str/blank? date-text) (nil? raw)))]
                   (let [day (try (LocalDate/parse date-text formatter)
                                  (catch DateTimeParseException _
                                    (fail! (str "Line " n ": \"" date-text "\" is not a date as " (or date-format "yyyy-MM-dd")
                                                " writes one.")
                                           {:row n})))]
                     (when-not raw (fail! (str "Line " n " has a date but no amount.") {:row n}))
                     (cond-> {:row n :date day
                              :amount (if (= "debit-positive" sign) (.negate ^BigDecimal raw) raw)}
                       (seq (cell r desci)) (assoc :description (cell r desci))
                       (seq (cell r idi)) (assoc :reference (cell r idi))
                       bi (assoc :balance (money r bi n "balance"))))))}))

(defn line-rows
  "Parsed lines as rows of statement.lines for statement account `account`. A line's id is the file's own
  reference when it has one; else its date, amount and description and which of the identical lines in its file it
  is, so the same line in two overlapping exports is one row."
  [account lines]
  (let [content #(str (:date %) "|" (.toPlainString (.stripTrailingZeros ^BigDecimal (:amount %))) "|" (:description %))
        occurrences (volatile! {})]
    (vec (for [l lines]
           (let [c (content l)
                 k (get (vswap! occurrences update c (fnil inc 0)) c)
                 id (if-let [r (:reference l)]
                      (str "statement:" account ":line:ref:" r)
                      (str "statement:" account ":line:" (subs (sha-256 c) 0 16) ":" k))]
             ;; A line names no file: the same line in two overlapping exports is one row, which each file lists.
             (-> (dissoc l :row)
                 (assoc :xt/id id :account account)))))))

(defn balance-breaks
  "Where a file's running balance does not follow from its lines: [{:row :expected :balance}], the first line
  checked against the opening when there is one."
  [opening lines]
  (loop [[l & more] lines prev opening breaks []]
    (if-not l
      breaks
      (let [b (:balance l)
            expected (when prev (+ prev (:amount l)))]
        (recur more (or b expected)
               (if (and b expected (not= 0 (.compareTo ^BigDecimal b ^BigDecimal expected)))
                 (conj breaks {:row (:row l) :expected expected :balance b})
                 breaks))))))

(defn source-account
  "A statement account as a source_accounts row."
  [{:keys [account name kind currency]}]
  (cond-> {:xt/id (account-id account)
           :source "statement"
           :source-table (sql-table (:files tables))
           :org (str "statement:" account)
           :kind (or kind "bank")
           :name (or name account)
           :status "active"}
    currency (assoc :currency currency)))

(defn bank-lines
  "Rows of statement.lines as bank_lines rows: posted, on their account's source account, money in positive."
  [currency rows]
  (for [r rows]
    (cond-> {:xt/id (str "line:" (:xt/id r))
             :account (account-id (:account r))
             :source "statement"
             :source-table (sql-table (:lines tables))
             :source-row (:xt/id r)
             :org (str "statement:" (:account r))
             :date (:date r)
             :amount (:amount r)
             :status "posted"}
      currency (assoc :currency currency)
      (:description r) (assoc :description (:description r))
      (:reference r) (assoc :reference (:reference r)))))
