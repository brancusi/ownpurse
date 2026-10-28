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
