//! `ownpurse walkthrough new|add-step|show|list|validate|render`: guided, replayable explanations played in the
//! TUI (walkthroughs design §3). Files live in `walkthroughs.dir` (default `walkthroughs/` beside
//! ownpurse.json). Read-only for the books: nothing here calls Xero or writes the record.
//! `validate` and `render` lay the steps out headlessly with the installed `ownpurse-tui`.

use crate::app::{App, CliError, Res};
use crate::obj;
use ownpurse_core::walkthrough::{self as wt, Walkthrough};
use serde_json::{Value, json};

fn dir(app: &App) -> std::path::PathBuf {
    wt::dir(&app.config)
}

fn read(app: &App, id: &str) -> Res<Walkthrough> {
    wt::read(&dir(app), id).map_err(|e| {
        CliError::usage(e).hint("`ownpurse walkthrough list` shows the walkthroughs there are.")
    })
}

/// An anchor argument as caretline-layers JSON: `row:code=6140` → `{"host": {"kind": "row", "key": "code=6140"}}`;
/// JSON passes through.
pub fn anchor_json(a: &str) -> Res<Value> {
    if a.starts_with('{') || a.starts_with('[') {
        return serde_json::from_str(a).map_err(|e| CliError::usage(format!("anchor {a}: {e}")));
    }
    match a.split_once(':') {
        Some((kind, key)) if !kind.is_empty() && !key.is_empty() => {
            Ok(json!({"host": {"kind": kind, "key": key}}))
        }
        _ => Err(CliError::usage(format!(
            "`{a}` is not an anchor: row:<anchor>, cell:<row>/<column>, diff:<path>, chip:<name>, panel:<name>, viz:<id>."
        ))),
    }
}

pub fn new(app: &mut App, id: &str, title: &str, audience: Option<&str>) -> Res<()> {
    if !wt::valid_id(id) {
        return Err(CliError::usage(format!(
            "`{id}` is not a walkthrough id (letters, digits, - and _)."
        )));
    }
    if let Some(a) = audience
        && a != "lay"
        && a != "informed"
    {
        return Err(CliError::usage("--audience is `lay` or `informed`."));
    }
    let d = dir(app);
    if d.join(format!("{id}.json")).exists() {
        return Err(CliError::usage(format!(
            "Walkthrough `{id}` already exists."
        )));
    }
    let mut w = wt::new(id, title);
    w.meta = wt::Meta {
        author: Some("owner".into()),
        created: Some(crate::ledger::today_utc()),
        record: None,
        audience: audience.map(str::to_string),
    }
    .to_value();
    let p = wt::write(&d, &w).map_err(|e| CliError::runtime("io", e))?;
    let mut out = app.out();
    out.field("walkthrough", id)
        .field("file", p.display().to_string());
    out.hint(format!(
        "Add a step: ownpurse walkthrough add-step {id} --title \"…\" --text \"…\" [--anchor row:code=6140 --hint \"…\"]"
    ));
    out.emit();
    Ok(())
}

pub struct StepArgs {
    pub title: String,
    pub text: String,
    pub why: Option<String>,
    pub anchors: Vec<String>,
    pub hint: Option<String>,
    pub hint_title: Option<String>,
    pub arrow: bool,
    pub ring: bool,
    pub spotlight: bool,
    pub verbs: Vec<String>,
    pub checks: Vec<String>,
    pub viz: Option<String>,
}

