(ns ownpurse.admin.docs.guide
  "How a bank line becomes a coding, in ten steps, told for someone new to all of it: each step says in plain words
  what happens and who does it, beside what it did to one real line of the sandbox (the reader picks which). The
  line's data come from the API, the same as the sandbox's, so the guide shows the pipeline as it runs today."
  (:require
   [clojure.string :as str]
   [ownpurse.admin.api :as api]
   [ownpurse.admin.route :as route]
   [ownpurse.admin.sandbox.line :as line]
   [ownpurse.admin.sandbox.run :as run]
   [ownpurse.admin.terms :refer [term]]
   [ownpurse.admin.ui :as ui :refer [cx]]
   [ownpurse.classify.outcome :as outcome]
   [uix.core :as uix :refer [$ defui]]))

(def sample "The invented scenario the guide follows when the record has none coded." "atlas")

(def examples
  "Lines of the sample worth following, each for what it shows."
  [{:id "d211" :title "A lunch in Lisbon" :why "Per diem, or hospitality? Only the note can tell."}
   {:id "d138" :title "A crane hire" :why "The bank sent no merchant code, so Jev has to guess the kind of shop."}
   {:id "d88/1" :title "A hotel room" :why "A receipt splits one payment into items, each coded on its own."}
   {:id "d109" :title "A hotel's refund" :why "Money back follows the purchase it returns."}
   {:id "d57" :title "A grill, far from home" :why "A check rule stops the line, however sure Jev is."}
   {:id "d217" :title "A cheeky note" :why "The note tells us how to code it. Should we listen?"}])

