//! `ownpurse-facts`: the fact model from a terminal. Every command reads or appends one log.

mod load;
mod out;

use clap::{Parser, Subcommand};
use ownpurse_facts::{Db, Eid, Instant, Kw, Pattern, TxData, Value, pull};
use ownpurse_store::Store;
use serde_json::{Value as Json, json};
use std::path::PathBuf;
use std::process::ExitCode;

#[derive(Parser)]
#[command(
    name = "ownpurse-facts",
    about = "The fact model: ingest evidence, transact, query, verify."
)]
struct Cli {
    /// The log (SQLite). Its drawer sits beside it: books.sqlite → books.drawer/
    #[arg(
        long,
        env = "OWNPURSE_LOG",
        global = true,
        default_value = "books.sqlite"
    )]
    log: PathBuf,
    /// Where the drawer is, if not beside the log
    #[arg(long, env = "OWNPURSE_DRAWER", global = true)]
    drawer: Option<PathBuf>,
    /// The clock, for reproducible logs (RFC 3339, UTC)
    #[arg(long, env = "OWNPURSE_AT", global = true)]
    at: Option<String>,
    /// Read the database as it was right after this transaction
    #[arg(long, global = true)]
    as_of: Option<u64>,
    /// JSON output
    #[arg(long, global = true)]
    json: bool,
    #[command(subcommand)]
    cmd: Cmd,
}

#[derive(Subcommand)]
enum Cmd {
    /// Create a log with genesis and the core vocabulary
    Init,
    /// Ingest one file with a projector
    Ingest {
        file: PathBuf,
        #[arg(long)]
        projector: String,
        /// Source id, e.g. xero:tallowmere
        #[arg(long)]
        source: String,
        #[arg(long, default_value = "")]
        source_kind: String,
        #[arg(long, default_value = "")]
        source_name: String,
        /// A companion file: role=path (mapping=…, reads=…)
        #[arg(long = "with")]
        with: Vec<String>,
        /// When the source was read (defaults to now)
        #[arg(long)]
        observed_at: Option<String>,
    },
    /// Ingest every file a manifest lists, in order
    Load { manifest: PathBuf },
    /// Append a transaction written as JSON (a file, or - for stdin)
    Transact {
        file: PathBuf,
        /// Who is writing: person:<name>, agent:<name>
        #[arg(long)]
        actor: String,
        #[arg(long)]
        reason: Option<String>,
        /// Show what would change, write nothing
        #[arg(long)]
        dry_run: bool,
    },
    /// Every current fact about an entity (id, or attr=value for a unique attribute)
    Entity { entity: String },
    /// Pull a pattern from an entity: '["*", {"obs/_doc": ["*"]}]'
    Pull { entity: String, pattern: String },
    /// Every assertion and retraction about an entity, with who and when
    History { entity: String },
    /// The transactions, newest last
    Log {
        #[arg(long, default_value_t = 30)]
        limit: usize,
    },
    /// Read-only SQL over the log's views (facts, history, transactions, files, observations, links, postings);
    /// dsum(text) sums decimals exactly
    Sql {
        #[arg(allow_hyphen_values = true)]
        query: Option<String>,
        /// Read the query from a file (samples/queries/*.sql)
        #[arg(short = 'f', long)]
        file: Option<PathBuf>,
    },
    /// The installed attributes (optionally one namespace prefix)
    Schema { prefix: Option<String> },
    /// Where sources did not fit the model, grouped by kind
    Pressure {
        /// Show every line, not just counts
        #[arg(long)]
        all: bool,
    },
    /// Run the link rules (same-event, evidences, counterpart) and append what they propose
    Link {
        /// Show the links, write nothing
        #[arg(long)]
        dry_run: bool,
    },
    /// Re-derive the hash chain and re-hash every cited drawer file
    Verify,
    /// The projectors this build knows
    Projectors,
}

fn now(cli: &Cli) -> Result<Instant, String> {
    if let Some(at) = &cli.at {
        return Instant::parse(at).map_err(|e| e.0);
    }
    let ms = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_err(|e| e.to_string())?
        .as_millis();
    Ok(Instant::from_unix_millis(
        i64::try_from(ms).unwrap_or(i64::MAX),
    ))
}

fn open(cli: &Cli) -> Result<Store, String> {
    Store::open(&cli.log, cli.drawer.clone()).map_err(|e| e.to_string())
}

fn db_view(cli: &Cli, store: &Store) -> Db {
    match cli.as_of {
        Some(t) => store.db().as_of(Eid(t)),
        None => store.db().clone(),
    }
}