pub fn add_step(app: &mut App, id: &str, a: StepArgs) -> Res<()> {
    let mut w = read(app, id)?;
    for c in &a.checks {
        wt::parse_check(c).map_err(CliError::usage)?;
    }
    for v in &a.verbs {
        let words: Vec<String> = v.split_whitespace().map(str::to_string).collect();
        let name = words.first().map(String::as_str).unwrap_or("");
        if !ownpurse_core::bridge::VIEW_ONLY.contains(&name) {
            return Err(CliError::usage(format!(
                "`{v}`: a step's view uses view verbs only ({}).",
                ownpurse_core::bridge::VIEW_ONLY.join(", ")
            )));
        }
        ownpurse_core::bridge::check_verb(&words).map_err(|(_, m)| CliError::usage(m))?;
    }
    let viz = match &a.viz {
        Some(s) => Some(
            serde_json::from_str::<Value>(s).map_err(|e| CliError::usage(format!("--viz: {e}")))?,
        ),
        None => None,
    };
    let n = w.steps.len() + 1;
    let mut sid = format!("s{n}");
    let mut k = n;
    while w.index(&sid).is_some() {
        k += 1;
        sid = format!("s{k}");
    }
    let mut layers = Vec::new();
    for (i, an) in a.anchors.iter().enumerate() {
        let hint = i == 0 && a.hint.is_some();
        let mut data = json!({});
        if hint {
            data = json!({"text": a.hint.clone().unwrap_or_default()});
            if let Some(t) = &a.hint_title {
                data["title"] = json!(t);
            }
        }
        layers.push(json!({
            "id": format!("{sid}/{i}"),
            "anchor": anchor_json(an)?,
            "kind": if hint { Value::from("hint") } else { Value::Null },
            "data": if hint { data } else { Value::Null },
            "place": {"arrow": hint && a.arrow, "ring": a.ring || !hint, "spotlight": a.spotlight},
        }));
    }
    let host = wt::StepHost {
        view: (!a.verbs.is_empty()).then(|| {
            json!({"verbs": a.verbs.iter().map(|v| v.split_whitespace().collect::<Vec<_>>()).collect::<Vec<_>>()})
        }),
        viz,
        checks: a.checks.clone(),
        why: a.why.clone(),
    };
    let mut step = json!({"id": sid, "narration": {"title": a.title, "text": a.text}});
    if !layers.is_empty() {
        step["layers"] = Value::Array(layers);
    }
    if let Some(h) = host.to_value() {
        step["host"] = h;
    }
    let step: wt::Step = serde_json::from_value(step).map_err(|e| {
        CliError::usage(format!("the step doesn't read as a walkthrough step: {e}"))
    })?;
    w.steps.push(step);
    // caretline-tour's lint on the whole walkthrough before it is written
    wt::parse(&wt::to_text(&w)).map_err(CliError::usage)?;
    let p = wt::write(&dir(app), &w).map_err(|e| CliError::runtime("io", e))?;
    let mut out = app.out();
    out.field("walkthrough", id)
        .field("step", n)
        .field("id", sid)
        .field("file", p.display().to_string());
    out.hint(format!("Check it: ownpurse walkthrough validate {id}"));
    out.emit();
    Ok(())
}

pub fn list(app: &mut App) -> Res<()> {
    let d = dir(app);
    let rows: Vec<_> = wt::list(&d)
        .into_iter()
        .map(|(id, w)| match w {
            Ok(w) => {
                let m = wt::Meta::of(&w);
                obj![
                    ("id", id),
                    ("title", w.title.clone()),
                    ("steps", w.steps.len()),
                    ("audience", m.audience.unwrap_or_default()),
                    ("created", m.created.unwrap_or_default()),
                    ("error", ""),
                ]
            }
            Err(e) => obj![
                ("id", id),
                ("title", ""),
                ("steps", 0),
                ("audience", ""),
                ("created", ""),
                ("error", e),
            ],
        })
        .collect();
    let n = rows.len();
    let mut out = app.out();
    out.field("dir", d.display().to_string());
    out.table(
        "walkthroughs",
        &["id", "title", "steps", "audience", "created", "error"],
        rows,
        None,
    );
    if n == 0 {
        out.hint("Create one: ownpurse walkthrough new <id> --title \"…\" (or record one from the TUI with an agent).");
    } else {
        out.hint("Play one in the TUI: W, or ownpurse-tui --walkthrough <id>");
    }
    out.emit();
    Ok(())
}

