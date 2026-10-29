(ns ownpurse.admin.ui
  "The admin's building blocks, and the only place their styles are written: a screen composes these and lays them
  out; it does not restyle them. Colours, type and radii come from ownpurse's design tokens through Tailwind
  (src/admin.css). Colour marks an outcome only, as the design system's coins do: verdigris right, coin wrong, gilt
  sent to a person needlessly, ink sent to a person rightly."
  (:require
   [clojure.string :as str]
   [uix.core :as uix :refer [$ defui]]))

(defn cx
  "Class names joined, nils and falses left out: (cx \"px-2\" (when on? \"bg-ink-1\"))."
  [& classes]
  (str/join " " (filter identity classes)))

(def world-context
  "The scenario's world (people, cards, projects, buckets), for the blocks that name things."
  (uix/create-context nil))

(defn use-world [] (uix/use-context world-context))

(defn pct [p] (str (js/Math.round (* 100 (or p 0))) "%"))

(defn count-of
  "\"1 card\", \"3 cards\": a count and its noun."
  ([xs noun] (count-of xs noun (str noun "s")))
  ([xs noun plural] (let [k (if (number? xs) xs (count xs))] (str k " " (if (= 1 k) noun plural)))))

(defn money [x] (.toFixed (js/Number x) 2))

;; ----------------------------------------------------------------------------------------------- outcomes

(def outcome-words
  {:right "coded, right" :wrong "coded, wrong" :unsafe "coded, but a person had to look"
   :stopped "to a person, rightly" :review "to a person, needlessly"})

(def ^:private outcome-dot
  {:right "bg-verdigris" :wrong "bg-coin" :unsafe "bg-coin-tint ring-2 ring-inset ring-coin"
   :review "bg-gilt" :stopped "ring-2 ring-inset ring-ink-3"})

(def ^:private outcome-pill
  {:right "bg-verdigris-tint text-verdigris" :wrong "bg-coin text-on-coin"
   :unsafe "bg-coin-tint text-coin-text ring-1 ring-inset ring-coin" :review "bg-gilt-tint text-gilt-text"
   :stopped "bg-surface-2 text-ink-2"})

