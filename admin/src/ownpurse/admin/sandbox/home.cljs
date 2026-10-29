(ns ownpurse.admin.sandbox.home
  "The sandbox's front: pick a set of lines (your books' or made-up samples', picked the same way), then a scenario
  that reads them, then see what a run would cost and run it, or open the runs kept. The two kinds of card look
  apart: lines as a short statement, a scenario as a stack of layers. Each row says what it holds as a query, with a
  peek; More opens it in place. Colour stays the outcomes'; the kinds are told apart by shape. The picks are in the URL:
  #/sandbox?lines=<set>&scenario=<name>."
  (:require
   [clojure.string :as str]
   [ownpurse.admin.api :as api]
   [ownpurse.admin.route :as route]
   [ownpurse.admin.sandbox.line :as line]
   [ownpurse.admin.scenarios.running :as running]
   [ownpurse.admin.ui :as ui]
   [uix.core :as uix :refer [$ defui]]))

(defn- set-size [{:keys [lines units]}]
  (str (ui/count-of lines "line") (when (and units (not= lines units)) (str " · " units " with items"))))

(defui step-title [{:keys [n children aside]}]
  ($ ui/section-title {:aside aside} ($ :span {} n " · " children)))

(defn- card-edge
  "A row's border: the one picked heavy, in ink; the rest light."
  [picked]
  (if picked "border-2 border-ink-1" "border border-line-2"))

(defui kind-label
  "What a row is, in its top line: LINES or SCENARIO, with a mark of its shape."
  [{:keys [mark children]}]
  ($ :span {:class "flex items-center gap-1.5 font-mono text-2xs font-medium uppercase tracking-[0.08em] text-ink-3"}
     mark children))

(def ^:private rows-mark
  ;; Three ruled rows: a statement.
  ($ :span {:class "inline-flex w-3 flex-col gap-[2px]" :aria-hidden true}
     ($ :span {:class "h-px bg-ink-3"}) ($ :span {:class "h-px bg-ink-3"}) ($ :span {:class "h-px bg-ink-3"})))

(def ^:private layers-mark
  ;; Two sheets, one on the other: layers.
  ($ :span {:class "relative inline-block size-3" :aria-hidden true}
     ($ :span {:class "absolute top-0 left-0 size-2 border border-ink-3"})
     ($ :span {:class "absolute right-0 bottom-0 size-2 border border-ink-3 bg-surface-0"})))

(defui pick-head
  "A row's head, a link that picks it: a round mark (filled when picked), its kind, its title and its size."
  [{:keys [href picked label title name detail]}]
  ($ :a {:href href :aria-current (when picked "true") :class "group flex items-start gap-3"}
     ($ :span {:aria-hidden true
               :class (ui/cx "mt-0.5 inline-flex size-4 flex-none items-center justify-center rounded-full border-2"
                             (if picked "border-ink-1" "border-line-2 group-hover:border-ink-3"))}
        (when picked ($ :span {:class "size-2 rounded-full bg-ink-1"})))
     ($ :span {:class "min-w-0 flex-1"}
        ($ :span {:class "flex items-center justify-between gap-3"}
           label
           ($ ui/muted {:class "text-xs whitespace-nowrap"} detail))
        ($ :span {:class "mt-0.5 flex flex-wrap items-baseline gap-x-2"}
           ($ :span {:class "font-display text-lg font-bold group-hover:underline"} title)
           (when name ($ ui/mono {:class "text-2xs text-ink-3"} name))))))

(defui query
  "What a row holds, said as the query that chooses it: field value · field value."
  [{:keys [where]}]
  ($ :div {:class "mt-1 flex flex-wrap gap-x-3 gap-y-0.5 font-mono text-xs"}
     (for [[field value] where]
       ($ :span {:key field}
          ($ :span {:class "text-ink-3"} field " ") ($ :span {:class "text-ink-1"} value)))))

(defui more-button [{:keys [open on-toggle children]}]
  ($ :button {:type "button" :on-click on-toggle :aria-expanded open
              :class "mt-2 font-mono text-2xs uppercase tracking-[0.06em] text-lapis hover:underline"}
     (if open "▴ Less" ($ :<> "▾ " children))))

(defui glance-row [{:keys [date merchant amount currency items]}]
  ($ :div {:class "grid grid-cols-[5.5rem_1fr_auto] gap-2 border-b border-dashed border-line-1 py-0.5 font-mono text-2xs"}
     ($ :span {:class "text-ink-3"} date)
     ($ :span {:class "truncate text-ink-2"} merchant (when items ($ :span {:class "text-ink-3"} " · " items " items")))
     ($ :span {:class "text-right tabular-nums text-ink-2"} ($ :span {:class "mr-1 text-ink-3"} currency) (ui/money amount))))

