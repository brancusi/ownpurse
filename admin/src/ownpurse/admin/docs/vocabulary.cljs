(ns ownpurse.admin.docs.vocabulary
  "Every word the admin uses, grouped by where it turns up, each with where it comes from: the page a dotted word
  links to."
  (:require
   [clojure.string :as str]
   [ownpurse.admin.route :as route]
   [ownpurse.admin.terms :as terms :refer [term]]
   [ownpurse.admin.ui :as ui :refer [cx]]
   [uix.core :as uix :refer [$ defui]]))

(defn- matches? [q {:keys [term short long origin-note]}]
  (or (str/blank? q)
      (str/includes? (str/lower-case (str/join " " (concat [term short origin-note] long))) (str/lower-case q))))

(defui entry [{:keys [t on?]}]
  (let [{:keys [id short long origin origin-note step see] title :term} t]
    ($ :article {:id (str "term-" (name id))
                 :class (cx "scroll-mt-28 border-t border-line-1 py-4 transition-colors duration-700"
                            (when on? "bg-lapis-tint"))}
       ($ :div {:class "grid gap-x-8 gap-y-2 md:grid-cols-[220px_minmax(0,1fr)]"}
          ($ :div
             ($ :h3 {:class "font-display text-lg font-bold leading-tight"} title)
             ($ :div {:class "mt-1.5"} ($ terms/origin-badge {:origin origin})))
          ($ :div {:class "max-w-[68ch]"}
             ($ :p {:class "text-base font-medium"} short)
             (for [[i p] (map-indexed vector long)]
               ($ :p {:key i :class "mt-2 text-sm leading-relaxed text-ink-2"} p))
             ($ :p {:class "mt-2.5 text-xs text-ink-3"}
                ($ :span {:class "font-mono uppercase tracking-[0.08em]"} "Where the word comes from · ") origin-note)
             ($ :div {:class "mt-2.5 flex flex-wrap items-baseline gap-x-4 gap-y-1 text-xs"}
                (when step
                  ($ :a {:class "font-mono text-lapis hover:underline" :href (route/href ["docs"] {:step step})}
                     "See it happen: step " step " →"))
                (when (seq see)
                  ($ :span {:class "text-ink-3"} "Related: "
                     (for [[i s] (map-indexed vector see)]
                       ($ :<> {:key (name s)} (when (pos? i) ", ") ($ term {:id s})))))))))))

(defui page [{:keys [query]}]
  (let [[q set-q!] (uix/use-state "")
        on (some-> (:term query) keyword)
        shown (filter #(matches? q %) terms/terms)
        by-group (group-by :group shown)]
    (uix/use-effect
     (fn []
       (when on
         (some-> (js/document.getElementById (str "term-" (name on))) (.scrollIntoView #js {:behavior "smooth"}))))
     [on])
    ($ :div {:class "max-w-[1100px]"}
       ($ ui/eyebrow {} "Docs / vocabulary")
       ($ :h1 {:class "mt-1 font-display text-2xl font-extrabold tracking-tight"} "The words, and where they come from")
       ($ :p {:class "mt-2 max-w-[70ch] text-base leading-relaxed text-ink-2"}
          "Every word the admin uses, in plain language. Some are an accountant's, some a bank's, some come from "
          "machine learning, and a few ownpurse made up. Each entry says which, so you know when a word will mean the "
          "same thing elsewhere. Anywhere in the admin, a word with a dotted underline shows its meaning when you hover it.")
       ($ :div {:class "mt-5 grid gap-3 md:grid-cols-5"}
          (for [[k n about] terms/origins]
            ($ :div {:key (name k) :class "border border-line-1 bg-surface-1 p-2.5"}
               ($ :div {:class "font-mono text-xs font-medium"} n)
               ($ :div {:class "mt-1 text-xs text-ink-2"} about))))
       ($ :div {:class "sticky top-[45px] z-10 mt-6 flex flex-wrap items-center gap-3 bg-surface-0 py-2"}
          ($ :input {:type "search" :value q :placeholder "Find a word…" :aria-label "Find a word"
                     :on-change #(set-q! (.. % -target -value))
                     :class "w-64 rounded-1 border border-line-2 bg-surface-0 px-2.5 py-1.5 text-sm placeholder:text-ink-3"})
          ($ :nav {:class "flex flex-wrap gap-x-3 gap-y-1 text-xs"}
             (for [[g n] terms/groups :when (seq (by-group g))]
               ($ :a {:key (name g) :href (str "#/docs/terms?group=" (name g))
                      :on-click (fn [e] (.preventDefault e)
                                  (some-> (js/document.getElementById (str "group-" (name g))) (.scrollIntoView #js {:behavior "smooth"})))
                      :class "text-ink-2 hover:text-ink-1 hover:underline"} n)))
          ($ ui/muted {:class "ml-auto font-mono text-xs"} (count shown) " of " (count terms/terms)))
       (for [[g n] terms/groups
             :let [ts (by-group g)]
             :when (seq ts)]
         ($ :section {:key (name g) :id (str "group-" (name g)) :class "mt-8 scroll-mt-28"}
            ($ :h2 {:class "border-b-2 border-ink-1 pb-1.5 font-mono text-xs font-medium uppercase tracking-[0.08em] text-ink-3"} n)
            (for [t ts] ($ entry {:key (name (:id t)) :t t :on? (= on (:id t))}))))
       (when (empty? shown)
         ($ ui/notice {} "No word matches “" q "”.")))))
