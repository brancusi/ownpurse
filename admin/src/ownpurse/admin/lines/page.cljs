(ns ownpurse.admin.lines.page
  "The sets of lines this workspace codes: its books' (each a scenario's lines layer, a selection over the books) and
  the made-up samples, each with the scenarios that read it and a link to run them in the sandbox."
  (:require
   [ownpurse.admin.api :as api]
   [ownpurse.admin.route :as route]
   [ownpurse.admin.ui :as ui]
   [uix.core :refer [$ defui]]))

(defui line-set [{:keys [id kind title note lines units scenarios]}]
  ($ ui/panel {:class "p-4"}
     ($ :div {:class "flex items-baseline justify-between gap-3"}
        ($ :span {:class "font-display text-xl font-bold"} title)
        ($ ui/muted {:class "text-xs whitespace-nowrap"}
           (ui/count-of lines "line") (when (and units (not= lines units)) (str " · " units " with items"))))
     (when note ($ :p {:class "mt-1 text-sm text-ink-2"} note))
     ($ ui/eyebrow {:class "mt-3 mb-1"} (if (= "books" kind) "Chosen by" "Read by"))
     ($ :p {:class "text-sm"}
        (if (= "books" kind)
          ($ :<> "The lines layer of " ($ :a {:class "text-lapis hover:underline" :href (route/href ["scenarios" (first scenarios)])}
                                          (first scenarios))
             (when (next scenarios) (str ", read by " (ui/count-of (next scenarios) "scenario") " standing on it")))
          "Its own rules and answer key, kept with the lines."))
     ($ :a {:class "mt-3 inline-block text-sm text-lapis hover:underline" :href (route/href ["sandbox"] {:lines id})}
        "Run them in the sandbox →")))

(defui page []
  (let [state (api/use-get "/sandbox/lines")
        {books "books" samples "sample"} (group-by :kind (get-in state [:data :lines]))]
    ($ :div
       ($ :h1 {:class "font-display text-2xl font-extrabold tracking-tight"} "Lines")
       ($ :p {:class "mt-1 max-w-3xl text-ink-3"}
          "Sets of bank lines to code: a selection over your books (accounts and dates), or made-up lines. Any of them is "
          "picked the same way in the sandbox. A key (lines a person coded) and evidence (a receipt, an order, a note) "
          "will be kept with the lines they belong to.")
       ($ ui/loading {:state state}
          ($ :<>
             (when (seq books)
               ($ :<>
                  ($ ui/section-title {:aside "Selections over this workspace's own bank lines"} "Your books")
                  ($ :div {:class "grid gap-4 md:grid-cols-2"}
                     (for [s books] ($ line-set (assoc s :key (:id s)))))))
             ($ ui/section-title {:aside "Invented lines with an answer key, for building and testing rules"}
                "Sample lines (made up)")
             ($ :div {:class "grid gap-4 md:grid-cols-2"}
                (for [s samples] ($ line-set (assoc s :key (:id s))))))))))
