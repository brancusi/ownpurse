//! `viz <kind> [target] [account] [--as-of] [--from] [--to] [--orgs] [--basis] [--top] [--spec JSON]`: the
//! explainer models the TUI draws (walkthroughs §4), as text or `--json`, so an agent can read the numbers it
//! is about to show. Read-only, zero calls; main or main + the current manifest, as every read command.

use crate::app::{App, CliError, Res};
use ownpurse_core::intercompany::{self as icm, IcContext};
use ownpurse_core::viz::{self, Extra, FlowBasis, Spec};
use serde_json::{Value, json};
use std::collections::HashMap;

pub struct Args {
    pub kind: String,
    pub target: Option<String>,
    pub account: Option<String>,
    pub as_of: Option<String>,
    pub from: Option<String>,
    pub to: Option<String>,
    pub orgs: Option<String>,
    pub basis: Option<String>,
    pub top: Option<usize>,
    pub spec: Option<String>,
}

fn need(v: &Option<String>, what: &str, kind: &str) -> Res<String> {
    v.clone()
        .ok_or_else(|| CliError::usage(format!("`viz {kind}` needs {what}.")))
}

/// The spec from the arguments (or `--spec`, which wins).
fn spec_of(a: &Args) -> Res<Spec> {
    if let Some(s) = &a.spec {
        return Spec::parse(s).map_err(|e| CliError::usage(e.message));
    }
    let k = a.kind.as_str();
    let s = match k {
        "bridge" => Spec::Bridge {
            pair: need(&a.target, "a pair id (`ownpurse ic` lists them)", k)?,
            as_of: a.as_of.clone(),
        },
        "flow" => Spec::Flow {
            orgs: a.orgs.as_ref().map(|o| {
                o.split(',')
                    .map(str::trim)
                    .filter(|s| !s.is_empty())
                    .map(str::to_string)
                    .collect()
            }),
            as_of: a.as_of.clone(),
            from: a.from.clone(),
            to: a.to.clone(),
            basis: match a.basis.as_deref() {
                None | Some("owed") => FlowBasis::Owed,
                Some("paid") => FlowBasis::Paid,
                Some(o) => {
                    return Err(CliError::usage(format!(
                        "--basis is `owed` or `paid`, got `{o}`."
                    )));
                }
            },
        },
        "t_account" => Spec::TAccount {
            org: need(&a.target, "an org alias", k)?,
            account: need(&a.account, "an account code", k)?,
            from: a.from.clone(),
            to: a.to.clone().or(a.as_of.clone()),
            top: a.top,
        },
        "timeline" => Spec::Timeline {
            org: need(&a.target, "an org alias", k)?,
            account: need(&a.account, "an account code", k)?,
            from: a.from.clone(),
            to: a.to.clone().or(a.as_of.clone()),
        },
        "before_after" => Spec::BeforeAfter {
            manifest: None,
            org: need(&a.target, "an org alias", k)?,
        },
        "checklist" => Spec::Checklist {
            file: need(
                &a.target,
                "the worklist file (relative to the config's folder)",
                k,
            )?,
        },
        other => {
            return Err(CliError::usage(format!(
                "Unknown viz kind `{other}`: {}.",
                viz::KINDS.join(", ")
            )));
        }
    };
    s.check().map_err(|e| CliError::usage(e.message))?;
    Ok(s)
}

pub fn viz(app: &mut App, a: &Args) -> Res<()> {
    let spec = spec_of(a)?;
    let st = &app.config.settings;
    let or_empty = |k: &str| {
        st.get(k)
            .filter(|v| !v.is_null())
            .cloned()
            .unwrap_or_else(|| json!({}))
    };
    let cfg = json!({"parties": or_empty("parties"), "intercompany": or_empty("intercompany")});
    let tenants: HashMap<String, String> = app.config.profile.orgs.iter().cloned().collect();
    let orgs: Vec<(String, String)> = app
        .config
        .profile
        .orgs
        .iter()
        .map(|(a, _)| (a.clone(), a.clone()))
        .collect();
    let as_of = match &a.as_of {
        Some(d) => d.clone(),
        None => match app.config.profile.orgs.first() {
            Some((_, tid)) => {
                let tid = tid.clone();
                let fy = app.record()?.fy_end(&tid)?;
                icm::default_as_of(fy, &crate::ledger::today_utc()).unwrap_or_default()
            }
            None => return Err(CliError::usage("No orgs in this profile.")),
        },
    };
    let effects = match &spec {
        Spec::BeforeAfter { .. } => {
            let Some(m) = app.current_manifest()? else {
                return Err(CliError::usage(
                    "before_after needs a manifest: pass --manifest <name> or switch to one.",
                ));
            };
            let ctx = app.view_ctx();
            let zero = ownpurse_core::manifest::zero_accounts(&app.config);
            let rec = app.record()?;
            Some(ownpurse_core::manifest::effects::diff(
                rec,
                &ctx,
                &m,
                &as_of,
                Some(&cfg),
                &zero,
            )?)
        }
        _ => None,
    };
    let base_dir = Some(app.config.root.clone());
    let today = crate::ledger::today_utc();
    let as_json = app.g.json;
    let model = {
        let v = app.view()?;
        let ctx = IcContext::new(v, tenants, &cfg, &as_of);
        let extra = Extra {
            today,
            base_dir,
            effects: effects.as_ref(),
            orgs,
        };
        viz::compute(&ctx, &spec, &extra).map_err(|e| match e.code {
            "usage" => CliError::usage(e.message),
            "not_found" => CliError::usage(e.message),
            _ => CliError::internal(e.message),
        })?
    };
    use std::io::Write;
    let mut so = std::io::stdout().lock();
    if as_json {
        let doc = json!({
            "spec": serde_json::to_value(&spec).unwrap_or(Value::Null),
            "as_of": as_of,
            "lead": model.lead(),
            "parts": model.ids(),
            "model": serde_json::to_value(&model).unwrap_or(Value::Null),
        });
        let _ = writeln!(
            so,
            "{}",
            serde_json::to_string_pretty(&doc).unwrap_or_default()
        );
    } else {
        for l in model.text() {
            let _ = writeln!(so, "{l}");
        }
    }
    Ok(())
}