(defui lines-full
  "Every line of a set (up to two hundred), the days they span and their currencies."
  [{:keys [s]}]
  (let [state (api/use-get (str "/sandbox/lines/" (:id s)))
        {:keys [from to currencies lines first]} (get-in state [:data :lines])]
    ($ ui/loading {:state state}
       ($ :<>
          ($ :p {:class "mt-2 text-sm text-ink-2"}
             (ui/count-of lines "line") " from " ($ :b {:class "text-ink-1"} from) " to " ($ :b {:class "text-ink-1"} to)
             (when (seq currencies)
               (str ", in " (str/join ", " (for [[c k] currencies] (str k " " c))))) ".")
          ($ :div {:class "mt-2 max-h-[28rem] max-w-3xl overflow-y-auto border-t border-line-1"}
             (for [l first] ($ glance-row (assoc l :key (:id l)))))
          (when (> lines (count first))
            ($ ui/say {} "The first " (count first) " of " lines ", oldest first. A run codes them all."))))))

(defui lines-row
  "A set of lines: what chooses them, its first lines as a short statement; More shows them all, in place."
  [{:keys [s picked]}]
  (let [{:keys [id kind title note lines peek where]} s
        [open set-open!] (uix/use-state false)]
    ($ :div {:class (ui/cx "bg-surface-0 p-3" (card-edge picked))}
       ($ pick-head {:href (route/href ["sandbox"] {:lines id}) :picked picked :title title :detail (set-size s)
                     :label ($ kind-label {:mark rows-mark} (cond (:saved s) "Lines · your books, saved" (= "books" kind) "Lines · your books" :else "Lines · made up"))})
       ($ :div {:class "pl-7"}
          ($ query {:where where})
          (when note ($ :div {:class "mt-0.5 text-xs text-ink-3"} note))
          (if open
            ($ lines-full {:s s})
            ($ :div {:class "mt-2 max-w-3xl border-t border-line-1"}
               (for [l peek] ($ glance-row (assoc l :key (:id l))))))
          ($ more-button {:open open :on-toggle #(set-open! not)}
             (if (> lines (count peek)) (str "All " lines " lines") "More"))))))

(defn- strategy-words [k] (some (fn [[s label]] (when (= s k) label)) line/strategies))

(defui scenario-full
  "A scenario all the way: what it stands on, how it decides, its buckets, its world and every rule."
  [{:keys [sc]}]
  (let [state (api/use-get (str "/sandbox/scenarios/" (:name sc)))
        {:keys [stack buckets rules projects trips threshold strategy] descriptions :codes} (get-in state [:data :world])
        label (into {} (for [{:keys [codes]} buckets [c l] codes] [(name c) l]))]
    ($ ui/loading {:state state}
       ($ :div {:class "mt-3 grid gap-x-8 gap-y-4 lg:grid-cols-[1fr_1.4fr]"}
          ($ :div {}
             (when (next stack)
               ($ :<>
                  ($ ui/eyebrow {:class "mb-1"} "Its layers stand on")
                  ($ :ol {:class "mb-3 text-sm"}
                     (for [{:keys [ref name]} (rest stack)]
                       ($ :li {:key ref} name " " ($ ui/mono {:class "text-ink-3"} ref))))))
             ($ ui/eyebrow {:class "mb-1"} "How it decides")
             ($ :p {:class "mb-3 text-sm text-ink-2"}
                (if strategy (strategy-words strategy) "Its runs compare every way of deciding, side by side")
                (when threshold (str "; a coding stands at " (ui/pct threshold) " or more, else a person looks")) ".")
             ($ ui/eyebrow {:class "mb-1"} "Buckets")
             ($ :div {:class "mb-3 space-y-1.5"}
                (for [{:keys [title codes]} buckets]
                  ($ :div {:key (or title (str (ffirst codes)))}
                     (when title ($ :div {:class "text-xs text-ink-3"} title))
                     ($ :div {:class "flex flex-wrap gap-1"}
                        (for [[c l] codes]
                          ($ :span {:key (name c) :title (get descriptions c)
                                    :class "rounded-1 bg-surface-2 px-1.5 py-0.5 text-xs text-ink-1"}
                             l))))))
             (when (seq projects)
               ($ :<>
                  ($ ui/eyebrow {:class "mb-1"} "The world")
                  ($ :p {:class "text-sm text-ink-2"}
                     (ui/count-of projects "project") ", " (ui/count-of trips "trip") ": "
                     (str/join "; " (for [[_ {:keys [name per-diem]}] projects]
                                      (str name (case per-diem true " (pays a per diem)" false " (no per diem)" ""))))))))
          ($ :div {}
             ($ ui/eyebrow {:class "mb-1"} (ui/count-of rules "rule"))
             ($ :ol {:class "divide-y divide-line-1 text-sm"}
                (for [{:keys [id code rule check]} rules]
                  ($ :li {:key id :class "grid grid-cols-[2.25rem_1fr] gap-2 py-1"}
                     ($ ui/mono {:class "text-ink-3"} id)
                     ($ :span {}
                        rule " "
                        ($ :span {:class "whitespace-nowrap text-xs text-ink-3"}
                           "→ " (get label (name code) (name code)) (when check " · a check")))))))))))

(defn- scenario-where
  "What a scenario is, said as a query: what it stands on, and its own layers by kind."
  [{:keys [base layers record]}]
  (cond-> []
    base (conj ["stands on" base])
    (seq layers) (conj ["adds" (str/join " · " (for [{:keys [id kind enabled]} layers]
                                                (str id (when (not= id kind) (str " (" kind ")")) (when-not enabled ", off"))))])
    (not record) (conj ["one file" "its world, buckets, rules and lines"])))

(defn run-over
  "Scenario `sc` run over set `lines`: {:over (the set's id, nil over the scenario's own lines) :run (the sandbox's id
  of that run) :ran (whether it has run so)}."
  [sc lines]
  (let [over (when (and (:record sc) (not= (:id lines) (:default-set sc))) (:id lines))
        run (or over "latest")]
    {:over over :run run :ran (boolean (some #(= run (:id %)) (:runs sc)))}))

(defui scenario-row
  "A scenario, drawn as a stack of layers (a sheet behind when it stands on another): what it adds, its buckets;
  More shows it all, in place."
  [{:keys [sc lines picked]}]
  (let [{:keys [name title rules buckets base]} sc
        {:keys [ran]} (if (:record sc) (run-over sc lines) {:ran (seq (:runs sc))})
        default (some #{name} (:scenarios lines))
        [open set-open!] (uix/use-state false)
        shown 8
        all (distinct buckets)]
    ($ :div {:class (ui/cx "relative" (when base "mr-1.5 mb-1.5"))}
       (when base
         ($ :span {:aria-hidden true :class "absolute inset-0 translate-x-1.5 translate-y-1.5 border border-line-2 bg-surface-1"}))
       ($ :div {:class (ui/cx "relative bg-surface-0 p-3" (card-edge picked))}
          ($ pick-head {:href (route/href ["sandbox"] {:lines (:id lines) :scenario name}) :picked picked
                        :title (or title name) :name (when title name)
                        :detail (str (ui/count-of rules "rule") " · " (if ran "run over these lines" "not run over them yet"))
                        :label ($ kind-label {:mark layers-mark}
                                  (if default "Scenario · reads these lines by default" "Scenario"))})
          ($ :div {:class "pl-7"}
             ($ query {:where (scenario-where sc)})
             (if open
               ($ scenario-full {:sc sc})
               ($ :div {:class "mt-2 flex flex-wrap gap-1"}
                  (for [b (take shown all)]
                    ($ :span {:key b :class "rounded-1 bg-surface-2 px-1.5 py-0.5 text-xs text-ink-1"} b))
                  (when (> (count all) shown)
                    ($ :span {:class "px-1 py-0.5 text-xs text-ink-3"} "+ " (ui/count-of (- (count all) shown) "bucket")))))
             ($ more-button {:open open :on-toggle #(set-open! not)} "Its rules, buckets and world"))))))

;; ----------------------------------------------------------------------------------------------- steps

(defui lines-step [{:keys [sets picked]}]
  (let [{books "books" samples "sample"} (group-by :kind sets)
        row (fn [s] ($ lines-row {:key (:id s) :s s :picked (= (:id s) (:id picked))}))]
    ($ :<>
       ($ step-title {:n 1 :aside "Which lines to code"} "Lines")
       (when (seq books)
         ($ :<>
            ($ ui/eyebrow {:class "mb-1"} "Your books")
            ($ :div {:class "mb-4 space-y-2"} (map row books))))
       ($ ui/eyebrow {:class "mb-1"} "Sample lines (made up)")
       ($ :div {:class "space-y-2"} (map row samples)))))

(defn- offered
  "The scenarios that can run over set `lines`: those that read it by default first, then every other of the
  workspace's. A sample's own rules run over its own lines only."
  [lines scenarios]
  (let [default (keep scenarios (:scenarios lines))]
    (into (vec default)
          (filter #(and (:record %) (not (some #{(:name %)} (:scenarios lines)))))
          (sort-by :name (vals scenarios)))))

(defui scenario-step [{:keys [lines scenarios picked]}]
  (let [reading (offered lines scenarios)]
    ($ :<>
       ($ step-title {:n 2 :aside "How to code them: the world, the buckets and the rules"} "Scenario")
       ($ :div {:class "space-y-2"}
          (for [sc reading]
            ($ scenario-row {:key (:name sc) :sc sc :lines lines :picked (= (:name sc) (:name picked))}))))))

(defui books-run [{:keys [sc lines]}]
  (let [{:keys [name runs]} sc
        {:keys [over run]} (run-over sc lines)]
  ($ :<>
     ($ running/run-bar {:ref name :over over :key (str name "/" over)})
     ($ :div {:class "mt-3 flex flex-wrap gap-4 text-sm"}
        (when-let [{:keys [id at]} (some #(when (= run (:id %)) %) runs)]
          ($ :a {:class "text-lapis hover:underline" :href (route/href ["sandbox" name id])}
             "Open its coding over these lines, last run " (subs at 0 16) " →"))
        ($ :a {:class "text-lapis hover:underline" :href (route/href ["scenarios" name])} "Change it in Scenarios →")))))

(defui sample-runs [{:keys [name runs]}]
  ($ :<>
     ($ ui/eyebrow {:class "mb-1"} "Runs, newest first")
     (if (seq runs)
       ($ :ul {:class "divide-y divide-line-1"}
          (for [{:keys [id at]} runs]
            ($ :li {:key id}
               ($ :a {:href (route/href ["sandbox" name id]) :class "flex justify-between py-1 text-sm hover:bg-surface-1"}
                  ($ ui/mono {} at)
                  ($ ui/muted {:class "text-xs"} "open →")))))
       ($ ui/muted {:class "text-sm"} "No runs kept on this server."))
     ($ ui/say {}
        "Made-up lines, each with the coding a bookkeeper would give it. Their runs are made by the classifier's eval "
        "and kept as files; running them from here comes with running any scenario over any lines.")))

(defui run-step [{:keys [lines scenario]}]
  ($ :<>
     ($ step-title {:n 3 :aside "What a run would ask of Jev, cost and take, before anything is asked"} "Run")
     ($ ui/panel {:class "p-4"}
        ($ :div {:class "mb-2 text-sm text-ink-2"}
           ($ :b {:class "text-ink-1"} (or (:title scenario) (:name scenario))) " over " ($ :b {:class "text-ink-1"} (:title lines))
           " (" (set-size lines) ")")
        (when (and (:record scenario) (= "sample" (:kind lines)))
          ($ ui/notice {}
             "This scenario's world (people, cards, trips, projects) is your books'; these made-up lines happened in "
             "another. Expect most of them to go to a person: a run here tests the rules' wording, not the coding."))
        (if (:record scenario)
          ($ books-run {:sc scenario :lines lines})
          ($ sample-runs scenario)))))

(defui page [{:keys [query]}]
  (let [sets-state (api/use-get "/sandbox/lines")
        sc-state (api/use-get "/sandbox/scenarios")
        sets (get-in sets-state [:data :lines])
        scenarios (into {} (map (juxt :name identity)) (get-in sc-state [:data :scenarios]))
        lines (or (some #(when (= (:lines query) (:id %)) %) sets) (first sets))
        can (when lines (offered lines scenarios))
        scenario (or (some #(when (= (:scenario query) (:name %)) %) can) (first can))]
    ($ :div
       ($ :h1 {:class "font-display text-2xl font-extrabold tracking-tight"} "Sandbox")
       ($ :p {:class "mt-1 max-w-3xl text-ink-3"}
          "Pick lines, from your books or made up, and a scenario (how to code them: the world, the buckets and rules); "
          "run it, and see how each line is coded, why, what it cost, and, where a person keyed it, whether it was right.")
       ($ ui/loading {:state sets-state}
          ($ ui/loading {:state sc-state}
             ($ :<>
                ($ lines-step {:sets sets :picked lines})
                (when lines ($ scenario-step {:lines lines :scenarios scenarios :picked scenario}))
                (when scenario ($ run-step {:lines lines :scenario scenario}))))))))
