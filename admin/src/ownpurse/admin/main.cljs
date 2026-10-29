(ns ownpurse.admin.main
  "The admin: one page, its sections by the URL's hash. ownpurse serves it at /admin over whatever record it runs on,
  so a workspace's admin shows that workspace's books."
  (:require
   [ownpurse.admin.api :as api]
   [ownpurse.admin.docs.guide :as docs.guide]
   [ownpurse.admin.docs.vocabulary :as docs.vocabulary]
   [ownpurse.admin.lines.page :as lines.page]
   [ownpurse.admin.route :as route]
   [ownpurse.admin.sandbox.home :as sandbox.home]
   [ownpurse.admin.sandbox.run :as sandbox.run]
   [ownpurse.admin.scenarios.home :as scenarios.home]
   [ownpurse.admin.scenarios.page :as scenarios.page]
   [ownpurse.admin.theme :as theme]
   [ownpurse.admin.ui :as ui]
   [clojure.string :as str]
   [uix.core :as uix :refer [$ defui]]
   [uix.dom]))

(def sections
  "The admin's sections, in the order the bar lists them."
  [{:id "sandbox" :title "Sandbox" :about "Pick lines and a scenario, run it, and see how each line is coded"}
   {:id "scenarios" :title "Scenarios" :about "The books read through layers: create, duplicate, change and code them, and compare"}
   {:id "lines" :title "Lines" :about "The sets of lines to code: selections over your books, and made-up samples"}
   {:id "docs" :title "Docs" :about "How a bank line is coded, step by step, and every word explained"}])

(defui workspace
  "Where this admin runs, in the bar on every page: a workspace's books by name, with how many and from where; or the
  demo, made-up lines only."
  []
  (let [state (api/use-get (when-not api/static? "/workspace"))
        {:keys [name kind books sources]} (:data state)
        chip "inline-flex items-baseline gap-1.5 rounded-1 border px-2 py-0.5 text-xs"]
    (uix/use-effect
     (fn [] (set! (.-title js/document) (str (or name (when (= "demo" kind) "Demo") "ownpurse") " · ownpurse admin")))
     [name kind])
    (cond
      api/static? ($ :span {:class (ui/cx chip "border-gilt text-gilt-text") :title "The public copy: no server, no books"}
                     "Demo on ownpurse.com")
      (nil? kind) nil
      (= "demo" kind) ($ :span {:class (ui/cx chip "border-gilt text-gilt-text")
                                :title "No books here: the sandbox's made-up lines only"}
                         "Demo · made-up lines only")
      :else ($ :span {:class (ui/cx chip "border-ink-2 text-ink-1")
                      :title "The workspace this admin reads and writes: its books and its scenarios"}
               ($ :span {:class "font-mono text-2xs uppercase tracking-[0.06em] text-ink-3"} "Workspace")
               ($ :b {} (or name "Your books"))
               ($ :span {:class "text-ink-3"}
                  (ui/count-of books "book") (when (seq sources) (str " · " (str/join ", " sources))))))))

(defui shell [{:keys [route children]}]
  (let [here (first (:path route))]
    ($ :div {:class "min-h-screen"}
       ($ :header {:class "sticky top-0 z-20 flex items-center gap-6 border-b border-line-1 bg-surface-0/95 px-6 py-2 backdrop-blur"}
          ($ :a {:href "#/" :class "font-display text-lg font-extrabold tracking-tight"} "ownpurse")
          ($ workspace)
          ($ :nav {:class "flex gap-1"}
             (for [{:keys [id title about]} sections]
               ($ :a {:key id :href (route/href [id]) :title about
                      :class (ui/cx "rounded-1 px-2.5 py-1 text-sm"
                                    (if (= id (or here "sandbox")) "bg-ink-1 text-ink-inverse" "text-ink-2 hover:bg-surface-1"))}
                  title)))
          ($ :div {:class "ml-auto"} ($ theme/switch)))
       (when api/static?
         ($ :div {:class "border-b border-line-1 bg-gilt-tint px-6 py-1.5 text-xs text-gilt-text"}
            "A live copy on ownpurse.com, over an invented studio: every name, card, trip and amount is made up. "
            "The lines were coded by Jev for real, in one of ownpurse's test runs. "
            ($ :a {:class "underline" :href "/docs/guides/classifying/"} "How it works")
            " · "
            ($ :a {:class "underline" :href "https://github.com/brancusi/ownpurse"} "Run it yourself")))
       ($ :main {:class "mx-auto max-w-[1440px] px-6 pt-5 pb-16"} children))))

(defui app []
  (let [{:keys [path] :as r} (route/use-route)
        [section scenario run] path]
    ($ shell {:route r}
       (case (or section "sandbox")
         "sandbox" (if scenario
                     ($ sandbox.run/page {:scenario scenario :run run :query (:query r) :key (str scenario "/" run)})
                     ($ sandbox.home/page {:query (:query r)}))
         "scenarios" (if scenario
                       ($ scenarios.page/page {:ref scenario :query (:query r) :key scenario})
                       ($ scenarios.home/page))
         "lines" ($ lines.page/page)
         "docs" (if (= "terms" scenario)
                  ($ docs.vocabulary/page {:query (:query r)})
                  ($ docs.guide/page {:query (:query r)}))
         ($ ui/notice {} "No such page. Pick a section above.")))))

(defonce ^:private root (delay (uix.dom/create-root (js/document.getElementById "app"))))

(defn ^:dev/after-load render []
  (uix.dom/render-root ($ app) @root))

(defn init []
  (theme/follow!)
  (render))
