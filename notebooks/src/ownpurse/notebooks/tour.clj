;; # A tour of ownpurse
;;
;; ownpurse keeps what each source of a group's books says as facts, and one model of books, accounts, entries and
;; postings over all of them. This notebook runs it from nothing over the synthetic samples: a roastery,
;; Tallowmere, whose books are in Xero. Every name, number and id here is invented. Nothing here reads a record of
;; the REPL's: the tour opens its own, in memory, and opens a new one each time it is shown.
(ns ownpurse.notebooks.tour
  {:nextjournal.clerk/no-cache true}
  (:require
   [clojure.java.io :as io]
   [ownpurse.api.books :as books]
   [ownpurse.close.interface :as close]
   [ownpurse.export.interface :as export]
   [ownpurse.notebook :as notebook]
   [ownpurse.store.interface :as store]
   [ownpurse.xero.interface :as xero])
  (:import
   (java.io File)
   (java.time LocalDate)))

;; ## A record
;;
;; The record is an XTDB node. A server keeps it on disk; the tour keeps it in memory.

^{:nextjournal.clerk/visibility {:code :hide :result :hide}}
(defonce ^:private opened (atom nil))

^{:nextjournal.clerk/visibility {:result :hide}}
(def node (do (some-> @opened store/stop) (reset! opened (store/start {:mode :in-memory}))))

;; ## Pulling a source
;;
;; ownpurse never calls Xero. It runs xero-cli, which does, and keeps what it prints. Here a stand-in for xero-cli
;; prints the samples' saved answers instead, as it does in the tests.

^{:nextjournal.clerk/visibility {:result :hide}}
(def root
  "This repository's folder, from where the notebook is on the classpath."
  (-> (io/resource "ownpurse/notebooks/tour.clj") io/file
      .getParentFile .getParentFile .getParentFile .getParentFile .getParentFile))

^{:nextjournal.clerk/visibility {:result :hide}}
(def tenant "c0ffee00-0000-4000-8000-000000000000")

^{:nextjournal.clerk/visibility {:result :hide}}
(def xero-cli
  ["bash" (str (io/file root "components/xero/test/ownpurse/xero/fake-xero-cli.sh"))
   (str (io/file root "samples/xero/pull-1")) "0" (str (File/createTempFile "tour" ".log"))])

(def pulled
  (xero/pull-orgs! node {:cli xero-cli :orgs [{:tenant tenant :name "Tallowmere"}]
                         :today (LocalDate/parse "2026-03-20")}))

;; Pulling again keeps only what changed: Xero's answers are the same, so nothing is written.

(get-in (xero/pull-orgs! node {:cli xero-cli :orgs [{:tenant tenant :name "Tallowmere"}]
                               :today (LocalDate/parse "2026-03-20")})
        ["Tallowmere" "Invoices"])

;; ## The book
;;
;; From the org's documents ownpurse makes the book's entries by Xero's own rules. The trial balance check asks
;; whether they add up to Xero's own trial balance, account by account, to the cent.

^{:nextjournal.clerk/visibility {:result :hide}}
(def date (LocalDate/parse "2026-03-31"))

^{:nextjournal.clerk/visibility {:result :hide}}
(def book (books/xero-book node {:org tenant :date date}))

(select-keys (:trial-balance book) [:date :year-start :tied?])

;; It does not tie, and it should not: the samples hold one month of documents and no opening balances, while
;; Xero's trial balance holds the whole year. Each account's difference says by how much.

(notebook/table (export/report node book "Trial balance"))

;; ## The close
;;
;; A close item proposes journals; nothing reaches Xero until a person approves a sealed package of them, and then
;; only through xero-cli. Until then the reports show the system's figures beside a pro-forma. Here, one proposed
;; journal moves two coffees from meals to cupping: they were tastings.

^{:nextjournal.clerk/visibility {:result :hide}}
(def accounts (into {} (map (fn [[id a]] [(:code a) id])) (:accounts book)))

(close/put-item! node {:ref "C-01" :book (:book book) :title "Tastings coded as meals" :status "proposed"
                       :journals [{:ref "J1" :date date :narration "Two tastings at Harbor Street"
                                   :postings [{:account (accounts "6600") :amount 9.50M :commodity "USD"}
                                              {:account (accounts "6100") :amount -9.50M :commodity "USD"}]}]})

;; The draft counts every journal not set aside; the approved view counts none yet.

^{:nextjournal.clerk/visibility {:result :hide}}
(def draft (books/xero-book node {:org tenant :date date :pro-forma "proposed"}))

(notebook/table (export/report node draft "Profit and loss"))

;; ## Your own books
;;
;; The same reports over a real record come from the book notebook, in the REPL that holds that record:
;;
;; ```clojure
;; (ownpurse.notebook/use-record! (fn [] node))
;; (ownpurse.notebook/show! :profit-and-loss {:org "Tallowmere" :date "2026-03-31"})
;; ```
