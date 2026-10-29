(ns ownpurse.admin.sandbox.home
  "The sandbox's front: pick a set of lines (your books' or made-up samples', picked the same way), then a scenario
  that reads them, then see what a run would cost and run it, or open the runs kept. The picks are in the URL:
  #/sandbox?lines=<set>&scenario=<name>."
  (:require
   [ownpurse.admin.api :as api]
   [ownpurse.admin.route :as route]
   [ownpurse.admin.scenarios.running :as running]
   [ownpurse.admin.ui :as ui]
   [uix.core :refer [$ defui]]))

(defui pick
  "One choice of a step: a card that is a link, marked when it is the one picked."
  [{:keys [picked href title name detail aside]}]
  ($ :a {:href href :aria-current (when picked "true")
         :class (ui/cx "block border p-3" (if picked "border-lapis bg-lapis-tint" "border-line-1 bg-surface-0 hover:bg-surface-1"))}
     ($ :div {:class "flex items-baseline justify-between gap-3"}
        ($ :span {:class "font-display text-lg font-bold"} title)
        ($ ui/muted {:class "text-xs whitespace-nowrap"} detail))
     (when name ($ :div {} ($ ui/mono {:class "text-2xs text-ink-3"} name)))
     (when aside ($ :div {:class "mt-1 text-sm text-ink-2"} aside))))

(defn- set-size [{:keys [lines units]}]
  (str (ui/count-of lines "line") (when (and units (not= lines units)) (str " · " units " with items"))))

(defui step-title [{:keys [n children aside]}]
  ($ ui/section-title {:aside aside} ($ :span {} n " · " children)))

(defui lines-step [{:keys [sets picked]}]
  (let [{books "books" samples "sample"} (group-by :kind sets)
        card (fn [{:keys [id title note] :as s}]
               ($ pick {:key id :picked (= id (:id picked)) :href (route/href ["sandbox"] {:lines id})
                        :title title :detail (set-size s) :aside note}))]
    ($ :<>
       ($ step-title {:n 1 :aside "Which lines to code"} "Lines")
       (when (seq books)
         ($ :<>
            ($ ui/eyebrow {:class "mb-1"} "Your books")
            ($ :div {:class "mb-3 grid gap-3 md:grid-cols-2"} (map card books))))
       ($ ui/eyebrow {:class "mb-1"} "Sample lines (made up)")
       ($ :div {:class "grid gap-3 md:grid-cols-2"} (map card samples)))))

(defui scenario-step [{:keys [lines scenarios picked]}]
  (let [reading (keep scenarios (:scenarios lines))
        others (- (count scenarios) (count reading))]
    ($ :<>
       ($ step-title {:n 2 :aside "How to code them: the world, the buckets and the rules"} "Scenario")
       ($ :div {:class "grid gap-3 md:grid-cols-2"}
          (for [{:keys [name title rules runs]} reading]
            ($ pick {:key name :picked (= name (:name picked)) :href (route/href ["sandbox"] {:lines (:id lines) :scenario name})
                     :title (or title name) :name (when title name)
                     :detail (str (ui/count-of rules "rule") " · " (if (seq runs) "run" "not run yet"))})))
       (when (pos? others)
         ($ ui/say {}
            (ui/count-of others "other scenario") " read other lines. Running any scenario over any lines comes next.")))))

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
