(ns ownpurse.admin.sandbox.home
  "The sandbox's front: the record's scenarios that code its own bank lines, then the invented ones, and their runs."
  (:require
   [ownpurse.admin.api :as api]
   [ownpurse.admin.route :as route]
   [ownpurse.admin.scenarios.running :as running]
   [ownpurse.admin.ui :as ui]
   [uix.core :refer [$ defui]]))

(defui card [{:keys [name title lines units rules runs record]}]
  ($ ui/panel {:class "p-4"}
     ($ :div {:class "flex items-baseline justify-between gap-3"}
        ($ :a {:href (route/href ["sandbox" name]) :class "font-display text-xl font-bold hover:underline"} (or title name))
        ($ ui/muted {:class "text-xs whitespace-nowrap"} lines " lines · " rules " rules"
           (when (not= lines units) (str " · " units " with items"))))
     (when title ($ ui/mono {:class "text-2xs text-ink-3"} name))
     ($ ui/eyebrow {:class "mt-3 mb-1"} (if record "Its coding" "Runs, newest first"))
     (if (seq runs)
       ($ :ul {:class "divide-y divide-line-1"}
          (for [{:keys [id at]} runs]
            ($ :li {:key id}
               ($ :a {:href (route/href ["sandbox" name id]) :class "flex justify-between py-1 text-sm hover:bg-surface-1"}
                  ($ ui/mono {} (if record (str "last run " (subs at 0 16)) at))
                  ($ ui/muted {:class "text-xs"} "open →")))))
       ($ ui/muted {:class "text-sm"} (if record "No runs yet." "No runs kept on this server.")))
     (when record ($ running/run-bar {:ref name}))))

(defui page []
  (let [state (api/use-get "/sandbox/scenarios")
        {record true invented false} (group-by (comp boolean :record) (get-in state [:data :scenarios]))]
    ($ :div
       ($ :h1 {:class "font-display text-2xl font-extrabold tracking-tight"} "Sandbox")
       ($ :p {:class "mt-1 max-w-3xl text-ink-3"}
          "Bank lines, the world they happened in (people, cards, trips, projects), and a rule set. Each run "
          "classified every line; open one to see how each line was coded, why, and what would change it.")
       ($ ui/loading {:state state}
          ($ :<>
             (when (seq record)
               ($ :<>
                  ($ ui/section-title {:aside "Real lines from the record, with no answer key: coded, or sent to a person"}
                     "The record's scenarios")
                  ($ :div {:class "grid gap-4 md:grid-cols-2"}
                     (for [s record] ($ card (assoc s :key (:name s)))))))
             ($ ui/section-title {:aside "Made-up lines with an answer key, for building and testing rules"}
                "Samples")
             ($ :div {:class "grid gap-4 md:grid-cols-2"}
                (for [s invented] ($ card (assoc s :key (:name s))))))))))