(defn bad? [outcome] (#{:wrong :unsafe} outcome))

(defui dot [{:keys [outcome]}]
  ($ :span {:class (cx "inline-block size-2.5 flex-none rounded-full" (outcome-dot outcome))
            :title (outcome-words outcome)}))

(defui pill [{:keys [outcome children]}]
  ($ :span {:class (cx "whitespace-nowrap rounded-1 px-2 py-1 font-mono text-2xs" (outcome-pill outcome))}
     (or children (outcome-words outcome))))

(defui chip
  "A line's id, coloured by its outcome, as a link."
  [{:keys [outcome href children]}]
  ($ :a {:href href :title (outcome-words outcome)
         :class (cx "rounded-1 px-1 py-0.5 font-mono text-2xs leading-none"
                    (case outcome
                      :right "bg-verdigris-tint text-verdigris"
                      :wrong "bg-coin text-on-coin"
                      :unsafe "bg-coin-tint text-coin-text ring-1 ring-inset ring-coin"
                      :review "bg-gilt-tint text-gilt-text"
                      "bg-surface-1 text-ink-2"))}
     children))

;; ----------------------------------------------------------------------------------------------- text

(defui eyebrow
  "The small capitals over a group: what follows."
  [{:keys [class children]}]
  ($ :div {:class (cx "font-mono text-2xs font-medium uppercase tracking-[0.08em] text-ink-3" class)} children))

(defui section-title [{:keys [children aside]}]
  ($ :div {:class "mt-8 mb-2 flex items-baseline justify-between gap-4 border-b-2 border-ink-1 pb-1.5"}
     ($ eyebrow {} children)
     (when aside ($ :div {:class "text-xs text-ink-3"} aside))))

(defui muted [{:keys [children class]}]
  ($ :span {:class (cx "text-ink-3" class)} children))

(defui mono [{:keys [children class title]}]
  ($ :span {:class (cx "font-mono text-xs" class) :title title} children))

(defui say
  "A sentence of explanation under a block."
  [{:keys [children class]}]
  ($ :p {:class (cx "mt-1.5 text-sm text-ink-2" class)} children))

(defui notice [{:keys [tone children]}]
  ($ :div {:class (cx "my-3 border-l-2 px-3 py-2 text-sm"
                      (if (= tone :bad) "border-coin bg-coin-tint text-coin-text" "border-line-2 text-ink-2"))}
     children))

;; ----------------------------------------------------------------------------------------------- figures

(defui amount [{:keys [amount currency]}]
  ($ :span {:class (cx "whitespace-nowrap text-right font-mono text-sm tabular-nums" (when (neg? amount) "text-coin-text"))}
     ($ :span {:class "mr-1 text-2xs text-ink-3"} currency) (money amount)))

(defui bar
  "A probability as a short bar; under the threshold, greyed."
  [{:keys [p threshold wide]}]
  ($ :span {:class (cx "relative inline-block h-1 bg-surface-2" (if wide "w-24" "w-9"))}
     ($ :span {:class (cx "absolute inset-y-0 left-0" (if (and threshold (< p threshold)) "bg-line-2" "bg-ink-2"))
               :style {:width (pct p)}})))

;; ----------------------------------------------------------------------------------------------- buckets

(defn group-of
  "Each bucket's group (its project's name, or the group's title) and its short label."
  [{:keys [buckets projects]}]
  (into {} (for [{:keys [project title codes]} buckets
                 [code label] codes]
             [code {:group (or (get-in projects [project :name]) title) :label label}])))

(defui tag
  "A bucket: its label, and its project as a quiet prefix. `miss` marks one the answer key does not accept."
  [{:keys [code miss]}]
  (let [world (use-world)
        groups (uix/use-memo #(group-of world) [world])
        {:keys [group label]} (get groups code {:group "" :label (some-> code name)})
        solo (#{"Studio" "Personal, or a person to look"} group)]
    ($ :span {:title (get-in world [:codes code])
              :class (cx "inline-flex max-w-full items-baseline gap-1.5 overflow-hidden text-ellipsis whitespace-nowrap rounded-1 px-1.5 py-0.5 text-xs"
                         (if miss "bg-coin-tint text-coin-text" "bg-surface-2 text-ink-1"))}
       (when-not (or solo (str/blank? group))
         ($ :i {:class (cx "font-mono text-2xs not-italic" (if miss "text-coin-text" "text-ink-3"))}
            (first (str/split group #" "))))
       label)))

;; ----------------------------------------------------------------------------------------------- layout

(defui panel
  "A bordered block on the page's paper."
  [{:keys [class children]}]
  ($ :div {:class (cx "border border-line-1 bg-surface-0" class)} children))

(defui disclosure
  "A summary that opens to more."
  [{:keys [summary children class]}]
  ($ :details {:class (cx "mt-1.5" class)}
     ($ :summary {:class "cursor-pointer text-xs text-lapis"} summary)
     ($ :div {:class "mt-1.5"} children)))

(defui code-block [{:keys [value]}]
  ($ :pre {:class "mt-1 max-h-[420px] overflow-auto whitespace-pre-wrap rounded-1 border border-line-1 bg-surface-0 p-2 font-mono text-2xs text-ink-2"}
     (js/JSON.stringify (clj->js value) nil 2)))

(defui loading
  "Its children once `state` (from api/use-get) has its data; meanwhile, or on an error, says so."
  [{:keys [state children]}]
  (cond
    (:error state) ($ notice {:tone :bad} (:error state))
    (:loading state) ($ notice {} "Loading…")
    :else children))

;; ----------------------------------------------------------------------------------------------- controls

(defui button
  "A button. `tone` :primary for the one thing a block is for, :danger for what drops something; else quiet."
  [{:keys [tone on-click disabled type title children]}]
  ($ :button {:type (or type "button") :on-click on-click :disabled disabled :title title
              :class (cx "whitespace-nowrap rounded-1 px-2.5 py-1 text-xs disabled:cursor-not-allowed disabled:opacity-50"
                         (case tone
                           :primary "bg-ink-1 text-ink-inverse hover:bg-ink-2"
                           :danger "border border-coin text-coin-text hover:bg-coin-tint"
                           "border border-line-2 text-ink-1 hover:bg-surface-1"))}
     children))

(def ^:private field-class
  "w-full rounded-1 border border-line-2 bg-surface-0 px-2 py-1 text-sm text-ink-1 focus:outline-2 focus:outline-focus")

(defui text-input
  "A one-line text field; `on-change` gets the text."
  [{:keys [value on-change placeholder class name aria-label]}]
  ($ :input {:type "text" :value (or value "") :placeholder placeholder :name name :aria-label aria-label
             :on-change #(on-change (.. % -target -value)) :class (cx field-class class)}))

(defui number-input
  [{:keys [value on-change min class aria-label]}]
  ($ :input {:type "number" :value (or value "") :min min :aria-label aria-label
             :on-change #(on-change (.. % -target -value)) :class (cx (str/replace field-class "w-full" "w-24") class)}))

(defui text-area
  "A field for a block of text, such as a layer as JSON; monospace, as tall as `rows`."
  [{:keys [value on-change rows name aria-label]}]
  ($ :textarea {:value (or value "") :rows (or rows 12) :name name :aria-label aria-label :spell-check false
                :on-change #(on-change (.. % -target -value))
                :class (cx field-class "font-mono text-2xs leading-snug")}))

(defui select
  "A choice among `options`, [value label] pairs; `on-change` gets the value."
  [{:keys [value on-change options aria-label class]}]
  ($ :select {:value (or value "") :aria-label aria-label :on-change #(on-change (.. % -target -value))
              :class (cx field-class "w-auto" class)}
     (for [[v label] options] ($ :option {:key v :value v} label))))

(defui field
  "A labelled control."
  [{:keys [label children class]}]
  ($ :label {:class (cx "block" class)}
     ($ eyebrow {:class "mb-1"} label)
     children))

(defui switch
  "On or off, as a small switch with its label."
  [{:keys [on on-change label disabled]}]
  ($ :label {:class (cx "inline-flex cursor-pointer items-center gap-1.5 text-xs" (when disabled "cursor-not-allowed opacity-50"))}
     ($ :input {:type "checkbox" :checked (boolean on) :disabled disabled :class "peer sr-only"
                :on-change #(on-change (.. % -target -checked))})
     ($ :span {:class (cx "relative inline-block h-3.5 w-6 rounded-full transition-colors"
                          (if on "bg-verdigris" "bg-line-2"))}
        ($ :span {:class (cx "absolute top-0.5 size-2.5 rounded-full bg-surface-0 transition-all" (if on "left-3" "left-0.5"))}))
     ($ :span {:class (if on "text-ink-1" "text-ink-3")} label)))

(defui table
  "A dense table: `head` the column titles, `children` the rows (tr)."
  [{:keys [head children class]}]
  ($ :div {:class (cx "overflow-x-auto border border-line-1" class)}
     ($ :table {:class "w-full border-collapse text-sm"}
        ($ :thead
           ($ :tr {:class "border-b border-line-1 bg-surface-1 text-left"}
              (for [[i h] (map-indexed vector head)]
                ($ :th {:key i :class "px-2 py-1 font-mono text-2xs font-medium uppercase tracking-[0.06em] text-ink-3"} h))))
        ($ :tbody {:class "divide-y divide-line-1"} children))))

(defui td [{:keys [children class col-span]}]
  ($ :td {:class (cx "px-2 py-1 align-top" class) :col-span col-span} children))

(defui tabs
  "Links along a page, the current one marked: `items` [[id label href]]."
  [{:keys [items current]}]
  ($ :nav {:class "mt-4 flex gap-1 border-b border-line-1"}
     (for [[id label href] items]
       ($ :a {:key id :href href
              :class (cx "-mb-px border-b-2 px-3 py-1.5 text-sm"
                         (if (= id current) "border-ink-1 text-ink-1" "border-transparent text-ink-3 hover:text-ink-1"))}
          label))))
