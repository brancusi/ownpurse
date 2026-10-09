//! Agent bridge v0.10.1 (PM's live test): a claimed change is visible (frame and state), the filter means
//! the same for keys and agents and shows its chip, state rows carry every column, the view window, and
//! the TB totals label under a filter.

use ownpurse_tui::app::Msg;
use ownpurse_tui::keymap::Key;
use ownpurse_tui::quiet::{Counting, Quiet};
use ownpurse_tui::runtime::Runtime;
use ownpurse_tui::snapshot::{self, fixture_app};
use ownpurse_tui::theme;
use ratatui::layout::Rect;
use ratatui::{Terminal, TerminalOptions, Viewport};
use serde_json::{Value, json};

type Rt = Runtime<Quiet<Counting>>;

fn rt_size(w: u16, h: u16) -> Rt {
    let backend = Quiet::fixed(Counting::default(), true, w, h);
    let terminal = Terminal::with_options(
        backend,
        TerminalOptions {
            viewport: Viewport::Fixed(Rect::new(0, 0, w, h)),
        },
    )
    .unwrap();
    let mut rt = Runtime::new(terminal, fixture_app(theme::LEDGER_DARK, false));
    rt.frame().unwrap();
    rt
}

fn rt() -> Rt {
    rt_size(100, 30)
}

fn send(rt: &mut Rt, verbs: &[&[&str]]) -> Value {
    rt.agent(&json!({"v": 1, "kind": "view", "op": "send", "verbs": verbs}))
}

fn state(rt: &mut Rt) -> Value {
    rt.agent(&json!({"v": 1, "kind": "view", "op": "state"}))["state"].clone()
}

fn key(rt: &mut Rt, k: &str) {
    rt.send(Msg::Key(Key::parse(k).unwrap().event()));
    rt.frame().unwrap();
}

/// The view as the user reads it, without the agent's own footprint: the header mark, the notice strip,
/// and on Activity the agent's own log rows (one per read and send, by design) and the counts they move.
fn view(rt: &Rt) -> String {
    // the agent mark says whether the agent acted (◆) or is only connected (◇): not the view itself
    let t = snapshot::text(&rt.current_frame()).replace('◆', "◇");
    let mut lines: Vec<String> = t.lines().map(str::to_string).collect();
    lines.pop(); // the status strip (the agent's notice lives there)
    let activity = rt.app.ui.tab == ownpurse_tui::app::Tab::Activity;
    lines
        .into_iter()
        // log rows (every agent request adds one at the top and scrolls the oldest off)
        .filter(|l| {
            !activity || {
                let t = l.trim_start_matches(['▌', ' ']);
                // (and the Recent calls band, whose count includes the agent's own request)
                !(t.len() > 8 && t.as_bytes()[2] == b':' && t.as_bytes()[5] == b':')
                    && !t.starts_with("Recent calls")
            }
        })
        .filter(|l| !activity || !l.starts_with(" #"))
        .map(|l| {
            if activity && (l.starts_with(" Today (UTC)") || l.starts_with(" filter:")) {
                l.chars().filter(|c| !c.is_ascii_digit()).collect()
            } else if activity {
                // the agent's org name can widen a column
                l.split_whitespace().collect::<Vec<_>>().join(" ")
            } else {
                l
            }
        })
        .collect::<Vec<_>>()
        .join("\n")
        .replace("◆ agent · ", "")
        .replace("◇ agent · ", "")
        .replace(" · ◆ · ", " · ")
        .replace(" · ◇ · ", " · ")
}

