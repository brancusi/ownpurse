(ns ownpurse.admin.scenarios.coding
  "A scenario's coding of bank lines: what a run would take, a run in the background and its progress, the bucket
  each line stands in, one line all the way through, and where two scenarios code a line differently. Every answer
  Jev gives is kept by its exact request, so a run after a change asks only what the change touched."
  (:require
   [clojure.string :as str]
   [ownpurse.admin.api :as api]
   [ownpurse.admin.route :as route]
   [ownpurse.admin.scenarios.common :refer [path]]
   [ownpurse.admin.ui :as ui :refer [cx]]
   [uix.core :as uix :refer [$ defui]]))

(defn- n [x] (if (keyword? x) (name x) (str x)))

(defn- here
  "This tab's link with `query` changed (nil values dropped)."
  [s query changes]
  (route/href ["scenarios" (:ref s)] (merge query changes)))

;; ----------------------------------------------------------------------------------------------- a run

(defui run-panel
  "The plan, the run going on, and a button to code the lines (all, or at most `limit`)."
  [{:keys [s]}]
  (let [[polling set-polling!] (uix/use-state false)
        state (api/use-get (path (:ref s) "coding") {:every (when polling 1500)})
        [limit set-limit!] (uix/use-state "")
        {:keys [send! busy error]} (api/use-send)
        {:keys [plan running runs]} (:data state)
        going (and running (not (:failed running)))]
    ;; Once the run is over, everything that reads its codings asks again.
    (uix/use-effect
     (fn []
       (cond
         (and going (not polling)) (set-polling! true)
         (and polling (not going) (:data state)) (do (set-polling! false) (api/invalidate!))))
     [going polling state])
    ($ ui/loading {:state state}
       ($ :div
          ($ :div {:class "flex flex-wrap items-end gap-4"}
             ($ :div {:class "text-sm"}
                ($ ui/eyebrow {} "What a run takes")
                (:lines plan) " lines, " (:units plan) " lines and receipt items to code, at most "
                (:at-most-requests plan) " requests to Jev; answers already in the record are not asked again.")
             ($ :form {:class "ml-auto flex items-end gap-2"
                       :on-submit (fn [e]
                                    (.preventDefault e)
                                    (-> (send! "POST" (path (:ref s) "coding")
                                               (if (seq limit) {:limit (js/parseInt limit)} {}))
                                        (.then #(when % (set-polling! true)))))}
                ($ ui/field {:label "At most (blank: all)"}
                   ($ ui/number-input {:value limit :on-change set-limit! :min 1 :aria-label "At most this many lines"}))
                ($ ui/button {:tone :primary :type "submit" :disabled (or busy going)}
                   (cond going "Coding…" (seq limit) (str "Code " limit " lines") :else "Code every line"))))
          (when going
            ($ :div {:class "mt-3"}
               ($ :div {:class "flex justify-between text-xs text-ink-2"}
                  ($ :span {} "Coding: " (:done running) " of " (:of running))
                  ($ ui/mono {:class "text-ink-3"} "started " (some-> (:started running) (subs 11 19))))
               ($ :div {:class "mt-1 h-1.5 bg-surface-2"}
                  ($ :div {:class "h-full bg-ink-2 transition-all"
                           :style {:width (str (* 100 (/ (:done running) (max 1 (:of running)))) "%")}}))))
          (when (:failed running) ($ ui/notice {:tone :bad} "The last run failed: " (:failed running)))
          (when error ($ ui/notice {:tone :bad} error))
          (when (seq runs)
            ($ ui/disclosure {:summary (str (count runs) " runs so far; the last " (some-> (:at (first runs)) (subs 0 16)))}
               ($ ui/table {:head ["At" "Lines" "Coded" "Asked Jev" "Answered from the record" "Model"]}
                  (for [{:keys [at units coded asked kept model]} runs]
                    ($ :tr {:key (str at)}
                       ($ ui/td {} ($ ui/mono {} (some-> at str (subs 0 19))))
                       ($ ui/td {} units) ($ ui/td {} coded) ($ ui/td {} asked) ($ ui/td {} kept)
                       ($ ui/td {} ($ ui/mono {:class "text-ink-3"} model)))))))))))

;; ----------------------------------------------------------------------------------------------- one unit

(defn- fact-text [v]
  (cond (map? v) (str/join ", " (for [[k x] v] (str (n k) " " (fact-text x))))
        (sequential? v) (str/join "; " (map fact-text v))
        :else (n v)))

(defui unit-detail [{:keys [s unit query]}]
  (let [state (api/use-get (str (path (:ref s) "codings") "/" (js/encodeURIComponent unit)))
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
             ($ ui/section-title {} "What the classifier knew")
             ($ :dl {:class "grid gap-x-3 gap-y-0.5 text-xs md:grid-cols-[9rem_1fr]"}
                (for [[k v] (sort-by (comp n key) (dissoc facts :id))]
                  ($ :<> {:key (n k)} ($ :dt {:class "text-ink-3"} (n k)) ($ :dd {:class "break-all"} (fact-text v)))))
             ($ ui/section-title {} "Each strategy's coding")
             ($ ui/table {:head ["Strategy" "Code" "How sure" "By rules"]}
                (for [[k {c :code :keys [confidence by reason]}] (sort-by (comp n key) decisions)]
                  ($ :tr {:key (n k)}
                     ($ ui/td {} ($ ui/mono {} (n k)))
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

(defui compare-tab [{:keys [s scenarios query]}]
  (let [other (or (:vs query) (:base s) (:duplicated-from s))
        state (api/use-get (when other (str (path (:ref s) "compare") "/" (js/encodeURIComponent other))))
        mine (api/use-get (path (:ref s) "codings"))
        theirs (api/use-get (when other (path other "codings")))
        diffs (get-in state [:data :differences])
        coded-mine (count (get-in mine [:data :codings]))
        coded-theirs (count (get-in theirs [:data :codings]))]
    ($ :div
       ($ :div {:class "flex flex-wrap items-end gap-3"}
          ($ ui/field {:label "Compare its coding with"}
             ($ ui/select {:value (or other "") :aria-label "Compare with"
                           :on-change #(route/go! ["scenarios" (:ref s)] {:tab "compare" :vs (not-empty %)})
                           :options (into [["" "pick a scenario"]]
                                          (for [{r :ref} scenarios :when (not= r (:ref s))] [r r]))}))
          (when other
            ($ ui/say {} (:ref s) " has coded " coded-mine " lines; " other " has coded " coded-theirs ".")))
       (when (and other (or (zero? coded-mine) (zero? coded-theirs)))
         ($ ui/notice {} "Code both scenarios' lines first (Coding tab): the comparison reads their last runs."))
       (when other
         ($ ui/loading {:state state}
            (if (empty? diffs)
              ($ ui/notice {} "They code every line they both coded the same way.")
              ($ :<>
                 ($ ui/section-title {:aside (str (count diffs) " lines")} "Where they differ")
                 ($ :div {:class "mb-2 flex flex-wrap gap-2 text-xs"}
                    (for [[[a b] k] (sort-by (comp - val) (frequencies (map (juxt (comp :code :a) (comp :code :b)) diffs)))]
                      ($ :span {:key (str a b) :class "rounded-1 bg-surface-1 px-2 py-0.5 font-mono"}
                         (or a "not coded") " → " (or b "not coded") " " k)))
                 ($ ui/table {:head ["Date" "Merchant" "Amount" (:ref s) other]}
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
                                ($ ui/muted {} "not coded"))))))))))))))
