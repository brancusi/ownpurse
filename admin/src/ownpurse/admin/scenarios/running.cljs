(ns ownpurse.admin.scenarios.running
  "Running a scenario's coding: what a run would ask of Jev now, cost and take, before anything is asked; a button to
  start one; its progress; and the runs so far. On a scenario's coding tab, and in the sandbox where a scenario of
  the record has no run yet."
  (:require
   [ownpurse.admin.api :as api]
   [ownpurse.admin.scenarios.common :refer [path]]
   [ownpurse.admin.ui :as ui]
   [uix.core :as uix :refer [$ defui]]))

(defn- about-seconds [n]
  (cond (< n 2) "a second or two" (< n 90) (str "about " n " seconds") :else (str "about " (js/Math.round (/ n 60)) " minutes")))

(defn- dollars [x]
  (let [x (js/Number x)]
    (cond (zero? x) "nothing" (< x 0.01) "under a cent" :else (str "about $" (.toFixed x 2)))))

(defui estimate-line
  "What a run would ask of Jev now, what it would cost and how long it would take, before anything is asked."
  [{:keys [ref limit]}]
  (let [state (api/use-get (str (path ref "coding" "estimate") (when (seq limit) (str "?limit=" limit))))
        {:keys [units requests asked kept input-tokens cost-usd seconds timed price-per-million-input-tokens]}
        (get-in state [:data :estimate])]
    ($ :div {:class "text-sm"}
       ($ ui/eyebrow {} "If you run it now")
       ($ ui/loading {:state state}
          ($ :<>
             ($ :p {}
                (ui/count-of units "line") " to code, " (ui/count-of requests "request") " to Jev: "
                (if (zero? asked)
                  ($ :<> ($ :b {} "nothing to ask") ", every answer is in the record already. ")
                  ($ :<> ($ :b {} asked " to ask") (when (pos? kept) (str ", " kept " answered from the record")) ". "))
                "It would cost " ($ :b {} (dollars cost-usd))
                (when (pos? asked)
                  (str " (" (.toLocaleString (js/Number input-tokens)) " input tokens at $"
                       price-per-million-input-tokens " a million; output is free)"))
                " and take " ($ :b {} (about-seconds seconds)) ".")
             ($ ui/say {}
                "Counted by walking every line through the classifier with the answers the record holds. "
                (if timed "The time is from this scenario's runs so far. " "The time is a guess until a run has been timed. ")
                "The price is TypeSafe's published one: the API reports tokens, not prices."))))))

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
          (when (and (empty? runs) (not going))
            ($ ui/notice {} "No runs yet: nothing of this scenario has been coded. Start one below."))
          ($ :div {:class "mt-3 flex flex-wrap items-end gap-4"}
             ($ :div {:class "max-w-3xl"}
                ($ estimate-line {:ref (:ref s) :limit limit})
                ($ ui/say {}
                   (ui/count-of (:lines plan) "line")
                   (when (not= (:lines plan) (:units plan)) (str " (" (:units plan) " to code, counting receipt items)"))
                   ", at most " (:at-most-requests plan) " requests. After a rule changes, most lines are asked "
                   "again (the rules go with each request); after a threshold or strategy changes, nothing is."))
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
               ($ ui/table {:head ["At" "Lines" "Coded" "Asked Jev" "Answered from the record" "Took" "Model"]}
                  (for [{:keys [at units coded asked kept model took-ms]} runs]
                    ($ :tr {:key (str at)}
                       ($ ui/td {} ($ ui/mono {} (some-> at str (subs 0 19))))
                       ($ ui/td {} units) ($ ui/td {} coded) ($ ui/td {} asked) ($ ui/td {} kept)
                       ($ ui/td {} (if took-ms (str (.toFixed (/ took-ms 1000) 1) " s") ($ ui/muted {} "—")))
                       ($ ui/td {} ($ ui/mono {:class "text-ink-3"} model)))))))))))
