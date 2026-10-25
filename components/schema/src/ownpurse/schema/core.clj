(ns ownpurse.schema.core
  (:require
   [malli.core :as m]
   [malli.error :as me]
   [malli.transform :as mt]
   [ownpurse.money.interface :as money])
  (:import
   (java.time LocalDate)
   (java.time.format DateTimeParseException)))

(defn- decode-amount
  "Strings and integers become BigDecimals. Anything else, a float included, is left for validation to refuse."
  [x]
  (if (or (string? x) (int? x))
    (try (money/amount x) (catch clojure.lang.ExceptionInfo _ x))
    x))

(defn- decode-date [x]
  (if (string? x)
    (try (LocalDate/parse x) (catch DateTimeParseException _ x))
    x))

(defn- postings-amounts [postings]
  (map (fn [{:posting/keys [amount commodity]}] {:amount amount :commodity commodity}) postings))

(defn- balanced-entry?
  "Only judged when every amount is exact; a float or a missing amount has its own error."
  [{:entry/keys [postings]}]
  (let [amounts (postings-amounts postings)]
    (or (not-every? (comp money/amount? :amount) amounts)
        (money/balanced? amounts))))

(def registry
  (merge
   (m/default-schemas)
   {:ownpurse/id [:string {:min 1}]
    :ownpurse/amount [:fn {:error/message "must be an exact decimal (a string such as \"12.50\" or a BigDecimal), never a float"
                           :decode/json decode-amount
                           :json-schema {:type "string" :pattern "^-?\\d+(\\.\\d+)?$"}}
                      money/amount?]
    :ownpurse/commodity [:re {:error/message "must be a three-letter code such as USD"} #"^[A-Z]{3}$"]
    :ownpurse/date [:fn {:error/message "must be a date, YYYY-MM-DD"
                         :decode/json decode-date
                         :decode/string decode-date
                         :json-schema {:type "string" :format "date"}}
                    #(instance? LocalDate %)]

    :ownpurse/source [:map {:closed true}
                      [:source/id :ownpurse/id]
                      [:source/tool [:string {:min 1}]]
                      [:source/org :ownpurse/id]
                      [:source/record [:string {:min 1}]]
                      [:source/chain-head {:optional true} :string]]

    :ownpurse/book [:map {:closed true}
                    [:book/id :ownpurse/id]
                    [:book/name [:string {:min 1}]]
                    [:book/currency :ownpurse/commodity]
                    [:book/source :ownpurse/id]
                    [:book/fiscal-year-end {:optional true}
                     [:map {:closed true} [:month [:int {:min 1 :max 12}]] [:day [:int {:min 1 :max 31}]]]]]

    :account/class [:enum :asset :liability :equity :income :expense]
    :ownpurse/account [:map {:closed true}
                       [:account/id :ownpurse/id]
                       [:account/book :ownpurse/id]
                       [:account/code {:optional true} :string]
                       [:account/name [:string {:min 1}]]
                       [:account/class :account/class]
                       [:account/status {:optional true} :string]
                       [:account/source-type {:optional true} :string]]

    :ownpurse/party [:map {:closed true}
                     [:party/id :ownpurse/id]
                     [:party/book :ownpurse/id]
                     [:party/name [:string {:min 1}]]
                     [:party/source-id {:optional true} :string]]

    :ownpurse/document [:map {:closed true}
                        [:document/id :ownpurse/id]
                        [:document/book :ownpurse/id]
                        [:document/kind [:string {:min 1}]]
                        [:document/source-id [:string {:min 1}]]
                        [:document/version pos-int?]
                        [:document/content-hash [:string {:min 1}]]
                        [:document/status {:optional true} :string]
                        [:document/date {:optional true} :ownpurse/date]
                        [:document/observed-at inst?]
                        [:document/payload map?]]

    :ownpurse/source-account [:map {:closed true}
                              [:xt/id :ownpurse/id]
                              [:source [:re #"^[a-z][a-z0-9-]*$"]]
                              [:source-table [:re #"^[a-z][a-z0-9_]*\.[a-z][a-z0-9_]*$"]]
                              [:org :ownpurse/id]
                              [:kind [:enum "org" "bank" "credit-card"]]
                              [:name [:string {:min 1}]]
                              [:code {:optional true} :string]
                              [:currency {:optional true} :ownpurse/commodity]
                              [:status {:optional true} :string]]

    :ownpurse/slug [:re {:error/message "must be lowercase letters, digits and hyphens, such as tallowmere"}
                    #"^[a-z0-9][a-z0-9-]*$"]
    :ownpurse/entity [:map {:closed true}
                      [:id :ownpurse/slug]
                      [:name [:string {:min 1}]]
                      [:kind [:enum "business" "household"]]]
    :link/role [:enum "system-of-record" "evidence"]
    :ownpurse/link [:map {:closed true}
                    [:entity :ownpurse/slug]
                    [:account :ownpurse/id]
                    [:role :link/role]
                    [:evidence-for {:optional true} :ownpurse/id]
                    [:from {:optional true} :ownpurse/date]]

    :ownpurse/posting [:map {:closed true}
                       [:posting/line nat-int?]
                       [:posting/account :ownpurse/id]
                       [:posting/amount :ownpurse/amount]
                       [:posting/commodity :ownpurse/commodity]
                       [:posting/party {:optional true} :ownpurse/id]
                       [:posting/description {:optional true} :string]]
    :ownpurse/entry [:and
                     [:map {:closed true}
                      [:entry/id :ownpurse/id]
                      [:entry/book :ownpurse/id]
                      [:entry/date :ownpurse/date]
                      [:entry/kind {:optional true} :string]
                      [:entry/narration {:optional true} :string]
                      [:entry/document {:optional true} :ownpurse/id]
                      [:entry/postings [:vector {:min 2} :ownpurse/posting]]]
                     [:fn {:error/fn (fn [{:keys [value]} _]
                                       (str "the postings do not balance: "
                                            (pr-str (money/imbalance (postings-amounts (:entry/postings value))))))}
                      balanced-entry?]]}))

(def ^:private options {:registry registry})

(def ^:private schema* (memoize (fn [k] (m/schema k options))))

(defn schema [k]
  (schema* k))

(def ^:private validator (memoize (fn [k] (m/validator (schema k)))))
(def ^:private decoder (memoize (fn [k] (m/decoder (schema k) mt/json-transformer))))

(defn valid? [k x]
  ((validator k) x))

(defn explain [k x]
  (some-> (m/explain (schema k) x) me/humanize))

(defn check! [k x]
  (if (valid? k x)
    x
    (let [errors (explain k x)]
      (throw (ex-info (str "This is not a valid " (name k) ": " (pr-str errors))
                      {:type :ownpurse/invalid :schema k :errors errors})))))

(defn coerce! [k x]
  (check! k ((decoder k) x)))
