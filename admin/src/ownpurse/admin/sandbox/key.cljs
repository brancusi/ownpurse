(ns ownpurse.admin.sandbox.key
  "Keying a line of the record: a person says which bucket it belongs in, and the scenario's codings of it are scored
  right or wrong from then on, as the invented lines are against their answer key. A line is keyed in one scenario
  of the stack: the one at the bottom by default, so every scenario standing on it is scored the same way; one
  higher up keys it otherwise when its policy differs (per diem on one trip only, say). Never sent to Jev."
  (:require
   [clojure.string :as str]
   [ownpurse.admin.api :as api]
   [ownpurse.admin.scenarios.common :refer [path]]
   [ownpurse.admin.ui :as ui]
   [uix.core :as uix :refer [$ defui]]))

(defn- bucket-options
  "Every bucket of the world, in its groups' order, then needs-person, as [value label]."
  [world]
  (let [listed (for [{:keys [codes]} (:buckets world) [c label] codes] [(name c) label])
        seen (set (map first listed))]
    (vec (concat listed
                 (for [c (keys (:codes world)) :when (not (seen (name c)))] [(name c) (name c)])
                 (when-not (seen "needs-person") [["needs-person" "Needs a person"]])))))

(defui confirm
  "What the key says of `unit`, and a form to key it: the bucket, in which scenario of the stack, whether a person
  must or may look, and why. `chosen` is the strategy's coding, offered first."
  [{:keys [unit chosen world]}]
  (let [stack (:stack world)
        root (:ref (last stack))
        [open set-open!] (uix/use-state false)
        [code set-code!] (uix/use-state (some-> (or (:expect unit) chosen) name))
        [in set-in!] (uix/use-state (or (:keyed-in unit) root))
        [review set-review!] (uix/use-state (some-> (:review unit) name))
        [why set-why!] (uix/use-state (or (:why unit) ""))
        {:keys [send! busy error]} (api/use-send)
        key! (fn [ref body] (-> (send! "POST" (path ref "key") (assoc body :unit (:id unit))) (.then #(when % (set-open! false)))))
        submit (fn [e]
                 (.preventDefault e)
                 (key! in (cond-> {:expect code} (seq review) (assoc :review review) (seq (str/trim why)) (assoc :why why))))]
    ($ :div {:class "flex flex-wrap items-center gap-x-3 gap-y-1.5"}
       (if (:expect unit)
         ($ :<>
            ($ ui/eyebrow {} "Belongs in") ($ ui/tag {:code (:expect unit)})
            ($ ui/muted {:class "text-xs"} "keyed in " (:keyed-in unit)
               (case (:review unit) :required ", a person must look" :allowed ", a person may look" nil)))
         ($ :<>
            ($ ui/muted {:class "text-xs"} "Not keyed: say where it belongs to score it")
            (when (and chosen (not= :needs-person chosen))
              ($ ui/button {:disabled busy :on-click #(key! root {:expect (name chosen)})
                            :title (str "Key it in " root ": every scenario standing on it is scored against it")}
                 "✓ " (name chosen) " is right"))))
       ($ ui/button {:on-click #(set-open! (not open))} (cond open "Cancel" (:expect unit) "Change" :else "Key it otherwise…"))
       (when (and (:expect unit) (:keyed-in unit))
         ($ ui/button {:tone :danger :disabled busy :on-click #(key! (:keyed-in unit) {:remove true})}
            "Remove from " (:keyed-in unit)))
       (when open
         ($ :form {:on-submit submit :class "mt-2 grid w-full gap-3 border border-line-1 bg-surface-0 p-3 md:grid-cols-4"}
            ($ ui/field {:label "Belongs in"}
               ($ ui/select {:value (or code "") :on-change set-code! :aria-label "Belongs in" :options (bucket-options world)}))
            ($ ui/field {:label "Key it in"}
               ($ ui/select {:value (or in "") :on-change set-in! :aria-label "Key it in"
                             :options (for [{:keys [ref name]} stack]
                                        [ref (str ref (when (= ref root) " (every scenario on it)") " — " name)])}))
            ($ ui/field {:label "A person"}
               ($ ui/select {:value (or review "") :on-change set-review! :aria-label "A person"
                             :options [["" "need not look"] ["allowed" "may look"] ["required" "must look"]]}))
            ($ ui/field {:label "Why (optional)"}
               ($ ui/text-input {:value why :on-change set-why! :placeholder "What a person knows that the line does not"}))
            ($ :div {:class "md:col-span-4 flex items-center gap-3"}
               ($ ui/button {:tone :primary :type "submit" :disabled (or busy (str/blank? code))} (if busy "Keying…" "Key it"))
               ($ ui/say {:class "mt-0"} "The key is never sent to Jev: it only scores the codings."))))
       (when error ($ ui/notice {:tone :bad} error)))))