pub fn show(app: &mut App, id: &str) -> Res<()> {
    let w = read(app, id)?;
    let rows: Vec<_> = w
        .steps
        .iter()
        .enumerate()
        .map(|(i, s)| {
            let host = wt::StepHost::of(s);
            obj![
                ("n", i + 1),
                ("id", s.id.as_str()),
                ("title", wt::step_title(s)),
                ("layers", s.layers.len()),
                (
                    "viz",
                    host.viz
                        .as_ref()
                        .and_then(|v| v["kind"].as_str())
                        .unwrap_or("")
                        .to_string()
                ),
                ("checks", host.checks.len()),
            ]
        })
        .collect();
    let m = wt::Meta::of(&w);
    let mut out = app.out();
    out.field("id", w.id.as_str())
        .field("title", w.title.as_str())
        .field("audience", m.audience.unwrap_or_default())
        .field("author", m.author.unwrap_or_default())
        .field("created", m.created.unwrap_or_default());
    out.table(
        "steps",
        &["n", "id", "title", "layers", "viz", "checks"],
        rows,
        None,
    );
    out.emit();
    Ok(())
}

fn tui() -> std::path::PathBuf {
    std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|d| d.join("ownpurse-tui")))
        .filter(|p| p.exists())
        .unwrap_or_else(|| std::path::PathBuf::from("ownpurse-tui"))
}

fn tui_cmd(app: &App) -> std::process::Command {
    let mut cmd = std::process::Command::new(tui());
    if let Some(p) = &app.config.path {
        cmd.arg("--config").arg(p);
    }
    if let Some(p) = &app.g.profile {
        cmd.arg("--profile").arg(p);
    }
    cmd.arg("--no-agent");
    cmd
}

fn run(cmd: &mut std::process::Command) -> Res<std::process::Output> {
    cmd.output().map_err(|e| {
        CliError::runtime("tui_missing", format!("Could not run ownpurse-tui: {e}"))
            .hint("Install both binaries with ./install.sh.")
    })
}

pub fn validate(app: &mut App, id: &str) -> Res<()> {
    read(app, id)?;
    let res = run(tui_cmd(app).arg("--walkthrough-validate").arg(id))?;
    let text = String::from_utf8_lossy(&res.stdout).to_string();
    if !res.status.success() && res.status.code() != Some(4) {
        return Err(CliError::runtime(
            "render",
            String::from_utf8_lossy(&res.stderr).trim().to_string(),
        ));
    }
    let rows: Vec<_> = text
        .lines()
        .filter_map(|l| serde_json::from_str::<Value>(l).ok())
        .filter_map(|v| v.as_object().cloned())
        .collect();
    let n = rows.len();
    let mut out = app.out();
    out.field("walkthrough", id)
        .field("ok", n == 0)
        .field("sizes", "140x40 100x30 80x24");
    out.table(
        "findings",
        &["step", "step_id", "size", "kind", "detail"],
        rows,
        None,
    );
    if n > 0 {
        out.hint("not_found / off_screen: the step's view doesn't show the anchor at that size; set the view (select, drill) or pick another anchor.");
        out.hint("check: the books changed since the walkthrough was written; update the step or its check.");
    }
    out.emit();
    if n > 0 {
        return Err(CliError::new(
            "walkthrough_invalid",
            4,
            format!("{n} finding{} (above).", if n == 1 { "" } else { "s" }),
        ));
    }
    Ok(())
}

pub fn render(
    app: &mut App,
    id: &str,
    out_dir: &std::path::Path,
    format: &str,
    sizes: &str,
) -> Res<()> {
    read(app, id)?;
    let res = run(tui_cmd(app)
        .arg("--walkthrough-render")
        .arg(id)
        .arg("--out")
        .arg(out_dir)
        .arg("--format")
        .arg(format)
        .arg("--sizes")
        .arg(sizes))?;
    if !res.status.success() {
        return Err(CliError::runtime(
            "render",
            String::from_utf8_lossy(&res.stderr).trim().to_string(),
        ));
    }
    let mut out = app.out();
    out.field("walkthrough", id)
        .field("out", out_dir.display().to_string())
        .field(
            "rendered",
            String::from_utf8_lossy(&res.stdout).trim().to_string(),
        );
    out.emit();
    Ok(())
}
