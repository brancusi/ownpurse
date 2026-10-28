(ns ownpurse.admin.terms
  "The admin's vocabulary: every word a newcomer may trip on, what it means here, and where it comes from (an
  accountant's word, a bank's, machine learning's, TypeSafe's, or one ownpurse chose). `term` underlines a word
  anywhere in the admin and shows its meaning on hover; the docs' vocabulary page lists them all."
  (:require
   [clojure.string :as str]
   [ownpurse.admin.route :as route]
   [ownpurse.admin.ui :as ui :refer [cx]]
   [uix.core :as uix :refer [$ defui]]))

(def origins
  "Where a word comes from, in the order the vocabulary explains them."
  [[:accounting "Accounting" "A bookkeeper's word, used the way accountants use it."]
   [:banking "Banks and cards" "A word from banking and the card networks (Visa, Mastercard)."]
   [:ml "Machine learning" "A word from machine learning and AI, where it is standard."]
   [:typesafe "TypeSafe" "TypeSafe's own word, from the documentation of Jev, the model we ask."]
   [:ownpurse "ownpurse's own" "A word this project chose. You will not find it in a textbook."]])

(def origin-names (into {} (map (fn [[k n]] [k n])) origins))

(def groups
  [[:books "The books"]
   [:line "The bank line"]
   [:facts "What code works out"]
   [:jev "Asking Jev"]
   [:rules "Rules"]
   [:deciding "Deciding"]
   [:testing "Testing it: the sandbox"]])

