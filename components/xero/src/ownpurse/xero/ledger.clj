(ns ownpurse.xero.ledger
  "Entries and postings from a Xero org's documents, by Xero's own rules, so that the org's trial balance can match
  Xero's. Ported from xero-axi's ledger (version 2): debits positive, credits negative.

    BankTransactions  SPEND* and RECEIVE* (AUTHORISED): the bank account against the line accounts (and tax);
                      SPEND-TRANSFER and RECEIVE-TRANSFER are skipped, since the bank transfer carries them
    BankTransfers     the to-account debited, the from-account credited
    ManualJournals    POSTED: each line, a positive LineAmount a debit
    Invoices          AUTHORISED or PAID: ACCREC debits receivables, ACCPAY credits payables, against the lines
    CreditNotes       AUTHORISED or PAID: the reverse of the invoice of the same side
    Payments          AUTHORISED: the bank against receivables or payables, by PaymentType

  A document in another currency is converted to the org's at its own CurrencyRate (Xero's rate is the document's
  currency per unit of the org's), each amount to the cent, the rounding put on its bank or control posting.
  Not posted, as Xero's API does not show them: tracked inventory's cost of sales, realised and unrealised currency
  gains, Xero's fixed asset depreciation, and conversion balances. A trial balance check shows them as differences.
  Pure."
  (:require
   [clojure.string :as str]
   [ownpurse.xero.source :as source])
  (:import
   (java.math RoundingMode)))

(def uncoded
  "The account named for a line whose account the chart does not hold."
  "(uncoded)")

(defn book-id
  "The id of org `tenant`'s book."
  [tenant]
  (str "book:xero:" tenant))

