//! `ic [pair] [--as-of]`: intercompany pairs from config, read-only, zero calls. `--json`
//! prints the intercompany envelope; text uses the axi format.

use crate::app::{App, CliError, Res};
use crate::cmds::{Row, g, truthy};
use crate::obj;
use crate::out::{py_json_opts, py_str};
use ownpurse_core::config::COMING;
use ownpurse_core::intercompany::{self as icm, IcContext};
use serde_json::{Map, Value, json};
use std::collections::HashMap;

/// `%Y-%m-%d %H:%M UTC` now.
fn stamp() -> String {
    let secs = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0);
    let (y, m, d) = ownpurse_core::dates::civil_from_days(secs.div_euclid(86_400));
    let rem = secs.rem_euclid(86_400);
    format!(
        "{} {:02}:{:02} UTC",
        ownpurse_core::dates::iso(y, m, d),
        rem / 3600,
        (rem % 3600) / 60
    )
}

fn obj_of(v: &Value) -> Row {
    v.as_object().cloned().unwrap_or_default()
}

pub fn ic(app: &mut App, pair: Option<&str>, as_of: Option<&str>) -> Res<()> {
    let st = &app.config.settings;
    let or_empty = |k: &str| {
        st.get(k)
            .filter(|v| truthy(v))
            .cloned()
            .unwrap_or_else(|| json!({}))
    };
    let cfg = json!({"parties": or_empty("parties"), "intercompany": or_empty("intercompany")});
    let pairs: Vec<Value> = cfg["intercompany"]
        .get("pairs")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    if pairs.is_empty() {
        return Err(CliError::usage("No intercompany pairs configured.").hint(
            "Add \"intercompany.pairs\" (and \"parties\") to ownpurse.json; see docs/ROADMAP.md §2.",
        ));
    }
    let id_of = |p: &Value| p.get("id").map(py_str).unwrap_or_default();
    if let Some(want) = pair
        && !pairs.iter().any(|p| id_of(p) == want)
    {
        return Err(
            CliError::usage(format!("Unknown pair '{want}'.")).hint(format!(
                "Pairs: {}",
                pairs.iter().map(id_of).collect::<Vec<_>>().join(", ")
            )),
        );
    }
    let tenants: HashMap<String, String> = app.config.profile.orgs.iter().cloned().collect();
    let first_pair = pairs
        .iter()
        .find(|p| pair.is_none_or(|w| id_of(p) == w))
        .expect("checked above");
    let first = first_pair["a"]["org"].as_str().unwrap_or("").to_string();
    let as_of = match as_of {
        Some(a) => a.to_string(),
        None => match tenants.get(&first) {
            Some(tid) => {
                let fy = app.record()?.fy_end(tid)?;
                icm::default_as_of(fy, &crate::ledger::today_utc()).unwrap_or_default()
            }
            None => {
                return Err(CliError::usage(
                    "Pass --as-of YYYY-MM-DD (the first pair's org has no alias in this profile).",
                ));
            }
        },
    };
    let store = app.config.store().display().to_string();
    let as_json = app.g.json;
    let result = {
        let v = app.view()?;
        let ctx = IcContext::new(v, tenants, &cfg, &as_of);
        let ids: Vec<String> = pair.map(|p| vec![p.to_string()]).unwrap_or_default();
        icm::build(&ctx, pair.map(|_| ids.as_slice())).map_err(CliError::internal)?
    };
    if as_json {
        let env = icm::envelope(&result, &store, &stamp());
        let mut s = py_json_opts(&env, false);
        s.push('\n');
        use std::io::Write;
        let mut so = std::io::stdout().lock();
        let _ = so.write_all(s.as_bytes());
        return Ok(());
    }
    let mut out = app.out();
    crate::manifest::view_note(&mut out, app);
    out.field("as_of", as_of.as_str())
        .field("basis", result["basis"].clone());
    let summary: Map<String, Value> = obj_of(&result["summary"])
        .into_iter()
        .filter(|(k, v)| truthy(v) || k == "pairs" || k == "unexplained")
        .collect();
    out.field("summary", Value::Object(summary));
    let mut reps: Vec<Value> = result["pairs"].as_array().cloned().unwrap_or_default();
    reps.sort_by(icm::sort_cmp("status"));
    let side = |org: &Value, net: Option<&Value>| {
        let n = net.filter(|v| truthy(v)).map(py_str).unwrap_or_default();
        format!("{} {n}", py_str(org)).trim().to_string()
    };
    let rows: Vec<Row> = reps
        .iter()
        .map(|p| {
            let reason = p.get("reason").filter(|r| truthy(r)).map(py_str);
            obj![
                ("pair", g(&obj_of(p), "id")),
                ("a", side(&p["a_org"], p.get("a_net"))),
                ("b", side(&p["b_org"], p.get("b_party_net"))),
                (
                    "difference",
                    p.get("difference").cloned().unwrap_or(Value::Null)
                ),
                (
                    "residual",
                    p.get("residual").cloned().unwrap_or(Value::Null)
                ),
                (
                    "status",
                    format!(
                        "{}{}",
                        py_str(&p["status"]),
                        reason.map(|r| format!(" ({r})")).unwrap_or_default()
                    )
                ),
            ]
        })
        .collect();
    out.table(
        "pairs",
        &["pair", "a", "b", "difference", "residual", "status"],
        rows,
        None,
    );
    let all: Vec<Value> = result["pairs"].as_array().cloned().unwrap_or_default();
    if pair.is_some()
        && let Some(p) = all.first()
        && !p.get("opening_difference").is_none_or(Value::is_null)
    {
        let get = |k: &str| p.get(k).cloned().unwrap_or(Value::Null);
        out.field(
            "bridge",
            Value::Object(obj![
                ("difference", get("difference")),
                ("opening", get("opening_difference")),
                ("movement", get("movement_difference")),
                ("explained", get("movement_explained")),
                ("residual", get("residual")),
            ]),
        );
        let items: Vec<Row> = p
            .get("movement_items")
            .and_then(Value::as_array)
            .map(|a| a.iter().map(obj_of).collect())
            .unwrap_or_default();
        out.table(
            "items",
            &["side", "date", "org", "account", "contact", "amount", "why"],
            items,
            None,
        );
        for k in p
            .get("known_items")
            .and_then(Value::as_array)
            .into_iter()
            .flatten()
        {
            let found = k.get("found").is_some_and(truthy);
            out.hint(format!(
                "{}{}",
                if found {
                    "✓ found: "
                } else {
                    "! not found in the record: "
                },
                k.get("text").map(py_str).unwrap_or_default()
            ));
        }
    }
    for p in &all {
        if p["status"] == "not compared" {
            out.hint(format!(
                "{}: {} — fetching it needs `ownpurse report` ({COMING}), 1 call per org.",
                py_str(&p["id"]),
                py_str(&p["reason"]),
            ));
        }
    }
    if pair.is_none() {
        out.hint(
            "`ownpurse ic <pair-id>` shows the bridge and its items; `--json` gives the full envelope.",
        );
    }
    out.emit();
    Ok(())
}
