(ns ownpurse.admin.sandbox.run
  "One run of a scenario: the world its lines happened in, the strategy and threshold to code them by, what that
  gives across every threshold, the buckets, and every line, each opening to its way through. The threshold and the
  strategy are the sandbox's levers: moving them scores every line again here, with the eval's own function
  (ownpurse.classify.outcome), without asking Jev anything."
  (:require
   [clojure.string :as str]
   [ownpurse.admin.api :as api]
   [ownpurse.admin.route :as route]
   [ownpurse.admin.sandbox.line :as line]
   [ownpurse.admin.scenarios.running :as running]
   [ownpurse.admin.terms :refer [term]]
   [ownpurse.admin.ui :as ui :refer [cx]]
   [ownpurse.classify.outcome :as outcome]
   [uix.core :as uix :refer [$ defui]]))

(def strategies
  "The ways of coding a line the run compared, in words (kept with the line's walk, which tells them)."
  line/strategies)

(defn keyed?
  "Whether any of a run's lines carries an answer key (every invented one does; a book's own, once a person keys it)."
  [lines]
  (boolean (some #(get-in % [:unit :expect]) lines)))

(defn- unkeyed? [lines] (boolean (some #(nil? (get-in % [:unit :expect])) lines)))

(defn- shown-outcomes
  "The outcomes a run's lines can have: right or wrong and the rest for keyed lines, coded or to a person for the others."
  [lines]
  (concat (when (keyed? lines) outcome/outcomes) (when (unkeyed? lines) outcome/unkeyed-outcomes)))

(defn- filters [scored]
  (vec (concat [[:all "All"]]
               (when (keyed? scored) [[:wrong nil] [:unsafe nil] [:review nil] [:stopped nil] [:right nil]])
               (when (unkeyed? scored) [[:asks nil] [:coded nil]])
               (when (some #(get-in % [:unit :hard]) scored) [[:hard "marked hard"]])
               (when (some :noted scored) [[:noted "with commentary"]]))))

(defn- score
  "Each line with its outcome under `strategy` at threshold `t`."
  [lines strategy t]
  (mapv #(assoc % :outcome (outcome/outcome (:unit %) (get-in % [:decisions strategy]) t)) lines))

(defn- sweep
  "At every threshold from 50% to 99%: the lines coded wrong or unsafely, and the lines sent to a person."
  [lines strategy]
  (vec (for [i (range 50 100)
             :let [fs (frequencies (map #(outcome/outcome (:unit %) (get-in % [:decisions strategy]) (/ i 100)) lines))]]
         {:t (/ i 100)
          :bad (+ (fs :wrong 0) (fs :unsafe 0))
          :person (+ (fs :stopped 0) (fs :review 0) (fs :asks 0))})))

;; ----------------------------------------------------------------------------------------------- the world

(defui world-strip [{:keys [world]}]
  (let [{:keys [projects trips entities cards home]} world
        person #(get-in entities [% :name] (some-> % name))]
    ($ :div {:class "mt-4 grid gap-px border border-line-1 bg-line-1 md:grid-cols-[repeat(auto-fit,minmax(260px,1fr))]"}
       (for [[id {:keys [name place from to work travellers]}] projects]
         ($ :div {:key (str id) :class "bg-surface-0 px-3 py-2.5 text-sm"}
            ($ ui/eyebrow {} name)
            ($ :div {} place " · " ($ ui/mono {} from " → " to))
            (when work ($ :div {:class "mt-0.5 text-ink-3"} work))
            ($ :div {:class "mt-1"}
               (str/join " · " (for [{:keys [person city place from to project]} trips :when (= id project)]
                                 (str (get-in entities [person :name]) " in " (or city place) " "
                                      (subs from 5) "–" (subs to 5)))))
            ($ :div {:class "text-ink-3"} "Travellers: " (str/join ", " (map person travellers)))))
       ($ :div {:class "bg-surface-0 px-3 py-2.5 text-sm"}
          ($ ui/eyebrow {} "Home and cards")
          ($ :div {} "At home in " (if (map? home) (str (:city home) ", " (:country home)) home) " on any other day.")
          (for [[id {:keys [name holder]}] cards]
            ($ :div {:key id} ($ ui/mono {} id) " "
               ($ ui/muted {} name (when-not (str/includes? name "held by") (str ", held by " (person holder))))))))))

;; ----------------------------------------------------------------------------------------------- the levers

(defui sweep-chart
  "Lines coded wrong or unsafely, and lines sent to a person, at every threshold: one count axis, the threshold
  marked. Hover reads a threshold's counts; a click moves the threshold there."
  [{:keys [points t on-pick]}]
  (let [[hover set-hover!] (uix/use-state nil)
        w 560 h 150 l 34 r 12 top 10 b 24
        ymax (max 1 (apply max (mapcat (juxt :bad :person) points)))
        x #(+ l (* (- w l r) (/ (- % 0.5) 0.49)))
        y #(- h b (* (- h top b) (/ % ymax)))
        path (fn [k] (str/join " " (map-indexed (fn [i p] (str (if (zero? i) "M" "L") (x (:t p)) "," (y (k p)))) points)))
        at (fn [e] (let [rect (.getBoundingClientRect (.-currentTarget e))
                         px (* w (/ (- (.-clientX e) (.-left rect)) (.-width rect)))
                         i (js/Math.round (* 49 (/ (- px l) (- w l r))))]
                     (nth points (max 0 (min 49 i)))))
        shown (or hover (some #(when (= (:t %) t) %) points))
        ticks (let [step (cond (> ymax 200) 100 (> ymax 80) 50 (> ymax 40) 20 :else 10)] (range 0 (inc ymax) step))]
    ($ :div
       ($ :div {:class "flex flex-wrap items-center gap-4 text-xs text-ink-2"}
          ($ :span {:class "inline-flex items-center gap-1.5"} ($ :span {:class "inline-block h-0.5 w-4 bg-coin"}) "coded wrong or unsafely")
          ($ :span {:class "inline-flex items-center gap-1.5"} ($ :span {:class "inline-block h-0.5 w-4 bg-ink-2"}) "sent to a person")
          (when shown
            ($ :span {:class "ml-auto font-mono text-ink-1"}
               "at " (ui/pct (:t shown)) ": " (:bad shown) " bad · " (:person shown) " to a person")))
       ($ :svg {:view-box (str "0 0 " w " " h) :class "mt-1 w-full cursor-crosshair select-none" :role "img"
                :aria-label "Lines coded wrong or unsafely, and lines sent to a person, by threshold"
                :on-mouse-move #(set-hover! (at %)) :on-mouse-leave #(set-hover! nil)
                :on-click #(on-pick (:t (at %)))}
          (for [v ticks]
            ($ :g {:key v}
               ($ :line {:x1 l :x2 (- w r) :y1 (y v) :y2 (y v) :stroke "var(--line-1)" :stroke-width 1})
               ($ :text {:x (- l 6) :y (+ (y v) 3) :text-anchor "end" :font-size 9 :fill "var(--ink-3)" :font-family "var(--font-mono)"} v)))
          (for [v [0.5 0.6 0.7 0.8 0.9]]
            ($ :text {:key v :x (x v) :y (- h 8) :text-anchor "middle" :font-size 9 :fill "var(--ink-3)" :font-family "var(--font-mono)"} (ui/pct v)))
          ($ :line {:x1 (x t) :x2 (x t) :y1 top :y2 (- h b) :stroke "var(--ink-1)" :stroke-width 1.5})
          (when hover
            ($ :line {:x1 (x (:t hover)) :x2 (x (:t hover)) :y1 top :y2 (- h b) :stroke "var(--line-2)" :stroke-dasharray "2 2"}))
          ($ :path {:d (path :person) :fill "none" :stroke "var(--ink-2)" :stroke-width 2 :stroke-linejoin "round"})
          ($ :path {:d (path :bad) :fill "none" :stroke "var(--coin)" :stroke-width 2 :stroke-linejoin "round"})
          (when shown
            ($ :g {}
               ($ :circle {:cx (x (:t shown)) :cy (y (:person shown)) :r 4 :fill "var(--ink-2)" :stroke "var(--surface-0)" :stroke-width 2})
               ($ :circle {:cx (x (:t shown)) :cy (y (:bad shown)) :r 4 :fill "var(--coin)" :stroke "var(--surface-0)" :stroke-width 2})))))))

(defui levers [{:keys [scored lines strategy t on-strategy on-threshold]}]
  (let [counts (frequencies (map :outcome scored))
        points (uix/use-memo #(sweep lines strategy) [lines strategy])]
    ($ ui/panel {:class "mt-4 grid gap-6 p-4 lg:grid-cols-[minmax(0,1fr)_minmax(0,1.2fr)]"}
       ($ :div
          ($ ui/eyebrow {} ($ term {:id :strategy} "Code lines by"))
          ($ :select {:value (name strategy) :on-change #(on-strategy (keyword (.. % -target -value)))
                      :class "mt-1 w-full rounded-1 border border-line-2 bg-surface-0 px-2 py-1.5 text-sm"}
             (for [[k title] strategies] ($ :option {:key (name k) :value (name k)} title)))
          ($ ui/say {} (some (fn [[k _ about]] (when (= k strategy) about)) strategies))
          ($ ui/eyebrow {:class "mt-4"} ($ term {:id :threshold} "Coded automatically at or above"))
          ($ :div {:class "mt-1 flex items-center gap-3"}
             ($ :input {:type "range" :min 50 :max 99 :value (js/Math.round (* 100 t))
                        :on-change #(on-threshold (/ (js/Number (.. % -target -value)) 100))
                        :class "w-full accent-[var(--ink-1)]"})
             ($ :span {:class "w-12 text-right font-display text-xl font-bold"} (ui/pct t)))
          ($ :div {:class "mt-4 flex flex-wrap items-baseline gap-x-5 gap-y-2"}
             (for [o (shown-outcomes lines)]
               ($ :span {:key (name o) :class "inline-flex items-baseline gap-1.5 text-sm"}
                  ($ :b {:class "font-display text-2xl leading-none"} (counts o 0))
                  ($ ui/dot {:outcome o}) ($ :span {:class "text-ink-2"} (ui/outcome-words o))))))
       ($ :div
          ($ ui/eyebrow {} "What each threshold gives")
          ($ sweep-chart {:points points :t t :on-pick on-threshold})
          ($ ui/say {} (cond
                         (not (keyed? lines))
                         (str "No line is keyed yet, so nothing here is right or wrong: the chart shows how many go to "
                              "a person at each threshold. Key lines (open one) to score them. Click the chart to move it.")
                         (unkeyed? lines)
                         (str "Coded wrongly counts the keyed lines only (" (count (filter #(get-in % [:unit :expect]) lines))
                              " of " (count lines) "); every line counts towards a person. Click the chart to move it.")
                         :else
                         "Raising the threshold sends more lines to a person and codes fewer wrongly. Click the chart to move it."))))))

(defui benchmark
  "How each strategy fares on the keyed lines at threshold `t`: the run's lines scored again, nothing asked. A row
  picks its strategy."
  [{:keys [lines strategy t on-strategy]}]
  (let [keyed (filterv #(get-in % [:unit :expect]) lines)]
    (when (seq keyed)
      ($ ui/panel {:class "mt-4 p-4"}
         ($ :div {:class "flex flex-wrap items-baseline justify-between gap-3"}
            ($ ui/eyebrow {} "Benchmark: " (ui/count-of keyed "keyed line") ", at " (ui/pct t))
            ($ ui/muted {:class "text-xs"} "Each strategy scored against the key; click one to code by it"))
         ($ ui/table {:class "mt-2" :head (into ["Strategy"] (map ui/outcome-words outcome/outcomes))}
            (for [[k title] strategies
                  :let [fs (frequencies (map #(outcome/outcome (:unit %) (get-in % [:decisions k]) t) keyed))]]
              ($ :tr {:key (name k) :on-click #(on-strategy k)
                      :class (cx "cursor-pointer hover:bg-surface-1" (when (= k strategy) "bg-surface-1 font-semibold"))}
                 ($ ui/td {} title)
                 (for [o outcome/outcomes]
                   ($ ui/td {:key (name o) :class (cx "font-mono" (when (and (ui/bad? o) (pos? (fs o 0))) "text-coin-text"))}
                      (fs o 0))))))))))

;; ----------------------------------------------------------------------------------------------- the buckets

(defui board [{:keys [world scored line-href]}]
  (let [expected (frequencies (map #(get-in % [:unit :expect]) scored))
        keyed (keyed? scored)
        placed (group-by :code scored)]
    ($ :div {:class "grid gap-4 md:grid-cols-[repeat(auto-fit,minmax(300px,1fr))]"}
       (for [{:keys [project title codes]} (:buckets world)]
         ($ :div {:key (or (some-> project name) title)}
            ($ :div {:class "mb-1 text-sm font-bold"} (or (get-in world [:projects project :name]) title))
            (for [[code label] codes]
              ($ :div {:key (name code) :class "grid grid-cols-[110px_1fr] gap-2 border-t border-line-1 py-1.5"}
                 ($ :div
                    ($ :div {:class "text-sm" :title (get-in world [:codes code])} label)
                    ($ ui/mono {:class "text-2xs text-ink-3"}
                       (when keyed (str (get expected code 0) " belong · ")) (count (placed code)) " put"))
                 ($ :div {:class "flex flex-wrap content-start gap-0.5"}
                    (for [{:keys [id short outcome]} (sort-by #(or (some-> (:short %) (subs 1) js/parseInt) (:id %)) (placed code))]
                      ($ ui/chip {:key id :outcome outcome :href (line-href id)} (or short id)))))))))))

;; ----------------------------------------------------------------------------------------------- the lines

(def ^:private cols
  "The row's columns, shared by the header and every row."
  "grid items-center gap-2.5 grid-cols-[12px_52px_78px_minmax(160px,1fr)_104px_minmax(140px,200px)_minmax(180px,260px)_84px_14px]")

(defui line-row [{:keys [row strategy t open on-toggle children]}]
  (let [{:keys [id short unit decisions outcome noted]} row
        {:keys [code confidence]} (get decisions strategy)
        acceptable (conj (set (:ok unit)) (:expect unit))
        blk (get-in decisions [:blocks :code])]
    ($ :div {:id id :class "border-b border-line-1"}
       ($ :button {:on-click on-toggle :aria-expanded open
                   :class (cx cols "min-h-[30px] w-full px-2 py-1.5 text-left hover:bg-surface-1"
                              (when open "bg-surface-1")
                              (when (ui/bad? outcome) "shadow-[inset_3px_0_0_var(--coin)]"))}
          ($ ui/dot {:outcome outcome})
          ($ :span {:class "font-mono text-xs text-ink-2" :title id} (or short id)
             (when noted ($ :span {:class "ml-1.5 font-mono text-2xs text-gilt-text" :title "Commentary on why it failed"} "✎")))
          ($ ui/mono {:class "text-ink-3"} (:date unit))
          ($ :span {:class "min-w-0 truncate"} (:merchant unit)
             (when (:item unit) ($ :span {:class "text-ink-2"} " · " (:item unit)))
             (when-let [n (:note unit)] ($ :span {:class "text-xs text-ink-3"} "  " n)))
          ($ ui/amount {:amount (:amount unit) :currency (:currency unit)})
          ($ :span {:class "min-w-0"} (if (:expect unit) ($ ui/tag {:code (:expect unit)}) ($ ui/muted {:class "text-xs"} "no key")))
          ($ :span {:class "flex min-w-0 items-center gap-1.5"}
             ($ ui/tag {:code code :miss (and (:expect unit) (not (acceptable code)))})
             ($ :span {:class "inline-flex flex-none items-center gap-1 font-mono text-2xs text-ink-2"}
                ($ ui/bar {:p confidence :threshold t}) (ui/pct confidence)))
          ($ :span {:class (cx "truncate font-mono text-2xs" (if (= blk code) "text-ink-3" "text-gilt-text"))
                    :title (str "The rule blocks said " (name blk))}
             (if (= blk code) "same" (str/replace (name blk) #"^[a-z]+-(?=[a-z])|^needs-" "")))
          ($ :span {:class (cx "text-2xs text-ink-3 transition-transform" (when open "rotate-90"))} "▶"))
       (when open children))))

(defui lines-table [{:keys [scored strategy t open show on-filter on-open line-view]}]
  (let [counts (frequencies (map :outcome scored))
        filters (filters scored)
        shown (case show
                :all scored
                :hard (filterv #(get-in % [:unit :hard]) scored)
                :noted (filterv :noted scored)
                (filterv #(= show (:outcome %)) scored))]
    ($ :div
       ;; The filters and the column headers stay under the page's bar as the lines scroll.
       ($ :div {:class "sticky top-[45px] z-10 bg-surface-0"}
          ($ :div {:class "flex flex-wrap gap-1 py-1.5"}
             (for [[k words] filters
                   :let [n (case k :all (count scored) :hard (count (filter #(get-in % [:unit :hard]) scored))
                                 :noted (count (filter :noted scored)) (counts k 0))
                         on? (= k show)]]
               ($ :button {:key (name k) :on-click #(on-filter k)
                           :class (cx "inline-flex items-center gap-1.5 rounded-1 border px-2.5 py-1 text-sm"
                                      (if on? "border-ink-1 bg-ink-1 text-ink-inverse" "border-line-2 hover:bg-surface-1"))}
                  (when-not words ($ ui/dot {:outcome k}))
                  (or words (ui/outcome-words k))
                  ($ :b {:class (cx "font-mono font-medium" (if on? "text-ink-inverse" "text-ink-3"))} n))))
          ($ :div {:class (cx cols "border-t-2 border-b border-t-ink-1 border-b-line-1 bg-surface-1 px-2 py-1.5 font-mono text-2xs uppercase tracking-[0.08em] text-ink-3")}
             ($ :span) ($ :span "line") ($ :span "date") ($ :span "merchant · item · note") ($ :span {:class "text-right"} "amount")
             ($ :span ($ term {:id :answer-key} "belongs in")) ($ :span ($ term {:id :strategy} "chosen"))
             ($ :span ($ term {:id :block} "blocks")) ($ :span)))
       ($ :div
          (for [row (sort-by #(case (:outcome %) :wrong 0 :unsafe 1 :asks 1 :review 2 :stopped 3 4) shown)]
            ($ line-row {:key (:id row) :row row :strategy strategy :t t :open (= open (:id row))
                         :on-toggle #(on-open (when-not (= open (:id row)) (:id row)))}
               (line-view row)))))))

;; ----------------------------------------------------------------------------------------------- the page

(defui page [{:keys [scenario run query]}]
  (let [scenarios (api/use-get "/sandbox/scenarios")
        world-state (api/use-get (str "/sandbox/scenarios/" scenario))
        world (get-in world-state [:data :world])
        this (some #(when (= scenario (:name %)) %) (get-in scenarios [:data :scenarios]))
        ;; The world says so too: a scenario made since the list was read is still the record's.
        record (or (:record this) (:record world))
        ;; A record scenario's one run is its coding as it stands, whatever the list read earlier said.
        newest (if record "latest" (:id (first (:runs this))))
        run-state (api/use-get (when run (str "/sandbox/scenarios/" scenario "/runs/" run)))
        ;; A record scenario's run over another set of lines than its own is that set's id.
        over (when (and record run (not= "latest" run)) run)
        ;; A record scenario's own threshold and strategy (the run's defaults when it sets none) are where it starts.
        t (/ (js/Number (or (:t query) (some-> (:threshold world) js/Number (* 100) js/Math.round) 85)) 100)
        strategy (keyword (or (:s query) (some-> (:strategy world) name) (if (:record world) "checked" "choice")))
        show (keyword (or (:f query) "all"))
        open (:line query)
        go! (fn [changes replace?] (route/go! ["sandbox" scenario run] (merge query changes) replace?))
        lines (get-in run-state [:data :lines])
        scored (uix/use-memo #(mapv (fn [r] (assoc r :code (get-in r [:decisions strategy :code])))
                                    (score lines strategy t))
                             [lines strategy t])]
    (uix/use-effect
     (fn [] (when (and (not run) newest) (route/go! ["sandbox" scenario newest] query true)))
     [run newest scenario query])
    ($ ui/world-context {:value world}
       ($ :div
          ($ :div {:class "flex flex-wrap items-baseline justify-between gap-3"}
             ($ :div
                ($ :div {:class "text-sm text-ink-3"} ($ :a {:href "#/sandbox" :class "hover:underline"} "Sandbox") " / " scenario
                   (when record
                     ($ :<> " · " ($ :a {:href (route/href ["scenarios" scenario]) :class "text-lapis hover:underline"}
                                     "lines from your books; change its rules on the scenario's page →")))
                   (when over ($ :<> " · over the lines " ($ ui/mono {} over))))
                ($ :h1 {:class "font-display text-2xl font-extrabold tracking-tight"} (or (:title this) "Bank lines into buckets")))
             ($ :label {:class "flex items-center gap-2 text-sm text-ink-3"} "Run"
                ($ :select {:value (or run "") :on-change #(route/go! ["sandbox" scenario (.. % -target -value)] query)
                            :class "rounded-1 border border-line-2 bg-surface-0 px-2 py-1 font-mono text-xs text-ink-1"}
                   (for [{:keys [id at over]} (some #(when (= scenario (:name %)) (:runs %)) (get-in scenarios [:data :scenarios]))]
                     ($ :option {:key id :value id} (if over (str "over " over ", " (subs at 0 16)) at))))))
          ($ ui/loading {:state world-state}
             ($ world-strip {:world world}))
          (cond
            (and record (:data run-state) (nil? (get-in run-state [:data :run])))
            ($ ui/panel {:class "mt-4 p-4"} ($ running/run-panel {:s {:ref scenario} :over over}))

            (and (not run) (:data scenarios) world (not newest))
            ($ ui/notice {}
               "These sample lines (made up) have no run kept in this server's runs folder. To code your books' own bank lines, "
               ($ :a {:href (route/href ["sandbox"]) :class "underline"} "pick a scenario over your books") ".")

            :else
          ($ ui/loading {:state (if run run-state {:loading true})}
             (let [{:keys [stale] info :run} (:data run-state)]
               ($ :<>
                  (when stale ($ ui/notice {:tone :bad} stale))
                  (when record
                    ($ ui/panel {:class "mt-4 px-4 pb-3"}
                       ($ running/run-bar {:ref scenario :over over})
                       ($ ui/disclosure {:summary (str (:results info) " lines coded"
                                                       " · a trial of a few lines, and the runs so far")}
                          ($ running/run-panel {:s {:ref scenario} :over over}))))
                  ($ :div {:class "mt-2 text-xs text-ink-3"}
                     ($ ui/mono {} (:model info)) " · " (:results info) " lines and items"
                     (when-let [{:keys [requests input-tokens]} (:usage info)] (str " · " requests " requests, " input-tokens " tokens in"))
                     (when-let [{:keys [commit dirty]} (:source info)] (str " · code " (subs (str commit) 0 7) (when dirty ", with changes"))))
                  ($ levers {:scored scored :lines lines :strategy strategy :t t
                             :on-strategy #(go! {:s (name %)} true) :on-threshold #(go! {:t (js/Math.round (* 100 %))} true)})
                  ($ benchmark {:lines lines :strategy strategy :t t :on-strategy #(go! {:s (name %)} true)})
                  ($ ui/section-title {:aside ($ :<> "Each bucket's lines, coloured by their " ($ term {:id :outcome}))} ($ term {:id :bucket} "The buckets"))
                  ($ board {:world world :scored scored
                            :line-href #(route/href ["sandbox" scenario run] (assoc query :line % :f nil))})
                  ($ ui/section-title {:aside "Open a line to walk it through"} "Every line")
                  ($ lines-table {:scored scored :strategy strategy :t t :open open :show show
                                  :on-filter #(go! {:f (when-not (= :all %) (name %))} true)
                                  :on-open #(go! {:line %} true)
                                  :line-view (fn [row] ($ line/walk {:scenario scenario :run run :row row :strategy strategy :t t}))})))))))))