(defn chart
  "The org's chart from its rows of xero.accounts: by Xero id, code and system account, each account's row id."
  [tenant accounts]
  (let [id #(source/row-id tenant "account" %)]
    {:tenant tenant
     :by-id (into {} (map (juxt :account-id identity)) accounts)
     :by-code (into {} (keep #(when (seq (:code %)) [(:code %) (:account-id %)])) accounts)
     :by-system (into {} (keep #(when (seq (:system-account %)) [(:system-account %) (:account-id %)])) accounts)
     :row-id id}))

(defn- placeholder [{:keys [tenant]} key]
  (source/row-id tenant "account" (str uncoded ":" key)))

(defn resolve-account
  "The row id of the account a line names, by its AccountID, else its code; a placeholder when the chart holds
  neither."
  [{:keys [by-id by-code row-id] :as chart} code account-id]
  (cond
    (and (seq account-id) (by-id account-id)) (row-id account-id)
    (and (seq code) (by-code code)) (row-id (by-code code))
    :else (placeholder chart (or (not-empty code) (not-empty account-id) ""))))

(defn system-account
  "The row id of the system account `system` (DEBTORS, CREDITORS, GST, ...), or a placeholder."
  [{:keys [by-system row-id] :as chart} system]
  (if-let [id (by-system system)]
    (row-id id)
    (placeholder chart (str "system:" system))))

(defn- amount [x]
  (if (some? x) (bigdec x) 0M))

(defn- net-and-tax
  "A line's amount before tax, and its tax: an Inclusive line's amount holds its tax."
  [line amount-types]
  (let [line-amount (amount (:line-amount line))
        tax (amount (:tax-amount line))]
    [(if (= "Inclusive" amount-types) (- line-amount tax) line-amount) tax]))

(defn- lines
  "Postings for a document's line items, with tax to the sales tax account, each times `sign` (1 a debit)."
  [chart doc sign]
  (mapcat (fn [line]
            (let [[net tax] (net-and-tax line (:line-amount-types doc))]
              (cond-> [{:account (resolve-account chart (:account-code line) (:account-id line))
                        :amount (* sign net)
                        :description (:description line)}]
                (not (zero? tax)) (conj {:account (system-account chart "GST") :amount (* sign tax)
                                         :description "tax"}))))
          (:line-items doc)))

(defn- convert
  "Postings in the document's currency as postings in the org's: each amount divided by `rate`, to the cent, and the
  rounding on the last posting (the bank or control account), so they still balance."
  [postings rate]
  (if (or (nil? rate) (zero? (bigdec rate)) (= 1M (.stripTrailingZeros (bigdec rate))))
    postings
    (let [rate (bigdec rate)
          converted (mapv #(update % :amount (fn [a] (.divide ^BigDecimal a rate 2 RoundingMode/HALF_UP))) postings)
          rest-sum (reduce + 0M (map :amount (butlast converted)))]
      (assoc-in converted [(dec (count converted)) :amount] (- rest-sum)))))

(defn- cents
  "`a` with at least two places: Xero writes 120 and 4.5."
  [^BigDecimal a]
  (if (< (.scale a) 2) (.setScale a 2) a))

(defn- foreign? [doc base]
  (and (seq (:currency-code doc)) (not= base (:currency-code doc))))

(defn- entry
  "An entry from `postings` (each {:account :amount :description}) for document `doc` of `resource`, or an
  {:unbalanced ...} when they do not sum to zero. Postings of zero are left out."
  [{:keys [tenant base]} resource id-key doc postings {:keys [kind narration]}]
  (let [id (get doc id-key)
        total (reduce + 0M (map :amount postings))
        postings (if (foreign? doc base) (convert postings (:currency-rate doc)) postings)
        kept (remove #(zero? (:amount %)) postings)
        {doc-kind :kind} (source/describe resource)
        party (some->> (get-in doc [:contact :contact-id]) (source/row-id tenant "contact"))]
    (cond
      (not (zero? total)) {:unbalanced {:resource resource :id id :date (:date doc) :imbalance total}}
      (< (count kept) 2) {:skipped (str resource ":empty")}
      :else {:entry (cond-> {:entry/id (str "entry:" (source/row-id tenant doc-kind id))
                             :entry/book (book-id tenant)
                             :entry/date (:date doc)
                             :entry/kind kind
                             :entry/document (source/row-id tenant doc-kind id)
                             :entry/postings (vec (map-indexed
                                                   (fn [i {:keys [account description] a :amount}]
                                                     (cond-> {:posting/line i
                                                              :posting/account account
                                                              :posting/amount (cents a)
                                                              :posting/commodity base}
                                                       party (assoc :posting/party party)
                                                       (seq description) (assoc :posting/description description)))
                                                   kept))}
                      (seq narration) (assoc :entry/narration narration))})))

(defn- title
  "SPEND-TRANSFER as Spend-Transfer."
  [t]
  (str/join "-" (map str/capitalize (str/split (str t) #"-"))))

(defn- bank-transaction [chart doc]
  (let [t (:type doc)]
    (if (or (not= "AUTHORISED" (:status doc)) (str/ends-with? (str t) "-TRANSFER"))
      {:skipped (str "BankTransactions:" t ":" (:status doc))}
      (let [sign (if (#{"SPEND" "SPEND-OVERPAYMENT" "SPEND-PREPAYMENT"} t) 1M -1M)]
        (entry chart "BankTransactions" :bank-transaction-id doc
               (concat (lines chart doc sign)
                       [{:account (resolve-account chart nil (get-in doc [:bank-account :account-id]))
                         :amount (* (- sign) (amount (:total doc))) :description "bank"}])
               {:kind (title t) :narration (or (get-in doc [:contact :name]) (:reference doc))})))))

(defn- bank-transfer [chart doc]
  (let [status (or (:status doc) "AUTHORISED")]
    (if (not= "AUTHORISED" status)
      {:skipped (str "BankTransfers:" status)}
      (let [a (amount (:amount doc))]
        (entry chart "BankTransfers" :bank-transfer-id doc
               [{:account (resolve-account chart nil (get-in doc [:to-bank-account :account-id]))
                 :amount a :description "transfer in"}
                {:account (resolve-account chart nil (get-in doc [:from-bank-account :account-id]))
                 :amount (- a) :description "transfer out"}]
               {:kind "Transfer" :narration (:reference doc)})))))

(defn- manual-journal [chart doc]
  (if (not= "POSTED" (:status doc))
    {:skipped (str "ManualJournals:" (:status doc))}
    (let [amount-types (or (not-empty (:line-amount-types doc)) "NoTax")]
      (entry chart "ManualJournals" :manual-journal-id doc
             (mapcat (fn [line]
                       (let [[net tax] (net-and-tax line amount-types)]
                         (cond-> [{:account (resolve-account chart (:account-code line) (:account-id line))
                                   :amount net :description (:description line)}]
                           (not (zero? tax)) (conj {:account (system-account chart "GST") :amount tax
                                                    :description "tax"}))))
                     (:journal-lines doc))
             {:kind "Journal" :narration (:narration doc)}))))

(defn- invoice [chart resource doc]
  (let [credit? (= "CreditNotes" resource)
        t (:type doc)]
    (if-not (#{"AUTHORISED" "PAID"} (:status doc))
      {:skipped (str resource ":" (:status doc))}
      (let [sales? (= t (if credit? "ACCRECCREDIT" "ACCREC"))
            sign (if (= credit? sales?) 1M -1M)]
        (entry chart resource (if credit? :credit-note-id :invoice-id) doc
               (concat (lines chart doc sign)
                       [{:account (system-account chart (if sales? "DEBTORS" "CREDITORS"))
                         :amount (* (- sign) (amount (:total doc))) :description "control"}])
               {:kind (cond credit? "Credit note" (= "ACCREC" t) "Invoice" (= "ACCPAY" t) "Bill" :else "Invoice")
                :narration (or (not-empty (if credit? (:credit-note-number doc) (:invoice-number doc)))
                               (:reference doc))})))))

(defn- payment [chart doc]
  (if (not= "AUTHORISED" (:status doc))
    {:skipped (str "Payments:" (:status doc))}
    (let [ptype (str (:payment-type doc))
          receivable? (or (str/starts-with? ptype "AR") (= "ACCRECPAYMENT" ptype))
          money-in? (#{"ACCRECPAYMENT" "APCREDITPAYMENT" "APOVERPAYMENTPAYMENT" "APPREPAYMENTPAYMENT"} ptype)
          s (if money-in? 1M -1M)
          a (amount (:amount doc))]
      (entry chart "Payments" :payment-id doc
             [{:account (system-account chart (if receivable? "DEBTORS" "CREDITORS")) :amount (* (- s) a)
               :description ptype}
              {:account (resolve-account chart (get-in doc [:account :code]) (get-in doc [:account :account-id]))
               :amount (* s a) :description "payment"}]
             {:kind "Payment" :narration (:reference doc)}))))

(defn entries
  "The entries of org `tenant`'s documents, its current rows by table ({:accounts :bank-transactions
  :bank-transfers :manual-journals :invoices :credit-notes :payments}, each a seq of rows as xero.<table> holds them),
  in the org's currency `base`. Returns {:entries [...] :unbalanced [{:resource :id :date :imbalance}] :skipped
  {reason count}}: a document whose postings do not balance is not posted but listed; a draft, a voided or a deleted
  one, and a transfer's legs, are counted as skipped."
  [tenant base {:keys [accounts bank-transactions bank-transfers manual-journals invoices credit-notes payments]}]
  (let [chart (assoc (chart tenant accounts) :base base)
        results (concat (map (partial bank-transaction chart) bank-transactions)
                        (map (partial bank-transfer chart) bank-transfers)
                        (map (partial manual-journal chart) manual-journals)
                        (map (partial invoice chart "Invoices") invoices)
                        (map (partial invoice chart "CreditNotes") credit-notes)
                        (map (partial payment chart) payments))]
    {:entries (vec (sort-by (juxt :entry/date :entry/id) (keep :entry results)))
     :unbalanced (vec (keep :unbalanced results))
     :skipped (frequencies (keep :skipped results))}))
