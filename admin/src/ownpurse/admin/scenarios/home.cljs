(ns ownpurse.admin.scenarios.home
  "Every scenario of the books: a named stack of layers over the record's facts, standing on another or on nothing.
  A new one starts empty, or standing on another with one layer of changes; nothing in one posts anywhere."
  (:require
   [clojure.string :as str]
   [ownpurse.admin.api :as api]
   [ownpurse.admin.route :as route]
   [ownpurse.admin.ui :as ui]
   [uix.core :as uix :refer [$ defui]]))

(def ref-pattern
  "What a scenario's or a layer's ref may be: the API's own rule, said before it is asked."
  #"^[A-Za-z0-9][A-Za-z0-9._-]*$")

(defn layers-summary
  "A scenario's layers in a few words: each kind's count, and how many are off."
  [layers]
  (let [off (count (remove :enabled layers))]
    (str (str/join " · " (for [[k n] (frequencies (map :kind layers))] (str n " " k)))
         (when (pos? off) (str " (" off " off)")))))

(defui new-form
  "A new scenario: empty, or standing on another with a coding layer to change its rules in."
  [{:keys [scenarios on-done]}]
  (let [[ref set-ref!] (uix/use-state "")
        [nm set-name!] (uix/use-state "")
        [base set-base!] (uix/use-state "")
        [note set-note!] (uix/use-state "")
        {:keys [send! busy error]} (api/use-send)
        bad-ref (and (seq ref) (not (re-matches ref-pattern ref)))
        submit (fn [e]
                 (.preventDefault e)
                 (-> (send! "PUT" (str "/scenarios/" (js/encodeURIComponent ref))
                            (cond-> {:name nm
                                     :layers (if (seq base)
                                               [{:id "changes" :kind "coding" :coding {}
                                                 :note "What this scenario changes in the one it stands on."}]
                                               [])}
                              (seq base) (assoc :base base)
                              (seq note) (assoc :note note)))
                     (.then #(when % (on-done ref)))))]
    ($ :form {:on-submit submit :class "grid gap-3 p-4 md:grid-cols-[1fr_1.4fr_1fr]"}
       ($ ui/field {:label "Ref (in the URL; letters, digits, . _ -)"}
          ($ ui/text-input {:value ref :on-change set-ref! :placeholder "per-diem-try" :name "ref"}))
       ($ ui/field {:label "Name"}
          ($ ui/text-input {:value nm :on-change set-name! :placeholder "What it tries" :name "name"}))
       ($ ui/field {:label "Stands on"}
          ($ ui/select {:value base :on-change set-base! :aria-label "Stands on"
                        :options (into [["" "nothing: start empty"]]
                                       (for [{r :ref n :name} scenarios] [r (str r " — " n)]))}))
       ($ ui/field {:label "Note (optional)" :class "md:col-span-2"}
          ($ ui/text-input {:value note :on-change set-note! :placeholder "Why it exists, what to look at" :name "note"}))
       ($ :div {:class "flex items-end gap-3"}
          ($ ui/button {:tone :primary :type "submit" :disabled (or busy (str/blank? ref) (str/blank? nm) bad-ref)}
             (if busy "Creating…" "Create scenario")))
       (when bad-ref ($ ui/notice {:tone :bad} "A ref is letters, digits, dots, dashes and underscores, starting with a letter or digit."))
       (when error ($ :div {:class "md:col-span-3"} ($ ui/notice {:tone :bad} error)))
       ($ ui/say {:class "md:col-span-3"}
          (if (seq base)
            (str "It starts with every layer of " base " under it and one empty coding layer, \"changes\", of its own: "
                 "a rule put there replaces the rule of its id below; a rule turned off there leaves the stack.")
            "It starts with no layers: add a world, a coding and a lines layer on its page.")))))

(defui page []
  (let [state (api/use-get "/scenarios")
        [adding set-adding!] (uix/use-state false)
        scenarios (get-in state [:data :scenarios])
        children (group-by :base scenarios)]
    ($ :div
       ($ :div {:class "flex items-baseline justify-between gap-4"}
          ($ :h1 {:class "font-display text-2xl font-extrabold tracking-tight"} "Scenarios")
          ($ ui/button {:tone (when-not adding :primary) :on-click #(set-adding! (not adding))}
             (if adding "Cancel" "New scenario")))
       ($ :p {:class "mt-1 max-w-3xl text-ink-3"}
          "A scenario reads the books through layers: who was where and when (world), the rules that code bank lines "
          "(coding), which lines (lines), journals of its own and the close register's. Each stands on another or on "
          "nothing; a layer turned off counts nowhere, and every version is kept. Try one, compare, throw it away: "
          "nothing in a scenario posts anywhere.")
       (when adding
         ($ ui/panel {:class "mt-4"}
            ($ new-form {:scenarios scenarios
                         :on-done (fn [ref] (set-adding! false) (route/go! ["scenarios" ref] nil))})))
       ($ ui/loading {:state state}
          ($ ui/table {:class "mt-5" :head ["Scenario" "Ref" "Stands on" "Layers" "Stood on by" ""]}
             (for [{r :ref n :name :keys [base built-in layers note duplicated-from]} scenarios]
               ($ :tr {:key r :class "hover:bg-surface-1"}
                  ($ ui/td {}
                     ($ :a {:href (route/href ["scenarios" r]) :class "font-medium hover:underline"} n)
                     (when note ($ :div {:class "max-w-xl text-xs text-ink-3"} note)))
                  ($ ui/td {} ($ ui/mono {} r))
                  ($ ui/td {} (if base ($ :a {:href (route/href ["scenarios" base]) :class "font-mono text-xs text-lapis"} base)
                                  ($ ui/muted {} "—")))
                  ($ ui/td {:class "text-xs"} (layers-summary layers))
                  ($ ui/td {:class "text-xs"}
                     (str/join ", " (map :ref (get children r))))
                  ($ ui/td {:class "text-2xs text-ink-3"}
                     (cond built-in "built in"
                           duplicated-from (str "copy of " duplicated-from))))))))))
