(ns ownpurse.admin.sandbox.home
  "The sandbox's front: pick a set of lines (your books' or made-up samples', picked the same way), then a scenario
  that reads them, then see what a run would cost and run it, or open the runs kept. The two kinds of card look
  apart: lines as a short statement, a scenario as a stack of layers; the one picked opens a preview under its step.
  Colour stays the outcomes'; the kinds are told apart by shape. The picks are in the URL:
  #/sandbox?lines=<set>&scenario=<name>."
  (:require
   [clojure.string :as str]
   [ownpurse.admin.api :as api]
   [ownpurse.admin.route :as route]
   [ownpurse.admin.sandbox.line :as line]
   [ownpurse.admin.scenarios.running :as running]
   [ownpurse.admin.ui :as ui]
   [uix.core :refer [$ defui]]))

(defn- set-size [{:keys [lines units]}]
  (str (ui/count-of lines "line") (when (and units (not= lines units)) (str " · " units " with items"))))

(defui step-title [{:keys [n children aside]}]
  ($ ui/section-title {:aside aside} ($ :span {} n " · " children)))

(defn- card-edge
  "A card's border: the one picked heavy, in ink; the rest light."
  [picked]
  (if picked "border-2 border-ink-1" "border border-line-2 hover:border-ink-3"))

(defui kind-label
  "What a card is, in its top corner: LINES or SCENARIO, with a mark of its shape, and whether it is the one picked."
  [{:keys [mark children picked]}]
  ($ :div {:class "flex items-center justify-between gap-2"}
     ($ :span {:class "flex items-center gap-1.5 font-mono text-2xs font-medium uppercase tracking-[0.08em] text-ink-3"}
        mark children)
     (when picked ($ :span {:class "font-mono text-2xs font-medium uppercase tracking-[0.08em] text-ink-1"} "✓ picked"))))

(def ^:private rows-mark
  ;; Three ruled rows: a statement.
  ($ :span {:class "inline-flex w-3 flex-col gap-[2px]" :aria-hidden true}
     ($ :span {:class "h-px bg-ink-3"}) ($ :span {:class "h-px bg-ink-3"}) ($ :span {:class "h-px bg-ink-3"})))

(def ^:private layers-mark
  ;; Two sheets, one on the other: layers.
  ($ :span {:class "relative inline-block size-3" :aria-hidden true}
     ($ :span {:class "absolute top-0 left-0 size-2 border border-ink-3"})
     ($ :span {:class "absolute right-0 bottom-0 size-2 border border-ink-3 bg-surface-0"})))

(defui glance-row [{:keys [date merchant amount currency items]}]
  ($ :div {:class "grid grid-cols-[5.5rem_1fr_auto] gap-2 border-b border-dashed border-line-1 py-0.5 font-mono text-2xs"}
     ($ :span {:class "text-ink-3"} date)
     ($ :span {:class "truncate text-ink-2"} merchant (when items ($ :span {:class "text-ink-3"} " · " items " items")))
     ($ :span {:class "text-right tabular-nums text-ink-2"} ($ :span {:class "mr-1 text-ink-3"} currency) (ui/money amount))))

(defui lines-card
  "A set of lines, drawn as a short statement: its first lines, ruled, and how many more."
  [{:keys [s picked]}]
  (let [{:keys [id kind title note lines peek]} s]
    ($ :a {:href (route/href ["sandbox"] {:lines id}) :aria-current (when picked "true")
           :class (ui/cx "block bg-surface-0 p-3" (card-edge picked))}
       ($ kind-label {:mark rows-mark :picked picked} (if (= "books" kind) "Lines · your books" "Lines · made up"))
       ($ :div {:class "mt-1 flex items-baseline justify-between gap-3"}
          ($ :span {:class "font-display text-lg font-bold"} title)
          ($ ui/muted {:class "text-xs whitespace-nowrap"} (set-size s)))
       (when note ($ :div {:class "text-xs text-ink-3"} note))
       ($ :div {:class "mt-2 border-t border-line-1"}
          (for [l peek] ($ glance-row (assoc l :key (:id l))))
          (when (> lines (count peek))
            ($ :div {:class "pt-0.5 font-mono text-2xs text-ink-3"} "+ " (- lines (count peek)) " more"))))))

(defui scenario-card
  "A scenario, drawn as a stack of layers (a sheet behind for each scenario it stands on): its buckets and rules."
  [{:keys [sc lines picked]}]
  (let [{:keys [name title rules runs buckets base]} sc
        shown 6]
    ($ :a {:href (route/href ["sandbox"] {:lines (:id lines) :scenario name}) :aria-current (when picked "true")
           :class (ui/cx "relative block" (when base "mr-1.5 mb-1.5"))}
       (when base
         ($ :span {:aria-hidden true
                   :class "absolute inset-0 translate-x-1.5 translate-y-1.5 border border-line-2 bg-surface-1"}))
       ($ :div {:class (ui/cx "relative bg-surface-0 p-3" (card-edge picked))}
          ($ kind-label {:mark layers-mark :picked picked} (if base (str "Scenario · stands on " base) "Scenario"))
          ($ :div {:class "mt-1 flex items-baseline justify-between gap-3"}
             ($ :span {:class "font-display text-lg font-bold"} (or title name))
             ($ ui/muted {:class "text-xs whitespace-nowrap"}
                (ui/count-of rules "rule") " · " (if (seq runs) "run" "not run yet")))
          (when title ($ :div {} ($ ui/mono {:class "text-2xs text-ink-3"} name)))
          ($ :div {:class "mt-2 flex flex-wrap gap-1"}
             (for [b (distinct (take shown buckets))]
               ($ :span {:key b :class "rounded-1 bg-surface-2 px-1.5 py-0.5 text-xs text-ink-1"} b))
             (when (> (count (distinct buckets)) shown)
               ($ :span {:class "px-1 py-0.5 text-xs text-ink-3"} "+ " (ui/count-of (- (count (distinct buckets)) shown) "bucket"))))))))

