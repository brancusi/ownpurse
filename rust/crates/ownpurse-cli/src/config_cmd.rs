//! `config [show|path|links|edit]`.

use crate::app::{App, CliError, Res};
use crate::cmds::{Row, g, gs};
use crate::obj;
use crate::out::py_str;
use ownpurse_core::config::CREATE_CONFIG;
use serde_json::{Map, Value};

/// Fill `{name}` placeholders; strict fails on a missing/empty value.
fn fill(template: &str, values: &Map<String, Value>, strict: bool) -> Option<String> {
    let mut out = String::with_capacity(template.len());
    let mut rest = template;
    while let Some(start) = rest.find('{') {
        out.push_str(&rest[..start]);
        let after = &rest[start + 1..];
        let name_len = after
            .chars()
            .take_while(|c| c.is_ascii_lowercase() || *c == '_')
            .count();
        if name_len > 0 && after[name_len..].starts_with('}') {
            let k = &after[..name_len];
            match values.get(k) {
                Some(v) if !matches!(v, Value::Null) && v.as_str() != Some("") => {
                    out.push_str(&py_str(v))
                }
                _ if strict => return None,
                _ => out.push_str(&rest[start..start + name_len + 2]),
            }
            rest = &after[name_len + 1..];
        } else {
            out.push('{');
            rest = after;
        }
    }
    out.push_str(rest);
    Some(out)
}

fn links(app: &mut App) -> Res<Vec<Row>> {
    let tids: Vec<String> = {
        let mut v: Vec<String> = Vec::new();
        for (_, t) in &app.config.profile.orgs {
            if !v.contains(t) {
                v.push(t.clone());
            }
        }
        v
    };
    let mut orgs: Vec<Row> = Vec::new();
    let mut banks: Vec<Row> = Vec::new();
    for tid in &tids {
        let alias = app.alias_of(tid);
        let rec = app.record()?;
        let o = match rec.organisation(tid)? {
            Value::Object(m) => m,
            _ => Map::new(),
        };
        let name = gs(&o, "Name");
        let short = g(&o, "ShortCode");
        orgs.push(obj![
            ("org", alias.as_str()),
            ("org_name", name.as_str()),
            ("short_code", short.clone()),
            ("tenant_id", tid.as_str()),
        ]);
        for a in ownpurse_core::view::View::main(rec).payloads(tid, "Accounts")? {
            let Value::Object(p) = a else { continue };
            if p.get("Type").and_then(Value::as_str) == Some("BANK")
                && p.get("Status").and_then(Value::as_str) == Some("ACTIVE")
            {
                banks.push(obj![
                    ("org", alias.as_str()),
                    ("org_name", name.as_str()),
                    ("short_code", short.clone()),
                    ("account_id", g(&p, "AccountID")),
                    ("account_name", g(&p, "Name")),
                ]);
            }
        }
    }
    let ctx = obj![
        ("client_id", app.config.profile.client_id.as_str()),
        ("redirect_uri", app.config.settings.redirect_uri()),
        ("profile", app.config.profile.name.as_str()),
    ];
    let mut out = Vec::new();
    let templates = app
        .config
        .settings
        .get("links")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    let app_scope = vec![Row::new()];
    for link in templates {
        let Value::Object(link) = link else { continue };
        let per = link
            .get("per")
            .and_then(Value::as_str)
            .unwrap_or("app")
            .to_string();
        if per == "document" {
            continue;
        }
        let scopes: &[Row] = match per.as_str() {
            "org" => &orgs,
            "bank_account" => &banks,
            _ => &app_scope,
        };
        for item in scopes {
            let mut values = ctx.clone();
            for (k, v) in item {
                values.insert(k.clone(), v.clone());
            }
            let (Some(name), Some(url)) = (
                fill(&gs(&link, "name"), &values, true),
                fill(&gs(&link, "url"), &values, true),
            ) else {
                continue;
            };
            let description = fill(&gs(&link, "description"), &values, false).unwrap_or_default();
            out.push(obj![
                ("name", name),
                ("url", url),
                ("description", description),
                ("per", per.as_str()),
                ("org", g(item, "org")),
            ]);
        }
    }
    Ok(out)
}

pub fn config(app: &mut App, action: &str) -> Res<()> {
    let mut out = app.out();
    let path_text = app.config.path.as_ref().map(|p| p.display().to_string());
    match action {
        "path" => {
            out.field(
                "config",
                path_text.unwrap_or_else(|| "(none: environment defaults)".into()),
            );
            out.field("profile", app.config.profile.name.as_str())
                .field(
                    "token_file",
                    app.config.profile.token_file.display().to_string(),
                )
                .field("store", app.config.store().display().to_string());
        }
        "links" => {
            let rows = links(app)?;
            let mut cols = vec!["name", "url", "per"];
            if app.g.full {
                cols.push("description");
            }
            out.table("links", &cols, app.limit(&rows), Some(rows.len()));
            out.hint(
                "Add links under `links` / `extra_links` in the config; placeholders: {org_name} \
                 {short_code} {tenant_id} {account_id} {account_name} {client_id} {redirect_uri}.",
            );
        }
        "edit" => {
            let Some(path) = app.config.path.clone() else {
                return Err(CliError::usage("No config file to edit.").hint(CREATE_CONFIG));
            };
            let editor = app
                .config
                .settings
                .get_str("tui.editor")
                .filter(|s| !s.is_empty())
                .map(str::to_string)
                .or_else(|| std::env::var("VISUAL").ok().filter(|s| !s.is_empty()))
                .or_else(|| std::env::var("EDITOR").ok().filter(|s| !s.is_empty()))
                .unwrap_or_else(|| "vi".into());
            let mut parts = editor.split_whitespace();
            let prog = parts.next().unwrap_or("vi");
            let _ = std::process::Command::new(prog)
                .args(parts)
                .arg(&path)
                .status();
            out.field("edited", path.display().to_string());
        }
        _ => {
            out.field("config", path_text)
                .field("profile", app.config.profile.name.as_str());
            let st = &app.config.settings;
            let mut rows: Vec<Row> = st
                .flat()
                .into_iter()
                .map(|(k, v, o)| {
                    let value = match &v {
                        Value::Array(a) => {
                            Value::String(a.iter().map(py_str).collect::<Vec<_>>().join(", "))
                        }
                        other => other.clone(),
                    };
                    obj![("key", k), ("value", value), ("origin", o)]
                })
                .collect();
            if !app.g.full {
                rows.retain(|r| {
                    gs(r, "origin") != "defaults" || !gs(r, "key").starts_with("xero.scopes")
                });
            }
            out.table("settings", &["key", "value", "origin"], rows, None);
            let n = st
                .get("links")
                .and_then(Value::as_array)
                .map(Vec::len)
                .unwrap_or(0);
            out.field("links", n);
            out.hint(
                "Override any key in the config file (top level or per profile); `--full` shows every scope.",
            );
        }
    }
    out.emit();
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fill_placeholders() {
        let v = obj![("org_name", "Tallowmere"), ("short_code", Value::Null)];
        assert_eq!(
            fill("Open {org_name}", &v, true).as_deref(),
            Some("Open Tallowmere")
        );
        assert_eq!(fill("x {short_code}", &v, true), None);
        assert_eq!(
            fill("x {short_code} {Org}", &v, false).as_deref(),
            Some("x {short_code} {Org}")
        );
    }
}