/// `123`, or `attr=value` for a unique attribute (the value read as that attribute's type).
fn entity(db: &Db, text: &str) -> Result<Eid, String> {
    if let Ok(n) = text.parse::<u64>() {
        return Ok(Eid(n));
    }
    let (a, v) = text
        .split_once('=')
        .ok_or_else(|| format!("{text:?}: give an id or attr=value"))?;
    let kw = Kw::new(a).map_err(|e| e.to_string())?;
    let def = db.attr(&kw).ok_or_else(|| format!("no attribute {a}"))?;
    let val = Value::from_json(def.ty, &Json::String(v.to_string()))
        .or_else(|_| Value::from_json(def.ty, &serde_json::from_str(v).unwrap_or(Json::Null)))
        .map_err(|e| e.0)?;
    db.lookup(&kw, &val)
        .ok_or_else(|| format!("no entity has {a} = {v}"))
}

fn run(cli: &Cli) -> Result<(), String> {
    match &cli.cmd {
        Cmd::Init => {
            let s = Store::create(&cli.log, cli.drawer.clone(), &now(cli)?)
                .map_err(|e| e.to_string())?;
            out::emit(
                cli.json,
                &json!({"log": cli.log.display().to_string(), "drawer": s.drawer().root().display().to_string(),
                        "attributes": s.db().schema().len(), "head": s.head().hex()}),
                &format!(
                    "created {} (drawer {}), {} attributes\nhelp[1]: ownpurse-facts load <manifest.json>",
                    cli.log.display(),
                    s.drawer().root().display(),
                    s.db().schema().len()
                ),
            );
        }
        Cmd::Ingest {
            file,
            projector,
            source,
            source_kind,
            source_name,
            with,
            observed_at,
        } => {
            let mut store = open(cli)?;
            let with = with
                .iter()
                .map(|w| {
                    w.split_once('=')
                        .map(|(r, p)| (r.to_string(), PathBuf::from(p)))
                        .ok_or(format!("--with {w:?}: role=path"))
                })
                .collect::<Result<Vec<_>, _>>()?;
            let observed_at = match observed_at {
                Some(t) => Instant::parse(t).map_err(|e| e.0)?,
                None => now(cli)?,
            };
            let kind = if source_kind.is_empty() {
                source.split(':').next().unwrap_or("").to_string()
            } else {
                source_kind.clone()
            };
            let job = load::Job {
                file: file.clone(),
                projector: projector.clone(),
                source: ownpurse_ingest::SourceSpec {
                    id: source.clone(),
                    kind,
                    name: if source_name.is_empty() {
                        source.clone()
                    } else {
                        source_name.clone()
                    },
                },
                with,
                observed_at,
            };
            let r = load::run(&mut store, &job, &now(cli)?)?;
            out::loaded(cli.json, &[r]);
        }
        Cmd::Load { manifest } => {
            let mut store = open(cli)?;
            let jobs = load::manifest(manifest)?;
            let mut done = Vec::new();
            for j in &jobs {
                done.push(load::run(&mut store, j, &now(cli)?)?);
            }
            out::loaded(cli.json, &done);
        }
        Cmd::Transact {
            file,
            actor,
            reason,
            dry_run,
        } => {
            let mut store = open(cli)?;
            let text = if file.as_os_str() == "-" {
                std::io::read_to_string(std::io::stdin()).map_err(|e| e.to_string())?
            } else {
                std::fs::read_to_string(file).map_err(|e| format!("{}: {e}", file.display()))?
            };
            let j: Json = serde_json::from_str(&text).map_err(|e| format!("not JSON: {e}"))?;
            let mut tx = TxData::from_json(&j).map_err(|e| e.0)?;
            tx.meta(&Kw::lit("tx/actor"), Value::str(actor));
            if let Some(r) = reason {
                tx.meta(&Kw::lit("tx/reason"), Value::str(r));
            }
            let at = now(cli)?;
            let r = if *dry_run {
                store.db().with(&tx, &at).map_err(|e| e.to_string())?
            } else {
                store.transact(&tx, &at).map_err(|e| e.to_string())?
            };
            out::report(cli.json, store.db(), &r, *dry_run);
        }
        Cmd::Entity { entity: e } => {
            let store = open(cli)?;
            let db = db_view(cli, &store);
            let e = entity(&db, e)?;
            out::json_pretty(&pull(&db, &Pattern::wild(), e));
        }
        Cmd::Pull { entity: e, pattern } => {
            let store = open(cli)?;
            let db = db_view(cli, &store);
            let e = entity(&db, e)?;
            let p: Json = serde_json::from_str(pattern)
                .map_err(|err| format!("pattern is not JSON: {err}"))?;
            let p = Pattern::from_json(&p)?;
            out::json_pretty(&pull(&db, &p, e));
        }
        Cmd::History { entity: e } => {
            let store = open(cli)?;
            let db = store.db();
            let e = entity(db, e)?;
            out::history(cli.json, db, e);
        }
        Cmd::Log { limit } => {
            let store = open(cli)?;
            out::sql(
                cli.json,
                store.conn(),
                &format!(
                    "SELECT * FROM (SELECT * FROM transactions ORDER BY tx DESC LIMIT {limit}) ORDER BY tx"
                ),
            )?;
        }
        Cmd::Sql { query, file } => {
            let q = match (query, file) {
                (Some(q), None) => q.clone(),
                (None, Some(f)) => {
                    std::fs::read_to_string(f).map_err(|e| format!("{}: {e}", f.display()))?
                }
                _ => return Err("give a query or --file <path>, not both".into()),
            };
            if !cli.log.exists() {
                return Err(format!(
                    "no log at {}; create one with `init` or pass --log",
                    cli.log.display()
                ));
            }
            let conn = rusqlite::Connection::open_with_flags(
                &cli.log,
                rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY,
            )
            .map_err(|e| format!("{}: {e}", cli.log.display()))?;
            ownpurse_store::sql::register(&conn).map_err(|e| e.to_string())?;
            out::sql(cli.json, &conn, &q)?;
        }
        Cmd::Schema { prefix } => {
            let store = open(cli)?;
            out::schema(cli.json, store.db(), prefix.as_deref());
        }
        Cmd::Pressure { all } => {
            let store = open(cli)?;
            out::pressure(cli.json, store.db(), *all);
        }
        Cmd::Link { dry_run } => {
            let mut store = open(cli)?;
            let mut tx = ownpurse_ingest::rules::all(store.db());
            tx.meta(
                &Kw::lit("tx/actor"),
                Value::str("rules:same-event@1,evidences@1,counterpart@1"),
            );
            tx.meta(
                &Kw::lit("tx/reason"),
                Value::str("witnesses of one movement of money, the evidence behind it, and invoices seen from both sides"),
            );
            let at = now(cli)?;
            let r = if *dry_run {
                store.db().with(&tx, &at).map_err(|e| e.to_string())?
            } else {
                store.transact(&tx, &at).map_err(|e| e.to_string())?
            };
            let links = r
                .datoms
                .iter()
                .filter(|d| {
                    d.added
                        && store
                            .db()
                            .schema()
                            .ident(d.a)
                            .is_some_and(|i| i.as_str() == "link/id")
                })
                .count();
            if !cli.json {
                println!(
                    "{links} new links{}",
                    if *dry_run {
                        " (dry run: nothing written)"
                    } else {
                        ""
                    }
                );
                println!("help[1]: ownpurse-facts sql \"SELECT * FROM links\"");
            } else {
                println!(
                    "{}",
                    json!({"links": links, "tx": r.tx.map(|t| t.0), "dry_run": dry_run})
                );
            }
        }
        Cmd::Verify => {
            let store = open(cli)?;
            let v = store.verify().map_err(|e| e.to_string())?;
            out::emit(
                cli.json,
                &json!({"transactions": v.transactions, "datoms": v.datoms, "files": v.files, "redacted": v.redacted, "head": v.head}),
                &format!(
                    "ok: {} transactions, {} datoms, {} files re-hashed, {} redacted\nhead {}",
                    v.transactions, v.datoms, v.files, v.redacted, v.head
                ),
            );
        }
        Cmd::Projectors => {
            let ps: Vec<Json> = ownpurse_ingest::projectors()
                .iter()
                .map(|p| json!({"id": p.id(), "reads": p.reads()}))
                .collect();
            let text: Vec<String> = ownpurse_ingest::projectors()
                .iter()
                .map(|p| format!("{:<20} {}", p.id(), p.reads()))
                .collect();
            out::emit(
                cli.json,
                &Json::Array(ps),
                &format!("projectors[{}]:\n{}", text.len(), text.join("\n")),
            );
        }
    }
    Ok(())
}

fn main() -> ExitCode {
    let cli = Cli::parse();
    match run(&cli) {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            if cli.json {
                println!("{}", json!({"error": e}));
            } else {
                eprintln!("error: {e}");
            }
            ExitCode::from(1)
        }
    }
}
