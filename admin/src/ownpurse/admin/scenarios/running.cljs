(ns ownpurse.admin.scenarios.running
  "Running a scenario's coding: what a run would ask of Jev now, cost and take, before anything is asked; a button to
  start one; its progress; and the runs so far. On a scenario's coding tab, and in the sandbox where a scenario of
  the record has no run yet."
  (:require
   [ownpurse.admin.api :as api]
   [ownpurse.admin.route :as route]
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
                  ($ :<> ($ :b {} "nothing to ask") ", every answer is kept already. ")
                  ($ :<> ($ :b {} asked " to ask") (when (pos? kept) (str ", " kept " answered from those kept")) ". "))
                "It would cost " ($ :b {} (dollars cost-usd))
                (when (pos? asked)
                  (str " (" (.toLocaleString (js/Number input-tokens)) " input tokens at $"
                       price-per-million-input-tokens " a million; output is free)"))
                " and take " ($ :b {} (about-seconds seconds)) ".")
             ($ ui/say {}
                "Counted by walking every line through the classifier with the answers kept. "
                (if timed "The time is from this scenario's runs so far. " "The time is a guess until a run has been timed. ")
                "The price is TypeSafe's published one: the API reports tokens, not prices."))))))

(defn use-run
  "A scenario's coding runs, polled while one is going: {:state :plan :running :runs :going :start! :busy :error}.
  `start!` codes every line, or at most `limit`. When a run ends, everything that reads codings asks again."
  [ref]
  (let [[polling set-polling!] (uix/use-state false)
        state (api/use-get (path ref "coding") {:every (when polling 1500)})
        {:keys [send! busy error]} (api/use-send)
        {:keys [plan running runs]} (:data state)
        going (boolean (and running (not (:failed running))))]
    (uix/use-effect
     (fn []
       (cond
         (and going (not polling)) (set-polling! true)
         (and polling (not going) (:data state)) (do (set-polling! false) (api/invalidate!))))
     [going polling state])
    {:state state :plan plan :running running :runs runs :going going :busy busy :error error
     :start! (fn [limit]
               (-> (send! "POST" (path ref "coding") (if limit {:limit limit} {}))
                   (.then #(when % (set-polling! true)))))}))

(defui progress [{:keys [running]}]
  ($ :div {:class "mt-3"}
     ($ :div {:class "flex justify-between text-xs text-ink-2"}
        ($ :span {} "Coding: " (:done running) " of " (:of running))
        ($ ui/mono {:class "text-ink-3"} "started " (some-> (:started running) (subs 11 19))))
     ($ :div {:class "mt-1 h-1.5 bg-surface-2"}
        ($ :div {:class "h-full bg-ink-2 transition-all"
                 :style {:width (str (* 100 (/ (:done running) (max 1 (:of running)))) "%")}}))))

(defui run-bar
  "One line and a button: what a run of every line would ask, cost and take now, and Run. Its progress while it
  goes. For the sandbox's cards and the top of a scenario's lines."
  [{:keys [ref]}]
  (let [{:keys [runs running going start! busy error]} (use-run ref)
        est (api/use-get (path ref "coding" "estimate"))
        {:keys [asked kept cost-usd seconds]} (get-in est [:data :estimate])]
    ($ :div {:class "mt-3"}
       ($ :div {:class "flex flex-wrap items-center justify-between gap-3"}
          ($ :span {:class "text-sm text-ink-2"}
             (cond
               going "Coding now…"
               (nil? asked) "Working out what a run would take…"
               (zero? asked) (str "A run now: nothing to ask (" kept " answers kept), costs nothing, " (about-seconds seconds) ".")
               :else (str "A run now: " asked " requests to ask Jev"
                          (when (pos? kept) (str ", " kept " kept")) ", " (dollars cost-usd) ", " (about-seconds seconds) ".")))
          ($ ui/button {:tone :primary :disabled (or busy going (nil? asked)) :on-click #(start! nil)}
             (cond going "Coding…" (seq runs) "Run again" :else "Run")))
       (when going ($ progress {:running running}))
       (when (:failed running) ($ ui/notice {:tone :bad} "The last run failed: " (:failed running)))
       (when error ($ ui/notice {:tone :bad} error)))))

(defui run-panel
  "The plan, the run going on, and a button to code the lines (all, or at most `limit`)."
  [{:keys [s]}]
  (let [{:keys [state plan running runs going start! busy error]} (use-run (:ref s))
        [limit set-limit!] (uix/use-state "")]
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
                                    (start! (when (seq limit) (js/parseInt limit))))}
                ($ ui/field {:label "At most (blank: all)"}
                   ($ ui/number-input {:value limit :on-change set-limit! :min 1 :aria-label "At most this many lines"}))
                ($ ui/button {:tone :primary :type "submit" :disabled (or busy going)}
                   (cond going "Coding…" (seq limit) (str "Code " limit " lines") :else "Code every line"))))
          (when going ($ progress {:running running}))
          (when (:failed running) ($ ui/notice {:tone :bad} "The last run failed: " (:failed running)))
          (when error ($ ui/notice {:tone :bad} error))
          (when (seq runs)
            ($ :a {:class "mt-3 inline-block text-sm text-lapis hover:underline" :href (route/href ["sandbox" (:ref s) "latest"])}
               "Open its coding in the sandbox →"))
          (when (seq runs)
            ($ ui/disclosure {:summary (str (count runs) " runs so far; the last " (some-> (:at (first runs)) (subs 0 16)))}
               ($ ui/table {:head ["At" "Lines" "Coded" "Asked Jev" "Answered from those kept" "Took" "Model"]}
                  (for [{:keys [at units coded asked kept model took-ms]} runs]
                    ($ :tr {:key (str at)}
                       ($ ui/td {} ($ ui/mono {} (some-> at str (subs 0 19))))
                       ($ ui/td {} units) ($ ui/td {} coded) ($ ui/td {} asked) ($ ui/td {} kept)
                       ($ ui/td {} (if took-ms (str (.toFixed (/ took-ms 1000) 1) " s") ($ ui/muted {} "—")))
                       ($ ui/td {} ($ ui/mono {:class "text-ink-3"} model)))))))))))