/// The state without its revision (and, on Activity, without the agent's own rows and their counts).
fn plain_state(rt: &Rt) -> Value {
    let mut s = rt.app.state_json(0);
    let m = s.as_object_mut().unwrap();
    m.remove("rev");
    // the canonical block carries the agent's own log (every request adds a row) and the history
    m.remove("canonical");
    m.remove("history");
    if rt.app.ui.tab == ownpurse_tui::app::Tab::Activity {
        if let Some(Value::Array(rows)) = m.get_mut("rows") {
            // the rows the window shows besides the agent's own (they push older rows off the bottom)
            rows.retain(|r| r["fields"]["org"] != "agent");
            for r in rows.iter_mut() {
                r.as_object_mut().unwrap().remove("row");
            }
            rows.truncate(8);
        }
        for k in ["filter_matches", "filter_total", "view", "anchors"] {
            m.remove(k);
        }
    }
    s
}

/// Screens to test on: (name, setup verbs).
fn screens() -> Vec<(&'static str, Vec<Vec<&'static str>>)> {
    vec![
        ("overview", vec![]),
        (
            "ledger-l1",
            vec![vec!["tab", "ledger"], vec!["scope", "roast"]],
        ),
        (
            "ledger-l2",
            vec![
                vec!["tab", "ledger"],
                vec!["scope", "roast"],
                vec!["select", "1010"],
                vec!["drill"],
            ],
        ),
        (
            "ledger-l3",
            vec![
                vec!["tab", "ledger"],
                vec!["scope", "roast"],
                vec!["select", "1010"],
                vec!["drill"],
                vec!["drill"],
            ],
        ),
        ("ledger-no-org", vec![vec!["tab", "ledger"]]),
        ("ic-l1", vec![vec!["tab", "intercompany"]]),
        (
            "ic-l2",
            vec![
                vec!["tab", "intercompany"],
                vec!["select", "hold-design"],
                vec!["drill"],
            ],
        ),
        ("activity", vec![vec!["tab", "activity"]]),
        ("system", vec![vec!["tab", "system"]]),
        ("compare", vec![vec!["compare", "roast,design"]]),
    ]
}

fn setup(steps: &[Vec<&str>]) -> Rt {
    let mut rt = rt();
    state(&mut rt); // connected: the header's ◇ agent is already there
    for s in steps {
        let r = send(&mut rt, &[s.as_slice()]);
        assert_eq!(r["ok"], true, "setup {s:?}: {r}");
    }
    key(&mut rt, "esc"); // dismiss the setup's notice
    rt
}

// ── 1. the invariant: a claimed change is visible in the frame and in state ─────────────────────

#[test]
fn every_claimed_change_shows_in_the_frame_and_in_state() {
    let verbs: Vec<Vec<&str>> = vec![
        vec!["tab", "2"],
        vec!["tab", "activity"],
        vec!["scope", "design"],
        vec!["scope", "all"],
        vec!["asof", "2025-06-30"],
        vec!["filter", "an"],
        vec!["filter", "Tallowmere "],
        vec!["clear-filter"],
        vec!["select", "#2"],
        vec!["drill"],
        vec!["up"],
        vec!["basis", "history"],
        vec!["differs", "on"],
        vec!["compare", "roast,hold"],
        vec!["reload"],
    ];
    let mut checked = 0;
    for (name, steps) in screens() {
        for v in &verbs {
            let mut rt = setup(&steps);
            let (frame0, state0) = (view(&rt), plain_state(&rt));
            let draws = rt.draws;
            let r = send(&mut rt, &[v.as_slice()]);
            let (frame1, state1) = (view(&rt), plain_state(&rt));
            let what = format!("{name} · {v:?}: {r}");
            if r["ok"] != true {
                assert!(r["exit"] == 2 || r["error"] == "not_found", "{what}");
                assert_eq!(frame1, frame0, "a refused verb changes nothing: {what}");
                assert_eq!(state1, state0, "{what}");
                continue;
            }
            let changed = r["changed"].as_array().unwrap();
            if changed.is_empty() {
                assert_eq!(frame1, frame0, "unchanged but the frame moved: {what}");
                // (Activity redraws: the agent's own log row is new on screen, by design)
                if !matches!(v[0], "reload") && rt.app.ui.tab != ownpurse_tui::app::Tab::Activity {
                    assert_eq!(rt.draws, draws, "unchanged must not redraw: {what}");
                }
            } else {
                assert_ne!(
                    frame1, frame0,
                    "claimed a change the frame doesn't show: {what}"
                );
                assert_ne!(
                    state1, state0,
                    "claimed a change state doesn't show: {what}"
                );
            }
            checked += 1;
        }
    }
    assert!(checked > 100, "{checked}");
}

