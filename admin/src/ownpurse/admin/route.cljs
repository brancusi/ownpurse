(ns ownpurse.admin.route
  "Where the admin is, in the URL's hash: #/sandbox/<scenario>/<run>?line=d211&t=85&s=choice&f=wrong. Every view is a
  link: a line, a threshold and a filter can be sent to someone and opened as they were."
  (:require
   [clojure.string :as str]
   [uix.core :refer [use-effect use-state]]))

(defn parse
  "The route in hash `h`: {:path [segments] :query {key string}}."
  [h]
  (let [[path query] (str/split (str/replace (or h "") #"^#/?" "") #"\?" 2)]
    {:path (vec (remove str/blank? (map js/decodeURIComponent (str/split (or path "") #"/"))))
     :query (into {} (for [kv (str/split (or query "") #"&") :when (seq kv)
                           :let [[k v] (str/split kv #"=" 2)]]
                       [(keyword k) (js/decodeURIComponent (or v ""))]))}))

(defn href
  "The hash for `path` (segments) and `query` (a map; nil values left out)."
  ([path] (href path nil))
  ([path query]
   (str "#/" (str/join "/" (map js/encodeURIComponent path))
        (when-let [q (seq (remove (comp nil? val) query))]
          (str "?" (str/join "&" (for [[k v] (sort-by key q)] (str (name k) "=" (js/encodeURIComponent (str v))))))))))

(defn go!
  "Moves to `path` with `query`, replacing the history entry when `replace?` (a slider's every step is not a page)."
  ([path query] (go! path query false))
  ([path query replace?]
   (let [h (href path query)]
     (if replace?
       (js/history.replaceState nil "" h)
       (set! (.-hash js/location) h))
     (js/dispatchEvent (js/HashChangeEvent. "hashchange")))))

(defn use-route
  "The current route, kept up to date as the hash changes."
  []
  (let [[route set-route!] (use-state (parse js/location.hash))]
    (use-effect
     (fn []
       (let [f #(set-route! (parse js/location.hash))]
         (js/addEventListener "hashchange" f)
         #(js/removeEventListener "hashchange" f)))
     [])
    route))
