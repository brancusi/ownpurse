(ns ownpurse.admin.scenarios.coding
  "A scenario's coding of bank lines: what a run would take, a run in the background and its progress, the bucket
  each line stands in, one line all the way through, and where two scenarios code a line differently. Every answer
  Jev gives is kept by its exact request, so a run after a change asks only what the change touched."
  (:require
   [clojure.string :as str]
   [ownpurse.admin.api :as api]
   [ownpurse.admin.route :as route]
   [ownpurse.admin.sandbox.run :as sandbox.run]
   [ownpurse.admin.scenarios.running :refer [run-panel]]
   [ownpurse.admin.scenarios.common :refer [path]]
   [ownpurse.admin.ui :as ui :refer [cx]]
   [uix.core :as uix :refer [$ defui]]))

(defn- n [x] (if (keyword? x) (name x) (str x)))

(defn- here
  "This tab's link with `query` changed (nil values dropped)."
  [s query changes]
  (route/href ["scenarios" (:ref s)] (merge query changes)))

;; ----------------------------------------------------------------------------------------------- one unit

(defn- fact-text [v]
  (cond (map? v) (str/join ", " (for [[k x] v] (str (n k) " " (fact-text x))))
        (sequential? v) (str/join "; " (map fact-text v))
        :else (n v)))

(def ^:private strategy-words (into {} (map (fn [[k label]] [(name k) label])) sandbox.run/strategies))

