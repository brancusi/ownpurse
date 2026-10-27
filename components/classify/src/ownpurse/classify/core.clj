(ns ownpurse.classify.core
  "Classifying a bank line: enrich it, ask every judgment the strategies need, and let each strategy decide. The
  strategies are compared in the eval; the questions are asked once and shared.

  - :choice     Jev picks the coding, given the facts and every rule as text
  - :reversed   the same Choice with its options in reverse order (Jev can lean to the first option)
  - :no-rules   Jev picks the coding from the facts alone: what the rules add
  - :rule-nouls Jev says whether each rule applies; code combines them
  - :blocks     each rule's building blocks decide, Jev answering only the fuzzy ones
  - :hybrid     :blocks, and where no rule or several disagreeing rules apply, the :choice answer"
  (:require
   [clojure.string :as str]
   [ownpurse.classify.blocks :as blocks]
   [ownpurse.classify.facts :as facts]))

(def strategies [:choice :reversed :no-rules :rule-nouls :blocks :hybrid])

(def ^:private yes 0.5M)

(defn- option [code] (str/replace (name code) "-" "_"))

(defn- code [option] (keyword (str/replace option "_" "-")))

(defn- code-question [codes reversed?]
  {:type "choice"
   :instructions (str "How should this bank line be coded? Apply the rules in `rules` to the facts in `transaction` "
                      "and `context`. If no rule decides it, or the facts contradict each other, choose needs_person.")
   :criteria (apply array-map (mapcat (fn [[c d]] [(option c) d]) (cond-> (seq codes) reversed? reverse)))})

(defn- plain-code-question [codes]
  {:type "choice"
   :instructions "How should this bank line be coded, given the facts in `transaction` and `context`?"
   :criteria (apply array-map (mapcat (fn [[c d]] [(option c) d]) codes))})

(defn- rule-question [{:keys [rule]}]
  {:type "noul"
   :instructions (str "Does this rule apply to this bank line, given the facts in `transaction` and `context`? Rule: "
                      rule)})

(defn- fuzzy-question [text]
  {:type "noul"
   :instructions (str text " Judge from the facts in `transaction` and `context`.")})

(defn enrichment
  "The enrichment request a line needs before its rules can run, or nil: a line with no merchant code gets its
  category judged from the merchant's name."
  [{:keys [categories] :as scenario} {:keys [mcc channel] :as line}]
  (when-not (or mcc (= :transfer channel))
    {:state (facts/state scenario (facts/enrich scenario line))
     :questions {:category {:type "choice"
                            :instructions "Which kind of merchant is `transaction.merchant`?"
                            :criteria (assoc (into {} (map (fn [[_ n]] [n nil])) categories)
                                             "none of these" "None of these, or it cannot be told from the name")}}}))

(defn enriched-category
  "The merchant code the enrichment answer chose, or nil when it chose none or was unsure."
  [{:keys [categories]} {:keys [choice confidence]}]
  (when (>= confidence yes)
    (some (fn [[mcc n]] (when (= n choice) mcc)) categories)))