// ── 1/2. the filter: key and agent identical, visible, refused where meaningless ────────────────

#[test]
fn the_key_filter_and_the_agent_filter_are_the_same_on_every_screen() {
    for (name, steps) in screens() {
        for q in ["Tallowmere ", "an", "zzz-nothing"] {
            let mut by_key = setup(&steps);
            let applies = by_key.app.filter_applies().is_ok();
            key(&mut by_key, "/");
            let note = by_key.app.ui.note.clone();
            for c in q.chars().filter(|_| applies) {
                let k = if c == ' ' {
                    "space".to_string()
                } else {
                    c.to_string()
                };
                key(&mut by_key, &k);
            }
            if applies {
                key(&mut by_key, "enter");
            }
            let mut by_agent = setup(&steps);
            let r = send(&mut by_agent, &[&["filter", q]]);
            if !applies {
                assert_eq!(r["error"], "not_applicable", "{name}: {r}");
                assert_eq!(r["exit"], 2);
                assert!(
                    by_key.app.ui.filter.is_empty(),
                    "{name}: the key never sets it"
                );
                assert!(by_key.app.ui.prompt.is_none(), "{name}");
                assert!(
                    note.as_deref()
                        .unwrap_or("")
                        .starts_with("A filter doesn't apply on"),
                    "{name}: {note:?}"
                );
                assert!(
                    by_agent.app.ui.filter.is_empty(),
                    "{name}: not set silently"
                );
                continue;
            }
            assert_eq!(r["ok"], true, "{name}: {r}");
            assert_eq!(by_key.app.ui.filter, q);
            // (on Activity the agent's own `send filter=…` row matches its own query, so only the rows
            // besides the agent's are compared there)
            if name != "activity" {
                assert_eq!(view(&by_key), view(&by_agent), "{name} {q:?}: same screen");
            }
            let (sk, sa) = (plain_state(&by_key), plain_state(&by_agent));
            assert_eq!(sk["rows"], sa["rows"], "{name} {q:?}: same rows");
            assert_eq!(sk["filter_matches"], sa["filter_matches"]);
            // the chip with its count is on screen, and in state
            let full = by_agent.app.state_json(0);
            let (n, total) = (
                full["filter_matches"].as_u64().unwrap(),
                full["filter_total"].as_u64().unwrap(),
            );
            let chip = format!("filter: \"{q}\" · {n} of {total}");
            let shown = snapshot::text(&by_agent.current_frame());
            assert!(shown.contains(&chip), "{name}: {chip}\n{shown}");
            assert!(n <= total);
            if q == "zzz-nothing" && name != "activity" {
                assert_eq!(n, 0, "{name}");
                assert!(
                    sa["rows"].as_array().unwrap().is_empty() || name == "system",
                    "{name}"
                );
            }
        }
    }
}

