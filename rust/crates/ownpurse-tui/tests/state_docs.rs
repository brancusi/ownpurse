//! Every field of `ui state` is in docs/TUI.md's field reference (V21 surface): a key the struct exports
//! but the reference doesn't name fails here, so the reference can't drift.

use ownpurse_tui::app::App;
use ownpurse_tui::snapshot::{self, fixture_app};
use ownpurse_tui::theme;
use serde_json::Value;

fn paths(v: &Value, prefix: &str, depth: usize, out: &mut std::collections::BTreeSet<String>) {
    if depth >= 3
        || prefix.ends_with("rows[].anchor")
        || prefix.ends_with("rows[].fields")
        || prefix == "selected"
    {
        return;
    }
    match v {
        Value::Object(m) => {
            for (k, x) in m {
                let p = if prefix.is_empty() {
                    k.clone()
                } else {
                    format!("{prefix}.{k}")
                };
                out.insert(p.clone());
                paths(x, &p, depth + 1, out);
            }
        }
        Value::Array(a) => {
            for x in a {
                paths(x, &format!("{prefix}[]"), depth, out);
            }
        }
        _ => {}
    }
}

fn states() -> Vec<App> {
    let mut v = Vec::new();
    for script in [
        "2 ]",
        "ctrl-p s y",
        "/ r o left",
        "ctrl-p s y n c space d e s i g n enter tab",
        "4",
        "6 A",
        "6 b",
    ] {
        let mut a = fixture_app(theme::LEDGER_DARK, false);
        snapshot::run_script(&mut a, script, 100, 30);
        v.push(a);
    }
    let mut a = fixture_app(theme::LEDGER_DARK, false);
    snapshot::agent_send(
        &mut a,
        &[
            &["tab", "ledger"],
            &["scope", "design"],
            &["caption", "x · y"],
        ],
        100,
        30,
    );
    a.update_available = Some(ownpurse_tui::canonical::Update {
        version: "9.9.9".into(),
        action: "restart".into(),
    });
    a.drift = Some(ownpurse_tui::canonical::Drift {
        from: 1,
        to: 2,
        config: None,
        classify: None,
        manifest: None,
    });
    v.push(a);
    let mut a = fixture_app(theme::LEDGER_DARK, false);
    snapshot::net_running(&mut a, false);
    v.push(a);
    v
}

#[test]
fn every_state_field_is_in_the_reference() {
    let doc = std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../docs/TUI.md"))
        .unwrap();
    let start = doc
        .find("<!-- state-fields -->")
        .expect("the field reference marker in docs/TUI.md");
    let reference = &doc[start..];
    let mut all = std::collections::BTreeSet::new();
    for mut a in states() {
        let (w, h) = a.ui.size;
        snapshot::render(&mut a, w, h);
        let v = serde_json::to_value(a.view_state(a.rev)).unwrap();
        paths(&v, "", 0, &mut all);
    }
    let missing: Vec<&String> = all
        .iter()
        .filter(|p| !reference.contains(&format!("`{p}`")))
        .collect();
    assert!(
        missing.is_empty(),
        "undocumented state fields:\n{}",
        missing
            .iter()
            .map(|s| s.as_str())
            .collect::<Vec<_>>()
            .join("\n")
    );
}
