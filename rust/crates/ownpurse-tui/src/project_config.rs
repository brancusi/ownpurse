//! Live reload of the project config (`ownpurse.json`): the file's folder is watched (the same OS events as
//! the record and the store; no timer), and a save is re-read. A valid change applies with the view kept:
//! the view state is exported and imported against the new config (scopes resolve by alias and group name;
//! a scope that no longer exists falls back to `all`), and the strip says what changed. An invalid file keeps
//! the last valid config, with the same error band as keys and themes.

use crate::app::App;
use crate::model::Source;

/// The band's `copy` prefix (a later valid save clears a band that starts with it).
pub const BAND: &str = "project config";

/// Re-read the config `old` came from: `Ok(None)` when nothing that matters changed, the new source
/// otherwise, or why the file can't be used.
pub fn load(old: &Source) -> Result<Option<Source>, String> {
    let Some(path) = old.config_file.as_ref() else {
        return Ok(None);
    };
    // the bytes as loaded: nothing to re-read
    if old.config_digest.is_some() && crate::model::file_digest(path) == old.config_digest {
        return Ok(None);
    }
    let env = ownpurse_core::config::Env::process();
    let cfg = ownpurse_core::config::load(&env, Some(&old.profile), Some(path))
        .map_err(|e| e.to_string())?;
    let mut new = Source::from_config(&cfg);
    new.config_arg = old.config_arg.clone();
    // groups given on the command line (`--group`) stay
    for g in &old.groups {
        if !new.groups.iter().any(|(n, _)| n == &g.0)
            && !cfg.settings.groups().iter().any(|(n, _)| n == &g.0)
        {
            new.groups.push(g.clone());
        }
    }
    Ok((new != *old).then_some(new))
}

/// What changed between two sources, in words (`orgs: +north −lab`, `groups`, `accents`, `account_map`, …).
pub fn describe(old: &Source, new: &Source) -> String {
    let mut parts = Vec::new();
    let names = |s: &Source| s.orgs.iter().map(|(a, _)| a.clone()).collect::<Vec<_>>();
    let (a, b) = (names(old), names(new));
    let added: Vec<String> = b
        .iter()
        .filter(|x| !a.contains(x))
        .map(|x| format!("+{x}"))
        .collect();
    let removed: Vec<String> = a
        .iter()
        .filter(|x| !b.contains(x))
        .map(|x| format!("−{x}"))
        .collect();
    if !added.is_empty() || !removed.is_empty() {
        parts.push(format!("orgs {}", [added, removed].concat().join(" ")));
    } else if old.orgs != new.orgs {
        parts.push("orgs".into());
    }
    if old.groups != new.groups {
        parts.push("groups".into());
    }
    if old.accents != new.accents || old.knobs != new.knobs || old.theme_pref != new.theme_pref {
        parts.push("theme settings".into());
    }
    if old.account_map != new.account_map {
        parts.push("account_map".into());
    }
    if old.ic_config != new.ic_config {
        parts.push("intercompany".into());
    }
    if old.view != new.view
        || old.as_of != new.as_of
        || old.log_rows != new.log_rows
        || old.compare_align != new.compare_align
        || old.label != new.label
    {
        parts.push("tui settings".into());
    }
    if parts.is_empty() {
        parts.push("settings".into());
    }
    parts.join(" · ")
}

/// The app on the new source, the view kept: export the view state, import it against the new config
/// (config drift allowed: this IS the config now), then carry over what belongs to this process. The note
/// says what changed, and whether the scope had to fall back.
pub fn apply(old: &App, source: Source) -> Result<App, String> {
    let text = old.canonical_value().to_string();
    let allow = crate::canonical::AllowDrift {
        record: false,
        config: true,
        classify: false,
        manifest: false,
        live: true,
    };
    let mut app = App::from_state_json_with(&text, source.clone(), allow)
        .map_err(|e| format!("{}: {}", e.code(), e.message()))?;
    // this process, not the view
    app.version_label = old.version_label.clone();
    app.build_label = old.build_label.clone();
    app.cli = old.cli.clone();
    app.cli_version = old.cli_version.clone();
    app.cli_shown = old.cli_shown.clone();
    app.paths = old.paths.clone();
    app.now = old.now;
    app.update_available = old.update_available.clone();
    app.user_config = old.user_config.clone();
    app.cmd_keys = old.cmd_keys;
    app.agent_client = old.agent_client.clone();
    app.data_refreshes = old.data_refreshes + 1;
    // `tui.label` applies unless the session was labelled otherwise (`--label`, the agent's `label`)
    app.label = if old.label == old.source.label {
        source.label.clone()
    } else {
        old.label.clone()
    };
    app.drift = None;
    let mut note = format!("config reloaded · {}", describe(&old.source, &source));
    let scope_before = old.scope_label(&old.ui.scope);
    let scope_after = app.scope_label(&app.ui.scope);
    if scope_before != scope_after {
        note.push_str(&format!(" · {scope_before} is gone, showing {scope_after}"));
    }
    // a band about a config file that is now valid goes
    if app
        .ui
        .result
        .as_ref()
        .is_some_and(|r| r.copy.starts_with(BAND))
    {
        app.ui.result = None;
    }
    app.ui.note = Some(note);
    app.agent.notice = None;
    app.rev = old.rev + 1;
    app.invalidate();
    app.dirty = true;
    Ok(app)
}

/// The error band for a config file that can't be used (the last valid config stays live).
pub fn invalid_band(why: &str) -> crate::run::ResultBand {
    crate::run::ResultBand {
        level: crate::run::ResultLevel::Error,
        lines: vec![
            format!("ownpurse.json: {why}"),
            "the previous config stays live; fix the file and save".into(),
        ],
        copy: format!("{BAND}: {why}"),
    }
}