#[test]
fn the_tb_filter_lists_only_matching_accounts_and_keeps_the_whole_tb_totals() {
    let mut rt = setup(&[vec!["tab", "ledger"], vec!["scope", "roast"]]);
    let all = state(&mut rt)["view"]["total_rows"].as_u64().unwrap();
    assert!(all >= 10, "{all}");
    let r = send(&mut rt, &[&["filter", "Tallowmere "]]);
    assert_eq!(r["changed"], json!(["filter “Tallowmere ” on Ledger"]));
    let st = &r["state"];
    let names: Vec<&str> = st["rows"]
        .as_array()
        .unwrap()
        .iter()
        .map(|x| x["fields"]["account"].as_str().unwrap())
        .collect();
    assert_eq!(
        names,
        [
            "Tallowmere Roastery Operating Account",
            "Tallowmere Green-Bean Card",
            "Tallowmere Paid-In Capital"
        ]
    );
    assert_eq!(
        (st["filter_matches"].clone(), st["filter_total"].clone()),
        (json!(3), json!(all))
    );
    assert_eq!(st["view"]["total_rows"], 3);
    let frame = view(&rt);
    assert!(
        frame.contains(&format!("filter: \"Tallowmere \" · 3 of {all}")),
        "{frame}"
    );
    assert!(!frame.contains("Wholesale Bean Sales"), "{frame}");
    assert!(
        frame.contains("Total (all accounts)"),
        "C11 under a filter:\n{frame}"
    );
    // code ranges
    let r = send(&mut rt, &[&["filter", "code:4000..6200"]]);
    let codes: Vec<&str> = r["state"]["rows"]
        .as_array()
        .unwrap()
        .iter()
        .map(|x| x["anchor"]["account_code"].as_str().unwrap())
        .collect();
    assert_eq!(codes, ["4010", "5020", "6140"]); // TB order: class bands (revenue, expenses), then code
    // clearing shows them all again, and "Total" again
    let r = send(&mut rt, &[&["clear-filter"]]);
    assert_eq!(r["state"]["view"]["total_rows"], json!(all));
    assert!(!view(&rt).contains("Total (all accounts)"));
}

#[test]
fn drilling_from_a_filtered_tb_starts_the_postings_unfiltered() {
    let mut rt = setup(&[vec!["tab", "ledger"], vec!["scope", "roast"]]);
    let r = send(
        &mut rt,
        &[&["filter", "Operating"], &["select", "#1"], &["drill"]],
    );
    assert_eq!(r["state"]["mode"], "postings", "{r}");
    assert_eq!(r["state"]["filter"], "");
    assert!(!r["state"]["rows"].as_array().unwrap().is_empty());
    // and the postings filter by their own fields
    let r = send(&mut rt, &[&["filter", "zzz-nothing"]]);
    assert_eq!(r["state"]["rows"], json!([]));
    assert!(view(&rt).contains("No posting matches the filter."));
}

// ── 3. state rows carry every column ─────────────────────────────────────────────────────────────

#[test]
fn every_visible_column_header_is_a_field_key() {
    use ownpurse_tui::agent::field_key;
    for (name, steps) in screens() {
        let mut rt = setup(&steps);
        let st = state(&mut rt);
        let rows = st["rows"].as_array().unwrap();
        if rows.is_empty() {
            continue;
        }
        let keys: Vec<String> = rows[0]["fields"]
            .as_object()
            .unwrap()
            .keys()
            .cloned()
            .collect();
        let titles: Vec<&str> = match name {
            "overview" => ownpurse_tui::views::overview::COLS
                .iter()
                .map(|c| c.title)
                .collect(),
            "ledger-l1" => ownpurse_tui::views::ledger::L1_COLS
                .iter()
                .map(|c| c.title)
                .collect(),
            "ledger-l2" => ownpurse_tui::views::ledger::L2_COLS
                .iter()
                .map(|c| c.title)
                .collect(),
            "ic-l1" => ownpurse_tui::views::intercompany::L1_COLS
                .iter()
                .map(|c| c.title)
                .collect(),
            "ic-l2" => ownpurse_tui::views::intercompany::ITEM_COLS
                .iter()
                .map(|c| c.title)
                .collect(),
            "activity" => ownpurse_tui::views::activity::LOG_COLS
                .iter()
                .map(|c| c.title)
                .collect(),
            "system" => ownpurse_tui::views::system::ORG_COLS
                .iter()
                .map(|c| c.title)
                .collect(),
            "compare" => vec!["code", "account"],
            _ => continue,
        };
        let frame = view(&rt);
        for t in titles {
            let k = field_key(t);
            assert!(
                keys.contains(&k),
                "{name}: column `{t}` → `{k}` missing from {keys:?}"
            );
            if t.len() > 2 {
                // a header the screen shows is a key (the table may hide low-priority columns when narrow)
                if frame.contains(t) {
                    assert!(keys.contains(&k));
                }
            }
        }
        // stable snake_case
        for k in &keys {
            assert!(
                k.chars()
                    .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_'),
                "{name}: {k}"
            );
        }
    }
    // the Overview row carries the parity, issues, TB fetched and calls columns (the PM's gap)
    let mut rt = setup(&[]);
    let f = state(&mut rt)["rows"][0]["fields"].clone();
    for k in [
        "org",
        "fetched",
        "parity_year",
        "parity_history",
        "issues",
        "tb_fetched",
        "calls_left",
    ] {
        assert!(f.get(k).is_some(), "{k}: {f}");
    }
    assert_eq!(f["parity_year"], "✓ exact");
    assert_eq!(f["calls_left"], "967/1,000");
}

