(ns ownpurse.admin.scenarios.world
  "A scenario's world, as its layers say it, and its editors: the projects, the trips, who is who, the cards, home and
  the business. Each entry is shown in its place, on or off (greyed), with the layer it comes from. A change, an
  addition or a switch is written into one of the scenario's own world layers, so the scenario under it stays as it
  was: a project, card or entity amended field by field, a trip replaced by one of its id."
  (:require
   [clojure.string :as str]
   [ownpurse.admin.api :as api]
   [ownpurse.admin.scenarios.common :refer [layer-body path]]
   [ownpurse.admin.ui :as ui :refer [cx]]
   [uix.core :as uix :refer [$ defui]]))

(defn- n [x] (if (keyword? x) (name x) (str x)))

(defn- slug [s] (-> (str/lower-case (or s "")) (str/replace #"[^a-z0-9]+" "-") (str/replace #"^-|-$" "")))

;; ----------------------------------------------------------------------------------------------- where it comes from

(defn where-text
  "A layer as an entry's provenance says it: \"here, layer changes\" for this scenario's own, else the scenario and
  layer."
  [s {:keys [scenario layer]}]
  (if (= scenario (:ref s)) (str "here, layer " layer) (str scenario ", layer " layer)))

(defui where
  "Where an entry comes from, on two lines: the scenario (this one, or a link to the one under it), then its layer."
  [{:keys [s at tab]}]
  (let [{:keys [scenario layer]} at]
    ($ :div {}
       (if (= scenario (:ref s))
         ($ :span {:class "text-ink-2"} "here")
         ($ :a {:class "font-mono text-lapis hover:underline" :href (str "#/scenarios/" scenario "?tab=" (or tab "rules"))}
            scenario))
       ($ :div {:class "font-mono text-2xs"} "layer " layer))))

(defui provenance
  "Where a ledger entry (a rule, a project, a trip) was said, which layers changed it, and where it was turned off."
  [{:keys [s entry]}]
  (let [{:keys [from changed-in off-in]} entry]
    ($ :<>
       (when from ($ where {:s s :at from}))
       (when (seq changed-in) ($ :div {} "changed in " (str/join "; " (map #(where-text s %) changed-in))))
       (when off-in ($ :div {:class "text-ink-2"} "turned off " (if (= (:scenario off-in) (:ref s))
                                                                (str "here, layer " (:layer off-in))
                                                                (str "in " (where-text s off-in))))))))

;; ----------------------------------------------------------------------------------------------- writing

(defn- own-world-layers [s] (filterv #(= "world" (:kind %)) (:layers s)))

(defn- default-layer
  "Where a change goes unless another is picked: the scenario's last world layer that is on, else a new one."
  [s]
  (or (:id (last (filter :enabled (own-world-layers s)))) "world-changes"))

(defn- write-world!
  "Writes `f` of world layer `layer-id`'s data (a new world layer of that id, when there is none) into scenario `s`."
  [send! s layer-id f]
  (let [layer (or (some #(when (= layer-id (:id %)) %) (:layers s))
                  {:kind "world" :enabled true
                   :note (str "What this scenario changes in the world of " (or (:base s) "its own layers") ".")})]
    (send! "PUT" (path (:ref s) "layers" layer-id) (assoc (layer-body layer) :world (f (or (:world layer) {}))))))

(defn trip-id
  "A trip's id, as ownpurse.scenario.coding/trip-id makes it: its :id, else whose it is and the day it starts."
  [{:keys [id person from]}]
  (or id (str person " " from)))

(defn- put-trip
  "`trips` with trip `t` in the place of the one of id `id`, or added."
  [trips id t]
  (let [trips (vec trips)
        at (first (keep-indexed #(when (= id (trip-id %2)) %1) trips))]
    (if at (assoc trips at t) (conj trips t))))

(defn- mine [layer-world id] (some #(when (= id (trip-id %)) %) (:trips layer-world)))

(defn- keep-id
  "Trip `v` as it is written for trip `id`: with that id, when its person or start no longer make it."
  [v id]
  (let [v (dissoc v :id :enabled)]
    (cond-> v (not= id (trip-id v)) (assoc :id id))))

(defn- turn-trip
  "Trip ledger entry `e` turned on or off, in world layer data `w` of layer `layer` of scenario `s`."
  [s layer {:keys [id value off-in]} on w]
  (let [here (mine w id)]
    (update w :trips
            (fn [trips]
              (cond
                (not on) (put-trip trips id (if here (assoc here :enabled false) {:id id :enabled false}))
                (and here (:person here) (= off-in {:scenario (:ref s) :layer layer})) (put-trip trips id (dissoc here :enabled))
                ;; Turned off here by its id alone: taking that out lets the trip under it stand again.
                (and here (= off-in {:scenario (:ref s) :layer layer})) (vec (remove #(= id (trip-id %)) trips))
                ;; Turned off under this scenario: said again here, on.
                :else (put-trip trips id (keep-id value id)))))))

;; ----------------------------------------------------------------------------------------------- forms

(defui date-input [{:keys [value on-change aria-label]}]
  ($ :input {:type "date" :value (or value "") :aria-label aria-label :on-change #(on-change (.. % -target -value))
             :class "rounded-1 border border-line-2 bg-surface-0 px-2 py-1 text-sm text-ink-1"}))

(defui entry-form
  "Fields for an entry, from `spec` ([key label kind options]: kind :text, :date, :select or :many), started from
  `value`. `on-save` gets the entry as the fields say it and returns a promise of the answer (nil when it failed);
  `check` of the entry says what is missing or wrong, if anything, and Save waits for it."
  [{:keys [spec value on-save on-done busy error check]}]
  (let [[d set-d!] (uix/use-state (or value {}))
        problem (when check (check d))
        put (fn [k] #(set-d! (fn [d] (if (str/blank? (str %)) (dissoc d k) (assoc d k %)))))]
    ($ :form {:class "grid gap-3 p-3"
              :on-submit (fn [e] (.preventDefault e) (-> (on-save d) (.then #(when % (on-done)))))}
       ($ :div {:class "grid gap-3 sm:grid-cols-2 lg:grid-cols-3"}
          (for [[k label kind options] spec]
            ($ ui/field {:key (name k) :label label :class (when (#{:many} kind) "sm:col-span-2 lg:col-span-3")}
               (case kind
                 :date ($ date-input {:value (get d k) :on-change (put k) :aria-label label})
                 :select ($ ui/select {:value (n (get d k "")) :on-change (put k) :options options :aria-label label})
                 :many ($ :div {:class "flex flex-wrap gap-x-4 gap-y-1"}
                          (for [[v l] options
                                :let [picked (set (get d k))]]
                            ($ :label {:key v :class "flex items-baseline gap-1.5 text-sm"}
                               ($ :input {:type "checkbox" :checked (contains? picked v)
                                          :on-change #(set-d! (fn [d] (let [p (set (get d k))
                                                                            p (if (contains? p v) (disj p v) (conj p v))]
                                                                        (if (seq p) (assoc d k (vec (sort p))) (dissoc d k)))))})
                               l)))
                 ($ ui/text-input {:value (get d k) :on-change (put k) :aria-label label})))))
       (when (or problem error) ($ ui/notice {:tone (if error :bad :neutral)} (or error problem)))
       ($ :div {:class "flex gap-2"}
          ($ ui/button {:tone :primary :type "submit" :disabled (or busy problem)} (if busy "Saving…" "Save"))
          ($ ui/button {:on-click on-done} "Cancel")))))

;; ----------------------------------------------------------------------------------------------- one kind of entry

(defui entry-rows
  "One ledger entry as a row: a switch, its cells (`cells` of its value), where it comes from and a Change button;
  changing it opens `spec`'s form under it."
  [{:keys [s entry cells spec check on-toggle on-save editable cols]}]
  (let [[editing set-editing!] (uix/use-state false)
        {:keys [send! busy error]} (api/use-send)
        {:keys [on value]} entry]
    ($ :<>
       ($ :tr {:class (cx "align-top" (if on "hover:bg-surface-1" "bg-surface-1/60 text-ink-3"))}
          ($ ui/td {:class "whitespace-nowrap"}
             ($ ui/switch {:on on :label (if on "on" "off") :disabled (or busy (not editable)) :on-change #(on-toggle send! %)}))
          (for [[i c] (map-indexed vector (cells entry))]
            ($ ui/td {:key i :class (when (and (zero? i) on) "font-medium")} c))
          ($ ui/td {:class "min-w-[10rem] text-xs text-ink-3"} ($ provenance {:s s :entry entry}))
          ($ ui/td {:class "whitespace-nowrap text-right"}
             (when editable ($ ui/button {:on-click #(set-editing! (not editing))} (if editing "Close" "Change")))))
       (when (and error (not editing)) ($ :tr {} ($ ui/td {:col-span cols} ($ ui/notice {:tone :bad} error))))
       (when editing
         ($ :tr {} ($ ui/td {:col-span cols :class "bg-surface-1"}
                      ($ entry-form {:spec spec :value value :busy busy :error error :check check
                                     :on-save #(on-save send! %) :on-done #(set-editing! false)})))))))

(defui adder
  "A button that opens `spec`'s form for a new entry."
  [{:keys [label spec value on-save check]}]
  (let [[open set-open!] (uix/use-state false)
        {:keys [send! busy error]} (api/use-send)]
    (if open
      ($ ui/panel {:class "mt-2"}
         ($ entry-form {:spec spec :value value :busy busy :error error :check check
                        :on-save #(on-save send! %) :on-done #(set-open! false)}))
      ($ :div {:class "mt-2"} ($ ui/button {:on-click #(set-open! true)} label)))))

(defui kind-table
  "A kind of entry (projects, trips, …) as a table in standing order, with an adder when it can be changed."
  [{:keys [s title aside head entries cells spec add-spec check add-check toggle! save! add! new-value editable add-label]}]
  ($ :<>
     ($ ui/eyebrow {:class "mt-5 mb-1 flex items-baseline justify-between gap-3"}
        ($ :span {} title) (when aside ($ :span {:class "normal-case tracking-normal text-ink-3"} aside)))
     (if (seq entries)
       ($ ui/table {:head (concat [""] head ["From" ""])}
          (for [e entries]
            ($ entry-rows {:key (:id e) :s s :entry e :cells cells :spec spec :check check :editable editable
                           :cols (+ 3 (count head))
                           :on-toggle (fn [send! on] (toggle! send! e on))
                           :on-save (fn [send! v] (save! send! e v))})))
       ($ ui/muted {:class "text-sm"} "None said."))
     (when editable ($ adder {:label add-label :spec (or add-spec spec) :value new-value :on-save add!
                              :check #(or (when add-check (add-check %)) (when check (check %)))}))))

;; ----------------------------------------------------------------------------------------------- the section

(defn- per-diem-text [p] (case (:per-diem p) true "pays one" false "none" ($ ui/muted {} "not said")))

(defn- from-select [v]
  (case v "true" true "false" false nil))

(defui section
  "The world: home and the business, the projects, the trips, who is who and the cards, each editable in place."
  [{:keys [s ledger]}]
  (let [editable (not (or (:built-in s) api/static?))
        [layer set-layer!] (uix/use-state (default-layer s))
        own (own-world-layers s)
        {:keys [projects trips entities cards home business]} ledger
        names (into {} (for [{:keys [id value]} entities] [id (or (:name value) id)]))
        who (fn [id] (get names id id))
        people (for [{:keys [id value on]} entities :when on] [id (or (:name value) id)])
        project-ids (for [{:keys [id on]} projects :when on] id)
        write! (fn [send! f] (write-world! send! s layer f))
        ;; A project, card or entity: amended field by field in the layer written to; on and off as :enabled.
        keyed-toggle (fn [k] (fn [send! {:keys [id]} on] (write! send! #(assoc-in % [k id :enabled] on))))
        keyed-save (fn [k clean] (fn [send! {:keys [id]} v]
                                   (write! send! (fn [w] (update-in w [k id] #(merge (dissoc % :per-diem) (clean v)))))))
        keyed-add (fn [k id-of clean] (fn [send! v] (write! send! #(assoc-in % [k (id-of v)] (clean (dissoc v :id))))))
        taken (fn [entries id-of what]
                (let [ids (set (map :id entries))]
                  (fn [v] (let [id (id-of v)]
                            (cond (str/blank? id) (str "Give it " what ".")
                                  (ids id) (str (pr-str id) " is taken: change that one, or pick another.")
                                  :else nil)))))
        needs (fn [& ks] (fn [v] (when-let [[_ label] (first (filter (fn [[k]] (str/blank? (n (get v k "")))) ks))]
                                   (str "Say " label "."))))
        name-id (comp slug :name)
        project-clean (fn [v] (let [v (dissoc v :enabled)]
                                (if-some [pd (from-select (n (:per-diem v "")))] (assoc v :per-diem pd) (dissoc v :per-diem))))
        project-spec [[:name "Name" :text] [:from "From" :date] [:to "To" :date] [:place "Where" :text]
                      [:bears "Its costs are borne by" :select (cons ["" "not said"] people)]
                      [:per-diem "Per diem" :select [["" "not said"] ["true" "pays one"] ["false" "pays none"]]]
                      [:work "The work (Jev reads this)" :text]
                      [:travellers "Who travels for it" :many people]]
        trip-spec [[:person "Who" :select (cons ["" "pick one"] people)] [:place "Country" :text] [:city "City" :text]
                   [:from "From" :date] [:to "To" :date]
                   [:project "For" :select (cons ["" "no project (personal)"] (for [id project-ids] [id id]))]]
        entity-spec [[:name "Name" :text] [:kind "Kind" :select [["household" "a person, a household"] ["business" "a business"]
                                                                 ["non-profit" "a non-profit"]]]]
        card-spec [[:name "Name" :text] [:holder "Held by" :select (cons ["" "pick one"] people)]
                   [:entity "Whose card (pays for)" :select (cons ["" "pick one"] (for [{:keys [id value]} entities] [id (or (:name value) id)]))]]
        {:keys [send! busy error]} (api/use-send)
        [h set-h!] (uix/use-state nil)
        [picking set-picking!] (uix/use-state false)]
    ($ :<>
       ($ ui/section-title {:aside "Who, where and when: what the classifier knows about a line before it asks anything"}
          "The world")
       ($ :p {:class "mb-2 max-w-4xl text-sm text-ink-2"}
          "Every project, trip, person and card its layers say, in their place; one turned off stays, greyed, and the "
          "classifier reads as if it were not there. A change, an addition or a switch is written into this "
          "scenario's own world layer, so the scenario it stands on is not changed.")
       (when editable
         ($ :div {:class "mb-2 flex flex-wrap items-end gap-3"}
            ($ ui/field {:label "Changes are written into"}
               ($ ui/select {:value layer :on-change set-layer! :aria-label "Changes are written into"
                             :options (concat (for [{:keys [id enabled]} own]
                                                [id (str "its world layer " id (when-not enabled " (off: changes will not show)"))])
                                              (when-not (some #(= "world-changes" (:id %)) own)
                                                [["world-changes" "a new world layer, world-changes"]]))}))))

       ($ ui/eyebrow {:class "mt-4 mb-1"} "Home, and the business")
       ($ :div {:class "grid gap-1 text-sm"}
          ($ :div {}
             "On no trip, people are at home in "
             ($ :span {:class "font-medium"} (let [v (:value home)] (if (map? v) (str/join ", " (remove str/blank? [(:city v) (:country v)])) (or v "— not said"))))
             (when (:from home) ($ ui/muted {:class "text-xs"} " · " (where-text s (:from home))))
             (when (and editable (not h))
               ($ :button {:class "ml-2 text-xs text-lapis hover:underline"
                           :on-click #(set-h! (let [v (:value home)] (if (map? v) v {:country v})))} "change")))
          (when h
            ($ :form {:class "flex flex-wrap items-end gap-3"
                      :on-submit (fn [e] (.preventDefault e)
                                   (-> (write! send! #(assoc % :home (into {} (remove (comp str/blank? val)) h)))
                                       (.then #(when % (set-h! nil)))))}
               ($ ui/field {:label "Country"} ($ ui/text-input {:value (:country h) :on-change #(set-h! (assoc h :country %))}))
               ($ ui/field {:label "City"} ($ ui/text-input {:value (:city h) :on-change #(set-h! (assoc h :city %))}))
               ($ ui/button {:tone :primary :type "submit" :disabled (or busy (str/blank? (:country h)))} "Save")
               ($ ui/button {:on-click #(set-h! nil)} "Cancel")))
          ($ :div {}
             "The business is " ($ :span {:class "font-medium"} (if business (who (:value business)) "— not said"))
             ": its costs are the business's, those of a project the one that bears it."
             (when (:from business) ($ ui/muted {:class "text-xs"} " · " (where-text s (:from business))))
             (when editable
               (if picking
                 ($ :span {:class "ml-2 inline-flex items-center gap-2"}
                    ($ ui/select {:value (or (:value business) "") :aria-label "The business"
                                  :options (cons ["" "pick one"] (for [{:keys [id value]} entities] [id (or (:name value) id)]))
                                  :on-change #(when (seq %) (-> (write! send! (fn [w] (assoc w :business %)))
                                                                (.then (fn [ok] (when ok (set-picking! false))))))})
                    ($ ui/button {:on-click #(set-picking! false)} "Cancel"))
                 ($ :button {:class "ml-2 text-xs text-lapis hover:underline" :on-click #(set-picking! true)} "change"))))
          (when error ($ ui/notice {:tone :bad} error)))

       ($ kind-table
          {:s s :title (ui/count-of (filter :on projects) "project")
           :aside "Work that ran over some days: a line in its days, or of a traveller on a trip for it, may be its cost"
           :head ["Project" "Name" "Days" "Where" "Borne by" "Travellers" "Per diem"]
           :entries projects :editable editable :add-label "Add a project"
           :cells (fn [{id :id p :value}] [($ ui/mono {} id) (:name p)
                           ($ ui/mono {:class "text-xs"} (:from p) " – " (:to p)) (:place p) (some-> (:bears p) who)
                           (str/join ", " (map who (:travellers p))) (per-diem-text p)])
           :spec project-spec :check (needs [:name "its name"] [:from "the day it starts"] [:to "the day it ends"])
           :add-check (taken projects name-id "a name")
           :toggle! (keyed-toggle :projects) :save! (keyed-save :projects project-clean)
           :add! (keyed-add :projects name-id project-clean)})

       ($ kind-table
          {:s s :title (ui/count-of (filter :on trips) "trip")
           :aside "Where someone was, and for which project; any other day they are at home"
           :head ["Who" "Where" "Days" "For"]
           :entries (sort-by (comp :from :value) trips) :editable editable :add-label "Add a trip"
           :cells (fn [{t :value}] [(who (:person t)) (str/join ", " (remove str/blank? [(:city t) (:place t)]))
                           ($ ui/mono {:class "whitespace-nowrap text-xs"} (:from t) " – " (:to t))
                           (cond
                             (not (:project t)) ($ ui/muted {} "personal")
                             ((set project-ids) (:project t)) ($ ui/mono {} (:project t))
                             :else ($ :span {} ($ ui/mono {} (:project t))
                                      ($ :div {:class "text-2xs text-gilt-text"} "a project that is off: read as personal")))])
           :spec trip-spec :check (needs [:person "whose trip it is"] [:place "the country"] [:from "the day it starts"]
                                         [:to "the day it ends"])
           :add-check (fn [v] (when (some #(= (trip-id v) (:id %)) trips)
                                (str "There is a trip of " (who (:person v)) " starting " (:from v) " already: change that one.")))
           :toggle! (fn [send! e on] (write! send! (partial turn-trip s layer e on)))
           :save! (fn [send! e v] (write! send! (fn [w] (update w :trips #(put-trip % (:id e) (cond-> (keep-id v (:id e))
                                                                                                     (not (:on e)) (assoc :enabled false)))))))
           :add! (fn [send! v] (write! send! (fn [w] (update w :trips #(put-trip % (trip-id v) v)))))})

       ($ kind-table
          {:s s :title "Who is who" :aside "The people and businesses lines are paid by and for"
           :head ["Id" "Name" "Kind"] :entries entities :editable editable :add-label "Add a person or business"
           :new-value {:kind "household"}
           :cells (fn [{id :id {nm :name kind :kind} :value}] [($ ui/mono {} id) nm (some-> kind n)])
           :spec entity-spec :check (needs [:name "the name"]) :add-check (taken entities name-id "a name")
           :toggle! (keyed-toggle :entities) :save! (keyed-save :entities #(dissoc % :enabled))
           :add! (keyed-add :entities name-id #(dissoc % :enabled))})

       ($ kind-table
          {:s s :title (ui/count-of (filter :on cards) "card")
           :aside "Whose card a line is on: who held it, and whom it pays for"
           :head ["Card" "Id" "Held by" "Pays for"] :entries cards :editable editable :add-label "Add a card"
           :cells (fn [{id :id c :value}] [(:name c) ($ ui/mono {:class "text-xs"} id) (some-> (:holder c) who) (some-> (:entity c) who)])
           :spec card-spec :add-spec (into [[:id "Its id, as the lines name the card or account" :text]] card-spec)
           :check (needs [:name "the name"] [:holder "who holds it"] [:entity "whom it pays for"])
           :add-check (taken cards (comp str/trim n :id) "the id its lines name it by")
           :toggle! (keyed-toggle :cards) :save! (keyed-save :cards #(dissoc % :enabled))
           :add! (keyed-add :cards (comp str/trim n :id) #(dissoc % :enabled))}))))
