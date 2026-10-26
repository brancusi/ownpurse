(ns ownpurse.xero.links
  "Links back to a document in Xero's web app, for a person to trace a posting to its source. Pure."
  (:require
   [ownpurse.xero.source :as source])
  (:import
   (java.net URLEncoder)
   (java.nio.charset StandardCharsets)))

(defn- url
  "A path in Xero's web app, through the org login when the org's short code is known, so the link opens in that
  org whichever org the person last used."
  [short-code path]
  (if (seq short-code)
    (str "https://go.xero.com/organisationlogin/default.aspx?shortcode=" short-code "&redirecturl="
         (URLEncoder/encode ^String path StandardCharsets/UTF_8))
    (str "https://go.xero.com" path)))

(defn- invoice-path [{:keys [type invoice-id]}]
  (str (if (= "ACCPAY" type) "/AccountsPayable/View.aspx" "/AccountsReceivable/View.aspx") "?InvoiceID=" invoice-id))

(defn- credit-note-path [{:keys [type credit-note-id]}]
  (str (if (= "ACCPAYCREDIT" type) "/AccountsPayable/ViewCreditNote.aspx" "/AccountsReceivable/ViewCreditNote.aspx")
       "?creditNoteID=" credit-note-id))

(defn links
  "{document row id → url} for org `tenant`'s documents (`docs`, rows by resource: Invoices, CreditNotes, Payments,
  BankTransactions, BankTransfers, ManualJournals). A payment links to what it pays."
  [tenant short-code docs]
  (let [id #(source/row-id tenant %1 %2)
        invoices (into {} (map (juxt :invoice-id identity)) (get docs "Invoices"))
        credit-notes (into {} (map (juxt :credit-note-id identity)) (get docs "CreditNotes"))
        link (partial url short-code)]
    (merge
     (into {} (for [d (get docs "Invoices")] [(id "invoice" (:invoice-id d)) (link (invoice-path d))]))
     (into {} (for [d (get docs "CreditNotes")] [(id "credit-note" (:credit-note-id d)) (link (credit-note-path d))]))
     (into {} (for [d (get docs "BankTransactions")]
                [(id "bank-transaction" (:bank-transaction-id d))
                 (link (str "/Bank/ViewTransaction.aspx?bankTransactionID=" (:bank-transaction-id d)))]))
     (into {} (for [d (get docs "BankTransfers")]
                [(id "bank-transfer" (:bank-transfer-id d))
                 (link (str "/Bank/ViewTransfer.aspx?bankTransferID=" (:bank-transfer-id d)))]))
     (into {} (for [d (get docs "ManualJournals")]
                [(id "manual-journal" (:manual-journal-id d))
                 (link (str "/Journal/View.aspx?invoiceID=" (:manual-journal-id d)))]))
     (into {} (for [d (get docs "Payments")
                    :let [paid (or (some-> (get-in d [:invoice :invoice-id]) invoices invoice-path)
                                   (some-> (get-in d [:credit-note :credit-note-id]) credit-notes credit-note-path))]
                    :when paid]
                [(id "payment" (:payment-id d)) (link paid)])))))
