(ns ownpurse.admin.scenarios.rules
  "The rules and the world a scenario's layers compose to, its bases' first: the rules that stand, the buckets, the
  threshold and strategy, the projects and trips. A rule is changed, added or turned off by writing it into one of
  the scenario's own coding layers, so the layer under it stays as it was."
  (:require
   [clojure.string :as str]
   [ownpurse.admin.api :as api]
   [ownpurse.admin.sandbox.run :as sandbox.run]
   [ownpurse.admin.scenarios.common :refer [json layer-body parse-json path]]
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

(defui rule-row [{:keys [s composed rule categories]}]
  (let [[editing set-editing!] (uix/use-state false)
        [off-asking set-off-asking!] (uix/use-state false)
        [layer set-layer!] (uix/use-state (default-layer s))
        {:keys [send! busy error]} (api/use-send)
        {:keys [id code over check]} rule]
    ($ :<>
       ($ :tr {:class "hover:bg-surface-1"}
          ($ ui/td {} ($ ui/mono {:class "font-medium"} id))
          ($ ui/td {} ($ ui/mono {} (n code)) (when check ($ :div {:class "text-2xs text-gilt-text"} "a check")))
          ($ ui/td {}
             ($ :div {} (:rule rule))
             ($ :ul {:class "mt-0.5 text-xs text-ink-3"}
                (for [[i b] (map-indexed vector (:when rule))] ($ :li {:key i} "· " (block-text b categories))))
             (when (seq over) ($ :div {:class "text-xs text-ink-3"} "wins over " (str/join ", " over))))
          ($ ui/td {:class "whitespace-nowrap"}
             (when-not (:built-in s)
               ($ :div {:class "flex flex-col items-end gap-1"}
                  ($ ui/button {:on-click #(set-editing! (not editing))} (if editing "Close" "Change"))
                  (if off-asking
                    ($ :div {:class "flex flex-col items-end gap-1"}
                       ($ layer-picker {:s s :value layer :on-change set-layer!})
                       ($ :div {:class "flex gap-1"}
                          ($ ui/button {:tone :danger :disabled busy
                                        :on-click #(write-rule! send! s layer {:id id :code (n code) :enabled false} nil)}
                             "Turn it off")
                          ($ ui/button {:on-click #(set-off-asking! false)} "Keep")))
                    ($ ui/button {:on-click #(set-off-asking! true)} "Turn off"))))))
       (when error ($ :tr {} ($ ui/td {:col-span 4} ($ ui/notice {:tone :bad} error))))
       (when editing
         ($ :tr {} ($ ui/td {:col-span 4}
                      ($ rule-editor {:s s :composed composed :rule rule :on-done #(set-editing! false)})))))))

(defui turned-off
  "The rules this scenario's own coding layers turn off, each to turn on again (its entry taken out of the layer)."
  [{:keys [s]}]
  (let [{:keys [send! busy error]} (api/use-send)
        off (for [l (own-coding-layers s) r (get-in l [:coding :rules]) :when (false? (:enabled r))] [l r])]
    (when (seq off)
      ($ :div
         ($ ui/section-title {} (str (ui/count-of off "rule") " turned off here"))
         (for [[l r] off]
           ($ :div {:key (str (:id l) "/" (:id r)) :class "flex items-center gap-3 py-1 text-sm"}
              ($ ui/mono {:class "font-medium"} (:id r)) ($ ui/muted {} "turned off in layer " (:id l))
              (when-not (:built-in s)
                ($ ui/button {:disabled busy
                              :on-click #(send! "PUT" (path (:ref s) "layers" (:id l))
                                                (update-in (layer-body l) [:coding :rules]
                                                           (partial filterv (fn [x] (not= (:id r) (:id x))))))}
                   "Turn on"))))
         (when error ($ ui/notice {:tone :bad} error))))))

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

(defui tab [{:keys [s]}]
  (let [state (api/use-get (path (:ref s) "composed"))
        composed (get-in state [:data :composed])
        [adding set-adding!] (uix/use-state false)
        {:keys [codes rules trips projects categories entities cards selects]} composed
        categories (update-keys (or categories {}) n)]
    ($ ui/loading {:state state}
       (when composed
         ($ :div
            ($ ui/say {} "What the scenario's layers that are on come to, its bases' first. A change here is written "
               "into one of its own coding layers: the rule of the same id under it is replaced, not changed.")
            ($ ui/section-title {} "Threshold and strategy")
            ($ levers {:s s :composed composed :key (str (:threshold composed) (:strategy composed))})
            ($ ui/section-title {:aside (when-not (:built-in s)
                                          ($ ui/button {:on-click #(set-adding! (not adding))} (if adding "Cancel" "Add a rule")))}
               (str (ui/count-of rules "rule") " that stand"))
            (when adding
              ($ ui/panel {:class "mb-3"} ($ rule-editor {:s s :composed composed :rule nil :on-done #(set-adding! false)})))
            ($ ui/table {:head ["Rule" "Codes to" "Says, and when" ""]}
               (for [r rules] ($ rule-row {:key (:id r) :s s :composed composed :rule r :categories categories})))
            ($ turned-off {:s s})
            ($ ui/section-title {} (ui/count-of codes "bucket"))
            ($ :dl {:class "grid gap-x-4 gap-y-1 text-sm md:grid-cols-[12rem_1fr]"}
               (for [[c about] (sort-by (comp n key) codes)]
                 ($ :<> {:key (n c)} ($ :dt {} ($ ui/mono {} (n c))) ($ :dd {:class "text-ink-2"} about))))
            ($ ui/section-title {} "Projects")
            ($ ui/table {:head ["Project" "Name" "From" "To" "Place" "Borne by" "Travellers"]}
               (for [[id {nm :name :keys [from to place bears travellers]}] (sort-by (comp n key) projects)]
                 ($ :tr {:key (n id)}
                    ($ ui/td {} ($ ui/mono {} (n id))) ($ ui/td {} nm) ($ ui/td {} ($ ui/mono {} from))
                    ($ ui/td {} ($ ui/mono {} to)) ($ ui/td {} place) ($ ui/td {} ($ ui/mono {} bears))
                    ($ ui/td {} (str/join ", " travellers)))))
            ($ ui/section-title {:aside "trips of every world layer, gathered"} "Trips")
            ($ ui/table {:head ["Who" "Where" "From" "To" "For"]}
               (for [[i {:keys [person place city from to project]}] (map-indexed vector (sort-by :from trips))]
                 ($ :tr {:key i}
                    ($ ui/td {} person) ($ ui/td {} (str/join ", " (remove nil? [city place])))
                    ($ ui/td {} ($ ui/mono {} from)) ($ ui/td {} ($ ui/mono {} to)) ($ ui/td {} ($ ui/mono {} project)))))
            ($ ui/section-title {} "Who, and which cards")
            ($ :div {:class "grid gap-1 text-sm"}
               (for [[id {nm :name kind :kind}] entities] ($ :div {:key (n id)} ($ ui/mono {} (n id)) " " nm " " ($ ui/muted {} kind)))
               (for [[id {nm :name :keys [holder]}] cards]
                 ($ :div {:key (n id) :class (cx "text-xs")} nm ", held by " holder " " ($ ui/mono {:class "text-ink-3"} (n id)))))
            ($ ui/section-title {} "Lines it reads")
            (if (seq selects)
              (for [[i {:keys [accounts from to]}] (map-indexed vector selects)]
                ($ :div {:key i :class "text-sm"} (str/join ", " accounts) ", " (or from "…") " → " (or to "…")))
              ($ ui/notice {} "No lines layer: it codes nothing yet.")))))))
