(ns ownpurse.admin.sandbox.home
  "The sandbox's front: the scenarios over this workspace's books, then the sample lines (made up), and their runs."
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
          "Run a scenario (how to code: the world, the buckets and rules) over lines, from your books or made up, and "
          "see how each line is coded, why, what it cost, and, where a person keyed it, whether it was right.")
       ($ ui/loading {:state state}
          ($ :<>
             (when (seq record)
               ($ :<>
                  ($ ui/section-title {:aside "This workspace's own bank lines: coded, or sent to a person; scored where a person keyed them"}
                     "Your books")
                  ($ :div {:class "grid gap-4 md:grid-cols-2"}
                     (for [s record] ($ card (assoc s :key (:name s)))))))
             ($ ui/section-title {:aside "Invented lines with an answer key, for building and testing rules"}
                "Sample lines (made up)")
             ($ :div {:class "grid gap-4 md:grid-cols-2"}
                (for [s invented] ($ card (assoc s :key (:name s))))))))))