;; ----------------------------------------------------------------------------------------------- previews

(defui preview-box
  "The picked card's preview, under its step."
  [{:keys [title children]}]
  ($ :div {:class "mt-3 border-l-2 border-ink-1 bg-surface-1 px-4 py-3"}
     ($ ui/eyebrow {:class "mb-2"} "Preview · " title)
     children))

(defui lines-preview [{:keys [s]}]
  (let [state (api/use-get (str "/sandbox/lines/" (:id s)))
        {:keys [from to currencies lines first]} (get-in state [:data :lines])]
    ($ preview-box {:title (:title s)}
       ($ ui/loading {:state state}
          ($ :<>
             ($ :p {:class "text-sm text-ink-2"}
                (ui/count-of lines "line") " from " ($ :b {:class "text-ink-1"} from) " to " ($ :b {:class "text-ink-1"} to)
                (when (seq currencies)
                  (str ", in " (str/join ", " (for [[c k] currencies] (str k " " c))))) ".")
             ($ :div {:class "mt-2 max-w-3xl"}
                ($ ui/table {:head ["Date" "Merchant" "Amount"]}
                   (for [{:keys [id date merchant amount currency items]} first]
                     ($ :tr {:key id}
                        ($ ui/td {} ($ ui/mono {:class "text-ink-3"} date))
                        ($ ui/td {} merchant (when items ($ ui/muted {:class "text-xs"} " · " items " items on its receipt")))
                        ($ ui/td {:class "text-right"} ($ ui/amount {:amount amount :currency currency}))))))
             (when (> lines (count first))
               ($ ui/say {} "The first " (count first) " of " lines ", oldest first. A run codes them all.")))))))

(defn- strategy-words [k] (some (fn [[s label]] (when (= s k) label)) line/strategies))

(defui scenario-preview [{:keys [sc]}]
  (let [state (api/use-get (str "/sandbox/scenarios/" (:name sc)))
        {:keys [stack buckets rules projects trips threshold strategy] descriptions :codes} (get-in state [:data :world])
        label (into {} (for [{:keys [codes]} buckets [c l] codes] [(name c) l]))]
    ($ preview-box {:title (or (:title sc) (:name sc))}
       ($ ui/loading {:state state}
          ($ :div {:class "grid gap-x-8 gap-y-4 lg:grid-cols-[1fr_1.4fr]"}
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
                              "→ " (get label (name code) (name code)) (when check " · a check"))))))))))))

;; ----------------------------------------------------------------------------------------------- steps

(defui lines-step [{:keys [sets picked]}]
  (let [{books "books" samples "sample"} (group-by :kind sets)
        card (fn [s] ($ lines-card {:key (:id s) :s s :picked (= (:id s) (:id picked))}))]
    ($ :<>
       ($ step-title {:n 1 :aside "Which lines to code"} "Lines")
       (when (seq books)
         ($ :<>
            ($ ui/eyebrow {:class "mb-1"} "Your books")
            ($ :div {:class "mb-3 grid gap-3 md:grid-cols-2 xl:grid-cols-3"} (map card books))))
       ($ ui/eyebrow {:class "mb-1"} "Sample lines (made up)")
       ($ :div {:class "grid gap-3 md:grid-cols-2 xl:grid-cols-3"} (map card samples))
       (when picked ($ lines-preview {:s picked :key (:id picked)})))))

(defui scenario-step [{:keys [lines scenarios picked]}]
  (let [reading (keep scenarios (:scenarios lines))
        others (- (count scenarios) (count reading))]
    ($ :<>
       ($ step-title {:n 2 :aside "How to code them: the world, the buckets and the rules"} "Scenario")
       ($ :div {:class "grid gap-3 md:grid-cols-2 xl:grid-cols-3"}
          (for [sc reading]
            ($ scenario-card {:key (:name sc) :sc sc :lines lines :picked (= (:name sc) (:name picked))})))
       (when (pos? others)
         ($ ui/say {}
            (ui/count-of others "other scenario") " read other lines. Running any scenario over any lines comes next."))
       (when picked ($ scenario-preview {:sc picked :key (:name picked)})))))

(defui books-run [{:keys [name runs]}]
  ($ :<>
     ($ running/run-bar {:ref name})
     ($ :div {:class "mt-3 flex flex-wrap gap-4 text-sm"}
        (when-let [{:keys [id at]} (first runs)]
          ($ :a {:class "text-lapis hover:underline" :href (route/href ["sandbox" name id])}
             "Open its coding, last run " (subs at 0 16) " →"))
        ($ :a {:class "text-lapis hover:underline" :href (route/href ["scenarios" name])} "Change it in Scenarios →"))))

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
        (if (:record scenario)
          ($ books-run scenario)
          ($ sample-runs scenario)))))

(defui page [{:keys [query]}]
  (let [sets-state (api/use-get "/sandbox/lines")
        sc-state (api/use-get "/sandbox/scenarios")
        sets (get-in sets-state [:data :lines])
        scenarios (into {} (map (juxt :name identity)) (get-in sc-state [:data :scenarios]))
        lines (or (some #(when (= (:lines query) (:id %)) %) sets) (first sets))
        scenario (or (when (some #{(:scenario query)} (:scenarios lines)) (scenarios (:scenario query)))
                     (some scenarios (:scenarios lines)))]
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