(def terms
  "Each term: :id, :term, :group, :short (the hover's one sentence), :long (paragraphs), :origin, :origin-note (where
  the word comes from, plainly), :step (the guide's step where it happens), :see (related terms)."
  [;; ----------------------------------------------------------------------------------- the books
   {:id :coding :term "Coding" :group :books :origin :accounting :step 1
    :short "Deciding which account a transaction belongs to: “this €41.60 is Harbour hospitality”."
    :long ["To code a transaction is to file it under one of the accounts in the books. Every bank line has to be coded before the books are done: a lunch is a meal, but whose meal, for which project, paid on whose behalf?"
           "Coding is the slow, repetitive part of bookkeeping. This whole pipeline exists to do most of it automatically, and to hand the rest to a person."]
    :origin-note "Standard bookkeeping language: accountants “code” transactions to accounts. Some tools say “categorise”."
    :see [:bucket :chart-of-accounts :bank-line]}

   {:id :bucket :term "Bucket" :group :books :origin :ownpurse :step 9
    :short "Where a line ends up once coded: an account, often for one project (“Harbour · Per diem”)."
    :long ["A bucket is the destination of a coding. In the sandbox each project has its own buckets (per diem, cost of goods, travel, hospitality); the studio has overhead; and there are two buckets for everything else: personal, and needs a person."
           "In real books a bucket stands for an account in the chart of accounts, plus the project the line is for."]
    :origin-note "ownpurse's word, used in the sandbox to keep things simple. Accountants would say “account” (and “category” or “tracking” for the project part)."
    :see [:chart-of-accounts :needs-a-person :coding]}

   {:id :chart-of-accounts :term "Chart of accounts" :group :books :origin :accounting
    :short "The list of every account a business keeps: rent, travel, materials, what it owes, what it is owed."
    :long ["A business's books are organised into accounts, and the chart of accounts is their list, each with a number and a name. Every transaction is recorded against one or more of them."
           "The sandbox's buckets are a small, simplified chart of accounts."]
    :origin-note "Standard accounting term."
    :see [:bucket :coding]}

   {:id :per-diem :term "Per diem" :group :books :origin :accounting
    :short "A fixed daily allowance for a traveller's own living costs (food, groceries, gym) while away for work."
    :long ["Latin for “per day”. Instead of claiming every coffee, a traveller gets a set amount a day for their living costs while travelling for a project, and those purchases are booked against it."
           "In the sandbox, food, groceries, cafés, gym and pharmacy bought by a traveller on a project trip are that project's per diem. The same lunch at home is personal."]
    :origin-note "Standard accounting and tax term (governments publish per diem rates)."
    :see [:hospitality :trip :traveller]}

   {:id :cost-of-goods :term "Cost of goods" :group :books :origin :accounting
    :short "What it costs to make the work itself: materials, tools, fabrication, shipping."
    :long ["The direct costs of making what a project delivers: for a sculpture, the plaster, the moulds, the tools and the shipping. Accountants often shorten it to COGS (cost of goods sold)."]
    :origin-note "Standard accounting term (COGS)."
    :see [:overhead :bucket]}

   {:id :overhead :term "Overhead" :group :books :origin :accounting
    :short "The studio's ordinary running costs that belong to no single project: software, phone, insurance."
    :long ["Costs of keeping the business running, whatever it is working on. They are not charged to a project."]
    :origin-note "Standard accounting term."
    :see [:cost-of-goods :bucket]}

   {:id :hospitality :term "Hospitality" :group :books :origin :accounting
    :short "Meals, drinks or gifts for other people (clients, collaborators), given for a project's work."
    :long ["When a traveller buys lunch for themselves it is per diem; when they buy lunch for the foundryman they work with, it is hospitality. The receipt looks the same: the difference is who ate, which is why it is a judgment and not a lookup."]
    :origin-note "Accounting and tax term (“meals and entertainment” in some charts)."
    :see [:per-diem :fuzzy-block]}

   {:id :reconciliation :term "Reconciliation" :group :books :origin :accounting
    :short "Checking that every line on the bank statement is accounted for in the books, and coded."
    :long ["Reconciling a bank account means matching each bank line to an entry in the books. Automatic reconciliation is the goal of this pipeline: most lines coded and matched without a person, the uncertain ones shown to one."]
    :origin-note "Standard accounting term. “Auto-reconciliation” is the common name for doing it by software."
    :see [:coding :auto-coded]}

   ;; ----------------------------------------------------------------------------------- the line
   {:id :bank-line :term "Bank line" :group :line :origin :banking :step 1
    :short "One row of a bank or card statement: a date, a merchant, an amount, and little else."
    :long ["What the bank sends: when, who was paid, how much, in what currency, by which card, and a merchant code. It never says what the purchase was for. Everything after this step is about filling that gap."
           "Xero calls these statement lines; ownpurse says bank line. In the sandbox the lines are invented, for testing."]
    :origin-note "Banking language: a statement line, or transaction."
    :see [:merchant-code :unit :scenario]}

   {:id :merchant-code :term "Merchant code (MCC)" :group :line :origin :banking :step 1
    :short "A four-digit number the card network gives every shop for its kind of business: 5812 is restaurants."
    :long ["Merchant Category Codes are an international standard (ISO 18245). When a shop starts taking cards, it is given the code for its trade: 5812 restaurants, 7011 hotels, 4511 airlines."
           "It is the most reliable clue a bank line carries about what was bought, so many rules start from it. Some lines arrive without one: then Jev guesses the kind of business from the merchant's name (enrichment)."]
    :origin-note "Card industry standard (ISO 18245), set by the card networks."
    :see [:category :enrichment]}

   {:id :category :term "Category" :group :line :origin :ownpurse :step 3
    :short "The merchant code in words: “Restaurants” instead of 5812."
    :long ["Code turns the merchant code into its name using the scenario's list of categories. When there is no merchant code, the category is Jev's guess from the merchant's name."]
    :origin-note "Plain word; here it means the merchant code's name, not an account."
    :see [:merchant-code :enrichment]}

   {:id :unit :term "Unit (and item)" :group :line :origin :ownpurse :step 2
    :short "What is coded: a whole bank line, or one item of it when a receipt lists what was bought."
    :long ["A hotel bill can hold a room (travel), a bar tab (per diem) and laundry. When a receipt or an online order shows the items, each item is coded on its own: a unit. Line d88 becomes units d88/1, d88/2, d88/3."
           "Without a receipt, the line is one unit and is coded whole."]
    :origin-note "ownpurse's word. Accountants call coding one payment to several accounts “splitting” a transaction."
    :see [:bank-line]}

   ;; ----------------------------------------------------------------------------------- facts
   {:id :facts :term "Facts" :group :facts :origin :ownpurse :step 3
    :short "What code can know for certain about a line: whose card, where they were that day, which project was running."
    :long ["Before anyone guesses anything, ordinary code works out everything that can be looked up or calculated: the cardholder, whether that day fell inside one of their trips, which projects were running, whether this is a refund of an earlier purchase."
           "The rule is simple: if it can be computed, it is computed, never guessed. Dates in particular are compared by code, never by the model."]
    :origin-note "ownpurse's word for this step's output."
    :see [:enrichment :whereabouts :state]}

   {:id :enrichment :term "Enrichment" :group :facts :origin :banking :step 3
    :short "Adding to a bare bank line what it does not say: who, where, which project, what kind of shop."
    :long ["Banks send very little. Enriching a line means attaching everything else that is known: the facts code works out, and, when the merchant code is missing, the kind of business Jev judges from the merchant's name (the enrichment call)."]
    :origin-note "Fintech industry term: “transaction enrichment” (Plaid and others sell it)."
    :see [:facts :merchant-code]}

   {:id :whereabouts :term "Whereabouts" :group :facts :origin :ownpurse :step 3
    :short "Where the cardholder was on the day of the purchase: at home, or on a trip (and which day of it)."
    :long ["Worked out from the trips the scenario lists: if the purchase date falls inside one of the cardholder's trips, they were there, on that project, on day 5 of 14. Otherwise they were at home."]
    :origin-note "Plain English, chosen for this field."
    :see [:trip :facts]}

   {:id :trip :term "Trip" :group :facts :origin :ownpurse
    :short "A stretch of dates someone was away, for a project or for themselves."
    :long ["Trips are part of the scenario's world. A project trip makes food per diem and transport travel; a personal trip makes them personal."]
    :origin-note "Plain word."
    :see [:whereabouts :traveller :per-diem]}

   {:id :traveller :term "Traveller" :group :facts :origin :ownpurse
    :short "Someone a project sends on its trips, whose living costs there it pays."
    :long ["Each project names its travellers. A meal for a traveller is per diem; a meal for anyone else, given for the work, is hospitality."]
    :origin-note "Plain word."
    :see [:per-diem :hospitality]}

   ;; ----------------------------------------------------------------------------------- Jev
   {:id :jev :term "Jev" :proper true :group :jev :origin :typesafe :step 6
    :short "The AI model we ask for judgments. It answers typed questions with probabilities, not prose."
    :long ["Jev is TypeSafe's model. Unlike a chatbot, it does not write text: you give it some facts (the state) and precise questions, and it answers each with a choice or a yes/no, plus how likely each answer is."
           "Think of a very fast junior bookkeeper who has read the rules and can say “I'm 97% sure this is per diem”, but who never decides on their own whether that is sure enough."]
    :origin-note "Product name, from TypeSafe."
    :see [:system-one :state :question :probability]}

   {:id :system-one :term "System One" :proper true :group :jev :origin :typesafe
    :short "TypeSafe's family of fast models that make quick, focused judgments; Jev is the first."
    :long ["The name echoes the psychologist Daniel Kahneman's “System 1”: the fast, intuitive kind of thinking, as opposed to slow, deliberate reasoning (System 2). A System One model gives a quick, calibrated judgment that software can use directly."]
    :origin-note "TypeSafe's term, borrowing from psychology."
    :see [:jev]}

   {:id :state :term "State" :group :jev :origin :typesafe :step 5
    :short "What Jev reads about a line: its facts written out in words, and nothing it does not need."
    :long ["Jev never sees the database. Code writes the facts out as short labelled sentences (“cardholder_was: in Lisbon, on a project trip for Harbour Exhibition, day 5 of 14”) and that bundle is the state."
           "If Jev gets something wrong, the state is the first place to look: did it have the fact it needed, in words it could read?"]
    :origin-note "TypeSafe's word for a question's input."
    :see [:facts :question :request]}

   {:id :question :term "Question (judgment)" :group :jev :origin :typesafe :step 6
    :short "One precise thing we ask Jev about the state, with its possible answers spelled out."
    :long ["Each question has instructions (“How should this bank line be coded?”) and criteria (the options, each described). There are two kinds here: a Choice, which picks one option, and a Noul, which answers yes or no."]
    :origin-note "TypeSafe calls each a question; the answer is a judgment."
    :see [:choice :noul :request]}

   {:id :choice :term "Choice" :proper true :group :jev :origin :typesafe :step 6
    :short "A question whose answer is one option from a list, with a probability for every option."
    :long ["“Which bucket?” is a Choice: Jev gives each bucket a probability (per diem 99%, hospitality 1%, …) and its pick is the most likely one."]
    :origin-note "TypeSafe primitive."
    :see [:noul :probability :confidence]}

   {:id :noul :term "Noul" :proper true :group :jev :origin :typesafe :step 6
    :short "A yes/no question, answered with the probability that the answer is yes."
    :long ["“Is this a meal for people who are not the travellers?” is a Noul. Jev answers 0.08 (likely no) or 0.93 (likely yes). Near 0.5 means it cannot tell, not that it is “half true”."
           "Every fuzzy block in a rule is a Noul."]
    :origin-note "TypeSafe's name for its yes/no primitive."
    :see [:choice :fuzzy-block :probability]}

   {:id :request :term "Request (call)" :group :jev :origin :ml :step 6
    :short "One trip to Jev: a state and several questions sent together, answered together."
    :long ["Questions about the same state go in one request and are answered in parallel, so they cannot see each other's answers. A line costs two requests (three when its merchant code is missing): one with the rules written out, one without."]
    :origin-note "Standard software word for one call to a service (an API)."
    :see [:state :question :token]}

   {:id :token :term "Token" :group :jev :origin :ml
    :short "The unit a model reads text in, roughly three quarters of a word. Usage and cost are counted in them."
    :long ["The run's header counts the tokens sent to Jev. More facts and longer rules mean more tokens per line."]
    :origin-note "Standard machine-learning term."
    :see [:request]}

   {:id :probability :term "Probability" :group :jev :origin :ml :step 7
    :short "How likely Jev thinks an answer is, from 0% to 100%."
    :long ["Every answer comes with one. For a Choice, the probabilities of all the options add up to 100%. For a Noul, it is the chance of yes."]
    :origin-note "Plain mathematics."
    :see [:confidence :calibration]}

   {:id :confidence :term "Confidence" :group :jev :origin :typesafe :step 7
    :short "How sure the answer is overall: high when one option clearly wins, low when it is split."
    :long ["For Jev's Choice, confidence says how concentrated its probabilities are on one answer. For the rule-based strategies, ownpurse works it out from the rules: a rule at 95% or 5% is certain either way; one at 50% is a coin toss. The coding is only as sure as its least certain rule."
           "The threshold is compared with this number."]
    :origin-note "TypeSafe reports it for every answer; the rule strategies compute their own."
    :see [:probability :threshold]}

   {:id :calibration :term "Calibration" :group :jev :origin :ml
    :short "Whether “90% sure” really means right 9 times out of 10. A calibrated model's numbers can be trusted."
    :long ["The sandbox is how we check it: run many lines whose right answer we know, and see whether the confident ones are in fact right."]
    :origin-note "Standard machine-learning term."
    :see [:confidence :eval]}

   ;; ----------------------------------------------------------------------------------- rules
   {:id :rule :term "Rule" :group :rules :origin :accounting :step 8
    :short "A sentence that says how a kind of line is coded, built from blocks code and Jev can check."
    :long ["“While a traveller is on a trip for the Harbour Exhibition, their restaurant, café and grocery purchases are Harbour per diem.” Each rule has an id (A1), the bucket it codes to, the sentence, and its blocks: the conditions that make it apply."
           "Rules are written once, by a person, and apply to every line. They are where your policy lives."]
    :origin-note "Bookkeeping tools (Xero, QuickBooks) call these bank rules: “if the description contains X, code it to Y”. ownpurse's are the same idea, with a judgment allowed in the conditions."
    :see [:block :exception :check-rule]}

   {:id :block :term "Block" :group :rules :origin :ownpurse :step 8
    :short "One condition inside a rule (“on a trip for Harbour”, “category is restaurants”). A rule applies when all its blocks hold."
    :long ["Rules are built by snapping conditions together, like building blocks. Most are exact: code checks them against the facts and the answer is plainly yes or no. One kind is fuzzy: a yes/no question only judgment can answer, which Jev answers."
           "A rule holds as much as its weakest block: if any block is a no, the rule does not apply."
           "In the sandbox's list, the Blocks column shows what the rules alone decided (the “rule blocks” strategy): “same” when it agrees with the chosen coding, otherwise its own answer."]
    :origin-note "ownpurse's word. Xero and QuickBooks call them the conditions of a bank rule."
    :see [:exact-block :fuzzy-block :rule]}

   {:id :exact-block :term "Exact block" :group :rules :origin :ownpurse :step 8
    :short "A condition code checks for certain: the category, the card, the dates of a trip. Always yes or no."
    :long ["Exact blocks cost nothing and are never wrong about the facts they read, so rules lean on them wherever they can."]
    :origin-note "ownpurse's word."
    :see [:block :fuzzy-block]}

   {:id :fuzzy-block :term "Fuzzy block" :group :rules :origin :ownpurse :step 8
    :short "A condition only judgment can settle (“was this a meal for someone else?”), asked of Jev as a yes/no."
    :long ["Some conditions cannot be looked up: whether a lunch was for a collaborator, whether a hotel was booked for an upcoming trip. A fuzzy block is that question, answered by Jev with a probability. They are shown with a dashed border."]
    :origin-note "ownpurse's word, after “fuzzy” as in fuzzy matching: not a strict yes or no."
    :see [:block :noul :exact-block]}

   {:id :exception :term "Exception" :group :rules :origin :ownpurse :step 8
    :short "A rule that wins over others it names: hospitality wins over per diem when both apply."
    :long ["A lunch with a collaborator on a Harbour trip matches both the per diem rule and the hospitality rule. The hospitality rule lists per diem as a rule it sets aside, so when both apply, hospitality decides."]
    :origin-note "Plain word, as in “the exception to the rule”."
    :see [:rule :conflict]}

   {:id :check-rule :term "Check rule" :group :rules :origin :ownpurse :step 8
    :short "A rule that does not code anything: when it applies, the line goes to a person, however sure Jev is."
    :long ["Some situations must be looked at whatever the answer: cash withdrawals, transfers, a shop in a different country from where the cardholder was, a note that names another project. A check rule catches them and stops automatic coding."]
    :origin-note "ownpurse's word. Auditors would call these red flags."
    :see [:needs-a-person :rule]}

   {:id :conflict :term "Conflict (and no rule)" :group :rules :origin :ownpurse :step 8
    :short "When rules disagree about a line, or no rule applies at all: then the rules alone cannot decide."
    :long ["If two rules that are not exceptions to each other point at different buckets, it is a conflict. If none applies, there is no rule. Either way the rule-based strategy says “needs a person”, and the hybrid strategy falls back on Jev's own pick."]
    :origin-note "Plain words."
    :see [:exception :strategy]}

   ;; ----------------------------------------------------------------------------------- deciding
   {:id :strategy :term "Strategy" :group :deciding :origin :ownpurse :step 9
    :short "One way of turning Jev's answers and the rules into a coding. The sandbox compares seven."
    :long ["Jev with the rules picks the bucket itself, reading the rules as text. The rule blocks decide by the rules alone, Jev answering only the fuzzy blocks. Blocks, else Jev uses the rules when they decide and Jev's pick when they do not. Jev, stopped by checks is Jev's pick unless a check rule applies. And three more for comparison: without the rules, rule by rule, and with the options in reverse order."
           "All seven are worked out from the same answers, so comparing them costs nothing extra. In use, one will be chosen."]
    :origin-note "ownpurse's word for these alternatives."
    :see [:block :threshold]}

   {:id :threshold :term "Threshold" :group :deciding :origin :ml :step 10
    :short "The confidence a coding needs to be applied automatically. Below it, a person looks."
    :long ["Set the threshold at 85% and every coding at 85% or more is applied without asking; the rest go to a person. Raise it and fewer lines are coded wrongly, but more go to a person. The chart in the sandbox shows that trade-off at every setting."]
    :origin-note "Standard term in machine learning and automation (a decision threshold, or cut-off)."
    :see [:confidence :auto-coded :needs-a-person]}

   {:id :auto-coded :term "Coded automatically" :group :deciding :origin :banking :step 10
    :short "A line coded with no person looking, because its confidence reached the threshold and no check stopped it."
    :long ["The aim is to code as many lines as possible this way, without ever coding one wrongly. Banks and insurers call it straight-through processing."]
    :origin-note "Finance's “straight-through processing” (STP)."
    :see [:threshold :human-in-the-loop]}

   {:id :needs-a-person :term "Needs a person" :group :deciding :origin :ownpurse :step 10
    :short "The answer that means: do not code this automatically, show it to someone."
    :long ["It is a bucket like the others, so Jev can choose it, a check rule can force it, and the answer key can expect it (when a line truly cannot be coded without asking)."]
    :origin-note "ownpurse's bucket name."
    :see [:human-in-the-loop :check-rule]}

   {:id :human-in-the-loop :term "Human in the loop" :group :deciding :origin :ml :step 10
    :short "A person reviews what the machine is unsure about, and their answers teach the system."
    :long ["Nothing here replaces the bookkeeper: it sorts the lines so a person sees only the ones that need them. Their decisions can become new rules, so the same question is not asked twice."]
    :origin-note "Standard machine-learning term (HITL)."
    :see [:needs-a-person :rule]}

   ;; ----------------------------------------------------------------------------------- testing
   {:id :sandbox :term "Sandbox" :group :testing :origin :ml
    :short "A safe place to try rules on lines whose right answer is known, without touching any books."
    :long ["Nothing in the sandbox is posted anywhere. It exists to see, before trusting it with real books, how each rule set and strategy would have coded a set of lines."]
    :origin-note "Software term for an isolated place to experiment."
    :see [:scenario :run :answer-key]}

   {:id :scenario :term "Scenario" :group :testing :origin :ownpurse
    :short "An invented world for testing: people, cards, trips, projects, rules, and a few hundred bank lines."
    :long ["“atlas” is one: a studio with two projects, two travellers and four cards. Its lines are made up but realistic, including the awkward ones (tips posted days later, refunds, notes in Portuguese)."]
    :origin-note "ownpurse's word."
    :see [:answer-key :run]}

   {:id :answer-key :term "Answer key" :group :testing :origin :ml
    :short "The right bucket for every line in a scenario, written by a person, to mark the pipeline's work against."
    :long ["Each line says where it belongs (“should be”), any other bucket that is also fine, and whether a person must look at it however sure the answer. It is what lets the sandbox say right or wrong."]
    :origin-note "Machine learning calls it ground truth, or labels."
    :see [:outcome :scenario]}

   {:id :outcome :term "Outcome" :group :testing :origin :ownpurse :step 10
    :short "How a line fared at a threshold: coded right, coded wrong, coded unsafely, or sent to a person (rightly or needlessly)."
    :long ["Coded, right: applied automatically and correct. Coded, wrong: applied automatically to the wrong bucket. Coded, but a person had to look: correct or not, the answer key says a person must see it. To a person, rightly: sent to a person, and the answer key says a person should or may look. To a person, needlessly: sent to a person although the answer key needed no one."
           "The first is the goal. The second and third are the costly mistakes. The last is only wasted time."]
    :origin-note "ownpurse's five words, from comparing each coding with the answer key."
    :see [:answer-key :threshold]}

   {:id :run :term "Run" :group :testing :origin :ownpurse
    :short "One pass of every line in a scenario through Jev, saved, so it can be studied without asking again."
    :long ["A run keeps every request, every answer and every strategy's coding. The sandbox scores it at any threshold instantly, because nothing needs asking twice."]
    :origin-note "ownpurse's word (machine learning would say an experiment)."
    :see [:eval :scenario]}

   {:id :eval :term "Eval" :group :testing :origin :ml
    :short "Measuring how well the pipeline does, by running it on lines with known answers and counting."
    :long ["Short for evaluation. The sandbox is ownpurse's eval made visible."]
    :origin-note "Standard machine-learning term."
    :see [:run :answer-key :calibration]}])

