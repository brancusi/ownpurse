(ns ownpurse.notebooks.cards
  "Bank lines into buckets: every line of an eval scenario as one row, with the bucket it belongs in and the bucket
  Jev chose with its confidence. A row opens to the raw line, its receipt, what code worked out, Jev's probabilities,
  and each payload sent to Jev with what came back. Filters at the top pick the wrong lines, those sent to a person,
  or the hard ones. Every name, number and place is invented.

  Params: `:scenario` (\"atlas\" when absent), `:run` (a run's file; the scenario's latest when absent),
  `:threshold` (0.7 when absent): `(show :cards {:scenario \"harbour\"})`."
  {:nextjournal.clerk/no-cache true
   :nextjournal.clerk/visibility {:code :hide :result :hide}}
  (:require
   [clojure.java.io :as io]
   [clojure.string :as str]
   [jsonista.core :as j]
   [nextjournal.clerk :as clerk]
   [ownpurse.classify.core :as core]
   [ownpurse.classify.interface :as classify]
   [ownpurse.notebook :as notebook]))

(def root
  "This repository's folder, from where the notebook is on the classpath."
  (-> (io/resource "ownpurse/notebooks/cards.clj") io/file
      .getParentFile .getParentFile .getParentFile .getParentFile .getParentFile))

(def params (notebook/params))

(def scenario-name (or (:scenario params) "atlas"))

(def run-file
  (or (some-> (:run params) io/file)
      (->> (.listFiles (io/file root "components/classify/runs"))
           (filter #(let [n (.getName ^java.io.File %)]
                      (and (str/starts-with? n (str scenario-name "-")) (str/ends-with? n ".edn"))))
           (sort-by #(.getName ^java.io.File %))
           last)))

(def run (classify/load-run run-file))

(def scenario (classify/scenario scenario-name))

(def threshold (bigdec (or (:threshold params) 0.7M)))

(def units (into {} (map (juxt :id identity)) (mapcat classify/units (:cases scenario))))

(def lines (into {} (map (juxt :id identity)) (:cases scenario)))

(def results (:results run))

(def groups
  "How the buckets are laid out: the scenario's :buckets, or, without them, its codes as one group."
  (or (:buckets scenario)
      [{:title "Codes" :codes (for [[c _] (:codes scenario)] [c (str/capitalize (str/replace (name c) "-" " "))])}]))

(def css
  "The page's look: ownpurse's design tokens (from the design system's tokens.css), its type, and the layout of the
  rows. Colour marks only an outcome, as the design system's coins do: verdigris right, coin wrong, gilt held for a
  person needlessly, ink sent to a person rightly."
  "
@import url('https://fonts.googleapis.com/css2?family=Bricolage+Grotesque:opsz,wght@12..96,400..800&family=DM+Mono:wght@400;500&display=swap');
.opx { --surface-0:#F5F3EE; --surface-1:#EDEBE4; --surface-2:#E2DFD6; --line-1:#DEDAD0; --line-2:#8F8C82;
  --ink-1:#23231F; --ink-2:#4D4C45; --ink-3:#6B695F; --coin:#D4402B; --coin-text:#B2341F; --coin-tint:#F8DDD5;
  --gilt:#B8862A; --gilt-text:#7F5C17; --gilt-tint:#F3E6C6; --verdigris:#2A6F5E; --verdigris-tint:#D6EAE3;
  --lapis:#2C5FA8; --lapis-tint:#DCE6F4;
  --display:'Bricolage Grotesque','Helvetica Neue',Arial,sans-serif; --mono:'DM Mono',ui-monospace,Menlo,monospace;
  font-family:var(--display); color:var(--ink-1); background:var(--surface-0); font-size:13px; line-height:1.35;
  padding:20px 24px 48px; border-radius:2px }
@media (prefers-color-scheme: dark) { .opx { --surface-0:#171715; --surface-1:#1F1F1C; --surface-2:#2A2A26;
  --line-1:#2E2D29; --line-2:#6E6C64; --ink-1:#EEEBE3; --ink-2:#C4C1B7; --ink-3:#9C998F; --coin:#EE5A3F;
  --coin-text:#F47D66; --coin-tint:#3A211B; --gilt:#E0B04F; --gilt-text:#E0B04F; --gilt-tint:#3A3020;
  --verdigris:#5FB39E; --verdigris-tint:#1C3330; --lapis:#86B2EA; --lapis-tint:#1E2A3D } }
.opx * { box-sizing:border-box }
.opx h1 { font:800 26px/1.1 var(--display); letter-spacing:-0.02em; margin:0 0 4px }
.opx h2 { font:500 11px/1.4 var(--mono); letter-spacing:.08em; text-transform:uppercase; color:var(--ink-3);
  margin:28px 0 8px; padding-bottom:6px; border-bottom:2px solid var(--ink-1) }
.opx .sub { color:var(--ink-3); margin:0 }
.opx .mono, .opx code { font-family:var(--mono); font-size:12px }
.opx a { color:inherit; text-decoration:none }
.opx .muted { color:var(--ink-3) }
/* the strip at the top: projects, people, cards, and the run's score */
.opx .strip { display:grid; grid-template-columns:repeat(auto-fit,minmax(260px,1fr)); gap:1px; background:var(--line-1);
  border:1px solid var(--line-1); margin-top:14px }
.opx .strip > div { background:var(--surface-0); padding:10px 12px }
.opx .label { font:500 10.5px/1.4 var(--mono); letter-spacing:.08em; text-transform:uppercase; color:var(--ink-3) }
.opx .score { display:flex; gap:18px; flex-wrap:wrap; align-items:baseline }
.opx .score b { font:700 22px/1 var(--display); margin-right:4px }
/* outcomes */
.opx .dot { width:9px; height:9px; border-radius:50%; display:inline-block; flex:none }
.opx .right .dot, .opx .dot.right { background:var(--verdigris) }
.opx .wrong .dot, .opx .dot.wrong { background:var(--coin) }
.opx .review .dot, .opx .dot.review { background:var(--gilt) }
.opx .stopped .dot, .opx .dot.stopped { background:transparent; box-shadow:inset 0 0 0 2px var(--ink-3) }
/* bucket tags: neutral; the project is the quiet prefix */
.opx .tag { display:inline-flex; gap:5px; align-items:baseline; font-size:12px; padding:2px 6px; border-radius:2px;
  background:var(--surface-2); color:var(--ink-1); white-space:nowrap; max-width:100%; overflow:hidden; text-overflow:ellipsis }
.opx .tag i { font-style:normal; font-family:var(--mono); font-size:10.5px; color:var(--ink-3) }
.opx .tag.miss { background:var(--coin-tint); color:var(--coin-text) }
.opx .tag.miss i { color:var(--coin-text) }
/* the buckets board */
.opx .board { display:grid; grid-template-columns:repeat(auto-fit,minmax(300px,1fr)); gap:16px }
.opx .group-h { font-weight:700; font-size:14px; margin-bottom:4px }
.opx .bucket { border-top:1px solid var(--line-1); padding:6px 0 7px; display:grid; grid-template-columns:110px 1fr; gap:8px }
.opx .bucket .n { font-family:var(--mono); font-size:11px; color:var(--ink-3) }
.opx .chips { display:flex; gap:3px; flex-wrap:wrap; align-content:flex-start }
.opx .chip { font:500 10.5px/1 var(--mono); padding:3px 4px; border-radius:2px; background:var(--surface-1); color:var(--ink-2) }
.opx .chip.right { background:var(--verdigris-tint); color:var(--verdigris) }
.opx .chip.wrong { background:var(--coin); color:#fff }
.opx .chip.review { background:var(--gilt-tint); color:var(--gilt-text) }
/* filters: radio buttons and :has(), no script */
.opx .filters { display:flex; gap:4px; flex-wrap:wrap; margin:0 0 8px; position:sticky; top:0; z-index:2;
  background:var(--surface-0); padding:6px 0 }
.opx .filters input { position:absolute; opacity:0; pointer-events:none }
.opx .filters label { display:inline-flex; gap:6px; align-items:center; padding:5px 9px; border:1px solid var(--line-2);
  border-radius:2px; cursor:pointer; font-size:12px; user-select:none }
.opx .filters label b { font-family:var(--mono); font-weight:500; color:var(--ink-3) }
.opx .filters input:checked + label { background:var(--ink-1); color:var(--surface-0); border-color:var(--ink-1) }
.opx .filters input:checked + label b { color:var(--surface-0) }
.opx .filters input:focus-visible + label { outline:2px solid var(--lapis); outline-offset:2px }
.opx:has(#f-wrong:checked) .row:not(.wrong), .opx:has(#f-review:checked) .row:not(.review),
.opx:has(#f-stopped:checked) .row:not(.stopped), .opx:has(#f-right:checked) .row:not(.right),
.opx:has(#f-hard:checked) .row:not(.hard) { display:none }
/* the rows */
.opx .rows { border-top:2px solid var(--ink-1) }
.opx .cols, .opx .row > summary { display:grid; align-items:center; gap:10px;
  grid-template-columns:12px 52px 78px minmax(180px,1fr) 104px minmax(150px,210px) minmax(190px,270px) 84px 14px }
.opx .cols { font:500 10.5px/1.4 var(--mono); letter-spacing:.08em; text-transform:uppercase; color:var(--ink-3);
  padding:6px 8px; background:var(--surface-1); border-bottom:1px solid var(--line-1) }
.opx .row { border-bottom:1px solid var(--line-1) }
.opx .row > summary { list-style:none; cursor:pointer; padding:5px 8px; min-height:30px }
.opx .row > summary::-webkit-details-marker { display:none }
.opx .row > summary:hover { background:var(--surface-1) }
.opx .row[open] > summary { background:var(--surface-1) }
.opx .row.wrong > summary { box-shadow:inset 3px 0 0 var(--coin) }
.opx .row .id { font-family:var(--mono); font-size:12px; color:var(--ink-2) }
.opx .row .what { min-width:0; white-space:nowrap; overflow:hidden; text-overflow:ellipsis }
.opx .row .what .item { color:var(--ink-2) }
.opx .row .what .note { color:var(--ink-3); font-size:12px }
.opx .amt { font-family:var(--mono); font-size:12.5px; font-variant-numeric:tabular-nums; text-align:right; white-space:nowrap }
.opx .amt .cur { color:var(--ink-3); font-size:10.5px; margin-right:4px }
.opx .amt.neg { color:var(--coin-text) }
.opx .jev { display:flex; gap:6px; align-items:center; min-width:0 }
.opx .conf { display:inline-flex; gap:5px; align-items:center; font-family:var(--mono); font-size:11px; color:var(--ink-2); flex:none }
.opx .bar { width:36px; height:4px; background:var(--surface-2); display:inline-block; position:relative }
.opx .bar > span { position:absolute; inset:0 auto 0 0; background:var(--ink-2) }
.opx .bar.low > span { background:var(--line-2) }
.opx .blk { font-family:var(--mono); font-size:11px; color:var(--ink-3); white-space:nowrap; overflow:hidden; text-overflow:ellipsis }
.opx .blk.differs { color:var(--gilt-text) }
.opx .chev { color:var(--ink-3); font-size:10px; transition:transform .12s }
.opx .row[open] .chev { transform:rotate(90deg) }
/* a row opened */
.opx .open-body { display:grid; grid-template-columns:minmax(240px,1fr) minmax(260px,1.2fr) minmax(260px,1.2fr); gap:20px;
  padding:10px 8px 14px 30px; background:var(--surface-1); border-top:1px dashed var(--line-1) }
.opx .kv { display:grid; grid-template-columns:58px minmax(0,1fr) 58px minmax(0,1fr); gap:3px 10px; font-size:12px;
  margin:4px 0 0; align-items:baseline }
.opx .kv dt, .opx .kv dd { margin:0; padding:0; line-height:1.4 }
.opx .kv dt { font-family:var(--mono); font-size:11px; color:var(--ink-3); font-weight:400 }
.opx .kv dd { min-width:0; overflow-wrap:anywhere }
.opx .kv .wide { grid-column:2 / -1 }
.opx .kv .wide-k { grid-column:1 }
.opx p, .opx ul, .opx li { margin-top:0; margin-bottom:0 }
.opx .receipt { margin:8px 0 0; padding:0; list-style:none; font-size:12px }
.opx .receipt li { display:flex; justify-content:space-between; gap:10px; padding:2px 0; border-bottom:1px dotted var(--line-1) }
.opx .receipt li.this { font-weight:700 }
.opx .probs { display:grid; grid-template-columns:minmax(0,1fr) 90px 34px; gap:3px 8px; align-items:center; font-size:12px; margin-top:4px }
.opx .probs .bar { width:90px }
.opx .probs .p { font-family:var(--mono); font-size:11px; text-align:right; color:var(--ink-2) }
.opx .say { font-size:12px; color:var(--ink-2); margin:6px 0 0 }
.opx .say b { color:var(--ink-1) }
.opx .why { font-size:12px; margin-top:8px; padding-left:8px; border-left:2px solid var(--line-2); color:var(--ink-2) }
.opx .calls { grid-column:1 / -1 }
.opx .calls summary { cursor:pointer; color:var(--lapis); font-size:12px }
.opx .calls .grid { display:grid; grid-template-columns:repeat(auto-fit,minmax(360px,1fr)); gap:10px; margin-top:8px }
.opx pre { font:11px/1.45 var(--mono); background:var(--surface-0); border:1px solid var(--line-1); border-radius:2px;
  padding:8px; white-space:pre-wrap; margin:3px 0 0; max-height:420px; overflow:auto; color:var(--ink-2) }
@media (max-width: 900px) {
  .opx { padding:16px }
  .opx .cols { display:none }
  .opx .row > summary { grid-template-columns:12px 46px minmax(0,1fr) auto; row-gap:4px }
  .opx .row > summary > .date, .opx .row > summary > .blk, .opx .row > summary > .chev { display:none }
  .opx .row > summary > .exp, .opx .row > summary > .jev { grid-column:3 / -1 }
  .opx .open-body { grid-template-columns:1fr; padding-left:12px }
}
")

(def group-of
  (into {} (for [{:keys [project title codes]} groups
                 [code label] codes]
             [code {:group (or (get-in scenario [:projects project :name]) title) :label label}])))

(defn- short-group [g] (first (str/split (str g) #" ")))

(defn- pct [p] (str (.setScale (.multiply (bigdec p) 100M) 0 java.math.RoundingMode/HALF_UP) "%"))

(defn- tag
  ([code] (tag code nil))
  ([code cls]
   (let [{:keys [group label]} (group-of code {:group "" :label (name code)})
         solo (#{"Studio" "Personal, or a person to look"} group)]
     [:span {:class (str "tag " cls) :title (get-in scenario [:codes code])}
      (when-not solo [:i (short-group group)]) label])))

(defn- amount [{:keys [amount currency]}]
  [:span {:class (str "amt" (when (neg? amount) " neg"))} [:span.cur currency] (.toPlainString ^BigDecimal amount)])

(defn- bar [p]
  [:span {:class (str "bar" (when (< p threshold) " low"))} [:span {:style {:width (pct p)}}]])

(defn- outcome [{:keys [id decisions]}]
  (classify/outcome (units id) (:choice decisions) threshold))

(def outcome-words
  {:right "coded, right" :wrong "coded, wrong" :stopped "to a person, rightly" :review "to a person, needlessly"})

(defn- json [x]
  [:pre (j/write-value-as-string x (j/object-mapper {:pretty true}))])

(defn- where-text [{:keys [whereabouts]}]
  (let [{:keys [place city trip project day days]} whereabouts
        at (if city (str city ", " place) place)]
    (case trip
      :project (str "in " at ", on a trip for " (get-in scenario [:projects project :name]) ", day " day " of " days)
      :personal (str "in " at ", on a personal trip")
      (str "at home in " at))))

(def by-outcome (group-by outcome results))

(defn- head []
  (let [home (:home scenario)]
    [:div
     [:h1 "Bank lines into buckets"]
     [:p.sub "Every line of the " [:b scenario-name] " scenario: where it belongs, and where Jev put it. "
      "Every name, number and place is invented. Click a line to open it."]
     [:div.strip
      (for [[id {:keys [name place from to work travellers]}] (:projects scenario)]
        [:div
         [:div.label name]
         [:div place " · " [:span.mono from " → " to]]
         (when work [:div.muted {:style {:margin-top 2}} work])
         [:div {:style {:margin-top 4}}
          (str/join " · " (for [{:keys [person city place from to project]} (:trips scenario) :when (= id project)]
                            (str (get-in scenario [:entities person :name]) " in " (or city place) " "
                                 (subs from 5) "–" (subs to 5))))]
         [:div.muted "Travellers: " (str/join ", " (map #(get-in scenario [:entities % :name]) travellers))]])
      [:div
       [:div.label "Home and cards"]
       [:div "At home in " (if (map? home) (str (:city home) ", " (:country home)) home) " on any other day."]
       (for [[id {:keys [name holder]}] (:cards scenario)]
         [:div [:span.mono id] " " [:span.muted name (when-not (str/includes? name "held by")
                                                        (str ", held by " (get-in scenario [:entities holder :name])))]])]
      [:div
       [:div.label "This run, automatic at " (pct threshold)]
       [:div.score {:style {:margin-top 4}}
        (for [o [:right :wrong :stopped :review]]
          [:span {:class (name o)} [:b (count (by-outcome o))] [:span.dot] " " (outcome-words o)])]
       [:div.muted {:style {:margin-top 6}}
        [:span.mono (.getName ^java.io.File run-file)] " · " [:span.mono (:model run)] " · " (count results)
        " lines and items"]]]]))

(defn- board []
  (let [expected (frequencies (map #(:expect (units (:id %))) results))
        placed (group-by #(get-in % [:decisions :choice :code]) results)]
    [:div
     [:h2 "The buckets: how many belong in each, and the lines Jev put there"]
     [:div.board
      (for [{:keys [project title codes]} groups]
        [:div
         [:div.group-h (or (get-in scenario [:projects project :name]) title)]
         (for [[code label] codes]
           [:div.bucket
            [:div [:div {:title (get-in scenario [:codes code])} label]
             [:div.n (get expected code 0) " belong · " (count (placed code)) " put"]]
            [:div.chips
             (for [{:keys [id] :as r} (sort-by :id (placed code))
                   :let [o (outcome r)]]
               [:a {:class (str "chip " (name o)) :href (str "#" id) :title (outcome-words o)} id])]])])]]))

(defn- open-body [{:keys [id facts decisions responses]}]
  (let [u (units id)
        line (lines (or (:line u) id))
        blocks (:blocks decisions)
        requests (core/requests scenario facts)
        enrich (core/enrichment scenario u)]
    [:div.open-body
     [:div
      [:div.label "The raw bank line"]
      [:dl.kv
       (for [k [:date :card :merchant :mcc :amount :currency :country :city :channel :note]
             :let [v (get line k)] :when (some? v)]
         (list [:dt {:class (when (#{:merchant :note} k) "wide-k")} (name k)]
               [:dd {:class (when (#{:merchant :note} k) "wide")}
                (cond (decimal? v) (.toPlainString ^BigDecimal v) (keyword? v) (name v) :else (str v))]))]
      (when-let [items (:items line)]
        [:div {:style {:margin-top 8}}
         [:div.label "The " (name (:items-from line)) ", scanned apart"]
         [:ul.receipt
          (for [{:keys [item amount]} items]
            [:li {:class (when (= item (:item u)) "this")} [:span item] [:span.amt (.toPlainString ^BigDecimal amount)]])]])]
     [:div
      [:div.label "Jev's Choice, with the rules"]
      [:div.probs
       (for [[option p] (take 5 (sort-by val > (get-in responses [:rules :answers :code :probabilities])))]
         (list [:span (tag (keyword (str/replace option "_" "-")))] (bar p) [:span.p (pct p)]))]
      [:p.say "Belongs in " (tag (:expect u))
       (when (seq (:ok u)) (list " · also fine " (interpose " " (map tag (:ok u)))))]]
     [:div
      [:div.label "What code worked out"]
      [:p.say (get-in scenario [:entities (:holder facts) :name]) " was " (where-text facts)
       (when (:country-mismatch facts) [:b "; the shop is not where they were recorded to be"]) "."]
      [:p.say "The rule blocks said " (tag (:code blocks))
       (when-let [by (seq (:by blocks))] [:span.mono " " (str/join " " by)])
       (when (:reason blocks) [:span.muted " (" (name (:reason blocks)) ")"])]
      (when-let [r (:refund-of facts)] [:p.say "Refunds " [:a.mono {:href (str "#" (:id r))} (:id r)] ", " (:date r) "."])
      (when-let [why (or (:hard u) (:why u))]
        [:div.why [:span.label (if (:hard u) "Hard " "Why ")] why])]
     [:details.calls
      [:summary "The calls to Jev: the facts, each payload and what came back"]
      [:div.grid
       [:div [:div.label "The facts code worked out"] (json (dissoc facts :id :line))]
       (when enrich
         [:div [:div.label "First: which kind of merchant"] (json (assoc enrich :model (:model run)))
          (json (get-in responses [:enrich :answers]))])
       [:div [:div.label "Call 1: with the rules"] (json (assoc (:rules requests) :model (:model run)))]
       [:div [:div.label "Call 1: what came back"] (json (get-in responses [:rules :answers]))]
       [:div [:div.label "Call 2: without the rules"] (json (assoc (:plain requests) :model (:model run)))]
       [:div [:div.label "Call 2: what came back"] (json (get-in responses [:plain :answers]))]]]]))

(defn- row [{:keys [id decisions] :as result}]
  (let [u (units id)
        {:keys [code confidence]} (:choice decisions)
        o (outcome result)
        acceptable (conj (set (:ok u)) (:expect u))
        blk (:code (:blocks decisions))]
    [:details {:id id :class (str "row " (name o) (when (:hard u) " hard"))}
     [:summary
      [:span.dot {:title (outcome-words o)}]
      [:span.id id]
      [:span.mono.muted.date (:date u)]
      [:span.what (:merchant u)
       (when (:item u) [:span.item " · " (:item u)])
       (when-let [n (:note u)] [:span.note "  " n])]
      (amount u)
      [:span.exp (tag (:expect u))]
      [:span.jev (tag code (when-not (acceptable code) "miss")) [:span.conf (bar confidence) (pct confidence)]]
      [:span {:class (str "blk" (when (not= blk code) " differs")) :title (str "The rule blocks said " (name blk))}
       (if (= blk code) "same" (str/replace (name blk) #"^(harbour|skyline)-|^needs-" ""))]
      [:span.chev "▶"]]
     (open-body result)]))

(defn- every-line []
  [:div
   [:h2 "Every line"]
   [:div.filters
    (for [[k words n] (concat [[:all "All" (count results)]]
                              (for [o [:wrong :review :stopped :right]] [o (outcome-words o) (count (by-outcome o))])
                              [[:hard "marked hard" (count (filter #(:hard (units (:id %))) results))]])]
      (list [:input {:type "radio" :name "f" :id (str "f-" (name k)) :default-checked (= :all k)}]
            [:label {:for (str "f-" (name k))} (when-not (#{:all :hard} k) [:span {:class (str "dot " (name k))}])
             words [:b n]]))]
   [:div.rows
    [:div.cols [:span] [:span "line"] [:span "date"] [:span "merchant · item · note"] [:span {:style {:text-align "right"}} "amount"]
     [:span "belongs in"] [:span "Jev chose"] [:span "blocks"] [:span]]
    (map row (sort-by #(case (outcome %) :wrong 0 :review 1 :stopped 2 3) results))]])

^{:nextjournal.clerk/visibility {:result :show}}
(clerk/html {:nextjournal.clerk/width :full}
            [:div.opx [:style css] (head) (board) (every-line)])
