(ns ownpurse.admin.main
  "The admin: one page, its sections by the URL's hash. ownpurse serves it at /admin over whatever record it runs on,
  so a workspace's admin shows that workspace's books."
  (:require
   [ownpurse.admin.route :as route]
   [ownpurse.admin.sandbox.home :as sandbox.home]
   [ownpurse.admin.sandbox.run :as sandbox.run]
   [ownpurse.admin.theme :as theme]
   [ownpurse.admin.ui :as ui]
   [uix.core :refer [$ defui]]
   [uix.dom]))

(def sections
  "The admin's sections, in the order the bar lists them."
  [{:id "sandbox" :title "Sandbox" :about "Build a rule set over bank lines, and see how each line is coded"}])

(defui shell [{:keys [route children]}]
  (let [here (first (:path route))]
    ($ :div {:class "min-h-screen"}
       ($ :header {:class "sticky top-0 z-20 flex items-center gap-6 border-b border-line-1 bg-surface-0/95 px-6 py-2 backdrop-blur"}
          ($ :a {:href "#/" :class "font-display text-lg font-extrabold tracking-tight"} "ownpurse")
          ($ :nav {:class "flex gap-1"}
             (for [{:keys [id title about]} sections]
               ($ :a {:key id :href (route/href [id]) :title about
                      :class (ui/cx "rounded-1 px-2.5 py-1 text-sm"
                                    (if (= id (or here "sandbox")) "bg-ink-1 text-ink-inverse" "text-ink-2 hover:bg-surface-1"))}
                  title)))
          ($ :div {:class "ml-auto"} ($ theme/switch)))
       ($ :main {:class "mx-auto max-w-[1440px] px-6 pt-5 pb-16"} children))))

(defui app []
  (let [{:keys [path] :as r} (route/use-route)
        [section scenario run] path]
    ($ shell {:route r}
       (case (or section "sandbox")
         "sandbox" (if scenario
                     ($ sandbox.run/page {:scenario scenario :run run :query (:query r) :key (str scenario "/" run)})
                     ($ sandbox.home/page))
         ($ ui/notice {} "No such page. Pick a section above.")))))

(defonce ^:private root (delay (uix.dom/create-root (js/document.getElementById "app"))))

(defn ^:dev/after-load render []
  (uix.dom/render-root ($ app) @root))

(defn init []
  (theme/follow!)
  (render))
