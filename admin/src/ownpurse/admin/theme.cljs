(ns ownpurse.admin.theme
  "Light or dark: the system's (auto, the default), or one picked in the bar and kept in this browser. The tokens
  read the theme from `data-theme` on <html>; index.html sets it before the first paint from the same stored choice."
  (:require
   [ownpurse.admin.ui :as ui]
   [uix.core :as uix :refer [$ defui]]))

(def ^:private stored "ownpurse.admin.theme")

(def ^:private dark-system (js/matchMedia "(prefers-color-scheme: dark)"))

(defn- choice
  "\"auto\", \"light\" or \"dark\": what this browser keeps, auto when nothing is kept or storage is closed."
  []
  (or (try (#{"light" "dark"} (.getItem js/localStorage stored)) (catch :default _ nil)) "auto"))

(defn- apply! [c]
  (.setAttribute js/document.documentElement "data-theme"
                 (if (= c "auto") (if (.-matches dark-system) "dark" "light") c)))

(defn- keep! [c]
  (try (if (= c "auto") (.removeItem js/localStorage stored) (.setItem js/localStorage stored c))
       (catch :default _ nil)))

(defn follow!
  "Applies the kept choice, and follows the system's setting as it changes while the choice is auto."
  []
  (apply! (choice))
  (.addEventListener dark-system "change" #(apply! (choice))))

(def ^:private options [["auto" "Auto" "Follow the system's light or dark setting"]
                        ["light" "Light" "Always light"]
                        ["dark" "Dark" "Always dark"]])

(defui switch
  "Auto, light or dark, for the bar."
  []
  (let [[c set-c!] (uix/use-state choice)]
    ($ :div {:role "radiogroup" :aria-label "Theme" :class "flex rounded-1 border border-line-2 p-0.5"}
       (for [[id words about] options
             :let [on? (= id c)]]
         ($ :button {:key id :role "radio" :aria-checked on? :title about
                     :on-click #(do (keep! id) (apply! id) (set-c! id))
                     :class (ui/cx "rounded-1 px-2 py-0.5 text-xs"
                                   (if on? "bg-ink-1 text-ink-inverse" "text-ink-2 hover:bg-surface-1"))}
            words)))))
