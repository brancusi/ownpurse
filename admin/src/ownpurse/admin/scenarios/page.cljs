(ns ownpurse.admin.scenarios.page
  "One scenario: what it stands on, its layers (each on or off, each editable as JSON), the rules and world they
  compose to, its coding of bank lines, how it differs from another, what it adds to a book, and every version of it,
  each one restorable. Every change is a new version; nothing here posts anywhere."
  (:require
   [clojure.string :as str]
   [ownpurse.admin.api :as api]
   [ownpurse.admin.route :as route]
   [ownpurse.admin.scenarios.coding :as coding]
   [ownpurse.admin.scenarios.common :refer [data-key json layer-body parse-json path scenario-body templates]]
   [ownpurse.admin.scenarios.home :as home]
   [ownpurse.admin.scenarios.rules :as rules]
   [ownpurse.admin.ui :as ui :refer [cx]]
   [uix.core :as uix :refer [$ defui]]))

(defn layer-summary [{:keys [kind world coding lines journals basis]}]
  (case kind
    "world" (let [{:keys [projects trips cards entities categories]} world]
              (str (count projects) " projects (" (str/join ", " (map (comp :name val) projects)) "), " (count trips)
                   " trips, " (count cards) " cards, " (count entities) " entities, " (count categories) " categories"))
    "coding" (let [{:keys [rules codes threshold strategy]} coding
                   {off false on true} (group-by #(not (false? (:enabled %))) rules)]
               (str/join " · " (remove nil? [(when (seq on) (str "rules " (str/join " " (map :id on))))
                                             (when (seq off) (str "turns off " (str/join " " (map :id off))))
                                             (when (seq codes) (str (count codes) " buckets"))
                                             (when threshold (str "threshold " threshold))
                                             (when strategy (str "strategy " (name strategy)))
                                             (when (and (empty? rules) (empty? codes) (not threshold) (not strategy))
                                               "empty: changes nothing yet")])))
    "lines" (let [{:keys [select cases]} lines]
              (str (when select (str (count (:accounts select)) " account(s), " (or (:from select) "…") " → " (or (:to select) "…")))
                   (when cases (str (count cases) " lines given"))))
    "journals" (str (count journals) " journals")
    "close" (str "the close register, " basis)
    ""))

;; ----------------------------------------------------------------------------------------------- the header

(defui edit-form [{:keys [s scenarios on-done]}]
  (let [[nm set-name!] (uix/use-state (:name s))
        [note set-note!] (uix/use-state (or (:note s) ""))
        [base set-base!] (uix/use-state (or (:base s) ""))
        {:keys [send! busy error]} (api/use-send)]
    ($ :form {:class "grid gap-3 p-4 md:grid-cols-[1.2fr_1.6fr_1fr_auto]"
              :on-submit (fn [e]
                           (.preventDefault e)
                           (-> (send! "PUT" (path (:ref s))
                                      (cond-> (-> (scenario-body s) (assoc :name nm) (dissoc :note :base))
                                        (seq note) (assoc :note note)
                                        (seq base) (assoc :base base)))
                               (.then #(when % (on-done)))))}
       ($ ui/field {:label "Name"} ($ ui/text-input {:value nm :on-change set-name! :name "name"}))
       ($ ui/field {:label "Note"} ($ ui/text-input {:value note :on-change set-note! :name "note"}))
       ($ ui/field {:label "Stands on"}
          ($ ui/select {:value base :on-change set-base! :aria-label "Stands on"
                        :options (into [["" "nothing"]] (for [{r :ref} scenarios :when (not= r (:ref s))] [r r]))}))
       ($ :div {:class "flex items-end gap-2"}
          ($ ui/button {:tone :primary :type "submit" :disabled busy} "Save")
          ($ ui/button {:on-click on-done} "Cancel"))
       (when error ($ :div {:class "md:col-span-4"} ($ ui/notice {:tone :bad} error))))))

(defui copy-form
  "Duplicate (`child` false: the same base and layers) or a new scenario standing on this one (`child` true)."
  [{:keys [s child on-done]}]
  (let [[ref set-ref!] (uix/use-state (str (:ref s) (if child "-try" "-copy")))
        [nm set-name!] (uix/use-state (str (:name s) (if child ", a try" ", a copy")))
        {:keys [send! busy error]} (api/use-send)]
    ($ :form {:class "grid gap-3 p-4 md:grid-cols-[1fr_1.6fr_auto]"
              :on-submit (fn [e]
                           (.preventDefault e)
                           (-> (if child
                                 (send! "PUT" (path ref) {:name nm :base (:ref s)
                                                          :layers [{:id "changes" :kind "coding" :coding {}
                                                                    :note (str "What this scenario changes in " (:ref s) ".")}]})
                                 (send! "POST" (path (:ref s) "duplicate") {:ref ref :name nm}))
                               (.then #(when % (on-done) (route/go! ["scenarios" ref] nil)))))}
       ($ ui/field {:label "New ref"} ($ ui/text-input {:value ref :on-change set-ref! :name "ref"}))
       ($ ui/field {:label "Name"} ($ ui/text-input {:value nm :on-change set-name! :name "name"}))
       ($ :div {:class "flex items-end gap-2"}
          ($ ui/button {:tone :primary :type "submit"
                        :disabled (or busy (not (re-matches home/ref-pattern ref)) (str/blank? nm))}
             (if child "Create it" "Duplicate"))
          ($ ui/button {:on-click on-done} "Cancel"))
       ($ ui/say {:class "md:col-span-3"}
          (if child
            (str "A new scenario with " (:ref s) " under it, and one empty coding layer of its own to change rules in. "
                 "A change to " (:ref s) " shows through it.")
            (str "A new scenario with " (:ref s) "'s base and layers as they stand now. The two change apart from then on.")))
       (when error ($ :div {:class "md:col-span-3"} ($ ui/notice {:tone :bad} error))))))

(defui drop-button [{:keys [s]}]
  (let [[asking set-asking!] (uix/use-state false)
        {:keys [send! busy error]} (api/use-send)]
    ($ :span {:class "inline-flex items-center gap-2"}
       (if asking
         ($ :<>
            ($ :span {:class "text-xs text-coin-text"} "Drop " (:ref s) "? Its versions stay in the record.")
            ($ ui/button {:tone :danger :disabled busy
                          :on-click #(-> (send! "DELETE" (path (:ref s)) nil)
                                         (.then (fn [r] (when r (route/go! ["scenarios"] nil)))))}
               "Drop it")
            ($ ui/button {:on-click #(set-asking! false)} "Keep it"))
         ($ ui/button {:tone :danger :on-click #(set-asking! true)} "Drop"))
       (when error ($ :span {:class "text-xs text-coin-text"} error)))))

(defui header [{:keys [s scenarios]}]
  (let [[open set-open!] (uix/use-state nil)
        {r :ref :keys [base built-in duplicated-from note]} s
        stood-on-by (filter #(= r (:base %)) scenarios)]
    ($ :div
       ($ :div {:class "text-xs text-ink-3"} ($ :a {:href (route/href ["scenarios"]) :class "hover:underline"} "Scenarios") " /")
       ($ :div {:class "flex flex-wrap items-baseline justify-between gap-3"}
          ($ :h1 {:class "font-display text-2xl font-extrabold tracking-tight"} (:name s))
          ($ :div {:class "flex flex-wrap gap-2"}
             (when-not built-in ($ ui/button {:on-click #(set-open! :edit)} "Edit"))
             ($ ui/button {:on-click #(set-open! :duplicate)} "Duplicate")
             ($ ui/button {:on-click #(set-open! :child)} "New scenario on this")
             (when-not built-in ($ drop-button {:s s}))))
       ($ :div {:class "mt-1 flex flex-wrap gap-x-4 text-xs text-ink-3"}
          ($ ui/mono {} r)
          (if base
            ($ :span {} "stands on " ($ :a {:href (route/href ["scenarios" base]) :class "font-mono text-lapis"} base))
            ($ :span {} "stands on nothing"))
          (when duplicated-from ($ :span {} "a copy of " ($ :a {:href (route/href ["scenarios" duplicated-from]) :class "font-mono text-lapis"} duplicated-from)))
          (when (seq stood-on-by)
            ($ :span {} "stood on by "
               (for [{c :ref} stood-on-by]
                 ($ :a {:key c :href (route/href ["scenarios" c]) :class "mr-1 font-mono text-lapis"} c))))
          (when built-in ($ :span {} "built in: duplicate it to change it")))
       (when note ($ ui/say {:class "max-w-4xl"} note))
       (when open
         ($ ui/panel {:class "mt-3"}
            (case open
              :edit ($ edit-form {:s s :scenarios scenarios :on-done #(set-open! nil)})
              :duplicate ($ copy-form {:s s :child false :on-done #(set-open! nil)})
              :child ($ copy-form {:s s :child true :on-done #(set-open! nil)})))))))

;; ----------------------------------------------------------------------------------------------- layers

(defui layer-editor
  "A layer's note and what its kind reads, as JSON, saved as a new version of the scenario."
  [{:keys [ref layer on-done]}]
  (let [k (data-key (:kind layer))
        [text set-text!] (uix/use-state (if (= :basis k) (or (:basis layer) "proposed") (json (get layer k))))
        [note set-note!] (uix/use-state (or (:note layer) ""))
        {:keys [send! busy error]} (api/use-send)
        [value parse-error] (if (= :basis k) [text nil] (parse-json text))]
    ($ :form {:class "mt-2 grid gap-2"
              :on-submit (fn [e]
                           (.preventDefault e)
                           (-> (send! "PUT" (path ref "layers" (:id layer))
                                      (cond-> (-> (layer-body layer) (assoc k value) (dissoc :note))
                                        (seq note) (assoc :note note)))
                               (.then #(when % (on-done)))))}
       ($ ui/field {:label "Note"} ($ ui/text-input {:value note :on-change set-note! :name "note"}))
       (if (= :basis k)
         ($ ui/field {:label "Basis"}
            ($ ui/select {:value text :on-change set-text! :aria-label "Basis"
                          :options [["approved" "approved items"] ["proposed" "every item not set aside"]]}))
         ($ ui/field {:label (str (name k) ", as JSON")}
            ($ ui/text-area {:value text :on-change set-text! :rows 18 :name (name k) :aria-label (str (name k) " as JSON")})))
       (when parse-error ($ ui/notice {:tone :bad} parse-error))
       (when error ($ ui/notice {:tone :bad} error))
       ($ :div {:class "flex gap-2"}
          ($ ui/button {:tone :primary :type "submit" :disabled (or busy parse-error)} (if busy "Saving…" "Save layer"))
          ($ ui/button {:on-click on-done} "Cancel")))))

(defui layer-card [{:keys [ref s layer index readonly]}]
  (let [[editing set-editing!] (uix/use-state false)
        [asking set-asking!] (uix/use-state false)
        {:keys [send! busy error]} (api/use-send)
        {:keys [id kind enabled note]} layer
        move! (fn [to]
                (let [ls (:layers s)
                      moved (-> (vec (concat (subvec ls 0 index) (subvec ls (inc index))))
                                (as-> v (vec (concat (subvec v 0 to) [layer] (subvec v to)))))]
                  (send! "PUT" (path ref) (scenario-body (assoc s :layers moved)))))]
    ($ :div {:class (cx "border-l-2 px-3 py-2" (if enabled "border-ink-2" "border-line-1 opacity-70"))}
       ($ :div {:class "flex flex-wrap items-center gap-3"}
          ($ ui/mono {:class "font-medium text-ink-1"} id)
          ($ :span {:class "rounded-1 bg-surface-2 px-1.5 py-0.5 font-mono text-2xs text-ink-2"} kind)
          ($ ui/switch {:on enabled :label (if enabled "on" "off") :disabled (or readonly busy)
                        :on-change #(send! "PUT" (path ref "layers" id) (assoc (layer-body layer) :enabled %))})
          (when-not readonly
            ($ :span {:class "ml-auto flex gap-2"}
               (when (pos? index) ($ ui/button {:on-click #(move! (dec index)) :disabled busy :title "Apply it earlier"} "↑"))
               (when (< index (dec (count (:layers s))))
                 ($ ui/button {:on-click #(move! (inc index)) :disabled busy :title "Apply it later"} "↓"))
               ($ ui/button {:on-click #(set-editing! (not editing))} (if editing "Close" "Edit"))
               (if asking
                 ($ :<>
                    ($ ui/button {:tone :danger :disabled busy
                                  :on-click #(send! "PUT" (path ref) (scenario-body (update s :layers (partial filterv (fn [l] (not= id (:id l))))))) }
                       "Remove it")
                    ($ ui/button {:on-click #(set-asking! false)} "Keep"))
                 ($ ui/button {:tone :danger :on-click #(set-asking! true)} "Remove")))))
       ($ :div {:class "mt-1 text-xs text-ink-2"} (layer-summary layer))
       (when note ($ :div {:class "text-xs text-ink-3"} note))
       (when error ($ ui/notice {:tone :bad} error))
       (if editing
         ($ layer-editor {:ref ref :layer layer :on-done #(set-editing! false)})
         ($ ui/disclosure {:summary "Show as JSON"} ($ ui/code-block {:value (get layer (data-key kind))}))))))

(defui add-layer [{:keys [ref s on-done]}]
  (let [[id set-id!] (uix/use-state "")
        [kind set-kind!] (uix/use-state "coding")
        [text set-text!] (uix/use-state (json (templates "coding")))
        {:keys [send! busy error]} (api/use-send)
        close? (= "close" kind)
        [value parse-error] (if close? [text nil] (parse-json text))
        taken (some #(= id (:id %)) (:layers s))]
    ($ :form {:class "grid gap-2 p-3"
              :on-submit (fn [e]
                           (.preventDefault e)
                           (-> (send! "PUT" (path ref "layers" id) {:kind kind (data-key kind) value})
                               (.then #(when % (on-done)))))}
       ($ :div {:class "grid gap-3 md:grid-cols-[1fr_1fr_2fr]"}
          ($ ui/field {:label "Layer id"} ($ ui/text-input {:value id :on-change set-id! :placeholder "per-diem" :name "layer-id"}))
          ($ ui/field {:label "Kind"}
             ($ ui/select {:value kind :aria-label "Kind"
                           :on-change (fn [k] (set-kind! k) (set-text! (if (= "close" k) "proposed" (json (templates k)))))
                           :options (for [k ["world" "coding" "lines" "journals" "close"]] [k k])}))
          ($ ui/say {} "It applies after every layer above it, and after the layers of the scenario this one stands on."))
       (if close?
         ($ ui/field {:label "Basis"}
            ($ ui/select {:value text :on-change set-text! :aria-label "Basis"
                          :options [["approved" "approved items"] ["proposed" "every item not set aside"]]}))
         ($ ui/field {:label (str (name (data-key kind)) ", as JSON")}
            ($ ui/text-area {:value text :on-change set-text! :rows 14 :aria-label "Layer data as JSON"})))
       (when parse-error ($ ui/notice {:tone :bad} parse-error))
       (when taken ($ ui/notice {:tone :bad} (str "There is a layer " id " already: edit it instead.")))
       (when error ($ ui/notice {:tone :bad} error))
       ($ :div {:class "flex gap-2"}
          ($ ui/button {:tone :primary :type "submit"
                        :disabled (or busy parse-error taken (not (re-matches home/ref-pattern id)))}
             "Add layer")
          ($ ui/button {:on-click on-done} "Cancel")))))

(defui inherited
  "The layers of the scenario `ref` stands under, and of those under it, read only, nearest last."
  [{:keys [ref]}]
  (let [state (api/use-get (path ref))
        s (get-in state [:data :scenario])]
    ($ ui/loading {:state state}
       (when s
         ($ :<>
            (when (:base s) ($ inherited {:ref (:base s)}))
            ($ :div {:class "mt-2"}
               ($ ui/eyebrow {} "From " ($ :a {:href (route/href ["scenarios" ref]) :class "text-lapis"} ref)
                  " (change them there)")
               (for [[i l] (map-indexed vector (:layers s))]
                 ($ layer-card {:key (:id l) :ref ref :s s :layer l :index i :readonly true}))))))))

(defui layers-tab [{:keys [s]}]
  (let [[adding set-adding!] (uix/use-state false)
        readonly (:built-in s)]
    ($ :div
       (when (:base s)
         ($ :<>
            ($ ui/section-title {:aside "applied first, in this order"} "Under it")
            ($ inherited {:ref (:base s)})))
       ($ ui/section-title {:aside (when-not readonly
                                     ($ ui/button {:on-click #(set-adding! (not adding))} (if adding "Cancel" "Add a layer")))}
          "Its own layers, applied in this order")
       (when adding ($ ui/panel {:class "mb-3"} ($ add-layer {:ref (:ref s) :s s :on-done #(set-adding! false)})))
       (if (empty? (:layers s))
         ($ ui/notice {} "No layers of its own yet.")
         (for [[i l] (map-indexed vector (:layers s))]
           ($ layer-card {:key (:id l) :ref (:ref s) :s s :layer l :index i :readonly readonly}))))))

;; ----------------------------------------------------------------------------------------------- versions

(defui versions-tab [{:keys [s]}]
  (let [state (api/use-get (path (:ref s) "versions"))
        {:keys [send! busy error]} (api/use-send)
        versions (reverse (get-in state [:data :versions]))
        latest (:version (first versions))]
    ($ :div
       ($ ui/say {} "Every change to a scenario is kept as a version. Restoring one keeps it as the newest version: "
          "the versions after it stay, so a restore can be undone the same way.")
       (when error ($ ui/notice {:tone :bad} error))
       ($ ui/loading {:state state}
          ($ ui/table {:class "mt-3" :head ["Version" "Kept at" "Name" "Stands on" "Layers" ""]}
             (for [v versions]
               ($ :tr {:key (:version v)}
                  ($ ui/td {} ($ ui/mono {} (:version v)))
                  ($ ui/td {} ($ ui/mono {:class "text-ink-3"} (some-> (:at v) (subs 0 19))))
                  ($ ui/td {} (:name v))
                  ($ ui/td {} ($ ui/mono {} (or (:base v) "—")))
                  ($ ui/td {:class "text-xs"}
                     (for [l (:layers v)]
                       ($ :div {:key (:id l) :class (when-not (:enabled l) "text-ink-3 line-through")}
                          ($ ui/mono {} (:id l)) " " (layer-summary l)))
                     ($ ui/disclosure {:summary "as JSON"} ($ ui/code-block {:value (scenario-body v)})))
                  ($ ui/td {}
                     (if (= latest (:version v))
                       ($ ui/muted {:class "text-xs"} "as it stands")
                       (when-not (:built-in s)
                         ($ ui/button {:disabled busy :on-click #(send! "PUT" (path (:ref s)) (scenario-body v))}
                            "Restore")))))))))))

;; ----------------------------------------------------------------------------------------------- books

(defui books-tab [{:keys [s query]}]
  (let [[book set-book!] (uix/use-state (or (:book query) ""))
        [from set-from!] (uix/use-state (or (:from query) ""))
        [to set-to!] (uix/use-state (or (:to query) ""))
        asked (:book query)
        state (api/use-get (when (seq asked)
                             (str (path (:ref s) "evaluation") "?book=" (js/encodeURIComponent asked)
                                  (when (seq (:from query)) (str "&from=" (:from query)))
                                  (when (seq (:to query)) (str "&to=" (:to query))))))
        {:keys [layers balances]} (get-in state [:data :evaluation])]
    ($ :div
       ($ ui/say {} "What the scenario's journals and close layers add to one book, account by account (debits "
          "positive), and what each layer adds. Coding layers propose codes for bank lines; they add to no book yet.")
       ($ :form {:class "mt-3 flex flex-wrap items-end gap-3"
                 :on-submit (fn [e] (.preventDefault e)
                              (route/go! ["scenarios" (:ref s)] {:tab "books" :book book :from (not-empty from) :to (not-empty to)}))}
          ($ ui/field {:label "Book (an org's id)"} ($ ui/text-input {:value book :on-change set-book! :name "book" :class "w-72"}))
          ($ ui/field {:label "From"} ($ ui/text-input {:value from :on-change set-from! :placeholder "2025-01-01" :name "from"}))
          ($ ui/field {:label "To"} ($ ui/text-input {:value to :on-change set-to! :placeholder "2025-12-31" :name "to"}))
          ($ ui/button {:tone :primary :type "submit" :disabled (str/blank? book)} "Evaluate"))
       (when (seq asked)
         ($ ui/loading {:state state}
            ($ :<>
               ($ ui/section-title {} "What it adds")
               (if (empty? balances)
                 ($ ui/notice {} "Nothing: no layer of it adds to this book over the period.")
                 ($ ui/table {:head ["Account" "Amount"]}
                    (for [[a amt] (sort balances)]
                      ($ :tr {:key a} ($ ui/td {} ($ ui/mono {} a)) ($ ui/td {:class "text-right"} ($ ui/mono {} amt))))))
               ($ ui/section-title {} "Layer by layer")
               ($ ui/table {:head ["Scenario" "Layer" "Kind" "On" "Accounts it moves"]}
                  (for [{:keys [scenario layer kind enabled] :as l} layers]
                    ($ :tr {:key (str scenario "/" layer)}
                       ($ ui/td {} ($ ui/mono {} scenario)) ($ ui/td {} ($ ui/mono {} layer)) ($ ui/td {} kind)
                       ($ ui/td {} (if enabled "on" "off"))
                       ($ ui/td {:class "text-xs"} (count (:balances l))))))))))))

;; ----------------------------------------------------------------------------------------------- the page

(def tab-items
  [["layers" "Layers"] ["rules" "Rules and world"] ["coding" "Coding"] ["compare" "Compare"] ["books" "Books"]
   ["versions" "Versions"]])

(defui page [{:keys [ref query]}]
  (let [state (api/use-get (path ref))
        all (api/use-get "/scenarios")
        s (get-in state [:data :scenario])
        scenarios (get-in all [:data :scenarios])
        tab (or (:tab query) "layers")]
    ($ ui/loading {:state state}
       (when s
         ($ :div
            ($ header {:s s :scenarios scenarios})
            ($ ui/tabs {:current tab
                        :items (for [[id label] tab-items] [id label (route/href ["scenarios" ref] {:tab id})])})
            ($ :div {:class "mt-3"}
               (case tab
                 "layers" ($ layers-tab {:s s})
                 "rules" ($ rules/tab {:s s})
                 "coding" ($ coding/tab {:s s :query query})
                 "compare" ($ coding/compare-tab {:s s :scenarios scenarios :query query})
                 "books" ($ books-tab {:s s :query query})
                 "versions" ($ versions-tab {:s s})
                 ($ ui/notice {} "No such tab."))))))))
