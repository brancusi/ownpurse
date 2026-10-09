//! Live reload of `ownpurse.json` (v0.12.2): a valid save applies with the view kept, a removed org in scope
//! falls back to `all` and says so, an invalid file keeps the last valid config.

use ownpurse_tui::app::{App, Scope};
use ownpurse_tui::model::{Model, Source};
use ownpurse_tui::{project_config, snapshot, theme};
use std::path::PathBuf;

/// A temp copy of the demo folder (config, record, classify files).
fn demo_copy(name: &str) -> PathBuf {
    let d = std::env::temp_dir().join(format!("xat-pc-{}-{name}", std::process::id()));
    let _ = std::fs::remove_dir_all(&d);
    std::fs::create_dir_all(d.join("db")).unwrap();
    let demo = snapshot::demo_dir();
    for f in [
        "ownpurse.json",
        "record.sqlite",
        "classify.toml",
        "tenants.json",
    ] {
        std::fs::copy(demo.join(f), d.join(f)).unwrap();
    }
    let _ = std::fs::copy(
        demo.join("db/classify.sqlite"),
        d.join("db/classify.sqlite"),
    );
    d
}

fn app_on(dir: &std::path::Path) -> App {
    let env = ownpurse_core::config::Env::process();
    let cfg = ownpurse_core::config::load(&env, None, Some(&dir.join("ownpurse.json"))).unwrap();
    let source = Source::from_config(&cfg);
    let now = snapshot::fixture_now();
    let model = Model::open(&source, now).unwrap();
    let mut a = App::new(model, source, theme::LEDGER_DARK, false, now);
    a.ui.size = (100, 30);
    a
}

fn edit(dir: &std::path::Path, f: impl FnOnce(&mut serde_json::Value)) {
    let p = dir.join("ownpurse.json");
    let mut v: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(&p).unwrap()).unwrap();
    f(&mut v);
    std::fs::write(&p, v.to_string()).unwrap();
}

#[test]
fn an_unchanged_file_is_no_reload() {
    let dir = demo_copy("same");
    let a = app_on(&dir);
    assert_eq!(project_config::load(&a.source).unwrap(), None);
}

#[test]
fn a_removed_org_in_scope_falls_back_to_all_with_a_note() {
    let dir = demo_copy("removed");
    let mut a = app_on(&dir);
    let lab = a.model.orgs.iter().position(|o| o.alias == "lab").unwrap();
    a.ui.scope = Scope::Org(lab);
    let rev = a.rev;
    edit(&dir, |c| {
        c["profiles"]["demo"]["orgs"]
            .as_object_mut()
            .unwrap()
            .remove("lab");
        for g in c["groups"].as_object_mut().unwrap().values_mut() {
            g["orgs"].as_array_mut().unwrap().retain(|x| x != "lab");
        }
    });
    let src = project_config::load(&a.source).unwrap().expect("a change");
    let b = project_config::apply(&a, src).unwrap();
    assert_eq!(b.ui.scope, Scope::All);
    assert!(b.model.orgs.iter().all(|o| o.alias != "lab"));
    let note = b.ui.note.clone().unwrap();
    assert!(note.starts_with("config reloaded · orgs −lab"), "{note}");
    assert!(note.ends_with("lab is gone, showing all"), "{note}");
    assert!(b.rev > rev && b.drift.is_none());
    assert_eq!(b.data_refreshes, a.data_refreshes + 1);
}

#[test]
fn a_valid_change_keeps_the_view() {
    let dir = demo_copy("keep");
    let mut a = app_on(&dir);
    let roast = a
        .model
        .orgs
        .iter()
        .position(|o| o.alias == "roast")
        .unwrap();
    a.ui.scope = Scope::Org(roast);
    edit(&dir, |c| {
        c["groups"]["duo"] = serde_json::json!({"orgs": ["roast", "hold"]});
    });
    let src = project_config::load(&a.source).unwrap().expect("a change");
    let b = project_config::apply(&a, src).unwrap();
    assert_eq!(b.scope_label(&b.ui.scope), "roast");
    assert_eq!(b.ui.tab, a.ui.tab);
    assert!(b.model.groups.iter().any(|g| g.name == "duo"));
    assert!(b.ui.note.as_deref().unwrap().contains("groups"));
}

#[test]
fn an_invalid_file_is_an_error_and_nothing_changes() {
    let dir = demo_copy("invalid");
    let a = app_on(&dir);
    std::fs::write(dir.join("ownpurse.json"), "{ \"profiles\": ").unwrap();
    let e = project_config::load(&a.source).unwrap_err();
    let band = project_config::invalid_band(&e);
    assert!(band.lines[1].contains("previous config stays live"));
    assert!(band.copy.starts_with(project_config::BAND));
}

#[test]
fn an_unchanged_file_is_no_reload_even_when_the_source_was_adjusted() {
    // a source tuned after load (another record path, as tests and `--record` do) is not "a change": only
    // the file's bytes decide (this was the upgrade pre-flight's spurious re-read)
    let dir = demo_copy("adjusted");
    let mut a = app_on(&dir);
    a.source.record = dir.join("elsewhere.sqlite");
    assert_eq!(project_config::load(&a.source).unwrap(), None);
    // a save that changes the bytes is re-read
    edit(&dir, |c| {
        c["groups"]["duo"] = serde_json::json!({"orgs": ["roast", "hold"]})
    });
    assert!(project_config::load(&a.source).unwrap().is_some());
}
