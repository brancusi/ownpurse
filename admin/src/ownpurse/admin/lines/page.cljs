(ns ownpurse.admin.lines.page
  "The sets of lines this workspace codes: its books' (each a scenario's lines layer, a selection over the books) and
  the made-up samples, each with the scenarios that read it and a link to run them in the sandbox."
  (:require
   [ownpurse.admin.api :as api]
   [ownpurse.admin.route :as route]
   [ownpurse.admin.ui :as ui]
   [clojure.string :as str]
   [uix.core :as uix :refer [$ defui]]))

(defn- slug [s] (-> (str/lower-case (or s "")) (str/replace #"[^a-z0-9]+" "-") (str/replace #"^-|-$" "")))

(defui new-set
  "Saves a set of lines: a selection over the books (some accounts, or every book's lines; days from and to) and a
  title. Any scenario then runs over it in the sandbox."
  []
  (let [accounts-state (api/use-get "/source-accounts")
        accounts (filterv #(not= "org" (:kind %)) (get-in accounts-state [:data :source-accounts]))
        [title set-title!] (uix/use-state "")
        [picked set-picked!] (uix/use-state #{})
        [all set-all!] (uix/use-state false)
        [from set-from!] (uix/use-state "")
        [to set-to!] (uix/use-state "")
        {:keys [send! busy error]} (api/use-send)
        name (slug title)
        save! (fn [e]
                (.preventDefault e)
                (-> (send! "PUT" (str "/lines/" name)
                           {:title title
                            :select (cond-> {}
                                      all (assoc :all true)
                                      (and (not all) (seq picked)) (assoc :accounts (vec picked))
                                      (seq from) (assoc :from from)
                                      (seq to) (assoc :to to))})
                    (.then #(when % (route/go! ["sandbox"] {:lines name})))))]
    ($ ui/panel {:class "p-4"}
       ($ :form {:on-submit save!}
          ($ ui/field {:label "Title"}
             ($ ui/text-input {:value title :on-change set-title! :placeholder "December's card lines"}))
          (when (seq name) ($ ui/say {} "Kept as " ($ ui/mono {} name) "."))
          ($ ui/eyebrow {:class "mt-3 mb-1"} "Accounts")
          ($ ui/switch {:on all :on-change set-all! :label "Every book's lines"})
          (when-not all
            ($ ui/loading {:state accounts-state}
               ($ :div {:class "mt-1 grid gap-x-4 sm:grid-cols-2"}
                  (for [{:keys [id name source]} accounts]
                    ($ :label {:key id :class "flex items-baseline gap-2 py-0.5 text-sm"}
                       ($ :input {:type "checkbox" :checked (contains? picked id)
                                  :on-change #(set-picked! (fn [p] (if (contains? p id) (disj p id) (conj p id))))})
                       name ($ ui/muted {:class "text-xs"} source))))))
          ($ :div {:class "mt-3 flex flex-wrap gap-4"}
             ($ ui/field {:label "From (blank: the first)"}
                ($ :input {:type "date" :value from :on-change #(set-from! (.. % -target -value))
                           :class "rounded-1 border border-line-2 bg-surface-0 px-2 py-1 text-sm text-ink-1"}))
             ($ ui/field {:label "To (blank: the last)"}
                ($ :input {:type "date" :value to :on-change #(set-to! (.. % -target -value))
                           :class "rounded-1 border border-line-2 bg-surface-0 px-2 py-1 text-sm text-ink-1"})))
          ($ :div {:class "mt-3 flex items-center gap-3"}
             ($ ui/button {:tone :primary :type "submit" :disabled (or busy (empty? name) (and (not all) (empty? picked)))}
                "Save, and pick it in the sandbox")
             ($ ui/muted {:class "text-xs"} "Nothing is asked of Jev until you run a scenario over it."))
          (when error ($ ui/notice {:tone :bad} error))))))

(defui line-set [{:keys [id kind title note lines units scenarios saved]}]
  ($ ui/panel {:class "p-4"}
     ($ :div {:class "flex items-baseline justify-between gap-3"}
        ($ :span {:class "font-display text-xl font-bold"} title)
        ($ ui/muted {:class "text-xs whitespace-nowrap"}
           (ui/count-of lines "line") (when (and units (not= lines units)) (str " · " units " with items"))))
     (when note ($ :p {:class "mt-1 text-sm text-ink-2"} note))
     ($ ui/eyebrow {:class "mt-3 mb-1"} (cond saved "Saved" (= "books" kind) "Chosen by" :else "Read by"))
     ($ :p {:class "text-sm"}
        (cond
          saved "A set of its own: no scenario reads it by default, and any runs over it."
          (= "books" kind)
          ($ :<> "The lines layer of " ($ :a {:class "text-lapis hover:underline" :href (route/href ["scenarios" (first scenarios)])}
                                          (first scenarios))
             (when (next scenarios) (str ", read by " (ui/count-of (next scenarios) "scenario") " standing on it")))
          :else "Its own rules and answer key, kept with the lines; your scenarios run over it too."))
     ($ :a {:class "mt-3 inline-block text-sm text-lapis hover:underline" :href (route/href ["sandbox"] {:lines id})}
        "Run them in the sandbox →")))

(defui page []
  (let [state (api/use-get "/sandbox/lines")
        {books "books" samples "sample"} (group-by :kind (get-in state [:data :lines]))]
    ($ :div
       ($ :h1 {:class "font-display text-2xl font-extrabold tracking-tight"} "Lines")
       ($ :p {:class "mt-1 max-w-3xl text-ink-3"}
          "Sets of bank lines to code: a selection over your books (accounts and days, one line, or every book's), or "
          "made-up lines. Any scenario runs over any of them in the sandbox; each run is kept apart. A key (lines a "
          "person coded) is kept with the set; evidence (a receipt, an order, a note) will be too.")
       ($ ui/loading {:state state}
          ($ :<>
             (when (seq books)
               ($ :<>
                  ($ ui/section-title {:aside "Selections over this workspace's own bank lines"} "Your books")
                  ($ :div {:class "grid gap-4 md:grid-cols-2"}
                     (for [s books] ($ line-set (assoc s :key (:id s)))))))
             (when-not api/static?
               ($ :<>
                  ($ ui/section-title {:aside "A selection over your books, for any scenario to run over"} "A new set")
                  ($ new-set)))
             ($ ui/section-title {:aside "Invented lines with an answer key, for building and testing rules"}
                "Sample lines (made up)")
             ($ :div {:class "grid gap-4 md:grid-cols-2"}
                (for [s samples] ($ line-set (assoc s :key (:id s))))))))))