// ── 4. the scroll window ─────────────────────────────────────────────────────────────────────────

#[test]
fn the_view_window_matches_rows_and_select_numbers() {
    let mut rt = rt_size(100, 20); // the smallest terminal: the TB scrolls
    for s in [vec!["tab", "ledger"], vec!["scope", "roast"]] {
        send(&mut rt, &[s.as_slice()]);
    }
    let st = state(&mut rt);
    let total = st["view"]["total_rows"].as_u64().unwrap();
    let rows = st["rows"].as_array().unwrap().clone();
    assert!((rows.len() as u64) < total, "only some rows fit");
    assert_eq!(st["view"]["first_visible"], 1);
    assert_eq!(st["view"]["last_visible"], rows.last().unwrap()["row"]);
    // move to the last row: the window scrolls and the numbers follow the whole list
    key(&mut rt, "end");
    let st = state(&mut rt);
    let rows = st["rows"].as_array().unwrap();
    assert_eq!(st["view"]["last_visible"].as_u64().unwrap(), total);
    assert!(st["view"]["first_visible"].as_u64().unwrap() > 1);
    assert_eq!(rows.first().unwrap()["row"], st["view"]["first_visible"]);
    assert!(rows.last().unwrap()["selected"] == true);
    // `select #n` takes those numbers
    let first = st["view"]["first_visible"].as_u64().unwrap();
    let r = send(&mut rt, &[&["select", &format!("#{first}")]]);
    let sel: Vec<&Value> = r["state"]["rows"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|x| x["selected"] == true)
        .collect();
    assert_eq!(sel[0]["row"], json!(first), "{r}");
    // a number off screen selects that row and scrolls it into view (v0.12: select anywhere in the list)
    let r = send(&mut rt, &[&["select", "#1"]]);
    assert_eq!(r["ok"], true, "{r}");
    let sel: Vec<&Value> = r["state"]["rows"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|x| x["selected"] == true)
        .collect();
    assert_eq!(sel[0]["row"], json!(1), "{r}");
}

// ── refusals for verbs that can't show ───────────────────────────────────────────────────────────

#[test]
fn basis_and_differs_are_refused_where_they_cannot_show() {
    let mut rt = setup(&[]);
    for v in [["basis", "history"], ["differs", "on"]] {
        let r = send(&mut rt, &[&v]);
        assert_eq!(
            (r["error"].clone(), r["exit"].clone()),
            (json!("not_applicable"), json!(2)),
            "{r}"
        );
        assert!(
            r["message"].as_str().unwrap().contains("not on Overview"),
            "{r}"
        );
    }
    let r = send(
        &mut rt,
        &[
            &["tab", "intercompany"],
            &["select", "hold-design"],
            &["drill"],
            &["filter", "x"],
        ],
    );
    assert_eq!(r["error"], "not_applicable");
    assert!(
        r["message"].as_str().unwrap().starts_with(
            "verb 4 (filter x) failed: A filter doesn't apply on the Intercompany bridge"
        ),
        "{r}"
    );
    // the earlier verbs are named as not applied (the send is atomic)
    assert!(
        r["message"].as_str().unwrap().contains(
            "Verbs 1 (tab intercompany), 2 (select hold-design), 3 (drill) not applied (the send is atomic). Nothing was changed."
        ),
        "{r}"
    );
}

