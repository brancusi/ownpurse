(ns ownpurse.admin.scenarios.rules
  "The rules and the world a scenario's layers compose to, its bases' first: the rules that stand, the buckets, the
  threshold and strategy, and the world (ownpurse.admin.scenarios.world). A rule is changed, added or turned off by writing it into one of
  the scenario's own coding layers, so the layer under it stays as it was."
  (:require
   [clojure.string :as str]
   [ownpurse.admin.api :as api]
   [ownpurse.admin.sandbox.run :as sandbox.run]
   [ownpurse.admin.scenarios.common :refer [json layer-body parse-json path]]
   [ownpurse.admin.scenarios.world :as world]
   [ownpurse.admin.ui :as ui :refer [cx]]
   [uix.core :as uix :refer [$ defui]]))

(defn- n [x] (if (keyword? x) (name x) (str x)))

(defn block-text
  "A rule's building block in words."
  [[op & args :as b] categories]
  (let [vals (fn [vs] (let [vs (sort (map n vs))]
                        (str/join ", " (for [v vs] (if-let [c (get categories v)] (str v " " c) v)))))]
    (case (n op)
      "category" (str "category in " (vals (second args)))
      "card" (str "card in " (vals (second args)))
      "channel" (str "paid by " (vals (second args)))
      "trip" (str "on a " (n (first args)) " trip")
      "on-trip-for" (str "on a trip for " (first args))
      "per-diem-trip" "on a trip for a project that pays a per diem"
      "at-home" "at home"
      "jev" (str "Jev says yes: “" (first args) "”")
      "not" (str "not (" (block-text (first args) categories) ")")
      "any" (str "any of (" (str/join " | " (map #(block-text % categories) args)) ")")
      "contains" (str (n (first args)) " contains “" (second args) "”")
      "is" (str (n (first args)) " is " (str/join ", " (map n (rest args))))
      "present" (str (n (first args)) " is said")
      "amount" (str "amount " (n (first args)) " " (str/join " and " (map str (rest args))))
      (js/JSON.stringify (clj->js b)))))

(defn own-coding-layers [s] (filterv #(= "coding" (:kind %)) (:layers s)))

(defn write-rule!
  "Writes `rule` (and a new bucket, when `bucket` is [code description]) into coding layer `layer-id` of scenario
  `s`, replacing the rule of its id there, or a new coding layer of that id."
  [send! s layer-id rule bucket]
  (let [layer (or (some #(when (= layer-id (:id %)) %) (:layers s))
                  {:kind "coding" :enabled true :coding {}
                   :note (str "What this scenario changes in " (or (:base s) "its own layers") ".")})
        rules (vec (get-in layer [:coding :rules]))
        at (first (keep-indexed #(when (= (:id rule) (:id %2)) %1) rules))
        coding (cond-> (assoc (:coding layer) :rules (if at (assoc rules at rule) (conj rules rule)))
                 bucket (assoc-in [:codes (first bucket)] (second bucket)))]
    (send! "PUT" (path (:ref s) "layers" layer-id) (assoc (layer-body layer) :coding coding))))

(defui layer-picker
  "Which of the scenario's own coding layers a change goes into, or a new one."
  [{:keys [s value on-change]}]
  (let [own (own-coding-layers s)]
    ($ ui/field {:label "Write it into"}
       ($ ui/select {:value value :on-change on-change :aria-label "Write it into"
                     :options (concat (for [{:keys [id]} own] [id (str "its coding layer " id)])
                                      (when-not (some #(= "changes" (:id %)) own) [["changes" "a new coding layer, changes"]]))}))))

(defn default-layer [s] (or (:id (peek (own-coding-layers s))) "changes"))

(defui rule-editor
  "A rule as fields: its id, the bucket it codes to (or a new one), its sentence, its blocks as JSON, the rules it
  wins over, whether it is a check."
  [{:keys [s composed rule on-done]}]
  (let [codes (:codes composed)
        [id set-id!] (uix/use-state (or (:id rule) ""))
        [code set-code!] (uix/use-state (n (or (:code rule) (first (keys codes)))))
        [new-code set-new-code!] (uix/use-state "")
        [new-about set-new-about!] (uix/use-state "")
        [sentence set-sentence!] (uix/use-state (or (:rule rule) ""))
        [blocks set-blocks!] (uix/use-state (json (or (:when rule) [["category" "in" ["5411"]]])))
        [over set-over!] (uix/use-state (str/join " " (:over rule)))
        [check set-check!] (uix/use-state (boolean (:check rule)))
        [layer set-layer!] (uix/use-state (default-layer s))
        {:keys [send! busy error]} (api/use-send)
        [when-value parse-error] (parse-json blocks)
        fresh? (= code "__new")
        code* (if fresh? new-code code)
        r (cond-> {:id id :code code* :rule sentence :when when-value}
            (seq (str/trim over)) (assoc :over (vec (remove str/blank? (str/split over #"[\s,]+"))))
            check (assoc :check true))]
    ($ :form {:class "grid gap-3 p-3"
              :on-submit (fn [e]
                           (.preventDefault e)
                           (-> (write-rule! send! s layer r (when fresh? [new-code new-about]))
                               (.then #(when % (on-done)))))}
       ($ :div {:class "grid gap-3 md:grid-cols-[8rem_1fr_1fr]"}
          ($ ui/field {:label "Id"} ($ ui/text-input {:value id :on-change set-id! :placeholder "R9" :name "rule-id" :aria-label "Rule id"}))
          ($ ui/field {:label "Codes to"}
             ($ ui/select {:value code :on-change set-code! :aria-label "Codes to"
                           :options (concat (for [c (sort (map n (keys codes)))] [c c]) [["__new" "a new bucket…"]])}))
          ($ layer-picker {:s s :value layer :on-change set-layer!}))
       (when fresh?
         ($ :div {:class "grid gap-3 md:grid-cols-[1fr_2fr]"}
            ($ ui/field {:label "New bucket"} ($ ui/text-input {:value new-code :on-change set-new-code! :placeholder "studio-rent" :name "new-code"}))
            ($ ui/field {:label "What it holds, for Jev"}
               ($ ui/text-input {:value new-about :on-change set-new-about! :name "new-about"
                                 :placeholder "Rent for the studio, paid on behalf of the business"}))))
       ($ ui/field {:label "The rule, as a person would say it (Jev reads this)"}
          ($ ui/text-input {:value sentence :on-change set-sentence! :name "sentence"}))
       ($ ui/field {:label "When: its building blocks, as JSON (every one must hold)"}
          ($ ui/text-area {:value blocks :on-change set-blocks! :rows 4 :aria-label "Blocks as JSON"}))
       ($ ui/say {}
          "Blocks: [\"category\",\"in\",[\"5411\"]] · [\"trip\",\"project\"] · [\"on-trip-for\",\"venice\"] · [\"per-diem-trip\"] · [\"at-home\"] · "
          "[\"jev\",\"a yes-or-no question\"] · [\"not\",block] · [\"any\",block,block] · [\"channel\",\"in\",[\"transfer\"]] · "
          "[\"contains\",\"merchant\",\"text\"] · [\"amount\",\"over\",100]")
       ($ :div {:class "grid gap-3 md:grid-cols-[1fr_auto]"}
          ($ ui/field {:label "Wins over (rule ids, when both hold)"}
             ($ ui/text-input {:value over :on-change set-over! :placeholder "H1 H2" :name "over"}))
          ($ :div {:class "flex items-end pb-1"}
             ($ ui/switch {:on check :on-change set-check! :label "a check: when it holds, a person looks"})))
       (when parse-error ($ ui/notice {:tone :bad} parse-error))
       (when error ($ ui/notice {:tone :bad} error))
       ($ :div {:class "flex gap-2"}
          ($ ui/button {:tone :primary :type "submit"
                        :disabled (or busy parse-error (str/blank? id) (str/blank? sentence) (str/blank? code*))}
             (if busy "Saving…" (str "Save rule into " layer)))
          ($ ui/button {:on-click on-done} "Cancel")))))

(defui rule-row
  "One rule in its place: a switch (on, or off and greyed), its id, the bucket it codes to, what it says and when it
  holds, and where it comes from. Off or on is written into this scenario's own coding layer."
  [{:keys [s composed entry categories labels]}]
  (let [[editing set-editing!] (uix/use-state false)
        {:keys [send! busy error]} (api/use-send)
        {:keys [id on off-in] rule :rule} entry
        {:keys [code over check]} rule
        layer (default-layer s)
        own-off (and off-in (= (:scenario off-in) (:ref s)))
        toggle! (fn [turn-on]
                  (cond
                    (not turn-on) (write-rule! send! s layer {:id id :code (n code) :enabled false} nil)
                    ;; Turned off here: taking the entry out lets the rule under it stand again.
                    own-off (let [l (some #(when (= (:layer off-in) (:id %)) %) (:layers s))]
                              (send! "PUT" (path (:ref s) "layers" (:id l))
                                     (update-in (layer-body l) [:coding :rules]
                                                (partial filterv (fn [x] (not= id (:id x)))))))
                    ;; Turned off in a scenario under it: this one says the rule again, on.
                    :else (write-rule! send! s layer (dissoc rule :enabled) nil)))
        dim (when-not on "text-ink-3")]
    ($ :<>
       ($ :tr {:class (cx "align-top" (if on "hover:bg-surface-1" "bg-surface-1/60"))}
          ($ ui/td {:class "whitespace-nowrap"}
             ($ ui/switch {:on on :label (if on "on" "off") :disabled (or busy (:built-in s)) :on-change toggle!}))
          ($ ui/td {} ($ ui/mono {:class (cx "font-medium" dim)} id))
          ($ ui/td {}
             ($ :div {:class (cx "text-sm" (if on "text-ink-1" "text-ink-3"))} (get labels (n code) (n code)))
             ($ ui/mono {:class "text-2xs text-ink-3"} (n code))
             (when check ($ :div {:class "text-2xs text-gilt-text"} "a check: a person looks")))
          ($ ui/td {}
             ($ :div {:class (if on "text-ink-1" "text-ink-3 line-through decoration-ink-3/60")} (:rule rule))
             ($ :ul {:class "mt-0.5 text-xs text-ink-3"}
                (for [[i b] (map-indexed vector (:when rule))] ($ :li {:key i} "· " (block-text b categories))))
             (when (seq over) ($ :div {:class "text-xs text-ink-3"} "wins over " (str/join ", " over))))
          ($ ui/td {:class "min-w-[11rem] text-xs text-ink-3"} ($ world/provenance {:s s :entry entry}))
          ($ ui/td {:class "whitespace-nowrap text-right"}
             (when-not (:built-in s)
               ($ ui/button {:on-click #(set-editing! (not editing))} (if editing "Close" "Change")))))
       (when error ($ :tr {} ($ ui/td {:col-span 6} ($ ui/notice {:tone :bad} error))))
       (when editing
         ($ :tr {} ($ ui/td {:col-span 6}
                      ($ rule-editor {:s s :composed composed :rule rule :on-done #(set-editing! false)})))))))

(defui levers
  "The threshold a coding needs to stand without a person, and the strategy whose coding stands."
  [{:keys [s composed]}]
  (let [[t set-t!] (uix/use-state (str (:threshold composed)))
        [strategy set-strategy!] (uix/use-state (n (:strategy composed)))
        [layer set-layer!] (uix/use-state (default-layer s))
        {:keys [send! busy error]} (api/use-send)
        bad-t (let [x (js/Number t)] (or (js/isNaN x) (< x 0) (> x 1)))]
    ($ :form {:class "flex flex-wrap items-end gap-3"
              :on-submit (fn [e]
                           (.preventDefault e)
                           (let [l (or (some #(when (= layer (:id %)) %) (:layers s)) {:kind "coding" :enabled true :coding {}})]
                             (send! "PUT" (path (:ref s) "layers" layer)
                                    (-> (layer-body l) (update :coding assoc :threshold (js/Number t) :strategy strategy)))))}
       ($ ui/field {:label "Threshold"} ($ ui/text-input {:value t :on-change set-t! :class "w-20" :name "threshold"}))
       ($ ui/field {:label "Strategy"}
          ($ ui/select {:value strategy :on-change set-strategy! :aria-label "Strategy"
                        :options (for [[k label] sandbox.run/strategies] [(name k) label])}))
       (when-not (:built-in s)
         ($ :<>
            ($ layer-picker {:s s :value layer :on-change set-layer!})
            ($ ui/button {:type "submit" :disabled (or busy bad-t)} "Save")))
       (when bad-t ($ ui/notice {:tone :bad} "A threshold is how sure a coding must be, from 0 to 1, such as 0.85."))
       (when error ($ ui/notice {:tone :bad} error)))))

(defn- strategy-about [k] (some (fn [[s _ about]] (when (= (n s) (n k)) about)) sandbox.run/strategies))

(defui tab [{:keys [s]}]
  (let [state (api/use-get (path (:ref s) "composed"))
        composed (get-in state [:data :composed])
        ledger (get-in state [:data :rules])
        world-ledger (get-in state [:data :world])
        [adding set-adding!] (uix/use-state false)
        account-names (into {} (map (juxt :id :name)) (get-in (api/use-get "/source-accounts") [:data :source-accounts]))
        {:keys [codes buckets categories selects threshold strategy]} composed
        categories (update-keys (or categories {}) n)
        labels (into {} (for [{:keys [codes]} buckets [c l] codes] [(n c) l]))
        on (count (filter :on ledger))
        off (- (count ledger) on)]
    ($ ui/loading {:state state}
       (when composed
         ($ :div
            ($ ui/section-title {:aside "Which strategy's pick stands, and how sure it must be"} "How it decides")
            ($ :p {:class "mb-2 max-w-4xl text-sm text-ink-2"}
               (strategy-about strategy) " A coding stands on its own when it is " (ui/pct threshold)
               " sure or more; under that, or when a check rule holds, a person looks.")
            ($ levers {:s s :composed composed :key (str threshold strategy)})

            ($ ui/section-title {:aside "What a line can be coded to; Jev reads each one's description"}
               (ui/count-of codes "bucket"))
            ($ :div {:class "grid gap-3 md:grid-cols-2 xl:grid-cols-3"}
               (for [{:keys [title] bcodes :codes}
                     (let [grouped (set (for [{:keys [codes]} buckets [c] codes] (n c)))
                           loose (for [[c _] codes :when (not (grouped (n c)))] [c (n c)])]
                       (cond-> (vec buckets)
                         (seq loose) (conj {:title (if (seq buckets) "In no group (shown by its code)" "Buckets") :codes loose})))]
                 ($ :div {:key (or title "buckets") :class "border border-line-1 p-3"}
                    (when title ($ ui/eyebrow {:class "mb-1"} title))
                    (for [[c l] bcodes]
                      ($ :div {:key (n c) :class "py-1"}
                         ($ :div {:class "text-sm font-medium"} l " " ($ ui/mono {:class "text-2xs font-normal text-ink-3"} (n c)))
                         ($ :div {:class "text-xs text-ink-2"} (get codes c (get codes (keyword (n c))))))))))

            ($ ui/section-title {:aside (when-not (:built-in s)
                                          ($ ui/button {:on-click #(set-adding! (not adding))} (if adding "Cancel" "Add a rule")))}
               (str (ui/count-of ledger "rule") ": " on " on" (when (pos? off) (str ", " off " off"))))
            ($ :p {:class "mb-2 max-w-4xl text-sm text-ink-2"}
               "In the order they stand. A rule holds when every one of its blocks does; where two hold, the one that "
               "wins over the other decides. A rule turned off stays in its place, greyed. Turning one off or on, or "
               "changing it, is written into this scenario's own coding layer, "
               ($ ui/mono {} (default-layer s)) ", so the scenario it stands on is not changed.")
            (when adding
              ($ ui/panel {:class "mb-3"} ($ rule-editor {:s s :composed composed :rule nil :on-done #(set-adding! false)})))
            ($ ui/table {:head ["" "Rule" "Codes to" "Says, and when it holds" "From" ""]}
               (for [e ledger]
                 ($ rule-row {:key (:id e) :s s :composed composed :entry e :categories categories :labels labels})))

            ($ world/section {:s s :ledger world-ledger})

            ($ ui/section-title {:aside "Its own lines; the Sandbox runs it over any others"} "Lines it reads by default")
            (if (seq selects)
              (for [[i {:keys [accounts from to]}] (map-indexed vector selects)]
                ($ :div {:key i :class "text-sm"} (str/join ", " (map #(get account-names % %) accounts)) ", "
                   (or from "the first") " to " (or to "the last")))
              ($ ui/notice {} "No lines layer: it reads no lines of its own. Run it over any set in the Sandbox.")))))))