(def by-id (into {} (map (juxt :id identity)) terms))

(defn href [id] (route/href ["docs" "terms"] {:term (name id)}))

(defn- said
  "The term as it reads mid-sentence: lower case, but for a name (Jev, Choice) and any parenthesis (MCC)."
  [{:keys [term proper]}]
  (let [t (str/replace term #" \(.*\)$" "")]
    (if proper t (str (str/lower-case (subs t 0 1)) (subs t 1)))))

(defui origin-badge [{:keys [origin]}]
  ($ :span {:class "inline-block rounded-1 border border-line-2 px-1.5 py-px font-mono text-2xs text-ink-2"}
     (origin-names origin)))

(defui term
  "A word from the vocabulary, dotted-underlined: hovering or focusing it shows what it means, and it links to the
  vocabulary's entry. `children` are the words as written (\"blocks\"); the term's name when there are none."
  [{:keys [id children]}]
  (let [{:keys [short origin] :as t} (by-id id)
        ;; Near the window's right edge the meaning opens leftwards, so it never runs off the page.
        [left? set-left!] (uix/use-state false)
        place! (fn [e] (set-left! (> (.-right (.getBoundingClientRect (.-currentTarget e))) (- js/innerWidth 300))))]
    (if-not t
      ($ :span children)
      ($ :span {:class "group/term relative inline" :on-pointer-enter place! :on-focus place!}
         ($ :a {:href (href id)
                :class "cursor-help underline decoration-line-2 decoration-dotted decoration-1 underline-offset-[3px] hover:decoration-lapis"}
            (or children (said t)))
         ($ :span {:role "tooltip"
                   :class (cx (if left? "right-0" "left-0")
                              "pointer-events-none absolute top-full z-50 mt-1.5 hidden w-72 rounded-2 border border-line-2"
                              "bg-surface-0 p-3 text-left font-sans text-xs font-normal tracking-normal normal-case text-ink-1"
                              ;; Out of the layout until shown: a hidden meaning near the edge must not widen the page.
                              "shadow-[var(--shadow-float)] animate-[fade_.12s_ease-out]"
                              "group-hover/term:block group-focus-within/term:block")}
            ($ :span {:class "flex items-baseline justify-between gap-2"}
               ($ :b {:class "text-sm"} (:term t)) ($ origin-badge {:origin origin}))
            ($ :span {:class "mt-1.5 block leading-normal text-ink-2"} short)
            ($ :span {:class "mt-2 block font-mono text-2xs text-lapis"} "Click for more in the vocabulary →"))))))
