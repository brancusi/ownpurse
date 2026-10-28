(ns ownpurse.admin.sandbox.home
  "The sandbox's front: each scenario of bank lines and its runs."
  (:require
   [ownpurse.admin.api :as api]
   [ownpurse.admin.route :as route]
   [ownpurse.admin.ui :as ui]
   [uix.core :refer [$ defui]]))

(defui page []
  (let [state (api/use-get "/sandbox/scenarios")]
    ($ :div
       ($ :h1 {:class "font-display text-2xl font-extrabold tracking-tight"} "Sandbox")
       ($ :p {:class "mt-1 max-w-3xl text-ink-3"}
          "Bank lines, the world they happened in (people, cards, trips, projects), and a rule set. Each run "
          "classified every line; open one to see how each line was coded, why, and what would change it.")
       ($ ui/loading {:state state}
          ($ :div {:class "mt-6 grid gap-4 md:grid-cols-2"}
             (for [{:keys [name lines units rules runs]} (get-in state [:data :scenarios])]
               ($ ui/panel {:key name :class "p-4"}
                  ($ :div {:class "flex items-baseline justify-between"}
                     ($ :a {:href (route/href ["sandbox" name]) :class "font-display text-xl font-bold hover:underline"} name)
                     ($ ui/muted {:class "text-xs"} lines " lines · " units " lines and items · " rules " rules"))
                  ($ ui/eyebrow {:class "mt-3 mb-1"} "Runs, newest first")
                  ($ :ul {:class "divide-y divide-line-1"}
                     (for [{:keys [id at]} runs]
                       ($ :li {:key id}
                          ($ :a {:href (route/href ["sandbox" name id]) :class "flex justify-between py-1 text-sm hover:bg-surface-1"}
                             ($ ui/mono {} at)
                             ($ ui/muted {:class "text-xs"} "open →"))))))))))))