#[test]
fn select_on_the_ledger_across_orgs_says_to_choose_an_org_first() {
    let mut rt = rt();
    let r = send(
        &mut rt,
        &[&["tab", "ledger"], &["scope", "all"], &["select", "1010"]],
    );
    assert_eq!(
        (r["error"].as_str(), r["exit"].as_i64()),
        (Some("not_found"), Some(2)),
        "{r}"
    );
    let m = r["message"].as_str().unwrap();
    assert!(
        m.starts_with("verb 3 (select 1010) failed: The Ledger lists no rows for scope all"),
        "{m}"
    );
    assert!(
        m.contains("Verbs 1 (tab ledger), 2 (scope all) not applied"),
        "{m}"
    );
    assert!(
        r["help"][0]
            .as_str()
            .unwrap()
            .starts_with("Ledger needs a single-org scope; send `scope <org>` first"),
        "{r}"
    );
    // one verb alone: no verb numbering
    let r = send(&mut rt, &[&["select", "zzz-nothing"]]);
    assert!(!r["message"].as_str().unwrap().starts_with("verb "), "{r}");
}

// ── 7. rows cut off say so; the whole list on request, read-only ────────────────────────────────

fn state_rows(rt: &mut Rt, rows: &str) -> Value {
    let r = rt.agent(&json!({"v": 1, "kind": "view", "op": "state", "rows": rows}));
    assert_eq!(r["ok"], true, "{r}");
    r["state"].clone()
}

#[test]
fn all_rows_is_the_full_list_and_the_visible_slice_matches() {
    for (w, h) in [(100u16, 30u16), (100, 20)] {
        for (name, steps) in screens() {
            let mut rt = rt_size(w, h);
            state(&mut rt);
            for s in &steps {
                send(&mut rt, &[s.as_slice()]);
            }
            key(&mut rt, "end"); // scroll to the bottom where a list scrolls
            let vis = state(&mut rt);
            let (rev, draws, logged) = (rt.app.rev, rt.draws, rt.app.agent.log.len());
            let all = state_rows(&mut rt, "all");
            assert_eq!(
                (rt.app.rev, rt.draws, rt.app.agent.log.len()),
                (rev, draws, logged),
                "{name}: read-only"
            );
            let total = vis["view"]["total_rows"].as_u64().unwrap() as usize;
            let rows = all["rows"].as_array().unwrap();
            assert_eq!(
                rows.len(),
                total,
                "{name} {w}x{h}: total_rows is the full set"
            );
            assert_eq!(all["truncated"], false);
            let numbers: Vec<u64> = rows.iter().map(|r| r["row"].as_u64().unwrap()).collect();
            assert_eq!(
                numbers,
                (1..=total as u64).collect::<Vec<_>>(),
                "{name}: screen order"
            );
            assert!(rows.iter().filter(|r| r["selected"] == true).count() <= 1);
            let shown = vis["rows"].as_array().unwrap();
            assert_eq!(vis["truncated"], json!(shown.len() < total), "{name}");
            if let (Some(a), Some(b)) = (
                vis["view"]["first_visible"].as_u64(),
                vis["view"]["last_visible"].as_u64(),
            ) {
                let slice = state_rows(&mut rt, &format!("{a}..{b}"));
                assert_eq!(
                    slice["rows"], vis["rows"],
                    "{name} {w}x{h}: the visible slice == --rows first..last"
                );
                assert_eq!(
                    &all["rows"].as_array().unwrap()[(a - 1) as usize..b as usize],
                    shown.as_slice(),
                    "{name}"
                );
            } else {
                assert!(shown.is_empty(), "{name}: rows without a window");
            }
        }
    }
}

