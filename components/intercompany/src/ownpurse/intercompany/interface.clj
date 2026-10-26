(ns ownpurse.intercompany.interface
  "Intercompany pairs: accounts in one book that mirror accounts in another (a loan and the receivable for it, a
  due-to and its due-from, a grant paid and the grant received), and a reconciliation of each pair from the
  books' postings. Knows nothing about any one source."
  (:require
   [ownpurse.intercompany.core :as core]))

(defn put-pair!
  "Adds or changes a pair: {:ref :name :a {:book :accounts [...] :parties [...] :assigned [{:entry :line}]
  :narration :opening {:amount :note}} :b {...} :from :to :note :explained [{:ref :amount :note}]}. A side may hold
  several accounts (one book spreads what another holds in one account); `parties` keeps only the postings of those
  parties (one account that holds several counterparties'), with the lines that name no party but are `assigned`
  to them (by entry, or entry and line) or whose narration or description matches the `narration` pattern.
  `opening` is a balance the side holds from before its postings (a conversion balance). `explained` pins known
  parts of the difference, such as a prior-period amount. Returns the pair."
  [node pair]
  (core/put-pair! node pair))

(defn pair
  "The pair `ref`, or nil."
  [node ref]
  (core/pair node ref))

(defn pairs
  "Every pair, by ref."
  [node]
  (core/pairs node))

(defn report
  "Reconciles every pair in force over `from` to `to`: {:from :to :days (default 5)}. Per pair, each side's
  :balance (its opening and its postings up to `to`) and :movement (within the period, clipped to the pair's own
  dates), the :difference (the two balances summed: zero when they mirror), the :explained parts and what is
  :unexplained, :reconciled? when nothing is, each side's movement by :months with their difference, and the lines
  of the period that the other side has no mirror for (:unmatched {:a :b}): a line matches one of the opposite
  amount at most `days` apart, else its entry's total does. A side with parties lists its lines that name no party
  and are not assigned to them as :unattributed."
  [node opts]
  (core/report node opts))
