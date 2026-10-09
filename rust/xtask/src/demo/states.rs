//! The frozen view-state fixtures (`rust/tests/fixtures/state/v0.10.2/`, `v0.10.3/`), rewritten IN PLACE on
//! the demo world (v0.10.4): same files, same schema version (`"v": 1`) and the same shape each
//! release wrote, new content. The gate test `every_frozen_state_still_imports` (tui `tests/v21.rs`) still
//! proves this release reads the previous releases' states.
//!
//! Each state is built by this release's TUI library from `snapshot::frozen_state_cases`, exported, and then
//! written in its release's shape: v0.10.3 is this code's own shape; v0.10.2 predates `selected_label` and the
//! rows' `*_value` twins (v0.10.3), so those keys are removed. Nothing is copied from an old binary or an old
//! fixture. The `.txt` beside each state is the frame this build draws when it imports that file.

use ownpurse_tui::app::App;
use ownpurse_tui::snapshot;
use serde_json::Value;
use std::path::Path;

/// The releases whose states are frozen, oldest first.
pub const RELEASES: [&str; 2] = ["0.10.2", "0.10.3"];

/// Remove what a release's states did not carry yet.
pub fn to_release_shape(v: &mut Value, release: &str) {
    if release != "0.10.2" {
        return;
    }
    if let Some(m) = v.as_object_mut() {
        m.shift_remove("selected_label");
    }
    for row in v
        .get_mut("rows")
        .and_then(Value::as_array_mut)
        .into_iter()
        .flatten()
    {
        if let Some(fields) = row.get_mut("fields").and_then(Value::as_object_mut) {
            fields.retain(|k, _| !k.ends_with("_value"));
        }
    }
}

fn frame(a: &mut App) -> String {
    let (w, h) = a.ui.size;
    snapshot::text(&snapshot::render(a, w, h))
}

/// Write every frozen state (JSON + frame). Returns the relative paths written.
pub fn write(fixtures: &Path) -> super::Result<Vec<String>> {
    let mut written = vec![];
    for release in RELEASES {
        let dir = fixtures.join("state").join(format!("v{release}"));
        if dir.exists() {
            std::fs::remove_dir_all(&dir).map_err(|e| format!("{}: {e}", dir.display()))?;
        }
        std::fs::create_dir_all(&dir).map_err(|e| format!("{}: {e}", dir.display()))?;
        for (name, mut a) in snapshot::frozen_state_cases(release) {
            frame(&mut a);
            let mut v: Value = serde_json::from_str(&a.view_state(a.rev).canonical())
                .map_err(|e| format!("{name}: {e}"))?;
            // no machine paths in a tracked fixture (the frames shown don't use them)
            v["canonical"]["env"]["cli"] = serde_json::json!("/usr/local/bin/ownpurse");
            v["canonical"]["env"]["paths"] = serde_json::json!({"cwd": "/w", "home": null});
            to_release_shape(&mut v, release);
            let text = ownpurse_core::canon::canonical(&v);
            let mut b = App::from_state_json(&text, a.source.clone(), false)
                .map_err(|e| format!("{name}: {}", e.message()))?;
            let drawn = frame(&mut b);
            for (ext, body) in [("json", format!("{text}\n")), ("txt", drawn)] {
                let p = dir.join(format!("{name}.{ext}"));
                std::fs::write(&p, body).map_err(|e| format!("{}: {e}", p.display()))?;
                written.push(format!("state/v{release}/{name}.{ext}"));
            }
        }
    }
    Ok(written)
}
