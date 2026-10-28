(ns ownpurse.notebooks.cards
  "Bank lines into buckets: every line of an eval scenario as one row, with the bucket it belongs in and the bucket
  Jev chose with its confidence. A row opens to the line's way through, in seven steps: the bank line; what code works
  out (only the open row's, worked out when it opens); what Jev reads (the state exactly as sent, field by field, with how code made each field and where to change
  it, and what Jev is not sent); what Jev is asked (the questions word for word, every option as Jev reads it, with
  its probability); the rules applied; the decision; and where to change it, with the levers this line points to
  marked. Filters at the top pick the wrong lines, those sent to a person, or the hard ones. Every name, number and
  place is invented.

  Params: `:scenario` (\"atlas\" when absent), `:run` (a run's file; the scenario's latest when absent),
  `:threshold` (0.85 when absent): `(show :cards {:scenario \"harbour\"})`."
  {:nextjournal.clerk/no-cache true
   :nextjournal.clerk/visibility {:code :hide :result :hide}}
  (:require
   [clojure.edn :as edn]
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
                      (and (str/starts-with? n (str scenario-name "-")) (str/ends-with? n ".edn")
                           (not (re-find #"\.(notes|partial)\.edn$" n)))))
           (sort-by #(.getName ^java.io.File %))
           last)))

(def run (classify/load-run run-file))

(def scenario (classify/scenario scenario-name))

(def threshold (bigdec (or (:threshold params) 0.85M)))

(def units (into {} (map (juxt :id identity)) (mapcat classify/units (:cases scenario))))

(def lines (into {} (map (juxt :id identity)) (:cases scenario)))

(def results (:results run))

(def notes
  "Commentary on this run's failing lines, by line id, from the notes file beside the run (its name with .notes.edn),
  when there is one."
  (let [f (io/file (.getParentFile ^java.io.File run-file) (str/replace (.getName ^java.io.File run-file) #"\.edn$" ".notes.edn"))]
    (when (.exists f) (edn/read-string (slurp f)))))

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
.opx .unsafe .dot, .opx .dot.unsafe { background:var(--coin-tint); box-shadow:inset 0 0 0 2px var(--coin) }
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
.opx .chip.unsafe { background:var(--coin-tint); color:var(--coin-text); box-shadow:inset 0 0 0 1px var(--coin) }
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
.opx:has(#f-wrong:checked) .row:not(.wrong), .opx:has(#f-unsafe:checked) .row:not(.unsafe), .opx:has(#f-review:checked) .row:not(.review),
.opx:has(#f-stopped:checked) .row:not(.stopped), .opx:has(#f-right:checked) .row:not(.right),
.opx:has(#f-hard:checked) .row:not(.hard), .opx:has(#f-noted:checked) .row:not(.noted) { display:none }
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
.opx .row.wrong > summary, .opx .row.unsafe > summary { box-shadow:inset 3px 0 0 var(--coin) }
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
.opx .open-body { padding:12px 12px 16px 30px; background:var(--surface-1); border-top:1px dashed var(--line-1) }
/* the verdict and the commentary */
.opx .verdict { display:flex; flex-wrap:wrap; gap:6px 14px; align-items:center; font-size:13px; padding-bottom:10px }
.opx .verdict .arrow { color:var(--ink-3) }
.opx .pill { font:500 11px/1 var(--mono); padding:4px 7px; border-radius:2px; white-space:nowrap }
.opx .pill.right { background:var(--verdigris-tint); color:var(--verdigris) }
.opx .pill.wrong { background:var(--coin); color:#fff }
.opx .pill.unsafe { background:var(--coin-tint); color:var(--coin-text); box-shadow:inset 0 0 0 1px var(--coin) }
.opx .pill.review { background:var(--gilt-tint); color:var(--gilt-text) }
.opx .pill.stopped { background:var(--surface-2); color:var(--ink-2) }
.opx .comment { display:grid; grid-template-columns:110px 1fr; gap:4px 12px; background:var(--surface-0);
  border:1px solid var(--line-1); border-left:3px solid var(--gilt); padding:10px 12px; margin-bottom:12px; font-size:12.5px;
  line-height:1.5 }
.opx .row.wrong .comment, .opx .row.unsafe .comment { border-left-color:var(--coin) }
.opx .comment .label { padding-top:2px }
/* the steps */
.opx .step { display:grid; grid-template-columns:170px minmax(0,1fr); gap:14px; padding:10px 0; border-top:1px solid var(--line-1) }
.opx .step-h { display:flex; flex-direction:column; gap:3px }
.opx .step-h .n { font:700 11px/1 var(--mono); width:20px; height:20px; border-radius:50%; background:var(--ink-1);
  color:var(--surface-0); display:inline-flex; align-items:center; justify-content:center }
.opx .step-h .t { font-weight:700; font-size:13px }
.opx .step-h .where { font-family:var(--mono); font-size:10.5px; color:var(--ink-3) }
.opx .cols2 { display:grid; grid-template-columns:repeat(auto-fit,minmax(300px,1fr)); gap:16px }
.opx .facts { display:grid; grid-template-columns:120px minmax(0,1fr) 170px; gap:3px 12px; font-size:12px; align-items:baseline }
.opx .facts .k { font-family:var(--mono); font-size:11px; color:var(--ink-3) }
.opx .facts .src { font-family:var(--mono); font-size:10.5px; color:var(--ink-3) }
.opx .facts .flag { color:var(--coin-text); font-weight:600 }
/* the rules table */
.opx .rules { display:grid; grid-template-columns:14px 34px minmax(130px,170px) minmax(0,1fr) 54px 54px; gap:4px 10px;
  font-size:12px; align-items:baseline }
.opx .rules .h { font:500 10px/1.4 var(--mono); letter-spacing:.06em; text-transform:uppercase; color:var(--ink-3) }
.opx .rules .rid { font-family:var(--mono); font-size:11.5px; font-weight:500 }
.opx .rules .num { font-family:var(--mono); font-size:11px; text-align:right }
.opx .rules .text { color:var(--ink-2); font-size:11.5px; margin-top:2px }
.opx .rules .off { opacity:.55 }
.opx .st { width:9px; height:9px; border-radius:50%; display:inline-block }
.opx .st.applies { background:var(--ink-1) }
.opx .st.check { background:var(--coin) }
.opx .st.overruled { background:transparent; box-shadow:inset 0 0 0 2px var(--ink-2) }
.opx .st.no { background:var(--surface-2) }
.opx .blocks { display:flex; flex-wrap:wrap; gap:3px }
.opx .b { display:inline-flex; gap:5px; align-items:baseline; font-size:11.5px; padding:1px 6px; border-radius:2px;
  border:1px solid var(--line-1); background:var(--surface-0) }
.opx .b .v { font:500 10.5px/1 var(--mono) }
.opx .b.yes { border-color:var(--verdigris) } .opx .b.yes .v { color:var(--verdigris) }
.opx .b.no { color:var(--ink-3) }
.opx .b.fuzzy { border-style:dashed }
.opx .more summary { cursor:pointer; color:var(--ink-3); font-size:11.5px; margin-top:6px }
.opx .decide { display:grid; grid-template-columns:repeat(auto-fit,minmax(240px,1fr)); gap:14px }
.opx .decide > div { background:var(--surface-0); border:1px solid var(--line-1); padding:8px 10px }
.opx .payload summary { cursor:pointer; color:var(--lapis); font-size:11.5px; margin-top:6px }
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
.opx .payload .grid { display:grid; grid-template-columns:repeat(auto-fit,minmax(340px,1fr)); gap:10px; margin-top:6px }
.opx .has-note { font:500 10px/1 var(--mono); color:var(--gilt-text); margin-left:6px }
.opx pre { font:11px/1.45 var(--mono); background:var(--surface-0); border:1px solid var(--line-1); border-radius:2px;
  padding:8px; white-space:pre-wrap; margin:3px 0 0; max-height:420px; overflow:auto; color:var(--ink-2) }
/* what Jev reads: the state, field by field */
.opx .reads { display:grid; grid-template-columns:minmax(150px,240px) minmax(0,1.3fr) minmax(180px,1fr); gap:0;
  font-size:12px; border:1px solid var(--line-1); background:var(--surface-0) }
.opx .reads > div { padding:3px 8px; border-top:1px solid var(--line-1); min-width:0; overflow-wrap:anywhere }
.opx .reads .sec { grid-column:1 / -1; font:500 10.5px/1.4 var(--mono); letter-spacing:.08em; text-transform:uppercase;
  color:var(--ink-3); background:var(--surface-1) }
.opx .reads .path { font-family:var(--mono); font-size:11px; color:var(--ink-2) }
.opx .reads .val { color:var(--ink-1) }
.opx .reads .val ul { padding-left:16px }
.opx .reads .how { font-size:11px; color:var(--ink-3) }
.opx .reads .how code { font-size:10.5px; color:var(--lapis) }
.opx .reads .new { box-shadow:inset 3px 0 0 var(--gilt) }
.opx .unsent { margin-top:8px; font-size:12px; color:var(--ink-2); border-left:2px solid var(--line-2); padding-left:8px }
.opx .unsent li { margin-left:16px }
/* what Jev is asked */
.opx .ask { background:var(--surface-0); border:1px solid var(--line-1); padding:8px 10px; margin-bottom:10px }
.opx .ask .q { font-size:12.5px; line-height:1.5; margin:4px 0 8px; padding-left:8px; border-left:2px solid var(--ink-1) }
.opx .opts { display:grid; grid-template-columns:minmax(160px,200px) minmax(0,1fr) 90px 38px; gap:3px 10px; font-size:12px;
  align-items:center }
.opx .opts .h { font:500 10px/1.4 var(--mono); letter-spacing:.06em; text-transform:uppercase; color:var(--ink-3) }
.opx .opts .desc { color:var(--ink-2); font-size:11.5px }
.opx .opts .p { font-family:var(--mono); font-size:11px; text-align:right }
.opx .opts .exp { font:500 9.5px/1 var(--mono); color:var(--verdigris); margin-left:4px; text-transform:uppercase }
.opx .opts .top { font-weight:700 }
.opx .opts .bar { width:90px }
/* where to change it */
.opx .levers { display:grid; grid-template-columns:minmax(200px,1fr) minmax(200px,1.2fr) minmax(160px,.8fr); gap:0;
  font-size:12px; border:1px solid var(--line-1); background:var(--surface-0) }
.opx .levers > div { padding:6px 8px; border-top:1px solid var(--line-1) }
.opx .levers .h { font:500 10px/1.4 var(--mono); letter-spacing:.06em; text-transform:uppercase; color:var(--ink-3);
  border-top:0; background:var(--surface-1) }
.opx .levers code { font-size:11px; color:var(--lapis) }
.opx .levers .hit { background:var(--gilt-tint) }
@media (max-width: 900px) {
  .opx { padding:16px }
  .opx .cols { display:none }
  .opx .row > summary { grid-template-columns:12px 46px minmax(0,1fr) auto; row-gap:4px }
  .opx .row > summary > .date, .opx .row > summary > .blk, .opx .row > summary > .chev { display:none }
  .opx .row > summary > .exp, .opx .row > summary > .jev { grid-column:3 / -1 }
  .opx .open-body { padding-left:12px }
  .opx .step, .opx .comment { grid-template-columns:1fr }
  .opx .facts { grid-template-columns:100px minmax(0,1fr) }
  .opx .facts .src { display:none }
  .opx .rules { grid-template-columns:14px 34px minmax(0,1fr) 44px }
  .opx .rules .c-code, .opx .rules .c-jev { display:none }
  .opx .reads, .opx .levers { grid-template-columns:1fr }
  .opx .opts { grid-template-columns:minmax(0,1fr) 60px 38px }
  .opx .opts .desc { display:none }
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
  {:right "coded, right" :wrong "coded, wrong" :unsafe "coded, but a person had to look" :stopped "to a person, rightly"
   :review "to a person, needlessly"})

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
        (for [o classify/outcomes]
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

(defn- block-text
  "A rule block in words; a category block names the line's own category when it is one of the block's."
  [[op & args] facts]
  (case op
    :category (let [codes (second args)
                    names (map #(get-in scenario [:categories %] %) (sort codes))]
                (cond
                  (contains? codes (:mcc facts)) (str "category is " (:category facts) " (one of " (count codes) ")")
                  (> (count names) 2) (str "category is one of " (count codes) ": " (first names) ", …")
                  :else (str "category is " (str/join " or " names))))
    :on-trip-for (str "on a trip for " (get-in scenario [:projects (first args) :name]))
    :trip (case (first args) :personal "on a personal trip" :project "on a project trip" "on a trip")
    :at-home "at home"
    :card (str "card is " (str/join " or " (sort (second args))))
    :channel (str "paid by " (str/join " or " (map name (second args))))
    :country-mismatch "the shop is not where the cardholder was"
    :present (str "the line has a " (name (first args)))
    :traveller "the cardholder travels for a running project"
    :during-project "a project is running"
    :jev (first args)
    :not (str "not: " (block-text (first args) facts))
    :any (str "any of: " (str/join "; " (map #(block-text % facts) args)))
    (pr-str (into [op] args))))

(defn- block-chip [facts {:keys [block holds]}]
  (let [fuzzy? (some #{:jev} (flatten (seq block)))
        yes? (>= holds 0.5M)]
    [:span {:class (str "b " (if yes? "yes" "no") (when fuzzy? " fuzzy"))
            :title (if fuzzy? "A yes/no question Jev answered, asked without the rules (call 2)" "Exact: code checks it")}
     (block-text block facts) [:span.v (if fuzzy? (pct holds) (if yes? "yes" "no"))]]))

(defn- rule-rows [facts rows overruled-by]
  (for [{:keys [id code check rule blocks holds jev]} rows
        :let [applies (>= holds 0.5M)
              by (overruled-by id)
              status (cond (and applies check) "check" (and applies by) "overruled" applies "applies" :else "no")]]
    (list
     [:span {:class (str "st " status)
             :title ({"check" "a check that applies: a person must look" "overruled" (str "applies, but set aside by " by)
                      "applies" "applies" "no" "does not apply"} status)}]
     [:span.rid {:class (when-not applies "off")} id]
     [:span.c-code {:class (when-not applies "off")} (tag code)]
     [:span {:class (when-not applies "off")}
      [:span.blocks (map #(block-chip facts %) blocks)]
      [:div.text rule (when by [:b " Set aside here by " by ", its exception."])
       (when-let [over (seq (:over (some #(when (= id (:id %)) %) (:rules scenario))))]
         [:span.muted " Exception to " (str/join ", " over) "."])]]
     [:span.num {:class (when-not applies "off")} (pct holds)]
     [:span.num.c-jev {:class (when-not applies "off") :title "Jev's own reading of the whole rule, given every rule (call 1)"}
      (when jev (pct jev))])))

(defn- step [n title where & body]
  [:div.step
   [:div.step-h [:span.n n] [:span.t title] [:span.where where]]
   (into [:div] body)])

(def ^:private facts-clj "facts.clj")
(def ^:private core-clj "core.clj")
(def ^:private scenario-edn (str scenario-name ".edn"))

(def ^:private paths
  "Each file a lever names, by its short name: where it is in the repository."
  {"facts.clj" "components/classify/src/ownpurse/classify/facts.clj"
   "core.clj" "components/classify/src/ownpurse/classify/core.clj"
   (str scenario-name ".edn") (str "components/classify/resources/classify/eval/" scenario-name ".edn")})

(defn- where-code
  "A lever's place, its files by short name, the full path on hover."
  [where]
  [:code {:title (str/join "\n" (keep #(paths (re-find #"^[^ ,;]+" (str/trim %))) (str/split where #"[,;]")))} where])

(def ^:private field-sources
  "How code made each field of the state Jev reads, and where to change it: [how, where]."
  {[:transaction :merchant] ["the bank line's merchant, as the bank sends it" "the bank's data"]
   [:transaction :amount] ["the bank line's amount and currency; for an item, the whole line's" "the bank's data"]
   [:transaction :how] ["the bank line's channel, in words" (str facts-clj " · state")]
   [:transaction :category] ["the bank's merchant code turned into its name by the scenario's list of categories; with no code, Jev's guess from the merchant's name (the enrichment call)" (str scenario-edn " · :categories")]
   [:transaction :merchant_country] ["the bank line's country" "the bank's data"]
   [:transaction :merchant_city] ["the bank line's city" "the bank's data"]
   [:transaction :note] ["the note the cardholder typed on the line" "the bank's data"]
   [:transaction :item] ["one item of the receipt or order, scanned apart from the bank line; the rest of it beside" (str scenario-edn " · the case's :items")]
   [:context :paid_with] ["the card's name, from the list of cards" (str scenario-edn " · :cards")]
   [:context :cardholder] ["who holds the card" (str scenario-edn " · :cards")]
   [:context :cardholder_was] ["where the cardholder was that day: the trips, by date (none on record: at home)" (str scenario-edn " · :trips; " facts-clj " · whereabouts")]
   [:context :projects_running] ["the projects whose dates include the day, with what each makes" (str scenario-edn " · :projects")]
   [:context :travellers_of_running_projects] ["each running project's travellers" (str scenario-edn " · :projects :travellers")]
   [:context :cardholder_travels_for_a_running_project] ["whether the cardholder is one of those travellers" (str facts-clj " · enrich")]
   [:context :cardholder_upcoming_trips] ["the cardholder's trips starting in the next 90 days" (str facts-clj " · upcoming-trips")]
   [:context :shop_is_where_cardholder_was] ["for a purchase in person: the shop's city and country against where the cardholder was" (str facts-clj " · enrich")]
   [:context :refund_of] ["for a refund: the earlier purchase it returns, same card, merchant and currency, by amount" (str facts-clj " · refund-of")]
   [:context :earlier_charge_here] ["the latest other charge on the same card at the same merchant in the week before" (str facts-clj " · earlier-here")]
   [:rules] ["every rule's id and text; not its exceptions (:over) nor its conditions (:when). Sent in call 1 only" (str scenario-edn " · :rules")]})

(defn- value [v]
  (cond (map? v) [:ul (for [[k x] v] [:li [:span.mono.muted (name k) ": "] (value x)])]
        (sequential? v) (if (every? map? v)
                          [:ul (for [x v] [:li (if (:id x) (list [:b.mono (:id x)] " " (:rule x)) (value x))])]
                          [:ul (for [x v] [:li (value x)])])
        (boolean? v) [:span.mono (str v)]
        :else (str v)))

(defn- reads
  "The state as Jev reads it: each field, its value exactly as sent, how code made it and where to change it."
  [state]
  [:div.reads
   (for [sec [:transaction :context]]
     (list [:div.sec (str (name sec) " — " (if (= :transaction sec) "the line itself" "what code adds about it"))]
           (for [[k v] (get state sec)
                 :let [[how where] (field-sources [sec k] ["" ""])]]
             (list [:div.path (str (name sec) "." (name k))] [:div.val (value v)]
                   [:div.how how (when (seq where) (list " · " (where-code where)))]))))
   (when-let [rules (:rules state)]
     (let [[how where] (field-sources [:rules])]
       (list [:div.sec "rules — sent with call 1 only"]
             [:div.path "rules"]
             [:div.val [:details [:summary {:style {:cursor "pointer"}} (count rules) " rules, each its id and text"]
                        (value rules)]]
             [:div.how how " · " (where-code where)])))])

(defn- options
  "A Choice's options as Jev reads them, each with its description and the probability Jev gave it; the buckets the
  answer key accepts are marked."
  [criteria probabilities u]
  (let [ok (conj (set (:ok u)) (:expect u))
        top (key (apply max-key val probabilities))]
    [:div.opts
     [:span.h "option"] [:span.h "what Jev reads it as"] [:span.h "probability"] [:span]
     (for [[option desc] (sort-by #(- (get probabilities (key %) 0M)) criteria)
           :let [p (get probabilities option 0M)
                 c (keyword (str/replace option "_" "-"))]]
       (list [:span {:class (when (= option top) "top")} (tag c)
              (when (ok c) [:span.exp (if (= c (:expect u)) "should be" "also fine")])]
             [:span.desc desc] (bar p) [:span.p (pct p)]))]))

(defn- levers
  "Where to change what this line shows: each kind of failure, what to change and where. The rows this line's
  answers point to are marked."
  [u decisions o]
  (let [ok (conj (set (:ok u)) (:expect u))
        good? #(ok (get-in decisions [% :code]))
        conf (get-in decisions [:choice :confidence])
        signs {:no-rules (and (not (good? :choice)) (not (good? :no-rules)))
               :rules (and (not (good? :choice)) (good? :no-rules))
               :blocks (and (not (good? :choice)) (good? :blocks) (not= :needs-person (get-in decisions [:blocks :code])))
               :check (= :unsafe o)
               :close (< (abs (- conf threshold)) 0.1M)}
        rows [[:fact "A field in step 3 is missing, wrong, or points the wrong way"
               "How code makes that field, or the scenario's data it comes from (trips, cards, projects)"
               (str facts-clj ", " scenario-edn)]
              [:no-rules "Jev picks a wrong bucket even without the rules (call 2, step 4)"
               "What Jev reads the buckets as (the option descriptions in step 4), or a fact it needs and is not given"
               (str scenario-edn " · :codes")]
              [:rules "Jev is wrong with the rules, right without them"
               "The text of the rule Jev leaned on (step 3, rules; step 5, the Jev column)"
               (str scenario-edn " · :rules")]
              [:blocks "The rule blocks (step 5) are right and the Choice is wrong"
               "The Choice does not see a rule's exceptions or conditions: say them in the rule's text, or send such disagreements to a person"
               (str scenario-edn " · :rules; " core-clj " · decide")]
              [:check "A person had to look, but Jev was sure of a bucket"
               "A check rule (:check true): exact blocks where code can tell, or one narrow yes/no question; the :checked strategy lets it stop the line"
               (str scenario-edn " · :rules")]
              [:close "Confidence within 10 points of the threshold"
               "It moves a few points between runs of the same questions (up to 25 seen): the threshold, or a rule that settles it"
               "the page's :threshold"]
              [:key "The answer key itself looks wrong"
               "The case's :expect, :ok and :review"
               (str scenario-edn " · :cases")]]]
    [:div.levers
     [:div.h "When you see"] [:div.h "What to change"] [:div.h "Where"]
     (for [[k see change where] rows
           :let [cls (when (signs k) "hit")]]
       (list [:div {:class cls} see (when cls [:b " — this line"])] [:div {:class cls} change]
             [:div {:class cls} (where-code where)]))]))

(defn- open-body [{:keys [id facts decisions responses] :as result}]
  (let [u (units id)
        line (lines (or (:line u) id))
        {:keys [code confidence]} (:choice decisions)
        blocks (:blocks decisions)
        checked (:checked decisions)
        o (outcome result)
        kept (classify/sent run result)
        requests (or kept (core/requests scenario facts))
        enrich (if kept (:enrich kept) (core/enrichment scenario u))
        rebuilt (when-not kept [:p.muted "This run did not keep its requests: these are rebuilt from today's code and scenario."])
        explained (classify/explain scenario result)
        applying (filter #(>= (:holds %) 0.5M) explained)
        near (filter #(and (< (:holds %) 0.5M) (or (>= (or (:jev %) 0M) 0.5M) (>= (:holds %) 0.25M))) explained)
        overruled-by (into {} (for [r applying
                                    :let [over (:over (some #(when (= (:id r) (:id %)) %) (:rules scenario)))]
                                    o over]
                                [o (:id r)]))
        {:keys [why fix]} (get-in notes [:lines id])]
    [:div.open-body
     [:div.verdict
      [:span.label "Should be"] (tag (:expect u))
      (when (seq (:ok u)) [:span.muted "also fine " (interpose " " (map tag (:ok u)))])
      (case (:review u)
        :required [:span.label "· a person must look"]
        :allowed [:span.muted "· a person may look"]
        nil)
      [:span.arrow "·"] [:span.label "Jev chose"] (tag code) [:span.mono (pct confidence)]
      [:span.arrow "·"] [:span.muted "automatic at " (pct threshold) " →"] [:span {:class (str "pill " (name o))} (outcome-words o)]]
     (when-let [w (or (:hard u) (:why u))]
       [:div.why {:style {:margin "0 0 10px"}} [:span.label (if (:hard u) "Why it is hard " "Why ")] w])
     (when why
       [:div.comment
        [:span.label "Why it failed"] [:span why]
        (when fix (list [:span.label "What would fix it"] [:span fix]))])

     (step 1 "The bank line" "as the bank sends it"
           [:div.cols2
            [:dl.kv
             (for [k [:date :card :merchant :mcc :amount :currency :country :city :channel :note]
                   :let [v (get line k)] :when (some? v)]
               (list [:dt {:class (when (#{:merchant :note} k) "wide-k")} (name k)]
                     [:dd {:class (when (#{:merchant :note} k) "wide")}
                      (cond (decimal? v) (.toPlainString ^BigDecimal v) (keyword? v) (name v) :else (str v))]))]
            (when-let [items (:items line)]
              [:div
               [:div.label "The " (name (:items-from line)) ", scanned apart from the bank line: one item is coded at a time"]
               [:ul.receipt
                (for [{:keys [item amount]} items]
                  [:li {:class (when (= item (:item u)) "this")} [:span item (when (= item (:item u)) "  ← this one")]
                   [:span.amt (.toPlainString ^BigDecimal amount)]])]])])

     (step 2 "What code works out" "facts/enrich: exact, no model"
           [:div.facts
            [:span.k "category"] [:span (or (:category facts) [:span.flag "none"]) (some->> (:mcc facts) (str " · "))]
            [:span.src (if (= :enrichment (:category-from facts))
                         (str "Jev guessed it from the name (" (pct (get-in responses [:enrich :answers :category :confidence] 0M)) ")")
                         "the bank's merchant code")]
            [:span.k "card"] [:span (:card-name facts)] [:span.src "the list of cards"]
            [:span.k "cardholder was"] [:span (get-in scenario [:entities (:holder facts) :name]) " was " (where-text facts)]
            [:span.src "the trips, by the date"]
            (when (= :in-store (:channel facts))
              (list [:span.k "shop"] [:span (if (:country-mismatch facts) [:span.flag "not where the cardholder was"] "where the cardholder was")]
                    [:span.src "the shop's city against the trip"]))
            [:span.k "projects running"] [:span (or (not-empty (str/join ", " (map #(get-in scenario [:projects % :name]) (:projects facts)))) "none")]
            [:span.src "the projects' dates"]
            (when (seq (:upcoming facts))
              (list [:span.k "trips coming"]
                    [:span (str/join "; " (for [{:keys [city place from project]} (:upcoming facts)]
                                            (str (or city place) " from " from (if project (str ", " (get-in scenario [:projects project :name])) ", personal"))))]
                    [:span.src "the trips, next 90 days"]))
            (when (:item facts)
              (list [:span.k "this item"] [:span (:item facts) (when (seq (:other-items facts)) [:span.muted " · beside it: " (str/join ", " (:other-items facts))])]
                    [:span.src "the receipt"]))
            (when-let [r (:refund-of facts)]
              (list [:span.k "refunds"] [:span [:a.mono {:href (str "#" (:id r))} (:id r)] ", " (:date r) ", when the cardholder was "
                                         (where-text {:whereabouts (:whereabouts r)})]
                    [:span.src "an earlier purchase, same card and merchant"]))
            (when-let [e (:earlier-here facts)]
              (list [:span.k "just before"] [:span [:a.mono {:href (str "#" (:id e))} (:id e)] ", " (:date e) ", "
                                             (:currency e) " " (.toPlainString ^BigDecimal (:amount e))
                                             (some->> (:note e) (str ", note: "))]
                    [:span.src "a charge in the week before, same card and merchant"]))]
           (when enrich
             [:details.payload [:summary "The enrichment call, which came first: Jev guessed the category from the name"]
              [:div.grid [:div [:div.label "Sent"] (json (assoc enrich :model (:model run)))]
               [:div [:div.label "Came back"] (json (get-in responses [:enrich :answers]))]]]))

     (step 3 "What Jev reads" "facts/state: the state, exactly as sent"
           [:p.say {:style {:margin "0 0 8px"}}
            "This is everything Jev knows about the line: no other line, no answer key, nothing else. Both calls send "
            "the same state; call 1 adds the rules. Read it as Jev would: is anything missing, wrong, or worded so it "
            "points the wrong way?"]
           rebuilt
           (reads (:state (:rules requests)))
           [:div.unsent [:b "Not sent to Jev"]
            [:ul [:li "the answer key, and why the line is hard"]
             [:li "each rule's exceptions (" [:code ":over"] ") and its conditions (" [:code ":when"] "): those are applied by code, in step 5"]
             [:li "the other lines on the card, except a refund's purchase or a charge in the week before, when found"]
             [:li "trips more than 90 days ahead, and anything about the cardholder beyond their trips"]]]
           [:details.payload [:summary "The raw state, as JSON"] (json (:state (:rules requests)))])

     (step 4 "What Jev is asked, and what it said" "core/requests: the questions, exactly as sent"
           (let [{:keys [questions]} (:rules requests)
                 code-q (:code questions)
                 plain-q (get-in (:plain requests) [:questions :code])]
             (list
              [:div.ask
               [:div.label "Call 1, the Choice that decides: given the state and the rules"]
               [:div.q (:instructions code-q)]
               (options (:criteria code-q) (get-in responses [:rules :answers :code :probabilities]) u)
               [:p.say "Asked again with the options in reverse order, it chose " (tag (get-in decisions [:reversed :code]))
                " " [:span.mono (pct (get-in decisions [:reversed :confidence]))] ". In the same call, one yes/no question per "
                "rule (\"does this rule apply?\"): its answers are the Jev column in step 5."]]
              [:details.ask {:style {:padding "8px 10px"}}
               [:summary.label {:style {:cursor "pointer"}} "Call 2, the same Choice without the rules: what the rules add"]
               [:div.q (:instructions plain-q)]
               (options (:criteria plain-q) (get-in responses [:plain :answers :code :probabilities]) u)
               [:p.say "Call 2 also asks the yes/no question in each dashed block of step 5, with the state alone."]])))

     (step 5 "The rules, applied" "blocks/holds: each block checked"
           [:div.rules
            [:span] [:span.h "rule"] [:span.h.c-code "codes to"] [:span.h "its blocks, and whether each holds"]
            [:span.h.num "holds"] [:span.h.num.c-jev {:title "Jev's reading of the whole rule, given every rule (call 1)"} "Jev"]
            (rule-rows facts (sort-by #(- (:holds %)) applying) overruled-by)
            (when (seq near)
              (list [:span] [:span.h {:style {:grid-column "2 / -1" :margin-top 6}}
                             "Did not apply, but Jev read them as applying (the Jev column), or a block came close"]
                    (rule-rows facts near overruled-by)))]
           (let [rest-rules (remove (set (concat applying near)) explained)]
             [:details.more [:summary (count rest-rules) " rules did not apply"]
              [:div.rules (rule-rows facts rest-rules overruled-by)]])
           [:details.payload [:summary "Call 2, raw: the yes/no questions in the dashed blocks, sent without the rules"]
            [:div.grid [:div [:div.label "Sent"] (json (update (assoc (:plain requests) :model (:model run)) :state (constantly "… the state above")))]
             [:div [:div.label "Came back"] (json (get-in responses [:plain :answers]))]]])

     (step 6 "The decision" "core/combine · the Choice · the threshold"
           [:div.decide
            [:div [:div.label "The rule blocks conclude"]
             [:p.say (tag (:code blocks)) " "
              (case (:reason blocks)
                :conflict [:span.muted "rules that apply disagree: " (str/join ", " (:by blocks))]
                :no-rule [:span.muted "no rule applies"]
                :check [:span.muted "a check applies: " (str/join ", " (:by blocks))]
                [:span.muted "by " (str/join ", " (:by blocks))
                 (when (seq (:overruled blocks)) (str "; set aside " (str/join ", " (:overruled blocks))))])]
             [:p.say [:span.muted "Their confidence is the least certain rule's: "] [:span.mono (pct (:confidence blocks))]]]
            [:div [:div.label "Jev's Choice, given the facts and every rule (call 1)"]
             [:div.probs
              (for [[option p] (take 5 (sort-by val > (get-in responses [:rules :answers :code :probabilities])))]
                (list [:span (tag (keyword (str/replace option "_" "-")))] (bar p) [:span.p (pct p)]))]
             [:p.say [:span.muted "Without the rules (call 2) it chose "] (tag (get-in decisions [:no-rules :code]))
              " " [:span.mono (pct (get-in decisions [:no-rules :confidence]))]]]
            [:div [:div.label "Checks, then the threshold"]
             [:p.say (if (= :check (:reason checked))
                       (list "A check holds (" (str/join ", " (:by checked)) "): a person looks, whatever Jev chose.")
                       "No check holds, so Jev's Choice stands.")]
             [:p.say "Confidence " [:span.mono (pct confidence)] (if (>= confidence threshold) " ≥ " " < ") [:span.mono (pct threshold)]
              (if (and (not= :needs-person code) (>= confidence threshold)) ": coded automatically." ": sent to a person.")]
             [:p.say [:span {:class (str "pill " (name o))} (outcome-words o)]]]]
           [:details.payload [:summary "Call 1, raw: the payload with the rules (the state is in step 3), and what came back"]
            [:div.grid [:div [:div.label "Sent"] (json (update (assoc (:rules requests) :model (:model run)) :state dissoc :transaction :context))]
             [:div [:div.label "Came back"] (json (get-in responses [:rules :answers]))]]])

     (step 7 "Where to change it" "the levers, by what you see"
           (levers u decisions o))]))

(defn- row
  "One line's summary row. Only the open line carries its walk-through: the page stays small, and opening a row asks
  the server for that line's."
  [{:keys [id decisions] :as result} open]
  (let [u (units id)
        {:keys [code confidence]} (:choice decisions)
        o (outcome result)
        acceptable (conj (set (:ok u)) (:expect u))
        blk (:code (:blocks decisions))]
    {:id id
     :class (str "row " (name o) (when (:hard u) " hard") (when (get-in notes [:lines id]) " noted"))
     :summary
     [:summary
      [:span.dot {:title (outcome-words o)}]
      [:span.id id (when (get-in notes [:lines id]) [:span.has-note {:title "Commentary on why it failed"} "✎"])]
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
     :body (when (= id open) (open-body result))}))

(defonce ^:private !open
  ;; The line whose row is open: a row that opens asks the server, through open!, for its walk-through.
  (atom nil))

(defn open!
  "Opens line `id`'s row; the page is worked out again with that line's walk-through."
  [id]
  (reset! !open id))

(def ^:private page-viewer
  {:transform-fn clerk/mark-presented
   :render-fn
   '(fn [{:keys [before cols rows open]}]
      (into before
            [(into [:div.rows cols]
                   (for [{:keys [id class summary body]} rows]
                     ^{:key id}
                     [:details {:id id :class class :open (= id open)
                                :on-toggle (fn [e]
                                             (when (and (.. e -target -open) (not= id open))
                                               (nextjournal.clerk.viewer/clerk-eval {:recompute? true}
                                                                                    (list 'ownpurse.notebooks.cards/open! id))))}
                      summary
                      (or body [:div.body {:style {:padding "14px 8px" :color "var(--ink-3)"}} "Working out this line…"])]))]))})

(defn- every-line []
  (let [open @!open]
    {:before
     [:div
      [:h2 "Every line"]
      [:p.muted {:style {:font-size "12.5px"}} "Open a row to walk its line through, one line at a time."]
      [:div.filters
       (for [[k words n] (concat [[:all "All" (count results)]]
                                 (for [o [:wrong :unsafe :review :stopped :right]] [o (outcome-words o) (count (by-outcome o))])
                                 [[:hard "marked hard" (count (filter #(:hard (units (:id %))) results))]]
                                 (when notes [[:noted "with commentary" (count (:lines notes))]]))]
         (list [:input {:type "radio" :name "f" :id (str "f-" (name k)) :default-checked (= :all k)}]
               [:label {:for (str "f-" (name k))} (when-not (#{:all :hard :noted} k) [:span {:class (str "dot " (name k))}])
                words [:b n]]))]]
     :cols [:div.cols [:span] [:span "line"] [:span "date"] [:span "merchant · item · note"] [:span {:style {:text-align "right"}} "amount"]
            [:span "belongs in"] [:span "Jev chose"] [:span "blocks"] [:span]]
     :open open
     :rows (mapv #(row % open) (sort-by #(case (outcome %) :wrong 0 :unsafe 1 :review 2 :stopped 3 4) results))}))

^{:nextjournal.clerk/visibility {:result :show}}
(clerk/with-viewer page-viewer {:nextjournal.clerk/width :full}
  (update (every-line) :before #(vector :div.opx [:style css] (head) (board) %)))