(defn record-examples
  "Lines of a record scenario's run worth following, each for what it shows, picked by what happened to them: the
  first line of each kind, and no line twice."
  [lines]
  (let [d #(get-in %1 [:decisions %2])
        person? #(= :needs-person (:code %))
        kinds [[#(and (= :per-diem (:code (d % :checked))) (get-in % [:unit :currency])
                      (not= "USD" (get-in % [:unit :currency])))
                "Code knows where the cardholder was that day, and Jev reads the trip."]
               [#(and (neg? (get-in % [:unit :amount] 0)) (not= :transfer (get-in % [:unit :channel])))
                "Money coming back, not going out."]
               [#(and (nil? (get-in % [:unit :mcc])) (not= :transfer (get-in % [:unit :channel])))
                "The bank sent no merchant code, so Jev guesses the kind of shop first."]
               [#(and (person? (d % :checked)) (not (person? (d % :choice))))
                "Jev picked a bucket, but a check rule stops the line: a person looks."]
               [#(and (not (person? (d % :checked))) (< (:confidence (d % :checked) 1) 0.85))
                "Jev is not sure enough, so the line goes to a person."]
               [#(and (not= :per-diem (:code (d % :checked))) (>= (:confidence (d % :checked) 0) 0.95))
                "A sure coding, applied without anyone looking."]]]
    (->> kinds
         (reduce (fn [[picked seen] [pred why]]
                   (if-let [l (first (filter #(and (pred %) (not (seen (:id %)))) lines))]
                     [(conj picked {:id (:id l) :short (:short l) :title (get-in l [:unit :merchant]) :why why})
                      (conj seen (:id l))]
                     [picked seen]))
                 [[] #{}])
         first)))

(def actors
  {:bank {:name "The bank" :glyph "▤" :about "Sends the line. Knows what was paid, never what for."}
   :code {:name "Code" :glyph "{ }" :about "A careful clerk: looks things up and calculates, never guesses."}
   :jev {:name "Jev" :glyph "◆" :about "A fast junior bookkeeper: judges, and says how sure it is."}
   :person {:name "A person" :glyph "●" :about "You: decides what the others cannot, and writes the rules."}})

(def steps
  [{:n 1 :actor :bank :short "Arrives" :title "A line arrives from the bank"}
   {:n 2 :actor :code :short "Split" :title "Split into items, when there's a receipt"}
   {:n 3 :actor :code :short "Facts" :title "Code works out the facts"}
   {:n 4 :actor :jev :short "Kind of shop" :title "If the kind of shop is unknown, Jev guesses it"}
   {:n 5 :actor :code :short "In words" :title "The facts are written out in words"}
   {:n 6 :actor :jev :short "Asked" :title "Jev is asked its questions"}
   {:n 7 :actor :jev :short "Answers" :title "Jev answers, with how sure it is"}
   {:n 8 :actor :code :short "Rules" :title "The rules are checked, block by block"}
   {:n 9 :actor :code :short "Strategies" :title "Each strategy makes its pick"}
   {:n 10 :actor :person :short "Code or ask" :title "Code it, or ask a person"}])

(defn- yes? [p] (>= (or p 0) 0.5))

;; ----------------------------------------------------------------------------------------------- pieces

(defui actor-chip [{:keys [actor]}]
  (let [{:keys [name glyph]} (actors actor)]
    ($ :span {:class "inline-flex items-center gap-1.5 rounded-full border border-line-2 px-2.5 py-0.5 text-xs"}
       ($ :span {:class "font-mono text-ink-3"} glyph) name)))

(defui p [{:keys [children]}]
  ($ :p {:class "mt-3 text-base leading-relaxed text-ink-1"} children))

(defui aside [{:keys [label children]}]
  ($ :div {:class "mt-4 border-l-2 border-lapis bg-lapis-tint px-3 py-2 text-sm leading-relaxed"}
     (when label ($ ui/eyebrow {:class "mb-1 text-lapis"} label))
     children))

(defui dev-note
  "Where a step lives in the code, for the developer reading along; with a record, the step's call on this line:
  what went in, what came out, and every query it made."
  [{:keys [children step]}]
  ($ ui/disclosure {:summary (if step "For developers: the call, what went in and came out, and the database"
                                 "For developers: where this happens")
                    :class "mt-5"}
     ($ :div {:class "font-mono text-xs leading-relaxed text-ink-2"} children)
     (when-let [{:keys [call about in out db]} step]
       ($ :div {:class "mt-3 grid gap-3"}
          ($ :div
             ($ ui/eyebrow {} "The call, on this line")
             ($ :code {:class "mt-1 block rounded-1 bg-surface-2 px-2 py-1 font-mono text-xs"} call)
             ($ :p {:class "mt-1.5 max-w-[80ch] text-sm leading-relaxed text-ink-2"} about))
          ($ :div {:class "grid gap-3 lg:grid-cols-2"}
             ($ :div
                ($ ui/eyebrow {} "In")
                (for [[k v] in]
                  ($ :div {:key (name k) :class "mt-1"}
                     ($ ui/mono {:class "text-2xs text-ink-3"} (name k))
                     ($ ui/code-block {:value v}))))
             ($ :div
                ($ ui/eyebrow {} "Out")
                ($ ui/code-block {:value out})))
          ($ :div
             ($ ui/eyebrow {} (if (seq db) (str "The database: " (count db) (if (= 1 (count db)) " query" " queries"))
                                  "The database: no query; this step only calculates"))
             (for [[i {:keys [sql params rows row shown]}] (map-indexed vector db)]
               ($ :div {:key i :class "mt-2 border-l-2 border-line-2 pl-2"}
                  ($ :code {:class "block font-mono text-xs text-ink-1"} sql)
                  ($ ui/mono {:class "text-2xs text-ink-3"}
                     "params " (js/JSON.stringify (clj->js params)) " → " rows (if (= 1 rows) " row" " rows")
                     (when shown (str "; " shown)))
                  (when row
                    ($ ui/disclosure {:summary "the row"} ($ ui/code-block {:value row}))))))))))

(defui kv
  "Rows of label and value; a value may be anything renderable."
  [{:keys [rows]}]
  ($ :dl {:class "grid grid-cols-[minmax(110px,170px)_minmax(0,1fr)] gap-x-4 text-sm"}
     (for [[i [k v how]] (map-indexed vector rows) :when (some? v)]
       ($ :<> {:key i}
          ($ :dt {:class "border-t border-line-1 py-1.5 text-ink-3"} k)
          ($ :dd {:class "border-t border-line-1 py-1.5"} v
             (when how ($ :span {:class "mt-0.5 block font-mono text-2xs text-ink-3"} how)))))))

(defui panel [{:keys [title children]}]
  ($ :div {:class "border border-line-1 bg-surface-1 p-4"}
     ($ ui/eyebrow {:class "mb-2.5"} title)
     children))

(defn- where-text [{:keys [projects]} {:keys [place city trip project day days]}]
  (let [at (if city (str city ", " place) place)]
    (case trip
      :project (str "in " at ", on a trip for " (get-in projects [project :name]) " (day " day " of " days ")")
      :personal (str "in " at ", on a personal trip")
      (str "at home in " at))))

(defn- fuzzy-text [instructions]
  (first (str/split instructions #" Judge from the facts")))

(def ^:private reason-words
  {:check "a check rule applies" :conflict "rules that apply disagree" :no-rule "no rule applies"})

;; ----------------------------------------------------------------------------------------------- the steps

(defui step-1 [{:keys [data]}]
  (let [{:keys [line]} data
        {:keys [date merchant amount currency card channel mcc country city note]} line
        world (ui/use-world)]
    ($ :<>
       ($ :div
          ($ p {} "Everything starts with one row of a bank or card statement: a " ($ term {:id :bank-line}) ". "
             "It is short. It says when, who was paid, how much, with which card, and a four-digit "
             ($ term {:id :merchant-code}) " for the kind of shop.")
          ($ p {} "What it never says is " ($ :b {} "what the money was for") ". A restaurant bill could be a "
             "traveller's dinner, a lunch with a client, or a personal treat. Each of those belongs in a different place "
             "in the books. Deciding which is called " ($ term {:id :coding}) ", and it is what the rest of this guide is about.")
          (if (:record world)
            ($ aside {:label "Your own books"}
               "This is a real line, from the record's bank lines, as the " ($ term {:id :scenario}) " "
               ($ :b {} (:name world)) " reads it. The scenario adds what the bank does not know: who holds each card, "
               "who was where and when, the projects, and the rules. Change those on the scenario's page and code again.")
            ($ aside {:label "In the sandbox"}
               "These lines are invented, for testing: a made-up studio with two projects, Harbour in Lisbon and Skyline in "
               "New York. That is a " ($ term {:id :scenario}) ". With a record, this guide follows its own lines instead.")))
       ($ panel {:title "Your line, as the bank sends it"}
          ($ :div {:class "border border-dashed border-line-2 bg-surface-0 p-4 font-mono"}
             ($ :div {:class "flex items-baseline justify-between gap-3 text-xs text-ink-3"}
                ($ :span {} date) ($ :span {} (some-> channel name str/upper-case)))
             ($ :div {:class "mt-2 flex items-baseline justify-between gap-3"}
                ($ :span {:class "text-lg font-semibold"} merchant)
                ($ ui/amount {:amount amount :currency currency}))
             (when note ($ :div {:class "mt-1 text-sm text-ink-2"} "“" note "”"))
             ($ :div {:class "mt-3 grid grid-cols-2 gap-x-4 gap-y-1 text-xs text-ink-2"}
                ($ :span {} "card " card)
                ($ :span {} (if mcc (str "merchant code " mcc) "no merchant code"))
                (when (or city country) ($ :span {} (str/join ", " (remove nil? [city country]))))))
          ($ :p {:class "mt-3 text-sm text-ink-2"}
             "That's all. Nothing here says whose card " card " is, where they were, or why they paid.")
          (when-let [c (get-in world [:cards card :name])]
            ($ :p {:class "mt-1 text-xs text-ink-3"} "(The scenario knows: " c ". Step 3 looks that up.)"))))))

(defui step-2 [{:keys [data]}]
  (let [{:keys [unit line]} data
        items (:items line)]
    ($ :<>
       ($ :div
          ($ p {} "One payment can hold several things. A hotel bill has the room, the bar and the laundry, and each "
             "belongs somewhere different: the room is travel, the bar is the traveller's " ($ term {:id :per-diem}) ".")
          ($ p {} "When there is a receipt or an online order listing what was bought, the line is split, and each item is "
             "coded on its own. Whatever gets coded, whole line or one item, is called a " ($ term {:id :unit}) ".")
          ($ aside {:label "Why it matters"}
             "Without the split, a $3,392 hotel bill would be coded all as travel, and the drinks would hide inside it."))
       ($ panel {:title (if (seq items) "Your line has a receipt" "Your line has no receipt")}
          (if (seq items)
            ($ :div
               ($ :div {:class "font-mono text-xs text-ink-3"} (:merchant line) " · " (:currency line) " " (ui/money (:amount line)))
               ($ :ul {:class "mt-2"}
                  (for [[i {:keys [item amount]}] (map-indexed vector items)
                        :let [this? (= item (:item unit))]]
                    ($ :li {:key i :class (cx "flex items-baseline justify-between gap-3 border-t border-line-1 py-1.5 text-sm"
                                              (if this? "font-semibold" "text-ink-3"))}
                       ($ :span {} (when this? "→ ") item ($ :span {:class "ml-2 font-mono text-2xs"} (:line unit) "/" (inc i)))
                       ($ ui/amount {:amount amount :currency (:currency line)}))))
               ($ :p {:class "mt-2 text-sm text-ink-2"} "This guide follows the item marked →, unit " (:id unit) "."))
            ($ :p {:class "text-sm text-ink-2"} "So it is coded whole: one line, one unit. Nothing to do at this step."))))))

(defui step-3 [{:keys [data]}]
  (let [world (ui/use-world)
        {:keys [facts]} data
        {:keys [holder card-name whereabouts projects traveller category mcc category-from channel country-mismatch
                refund-of earlier-here upcoming]} facts
        person (get-in world [:entities holder :name])]
    ($ :<>
       ($ :div
          ($ p {} "Before anything is guessed, ordinary code looks up and calculates everything it can know for certain. "
             "These are the line's " ($ term {:id :facts}) ".")
          ($ p {} "Whose card is it? Where was that person on that day: at home, or on one of their trips? Which projects "
             "were running? Is this money coming back from an earlier purchase? None of that needs judgment. It needs the "
             "card list, the calendar of trips, and some arithmetic.")
          ($ aside {:label "The golden rule"}
             "If it can be computed, it is computed, never guessed. Dates in particular are compared by code, never by the "
             "model: “was the 7th of November inside the Lisbon trip?” is a calculation, and calculations don't make mistakes.")
          ($ p {} "Adding all this to a bare bank line is what the industry calls " ($ term {:id :enrichment}) "."))
       ($ panel {:title "What code worked out for your line"}
          ($ kv {:rows [["Paid with" card-name "from the scenario's list of cards"]
                        ["Cardholder" person nil]
                        ["That day they were" (where-text world whereabouts) "the purchase date against their trips' dates"]
                        ["Projects running" (if (seq projects) (str/join ", " (map #(get-in world [:projects % :name]) projects)) "none")
                         "the date against each project's dates"]
                        ["Travels for one" (if traveller "yes" "no") "each running project's list of travellers"]
                        ["Kind of shop" (when category (str category " (" mcc ")"))
                         (if (= :enrichment category-from) "Jev's guess from the name (step 4)" "the merchant code, looked up")]
                        ["Shop is where they were" (when (= :in-store channel) (if country-mismatch "no: a different country or city" "yes"))
                         "the shop's country and city against theirs"]
                        ["Money back for" (when refund-of (str "the purchase of " (:date refund-of) ", " (:currency refund-of) " " (ui/money (:amount refund-of))))
                         "an earlier purchase, same card, same shop"]
                        ["Earlier charge here" (when earlier-here (str (:date earlier-here) ", " (:currency earlier-here) " " (ui/money (:amount earlier-here))))
                         "a tip or a stay posted after the bill"]
                        ["Trips coming up" (when (seq upcoming) (str (count upcoming) " in the next 90 days")) "what a booking may be for"]]})))))

(defui step-4 [{:keys [data]}]
  (let [{:keys [enrich responses facts unit]} data
        answer (get-in responses [:enrich :answers :category])
        top (take 5 (sort-by (comp - val) (:probabilities answer)))]
    ($ :<>
       ($ :div
          ($ p {} "Most rules start from the kind of shop, so the " ($ term {:id :merchant-code}) " matters. But some lines "
             "arrive without one: an online invoice, a small business, a foreign terminal.")
          ($ p {} "Then this is the first time " ($ term {:id :jev}) " is asked anything. It reads the merchant's name and "
             "the few facts there are, and picks the kind of business from the list, or “none of these”.")
          ($ aside {:label "Only when it's sure enough"}
             "The guess is used only when Jev gives it at least 50%. Otherwise the line goes on without a kind of shop, and "
             "the rules that need one simply won't apply."))
       ($ panel {:title (if enrich "Your line had no merchant code" "Your line had a merchant code")}
          (if (and enrich answer)
            ($ :div
               ($ :p {:class "text-sm"} "Asked: " ($ :i {} "“Which kind of merchant is " (:merchant unit) "?”"))
               ($ :div {:class "mt-3 grid grid-cols-[minmax(0,1fr)_100px_40px] items-center gap-x-3 gap-y-1.5 text-sm"}
                  (for [[option pr] top]
                    ($ :<> {:key option}
                       ($ :span {:class (when (= option (:choice answer)) "font-semibold")} option)
                       ($ ui/bar {:p pr :wide true})
                       ($ ui/mono {:class "text-right text-2xs"} (ui/pct pr)))))
               ($ :p {:class "mt-3 text-sm text-ink-2"}
                  (if (:category facts)
                    (str "Jev's pick, " (:choice answer) ", is used: it reached " (ui/pct (:confidence answer)) ".")
                    "Jev wasn't sure enough, so the line goes on without a kind of shop.")))
            ($ :p {:class "text-sm text-ink-2"}
               "Nothing to guess: the bank sent " (:mcc facts) ", which is " (:category facts)
               ". Jev isn't asked, and the line costs one call less."))))))

(defui step-5 [{:keys [data]}]
  (let [state (get-in data [:requests :plain :state])
        show (fn [m] ($ :dl {:class "font-mono text-xs"}
                        (for [[k v] m]
                          ($ :div {:key (name k) :class "grid grid-cols-[minmax(120px,190px)_minmax(0,1fr)] gap-x-3 border-t border-line-1 py-1"}
                             ($ :dt {:class "min-w-0 text-ink-3 [overflow-wrap:anywhere]"}
                                ;; A long field name wraps after its underscores, not over the value.
                                (for [[i part] (map-indexed vector (str/split (name k) #"(?<=_)"))]
                                  ($ :<> {:key i} part ($ :wbr))))
                             ($ :dd {:class "min-w-0 text-ink-1"}
                                (cond (sequential? v) ($ :ul {} (for [[i x] (map-indexed vector v)] ($ :li {:key i} "· " x)))
                                      (boolean? v) (str v)
                                      (map? v) (pr-str v)
                                      :else v))))))]
    ($ :<>
       ($ :div
          ($ p {} "Jev doesn't read databases. It reads words. So code writes the facts out as short labelled "
             "sentences: “cardholder_was: in Lisbon, on a trip for Harbour Exhibition (day 5 of 14)”.")
          ($ p {} "This bundle is called the " ($ term {:id :state}) ". It holds what the bank said, under "
             ($ :code {:class "font-mono text-sm"} "transaction") ", and what code worked out, under "
             ($ :code {:class "font-mono text-sm"} "context") ". Nothing else: no ids, no database rows, nothing Jev doesn't need.")
          ($ aside {:label "When Jev gets one wrong"}
             "Look here first. Did it have the fact it needed, in words it could read? Most mistakes are a missing or "
             "muddled fact, not a bad judgment."))
       ($ panel {:title "Exactly what Jev reads about your line"}
          ($ ui/eyebrow {:class "mt-1"} "transaction")
          (show (:transaction state))
          ($ ui/eyebrow {:class "mt-3"} "context")
          (show (:context state))))))

(defui step-6 [{:keys [data]}]
  (let [world (ui/use-world)
        {:keys [requests enrich]} data
        rq (:questions (:rules requests))
        pq (:questions (:plain requests))
        fuzzy (sort-by key (filter #(str/starts-with? (name (key %)) "fuzzy") pq))
        calls (if enrich 3 2)]
    ($ :<>
       ($ :div
          ($ p {} "Now the questions. Each " ($ term {:id :question}) " is precise, and its possible answers are spelled "
             "out. There are two kinds:")
          ($ :ul {:class "mt-2 space-y-1.5 text-base"}
             ($ :li {} "→ a " ($ term {:id :choice}) ": pick one from a list (“which bucket?”)")
             ($ :li {} "→ a " ($ term {:id :noul}) ": yes or no (“does rule A15 apply?”)"))
          ($ p {} "They are sent in two " ($ term {:id :request} "requests") " with the same state. One also includes "
             "every " ($ term {:id :rule}) " written out, so Jev can apply them. The other leaves the rules out, so we can see "
             "what Jev thinks from the facts alone, and asks the small yes/no questions some rules depend on.")
          ($ aside {:label "All at once"}
             "Questions in one request are answered together, in parallel, and can't see each other's answers. That keeps "
             "each one honest and the whole thing fast: your line took " calls " calls."))
       ($ :div {:class "grid gap-3"}
          ($ panel {:title (str "Request 1 · with the rules · " (count rq) " questions")}
             ($ :p {:class "text-sm"} ($ :b {} "Which bucket? ") ($ :i {:class "text-ink-2"} (get-in rq [:code :instructions])))
             ($ :p {:class "mt-2 text-sm text-ink-2"} "The same question again with the options in reverse order (does Jev "
                "favour whatever comes first?), and one yes/no for each of the " (count (:rules world)) " rules: "
                ($ :i {} "“" (get-in rq [:rule_A1 :instructions] (some-> (first (filter #(str/starts-with? (name (key %)) "rule") rq)) val :instructions)) "”")))
          ($ panel {:title (str "Request 2 · without the rules · " (count pq) " questions")}
             ($ :p {:class "text-sm"} ($ :b {} "Which bucket? ") ($ :i {:class "text-ink-2"} (get-in pq [:code :instructions])))
             ($ :p {:class "mt-2 text-sm"} "And the " (count fuzzy) " yes/no questions some rules depend on, their "
                ($ term {:id :fuzzy-block} "fuzzy blocks") ":")
             ($ :ul {:class "mt-1.5 space-y-1 text-sm text-ink-2"}
                (for [[k q] fuzzy] ($ :li {:key (name k)} "· " (fuzzy-text (:instructions q))))))))))

(defui step-7 [{:keys [data]}]
  (let [{:keys [requests responses unit]} data
        code (get-in responses [:rules :answers :code])
        pq (get-in requests [:plain :questions])
        pa (get-in responses [:plain :answers])
        fuzzy (sort-by key (filter #(str/starts-with? (name (key %)) "fuzzy") pq))]
    ($ :<>
       ($ :div
          ($ p {} "Jev doesn't answer in sentences. For every option it gives a " ($ term {:id :probability}) ": per diem "
             "99%, hospitality 1%, and so on. Its pick is the most likely one.")
          ($ p {} "It also says how sure it is overall, its " ($ term {:id :confidence}) ". When one option clearly wins, "
             "confidence is high. When two are close, it is low, and that is useful: it tells us which lines are genuinely hard.")
          ($ aside {:label "Think of it as"}
             "A junior bookkeeper who says “I'm 97% sure this is per diem” instead of just “per diem”. Whether 97% is sure "
             "enough to act on isn't Jev's call. That comes at step 10.")
          ($ p {} "For the yes/no questions, the number is the chance of yes. Around 50% means it can't tell, not that the "
             "answer is “half true”."))
       ($ :div {:class "grid gap-3"}
          ($ panel {:title (str "Which bucket? Jev picked " (some-> (:choice code) (str/replace "_" " ")) " at " (ui/pct (:confidence code)))}
             ($ line/option-rows {:criteria (get-in requests [:rules :questions :code :criteria])
                                  :probabilities (:probabilities code) :unit unit}))
          ($ panel {:title "The yes/no questions"}
             ($ :div {:class "grid grid-cols-[minmax(0,1fr)_100px_40px] items-center gap-x-3 gap-y-1.5 text-sm"}
                (for [[k q] fuzzy
                      :let [pr (get-in pa [k :noul])]]
                  ($ :<> {:key (name k)}
                     ($ :span {:class (if (yes? pr) "text-ink-1" "text-ink-3")} (fuzzy-text (:instructions q)))
                     ($ ui/bar {:p pr :wide true})
                     ($ ui/mono {:class "text-right text-2xs"} (ui/pct pr))))))))))

(defui step-8 [{:keys [data]}]
  (let [world (ui/use-world)
        {:keys [facts explained]} data]
    ($ :<>
       ($ :div
          ($ p {} "A " ($ term {:id :rule}) " is a sentence a person wrote once, for every line: “While a traveller is on a "
             "trip for Harbour, their restaurant and café purchases are Harbour per diem.”")
          ($ p {} "Each rule is built from " ($ term {:id :block} "blocks") ", the conditions that make it apply, snapped "
             "together like building blocks. Most are " ($ term {:id :exact-block} "exact") ": code checks them against the "
             "facts, and the answer is plainly yes or no (“on a trip for Harbour”: yes). A few are "
             ($ term {:id :fuzzy-block} "fuzzy") ": only judgment can settle them (“was this a meal for someone else?”), so "
             "they are Jev's yes/no answers from the step before.")
          ($ p {} "A rule applies when all its blocks hold. Then:")
          ($ :ul {:class "mt-1.5 space-y-1.5 text-base"}
             ($ :li {} "→ an " ($ term {:id :exception}) " wins over the rules it names (hospitality over per diem)")
             ($ :li {} "→ a " ($ term {:id :check-rule}) " sends the line to a person, whatever else applies")
             ($ :li {} "→ if the rest disagree, or none applies, the rules can't decide: a " ($ term {:id :conflict})))
          ($ aside {:label "Where your policy lives"}
             "Rules are the part you control. When a line is coded wrongly, the fix is usually a rule: a new one, a sharper "
             "block, or an exception."))
       ($ panel {:title "The rules, on your line"}
          ($ line/rules-panel {:world world :facts facts :explained explained})))))

(defui step-9 [{:keys [data]}]
  (let [{:keys [decisions]} data]
    ($ :<>
       ($ :div
          ($ p {} "Now there are plenty of answers: Jev's own pick with the rules, its pick without them, a yes or no for "
             "every rule, and the blocks. How should they be combined into one coding?")
          ($ p {} "There's more than one sensible way, and each is a " ($ term {:id :strategy}) ". Trust Jev's pick? Trust "
             "only the rules? Use the rules where they decide and Jev where they can't? The sandbox works out all seven "
             "from the same answers, so comparing them costs nothing.")
          ($ aside {:label "The Blocks column in the sandbox"}
             "is one of these: what the rule blocks decided alone. “same” means it agrees with the chosen strategy.")
          ($ p {} "In daily use, one strategy will be chosen, the one that codes the most lines without ever coding one "
             "wrongly. Finding it is what the sandbox is for."))
       ($ panel {:title "Each strategy's coding of your line"}
          ($ :div {:class "grid grid-cols-[minmax(0,1fr)_auto_48px] items-center gap-x-3 text-sm"}
             (for [[k n about] run/strategies
                   :let [{:keys [code confidence reason fallback]} (get decisions k)]]
               ($ :<> {:key (name k)}
                  ($ :div {:class "border-t border-line-1 py-2"}
                     ($ :div {:class "font-medium"} n)
                     ($ :div {:class "text-xs text-ink-3"} about
                        (when-let [r (or reason fallback)]
                          ($ :b {:class "text-ink-2"} " Here: " (reason-words r) "."))))
                  ($ :div {:class "border-t border-line-1 py-2"} ($ ui/tag {:code code}))
                  ($ ui/mono {:class "border-t border-line-1 py-2 text-right"} (ui/pct confidence)))))))))

(defui step-10 [{:keys [data t set-t! strategy set-strategy!]}]
  (let [{:keys [unit decisions]} data
        {:keys [code confidence] :as d} (get decisions strategy)
        auto? (and (not= :needs-person code) (>= confidence t))
        o (outcome/outcome unit d t)]
    ($ :<>
       ($ :div
          ($ p {} "Last question: is this coding sure enough to apply without anyone looking?")
          ($ p {} "That's the " ($ term {:id :threshold}) ". Set it at 85% and every coding at 85% or more is applied "
             ($ term {:id :auto-coded} "automatically") "; the rest go to a person, with everything this guide has shown "
             "laid out for them. A coding of " ($ term {:id :needs-a-person}) " always goes to a person.")
          ($ p {} "Raise the threshold and fewer lines are coded wrongly, but more land on someone's desk. Lower it and the "
             "reverse. Try it with the slider: the sandbox's chart shows the same trade-off across every line.")
          ($ aside {:label "The answer key"}
             "In the sandbox a person has written down the right bucket for every line: the " ($ term {:id :answer-key}) ". "
             "That's how we can say right or wrong, the line's " ($ term {:id :outcome}) ". With real books there is no "
             "key: a person's answer becomes the truth, and often a new rule. That's the "
             ($ term {:id :human-in-the-loop}) "."))
       ($ panel {:title "Try it"}
          ($ :div {:class "grid gap-3"}
             ($ :label {:class "text-xs text-ink-3"} "Strategy"
                ($ :select {:value (name strategy) :on-change #(set-strategy! (keyword (.. % -target -value)))
                            :class "mt-1 block w-full rounded-1 border border-line-2 bg-surface-0 px-2 py-1.5 text-sm text-ink-1"}
                   (for [[k n] run/strategies] ($ :option {:key (name k) :value (name k)} n))))
             ($ :label {:class "text-xs text-ink-3"} "Coded automatically at or above"
                ($ :div {:class "mt-1 flex items-center gap-3"}
                   ($ :input {:type "range" :min 50 :max 99 :value (js/Math.round (* 100 t))
                              :on-change #(set-t! (/ (js/Number (.. % -target -value)) 100))
                              :class "w-full accent-[var(--ink-1)]"})
                   ($ :span {:class "w-12 text-right font-display text-xl font-bold"} (ui/pct t)))))
          ($ :ol {:class "mt-4 space-y-2.5 text-sm"}
             ($ :li {:class "flex flex-wrap items-center gap-2"}
                ($ :span {:class "text-ink-3"} "1. It's coded") ($ ui/tag {:code code}) ($ :span {} "at " (ui/pct confidence)))
             ($ :li {:class "flex flex-wrap items-center gap-2"}
                ($ :span {:class "text-ink-3"} "2. So it's")
                ($ :b {} (cond (= :needs-person code) "sent to a person: the coding itself says so"
                               auto? (str "coded automatically: " (ui/pct confidence) " reaches " (ui/pct t))
                               :else (str "sent to a person: " (ui/pct confidence) " is under " (ui/pct t)))))
             (if-not (:expect unit)
               ($ :li {:class "flex flex-wrap items-center gap-2"}
                  ($ :span {:class "text-ink-3"} "3. No answer key:")
                  ($ :span {} "a line of your books is right when a person says so"))
             ($ :li {:class "flex flex-wrap items-center gap-2"}
                ($ :span {:class "text-ink-3"} "3. The answer key says") ($ ui/tag {:code (:expect unit)})
                (when (seq (:ok unit)) ($ :span {:class "text-ink-3"} "(also fine: " (str/join ", " (map name (:ok unit))) ")"))
                (case (:review unit) :required ($ :span {} "and a person must look") :allowed ($ :span {} "and a person may look") nil)))
             ($ :li {:class "flex flex-wrap items-center gap-2"}
                ($ :span {:class "text-ink-3"} "4. Outcome") ($ ui/pill {:outcome o})))
          (when-let [w (or (:hard unit) (:why unit))]
            ($ :div {:class "mt-4 border-t border-line-1 pt-3 text-sm text-ink-2"}
               ($ ui/eyebrow {:class "mb-1"} "Why this line is tricky, says whoever wrote the key") w))
          ($ :div {:class "mt-4 border-t border-line-1 pt-3"}
             ($ ui/eyebrow {:class "mb-2"} "Every strategy at " (ui/pct t))
             ($ :div {:class "flex flex-wrap gap-x-4 gap-y-1.5 text-xs"}
                (for [[k n] run/strategies
                      :let [oc (outcome/outcome unit (get decisions k) t)]]
                  ($ :span {:key (name k) :class "inline-flex items-center gap-1.5"} ($ ui/dot {:outcome oc}) n))))))))

;; ----------------------------------------------------------------------------------------------- the page

(defui rail
  "The ten steps as stations on a line, the coin at the current one."
  [{:keys [at on-go]}]
  ($ :nav {:aria-label "Steps" :class "relative"}
     ($ :div {:class "absolute top-[13px] right-[5%] left-[5%] h-0.5 bg-line-1"})
     ($ :div {:class "absolute top-[13px] left-[5%] h-0.5 bg-ink-1 transition-all duration-500"
              :style {:width (str (* 90 (/ (dec at) (dec (count steps)))) "%")}})
     ($ :ol {:class "relative grid grid-cols-10"}
        (for [{:keys [n actor short]} steps
              :let [here (= n at) done (< n at)]]
          ($ :li {:key n :class "flex flex-col items-center"}
             ($ :button {:on-click #(on-go n) :aria-current (when here "step")
                         :title (:name (actors actor))
                         :class (cx "relative z-10 flex size-7 items-center justify-center rounded-full font-mono text-xs transition-all duration-300"
                                    (cond here "scale-125 bg-coin text-on-coin shadow-[0_0_0_4px_var(--surface-0)]"
                                          done "bg-ink-1 text-ink-inverse"
                                          :else "border border-line-2 bg-surface-0 text-ink-3 hover:border-ink-2"))}
                n)
             ($ :span {:class (cx "mt-1.5 hidden text-center text-2xs leading-tight md:block" (if here "font-semibold text-ink-1" "text-ink-3"))}
                short)
             ($ :span {:class "hidden font-mono text-[9.5px] text-ink-3 md:block"} (:glyph (actors actor))))))))

(defui cast-of [{}]
  ($ :div {:class "mt-5 grid gap-2 sm:grid-cols-2 lg:grid-cols-4"}
     (for [[k {:keys [name glyph about]}] actors]
       ($ :div {:key (cljs.core/name k) :class "flex gap-3 border border-line-1 p-3"}
          ($ :span {:class "w-9 flex-none whitespace-nowrap font-mono text-lg text-ink-3"} glyph)
          ($ :div
             ($ :div {:class "text-sm font-semibold"} name)
             ($ :div {:class "text-xs text-ink-2"} about))))))

(defui picker [{:keys [examples at-id on-pick]}]
  ($ :div {:class "mt-6"}
     ($ ui/eyebrow {:class "mb-2"} "Pick a line to follow")
     ($ :div {:class "grid gap-2 sm:grid-cols-2 lg:grid-cols-6"}
        (for [{:keys [id short title why]} examples
              :let [on? (= id at-id)]]
          ($ :button {:key id :on-click #(on-pick id) :title id
                      :class (cx "flex flex-col items-start gap-1 border p-3 text-left transition-colors"
                                 (if on? "border-ink-1 bg-surface-1" "border-line-1 hover:border-line-2 hover:bg-surface-1"))}
             ($ :span {:class "flex w-full items-baseline justify-between gap-2"}
                ($ :span {:class "text-sm font-semibold"} title)
                ($ :span {:class "font-mono text-2xs text-ink-3"} (or short id)))
             ($ :span {:class "text-xs leading-snug text-ink-2"} why))))))

(defui page [{:keys [query]}]
  (let [at (min 10 (max 1 (or (some-> (:step query) js/parseInt) 1)))

        [t set-t!] (uix/use-state 0.85)
        [strategy set-strategy!] (uix/use-state :choice)
        go! (uix/use-callback (fn [q] (route/go! ["docs"] q true)) [])
        scenarios (api/use-get "/sandbox/scenarios")
        ;; The record's own lines when one of its scenarios is coded: the one asked for, else the one with the most
        ;; lines coded (the first, on a tie). Else the invented sample's.
        coded (filterv #(and (:record %) (seq (:runs %))) (get-in scenarios [:data :scenarios]))
        rec (or (some #(when (= (:scenario query) (:name %)) %) coded)
                (reduce #(if (> (:coded %2 0) (:coded %1 0)) %2 %1) (first coded) (rest coded)))
        scenario (if rec (:name rec) sample)
        run-id (some->> (get-in scenarios [:data :scenarios]) (some #(when (= scenario (:name %)) %)) :runs first :id)
        run-state (api/use-get (when rec (str "/sandbox/scenarios/" scenario "/runs/" run-id)))
        shown (if rec (record-examples (get-in run-state [:data :lines])) examples)
        id (or (:line query) (:id (first shown)))
        to-step (uix/use-callback (fn [n] (go! {:step n :line id :scenario (:scenario query)})) [go! id query])
        world-state (api/use-get (str "/sandbox/scenarios/" scenario))
        line-state (api/use-get (when (and run-id id) (str "/sandbox/scenarios/" scenario "/runs/" run-id "/lines/" id)))
        trace-state (api/use-get (when (and rec id) (str "/sandbox/scenarios/" scenario "/trace/" id)))
        world (get-in world-state [:data :world])
        {:keys [title]} (nth steps (dec at))]
    (uix/use-effect
     (fn []
       (let [f (fn [e]
                 (when-not (#{"INPUT" "SELECT" "TEXTAREA"} (.. e -target -tagName))
                   (case (.-key e)
                     "ArrowRight" (when (< at 10) (to-step (inc at)))
                     "ArrowLeft" (when (> at 1) (to-step (dec at)))
                     nil)))]
         (js/addEventListener "keydown" f)
         #(js/removeEventListener "keydown" f)))
     [at to-step])
    ($ ui/world-context {:value world}
       ($ :div {:class "max-w-[1240px]"}
          ($ ui/eyebrow {} "Docs / how a line is coded")
          ($ :h1 {:class "mt-1 font-display text-2xl font-extrabold tracking-tight"} "From a bank line to a bucket")
          ($ :p {:class "mt-2 max-w-[72ch] text-base leading-relaxed text-ink-2"}
             "Every payment on a card has to be filed somewhere in the books. This is how ownpurse does it, one step at a "
             "time, on a real line" (if rec " of your own books." " from the sandbox.") " Four characters do the work:")
          ($ cast-of {})
          (when (next coded)
            ($ :label {:class "mt-6 flex flex-wrap items-center gap-2 text-sm text-ink-3"} "Following the scenario"
               ($ :select {:value scenario :on-change #(go! {:step at :scenario (.. % -target -value)})
                           :class "rounded-1 border border-line-2 bg-surface-0 px-2 py-1 text-sm text-ink-1"}
                  (for [{:keys [name title]} coded] ($ :option {:key name :value name} (or title name))))))
          ($ picker {:examples shown :at-id id :on-pick #(go! {:step at :line % :scenario (:scenario query)})})
          ($ :div {:class "sticky top-[45px] z-10 -mx-6 mt-6 border-b border-line-1 bg-surface-0/95 px-6 pt-3 pb-2 backdrop-blur"}
             ($ rail {:at at :on-go to-step}))
          ($ ui/loading {:state (cond (:error scenarios) scenarios (:error world-state) world-state
                                      (or (nil? world) (nil? run-id) (nil? id)) {:loading true} :else line-state)}
             (when-let [data (:data line-state)]
               ($ :section {:key (str id "/" at) :class "mt-6 animate-[fade_.25s_ease-out]"}
                  ($ :div {:class "flex flex-wrap items-center gap-3"}
                     ($ :span {:class "font-display text-4xl font-extrabold text-ink-3"} at)
                     ($ :h2 {:class "font-display text-xl font-bold"} title)
                     ($ actor-chip {:actor (:actor (nth steps (dec at)))}))
                  (when (seq (:unanswered data))
                    ($ ui/notice {:tone :bad}
                       "This scenario changed since the line was coded, and the record keeps no answer to its new "
                       "requests: Jev's answers below are missing until the scenario is run again. Pick another "
                       "scenario above, or run this one from the sandbox."))
                  ($ :div {:class (cx "mt-2 grid gap-8"
                                      ;; Jev's answers and the rules are wide tables: they go under the words, full width.
                                      (if (#{7 8} at) "[&>div:first-child]:max-w-[72ch]" "lg:grid-cols-[minmax(0,5fr)_minmax(0,6fr)]"))}
                     (case at
                       1 ($ step-1 {:data data})
                       2 ($ step-2 {:data data})
                       3 ($ step-3 {:data data})
                       4 ($ step-4 {:data data})
                       5 ($ step-5 {:data data})
                       6 ($ step-6 {:data data})
                       7 ($ step-7 {:data data})
                       8 ($ step-8 {:data data})
                       9 ($ step-9 {:data data})
                       ($ step-10 {:data data :t t :set-t! set-t! :strategy strategy :set-strategy! set-strategy!})))
                  ($ dev-note {:step (get-in trace-state [:data :trace :steps (dec at)])}
                     (case at
                       1 (if rec
                           "The line: a bank_lines row of the record, chosen by the scenario's lines layer (ownpurse.scenario.coding/read-lines)."
                           "The line: a scenario's :cases (components/classify/resources/classify/eval/<name>.edn). With a record, a book's bank lines.")
                       2 "ownpurse.classify.facts/units"
                       3 "ownpurse.classify.facts/enrich: whereabouts, running-projects, refund-of, earlier-here"
                       4 "ownpurse.classify.core/enrichment and enriched-category (used at 50% or more)"
                       5 "ownpurse.classify.facts/state"
                       6 "ownpurse.classify.core/requests: :rules and :plain; ownpurse.classify.jev/ask sends them"
                       7 (if rec
                           "the answers kept in the record's judgments table, by each request's hash (ownpurse.scenario.run/caching-ask)"
                           "the responses kept in the run file, components/classify/runs/<run>.edn")
                       8 "ownpurse.classify.blocks/holds and rule-holds; ownpurse.classify.core/combine and explain"
                       9 "ownpurse.classify.core/decide"
                       "ownpurse.classify.outcome/outcome (shared with the browser); the eval in ownpurse.classify.eval"))
                  ($ :div {:class "mt-8 flex items-center justify-between border-t border-line-1 pt-4"}
                     (if (> at 1)
                       ($ :button {:on-click #(to-step (dec at)) :class "rounded-1 border border-line-2 px-3 py-1.5 text-sm hover:bg-surface-1"}
                          "← " (:title (nth steps (- at 2))))
                       ($ :span))
                     ($ ui/muted {:class "hidden text-xs md:block"} "← → keys move between steps")
                     (if (< at 10)
                       ($ :button {:on-click #(to-step (inc at)) :class "rounded-1 bg-ink-1 px-3 py-1.5 text-sm text-ink-inverse hover:opacity-90"}
                          (:title (nth steps at)) " →")
                       ($ :span {:class "flex gap-4 text-sm"}
                          ($ :a {:class "text-lapis hover:underline" :href (route/href ["sandbox" scenario run-id] {:line id})} "Open this line in the sandbox →")
                          ($ :a {:class "text-lapis hover:underline" :href (route/href ["docs" "terms"])} "Every word, explained →")))))))))))