(defui unit-detail [{:keys [s unit query]}]
  (let [state (api/use-get (str (path (:ref s) "codings") "/" (js/encodeURIComponent unit)))
        composed (get-in (api/use-get (path (:ref s) "composed")) [:data :composed])
        threshold (:threshold composed)
        deciding (some-> (:strategy composed) n)
        {:keys [facts decisions code proposed confidence auto merchant date amount currency]} (get-in state [:data :coding])]
    ($ ui/panel {:class "p-4"}
       ($ :div {:class "flex items-baseline justify-between"}
          ($ ui/eyebrow {} "One line, all the way through")
          ($ :a {:href (here s query {:unit nil}) :class "text-xs text-lapis"} "close ×"))
       ($ ui/loading {:state state}
          ($ :div
             ($ :div {:class "mt-1 text-lg font-medium"} merchant)
             ($ :div {:class "text-sm text-ink-2"} ($ ui/mono {} date) " · " ($ ui/amount {:amount (or amount 0) :currency currency}))
             ($ :div {:class "mt-2 flex items-center gap-2"}
                ($ ui/mono {:class "font-medium"} code)
                (when (not= code proposed) ($ ui/muted {:class "text-xs"} "(proposed " proposed ")"))
                ($ ui/bar {:p confidence :wide true}) ($ ui/mono {:class "text-xs"} (ui/pct confidence))
                ($ :span {:class (cx "text-xs" (if auto "text-verdigris" "text-gilt-text"))}
                   (if auto "stands without a person" "a person looks")))
             (when (and threshold (not auto))
               ($ ui/say {}
                  (cond (= "needs-person" proposed) "The classifier proposes that a person looks."
                        (< confidence threshold) (str "Below the threshold: " (ui/pct confidence) " sure, and a coding needs "
                                                      (ui/pct threshold) " to stand without a person.")
                        :else "A check rule holds, so a person looks.")))
             ($ ui/section-title {} "What the classifier knew")
             ($ :dl {:class "grid gap-x-3 gap-y-0.5 text-xs md:grid-cols-[9rem_1fr]"}
                (for [[k v] (sort-by (comp n key) (dissoc facts :id))]
                  ($ :<> {:key (n k)} ($ :dt {:class "text-ink-3"} (n k)) ($ :dd {:class "break-all"} (fact-text v)))))
             ($ ui/section-title {} "Each strategy's coding")
             ($ ui/say {} "The strategy marked ◆ decides; the others are shown to compare.")
             ($ ui/table {:head ["Strategy" "Code" "How sure" "By rules"]}
                (for [[k {c :code :keys [confidence by reason]}] (sort-by #(if (= deciding (n (key %))) "" (n (key %))) decisions)]
                  ($ :tr {:key (n k) :class (when (= deciding (n k)) "bg-surface-1")}
                     ($ ui/td {} (when (= deciding (n k)) "◆ ") (get strategy-words (n k) (n k))
                        ($ :div {:class "font-mono text-2xs text-ink-3"} (n k)))
                     ($ ui/td {} ($ ui/mono {} (n c)) (when reason ($ :div {:class "text-2xs text-ink-3"} (n reason))))
                     ($ ui/td {} ($ ui/mono {} (ui/pct confidence)))
                     ($ ui/td {} ($ ui/mono {} (str/join " " by)))))))))))

;; ----------------------------------------------------------------------------------------------- the codings

(defui codings-table [{:keys [s query]}]
  (let [state (api/use-get (path (:ref s) "codings"))
        codings (get-in state [:data :codings])
        {:keys [code stands q unit]} query
        counts (frequencies (map :code codings))
        shown (cond->> codings
                (seq code) (filter #(= code (:code %)))
                (= "auto" stands) (filter :auto)
                (= "person" stands) (remove :auto)
                (seq q) (filter #(str/includes? (str/lower-case (str (:merchant %))) (str/lower-case q))))
        [text set-text!] (uix/use-state (or q ""))]
    ($ ui/loading {:state state}
       (if (empty? codings)
         ($ ui/notice {} "Not coded yet: code its lines above. A trial of a few lines first costs little.")
         ($ :div
            ($ :div {:class "flex flex-wrap items-center gap-1.5"}
               ($ :a {:href (here s query {:code nil}) :class (cx "rounded-1 px-2 py-0.5 text-xs" (if (seq code) "bg-surface-1" "bg-ink-1 text-ink-inverse"))}
                  "every bucket " (count codings))
               (for [[c k] (sort-by (comp - val) counts)]
                 ($ :a {:key c :href (here s query {:code c})
                        :class (cx "rounded-1 px-2 py-0.5 font-mono text-xs" (if (= c code) "bg-ink-1 text-ink-inverse" "bg-surface-1 hover:bg-surface-2"))}
                    c " " k))
               ($ :span {:class "mx-2 text-line-2"} "|")
               (for [[v label] [[nil "all"] ["auto" "stand alone"] ["person" "a person looks"]]]
                 ($ :a {:key (str label) :href (here s query {:stands v})
                        :class (cx "rounded-1 px-2 py-0.5 text-xs" (if (= v stands) "bg-ink-1 text-ink-inverse" "bg-surface-1 hover:bg-surface-2"))}
                    label))
               ($ :form {:class "ml-auto" :on-submit (fn [e] (.preventDefault e) (route/go! ["scenarios" (:ref s)] (assoc query :q (not-empty text))))}
                  ($ ui/text-input {:value text :on-change set-text! :placeholder "merchant contains…" :class "w-52" :name "q"})))
            ($ :div {:class (cx "mt-3 grid gap-4" (when unit "lg:grid-cols-[1fr_28rem]"))}
               ($ ui/table {:head ["Date" "Merchant" "Amount" "Stands in" "Proposed" "How sure" ""]}
                  (for [{u :unit :keys [date merchant amount currency proposed confidence auto] c :code} shown]
                    ($ :tr {:key u :class (cx "hover:bg-surface-1" (when (= u unit) "bg-surface-1"))}
                       ($ ui/td {} ($ ui/mono {} date))
                       ($ ui/td {} ($ :a {:href (here s query {:unit u}) :class "hover:underline"} merchant))
                       ($ ui/td {:class "text-right"} ($ ui/amount {:amount (or amount 0) :currency currency}))
                       ($ ui/td {} ($ ui/mono {:class (when-not auto "text-gilt-text")} c))
                       ($ ui/td {} (when (not= c proposed) ($ ui/mono {:class "text-ink-3"} proposed)))
                       ($ ui/td {} ($ ui/bar {:p confidence}) " " ($ ui/mono {:class "text-2xs"} (ui/pct confidence)))
                       ($ ui/td {} ($ :a {:href (here s query {:unit u}) :class "text-xs text-lapis"} "why")))))
               (when unit ($ :div {:class "lg:sticky lg:top-14 lg:self-start"} ($ unit-detail {:s s :unit unit :query query}))))
            ($ ui/say {} (count shown) " of " (count codings) " lines shown."))))))

(defui tab [{:keys [s query]}]
  ($ :div
     ($ run-panel {:s s})
     ($ ui/section-title {:aside "as its last run left them"} "How its lines are coded")
     ($ codings-table {:s s :query query})))

;; ----------------------------------------------------------------------------------------------- comparing

(defn- totals
  "The lines' amounts summed per currency, in words: \"USD 53.69 · EUR 12.00\"."
  [lines]
  (str/join " · " (for [[c xs] (sort-by key (group-by :currency lines))]
                    (str (or c "?") " " (ui/money (reduce + (map #(js/Math.abs (or (:amount %) 0)) xs)))))))

(defui diff-table [{:keys [s other diffs]}]
  ($ :<>
     ($ :div {:class "mb-2 flex flex-wrap gap-2 text-xs"}
        (for [[[a b] k] (sort-by (comp - val) (frequencies (map (juxt (comp :code :a) (comp :code :b)) diffs)))]
          ($ :span {:key (str a b) :class "rounded-1 bg-surface-1 px-2 py-0.5 font-mono"}
             (or a "not coded") " → " (or b "not coded") " " k)))
     ($ ui/table {:head ["Date" "Merchant" "Amount" (str "In " other) (str "In " (:ref s))]}
        (for [{:keys [unit date merchant amount currency a b]} diffs]
          ($ :tr {:key unit}
             ($ ui/td {} ($ ui/mono {} date))
             ($ ui/td {} ($ :a {:href (route/href ["scenarios" (:ref s)] {:tab "coding" :unit unit}) :class "hover:underline"} merchant))
             ($ ui/td {:class "text-right"} ($ ui/amount {:amount (or amount 0) :currency currency}))
             (for [[side x] [[:a a] [:b b]]]
               ($ ui/td {:key (name side)}
                  (if x
                    ($ :<> ($ ui/mono {} (:code x))
                       (when (not= (:code x) (:proposed x)) ($ ui/muted {:class "text-2xs"} " (proposed " (:proposed x) ")"))
                       ($ :div {:class "text-2xs text-ink-3"} (ui/pct (:confidence x))))
                    ($ ui/muted {} "not coded")))))))))

(defui compare-tab [{:keys [s scenarios query]}]
  (let [other (or (:vs query) (:base s) (:duplicated-from s))
        ;; From the other scenario to this one: what this one changes.
        state (api/use-get (when other (str (path other "compare") "/" (js/encodeURIComponent (:ref s)))))
        mine (api/use-get (path (:ref s) "codings"))
        theirs (api/use-get (when other (path other "codings")))
        diffs (sort-by (juxt :date :unit) (get-in state [:data :differences]))
        {moved true surer false} (group-by #(not= (get-in % [:a :proposed]) (get-in % [:b :proposed])) diffs)
        coded-mine (count (get-in mine [:data :codings]))
        coded-theirs (count (get-in theirs [:data :codings]))]
    ($ :div
       ($ :div {:class "flex flex-wrap items-end gap-3"}
          ($ ui/field {:label "What it changes from"}
             ($ ui/select {:value (or other "") :aria-label "Compare with"
                           :on-change #(route/go! ["scenarios" (:ref s)] {:tab "compare" :vs (not-empty %)})
                           :options (into [["" "pick a scenario"]]
                                          (for [{r :ref} scenarios :when (not= r (:ref s))] [r r]))}))
          (when other
            ($ ui/say {} other " has coded " (ui/count-of coded-theirs "line") "; " (:ref s) " has coded " coded-mine ".")))
       (when (and other (or (zero? coded-mine) (zero? coded-theirs)))
         ($ ui/notice {} "Code both scenarios' lines first (Coding tab): the comparison reads their last runs."))
       (when other
         ($ ui/loading {:state state}
            (if (empty? diffs)
              ($ ui/notice {} "They code every line they both coded the same way.")
              ($ :<>
                 ($ ui/section-title {:aside (str (ui/count-of moved "line") " · " (totals moved))}
                    "Where the bucket proposed changes")
                 (if (seq moved)
                   ($ diff-table {:s s :other other :diffs moved})
                   ($ ui/notice {} "None: every line is proposed for the same bucket in both."))
                 ($ ui/section-title {:aside (str (ui/count-of surer "line") " · " (totals surer))}
                    "Same bucket proposed, but sure enough in one only")
                 ($ ui/say {} "These lines stand without a person in one scenario and go to a person in the other: the "
                    "proposal is the same, only how sure the classifier is changed. A new rule's wording can do that.")
                 (when (seq surer) ($ diff-table {:s s :other other :diffs surer})))))))))