(defn requests
  "The two requests for a line's facts: :rules (state with the rules: the coding Choices and a Noul per rule) and
  :plain (state alone: the coding Choice without rules, and each fuzzy block)."
  [{:keys [codes rules] :as scenario} line-facts]
  (let [state (facts/state scenario line-facts)]
    {:rules {:state (assoc state :rules (mapv #(select-keys % [:id :rule]) rules))
             :questions (into {:code (code-question codes false)
                               :reversed (code-question codes true)}
                              (map (fn [{:keys [id] :as r}] [(keyword (str "rule_" id)) (rule-question r)]))
                              rules)}
     :plain {:state state
             :questions (into {:code (plain-code-question codes)}
                              (map-indexed (fn [i q] [(keyword (str "fuzzy_" i)) (fuzzy-question q)]))
                              (blocks/fuzzy-questions rules))}}))

(defn- chose [{:keys [choice confidence]}]
  {:code (code choice) :confidence confidence})

(defn- certainty [p] (abs (- (* 2M p) 1M)))

(defn combine
  "One coding from each rule's probability of applying (`ps`, by rule id). A check rule that applies stops the line;
  otherwise the rules that apply must agree. The confidence is the least certain rule's."
  [rules ps]
  (let [applying (filter #(>= (get ps (:id %)) yes) rules)
        confidence (reduce min 1M (map certainty (vals ps)))
        by (mapv :id applying)
        codes (distinct (map :code (remove :check applying)))]
    (cond
      (some :check applying) {:code :needs-person :reason :check :by by :confidence confidence}
      (empty? codes) {:code :needs-person :reason :no-rule :by by :confidence confidence}
      (next codes) {:code :needs-person :reason :conflict :by by :confidence confidence}
      :else {:code (first codes) :by by :confidence confidence})))

(defn- fuzzy-answers
  "Each fuzzy block's question in `rules` with Jev's answer to it, from the answers to the :plain request."
  [rules plain-answers]
  (let [questions (blocks/fuzzy-questions rules)]
    (zipmap questions (map-indexed (fn [i _] (get-in plain-answers [(keyword (str "fuzzy_" i)) :noul])) questions))))

(defn decide
  "Each strategy's coding of a line, from its facts and the answers to its two requests."
  [{:keys [rules]} line-facts {:keys [rules-answers plain-answers]}]
  (let [fuzzy (fuzzy-answers rules plain-answers)
        by-blocks (combine rules (into {} (map (fn [r] [(:id r) (blocks/rule-holds r line-facts fuzzy)])) rules))
        choice (chose (:code rules-answers))]
    {:choice choice
     :reversed (chose (:reversed rules-answers))
     :no-rules (chose (:code plain-answers))
     :rule-nouls (combine rules (into {} (map (fn [{:keys [id]}]
                                                [id (get-in rules-answers [(keyword (str "rule_" id)) :noul])]))
                                      rules))
     :blocks by-blocks
     :hybrid (if (#{:no-rule :conflict} (:reason by-blocks))
               (assoc choice :fallback (:reason by-blocks) :by (:by by-blocks))
               by-blocks)}))

(defn classify
  "Classifies `line` in `scenario`, calling `ask` (state, questions → a System One response) for every judgment:
  enrichment first when the line needs it, then its two requests. Returns the facts, the raw responses and each
  strategy's coding."
  [scenario ask line]
  (let [enrich-request (enrichment scenario line)
        enrich-response (when enrich-request (ask (:state enrich-request) (:questions enrich-request)))
        category (some->> enrich-response :answers :category (enriched-category scenario))
        line-facts (facts/enrich scenario line category)
        {:keys [rules plain]} (requests scenario line-facts)
        rules-response (ask (:state rules) (:questions rules))
        plain-response (ask (:state plain) (:questions plain))]
    {:id (:id line)
     :facts line-facts
     :responses (cond-> {:rules rules-response :plain plain-response}
                  enrich-response (assoc :enrich enrich-response))
     :decisions (decide scenario line-facts {:rules-answers (:answers rules-response)
                                             :plain-answers (:answers plain-response)})}))

(defn explain
  "Why each rule did or did not apply to a classified line (a result of `classify`): each of its building blocks
  with the probability it holds, the rule's (the least of them), and Jev's own judgment of the whole rule."
  [{:keys [rules]} {:keys [facts responses]}]
  (let [fuzzy (fuzzy-answers rules (get-in responses [:plain :answers]))]
    (vec (for [{:keys [id] :as rule} rules]
           (assoc (select-keys rule [:id :code :check :rule])
                  :blocks (mapv (fn [block] {:block block :holds (blocks/holds block facts fuzzy)}) (:when rule))
                  :holds (blocks/rule-holds rule facts fuzzy)
                  :jev (get-in responses [:rules :answers (keyword (str "rule_" id)) :noul]))))))