#[test]
fn postings_carry_debit_credit_and_signed_amount_and_balance() {
    let mut rt = setup(&[
        vec!["tab", "ledger"],
        vec!["scope", "roast"],
        vec!["select", "1010"],
        vec!["drill"],
    ]);
    let all = state_rows(&mut rt, "all");
    let rows = all["rows"].as_array().unwrap();
    assert!(!rows.is_empty());
    for r in rows {
        let f = &r["fields"];
        let amount: f64 = f["amount"].as_str().unwrap().parse().unwrap_or(0.0);
        let (dr, cr) = (f["debit"].as_str().unwrap(), f["credit"].as_str().unwrap());
        // debit positive, credit negative
        assert_eq!(amount > 0.0, !dr.is_empty(), "{f}");
        assert_eq!(amount < 0.0, !cr.is_empty(), "{f}");
        assert!(
            // v0.10.3: the twin is an exact number (it was a decimal string)
            f["balance_value"].is_number() && f["balance"].as_str().is_some(),
            "{f}"
        );
    }
    let r = rt.agent(&json!({"v": 1, "kind": "view", "op": "state", "rows": "3..1"}));
    assert_eq!(
        (r["error"].clone(), r["exit"].clone()),
        (json!("usage"), json!(2))
    );
}

// ── 8. `up` is one level; `back` / `forward` are navigation history (v0.10.2) ─────────────────

#[test]
fn up_goes_one_level_and_back_forward_walk_the_history() {
    let mut rt = setup(&[
        vec!["tab", "ledger"],
        vec!["scope", "roast"],
        vec!["select", "1010"],
        vec!["drill"],
    ]);
    let r = send(&mut rt, &[&["up"]]);
    assert_eq!(r["changed"], json!(["went up a level"]));
    assert_eq!(r["state"]["mode"], "trial balance");
    let r = send(&mut rt, &[&["back"]]);
    assert_eq!(
        r["state"]["mode"], "postings",
        "back returns to where `up` left: {r}"
    );
    assert!(
        r["changed"][0].as_str().unwrap().starts_with("back → "),
        "{r}"
    );
    let r = send(&mut rt, &[&["forward"]]);
    assert_eq!(r["state"]["mode"], "trial balance");
}

// ── 9. the version: footer's last item, never in the header ─────────────────────────────────────

#[test]
fn the_version_is_the_footers_last_item_at_every_width() {
    let v = format!("v{}", snapshot::TEST_VERSION);
    for st in snapshot::shell_states()
        .into_iter()
        .filter(|s| ["overview", "ledger-filter", "agent-notice", "palette"].contains(&s.name))
    {
        for w in (60..=220u16).step_by(7) {
            let h = 24;
            let mut a = snapshot::state_app(&st, theme::LEDGER_DARK, w, h);
            let buf = snapshot::render(&mut a, w, h);
            let foot = snapshot::row_text(&buf, h - 1);
            assert!(foot.trim_end().ends_with(&v), "{} {w}: {foot}", st.name);
            assert!(
                !snapshot::row_text(&buf, 0).contains(&v),
                "{} {w}: header",
                st.name
            );
        }
    }
    let mut rt = rt();
    let st = state(&mut rt);
    assert_eq!(st["version"], snapshot::TEST_VERSION);
    assert_eq!(st["update_available"], Value::Null);
}

#[test]
fn selecting_the_selected_row_is_unchanged_and_labels_name_the_account() {
    let mut rt = setup(&[vec!["tab", "ledger"], vec!["scope", "roast"]]);
    let draws = rt.draws;
    let r = send(&mut rt, &[&["select", "1010"]]);
    assert_eq!(r["changed"], json!([]), "already selected: {r}");
    assert_eq!(rt.draws, draws);
    let r = send(&mut rt, &[&["select", "1060"]]);
    assert_eq!(
        r["changed"],
        json!(["selected 1060 Tallowmere Green-Bean Card"])
    );
}
