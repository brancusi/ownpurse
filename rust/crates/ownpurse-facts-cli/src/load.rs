//! The ingest shell: read files, put them in the drawer, plan with a projector, transact.

use ownpurse_facts::{Instant, TxReport};
use ownpurse_ingest::{FileIn, Input, Pressure, SourceSpec};
use ownpurse_store::Store;
use serde_json::Value as Json;
use std::path::{Path, PathBuf};

/// One file to ingest, as the CLI flags or a manifest line describe it.
#[derive(Debug, Clone)]
pub struct Job {
    pub file: PathBuf,
    pub projector: String,
    pub source: SourceSpec,
    /// (role, path): a mapping, the file an extraction reads
    pub with: Vec<(String, PathBuf)>,
    pub observed_at: Instant,
}

#[derive(Debug, Clone)]
pub struct Loaded {
    pub file: String,
    pub projector: String,
    pub installed: usize,
    pub tx: Option<u64>,
    pub asserted: usize,
    pub retracted: usize,
    pub pressure: Vec<Pressure>,
}

pub fn media_type(p: &Path) -> &'static str {
    match p
        .extension()
        .and_then(|e| e.to_str())
        .map(str::to_ascii_lowercase)
        .as_deref()
    {
        Some("json") => "application/json",
        Some("csv") => "text/csv",
        Some("ofx" | "qfx") => "application/x-ofx",
        Some("pdf") => "application/pdf",
        Some("png") => "image/png",
        Some("jpg" | "jpeg") => "image/jpeg",
        Some("xlsx") => "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet",
        _ => "application/octet-stream",
    }
}

fn counts(r: &TxReport) -> (usize, usize) {
    let a = r.datoms.iter().filter(|d| d.added).count();
    (a, r.datoms.len() - a)
}

pub fn run(store: &mut Store, job: &Job, now: &Instant) -> Result<Loaded, String> {
    let projector = ownpurse_ingest::projector(&job.projector)
        .ok_or_else(|| format!("no projector {:?}; `projectors` lists them", job.projector))?;
    let read = |p: &Path| std::fs::read(p).map_err(|e| format!("{}: {e}", p.display()));
    let main_bytes = read(&job.file)?;
    let mut with_bytes = Vec::new();
    for (role, p) in &job.with {
        with_bytes.push((role.clone(), p.clone(), read(p)?));
    }
    let put = |b: &[u8]| {
        store
            .drawer()
            .put(b)
            .map(|(h, _)| h)
            .map_err(|e| format!("drawer: {e}"))
    };
    let main_hash = put(&main_bytes)?;
    let mut with_hashes = Vec::new();
    for (_, _, b) in &with_bytes {
        with_hashes.push(put(b)?);
    }
    let name = |p: &Path| {
        p.file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("")
            .to_string()
    };
    let main_name = name(&job.file);
    let with_names: Vec<String> = with_bytes.iter().map(|(_, p, _)| name(p)).collect();
    let input = Input {
        file: FileIn {
            hash: main_hash,
            name: &main_name,
            media_type: media_type(&job.file),
            bytes: &main_bytes,
            role: "main",
        },
        with: with_bytes
            .iter()
            .zip(&with_hashes)
            .zip(&with_names)
            .map(|(((role, p, b), h), n)| FileIn {
                hash: *h,
                name: n,
                media_type: media_type(p),
                bytes: b,
                role,
            })
            .collect(),
        source: job.source.clone(),
        observed_at: job.observed_at.clone(),
    };
    let plan = ownpurse_ingest::plan(store.db(), projector, &input).map_err(|e| e.0)?;
    let installed = if plan.install.is_empty() {
        0
    } else {
        let r = store
            .transact(&plan.install, now)
            .map_err(|e| e.to_string())?;
        r.datoms
            .iter()
            .filter(|d| d.added && d.a == ownpurse_facts::schema::DB_IDENT)
            .count()
    };
    // the plan was made against the database before the install; re-plan so lookups see new attributes
    let plan = if installed > 0 {
        ownpurse_ingest::plan(store.db(), projector, &input).map_err(|e| e.0)?
    } else {
        plan
    };
    let r = store
        .transact(&plan.tx, now)
        .map_err(|e| format!("{}: {e}", job.file.display()))?;
    let (asserted, retracted) = counts(&r);
    Ok(Loaded {
        file: job.file.display().to_string(),
        projector: projector.id().to_string(),
        installed,
        tx: r.tx.map(|t| t.0),
        asserted,
        retracted,
        pressure: plan.pressure,
    })
}

/// A manifest: `{"files": [{"file", "projector", "source": {"id","kind","name"}, "with": {role: path},
/// "observed_at"}]}`, paths relative to the manifest.
pub fn manifest(path: &Path) -> Result<Vec<Job>, String> {
    let text = std::fs::read_to_string(path).map_err(|e| format!("{}: {e}", path.display()))?;
    let j: Json = serde_json::from_str(&text).map_err(|e| format!("{}: {e}", path.display()))?;
    let base = path.parent().unwrap_or(Path::new("."));
    let sources = j.get("sources").and_then(Json::as_object);
    let mut out = Vec::new();
    for (i, f) in j
        .get("files")
        .and_then(Json::as_array)
        .ok_or("a manifest has \"files\": [...]")?
        .iter()
        .enumerate()
    {
        let at = |m: &str| format!("files[{i}]: {m}");
        let s = |k: &str| f.get(k).and_then(Json::as_str);
        let src_id = s("source").ok_or_else(|| at("needs \"source\" (an id from \"sources\")"))?;
        let src = sources
            .and_then(|m| m.get(src_id))
            .ok_or_else(|| at(&format!("source {src_id:?} is not in \"sources\"")))?;
        let source = SourceSpec {
            id: src_id.to_string(),
            kind: src
                .get("kind")
                .and_then(Json::as_str)
                .unwrap_or("")
                .to_string(),
            name: src
                .get("name")
                .and_then(Json::as_str)
                .unwrap_or(src_id)
                .to_string(),
        };
        let observed_at =
            Instant::parse(s("observed_at").ok_or_else(|| at("needs \"observed_at\""))?)
                .map_err(|e| at(&e.0))?;
        let with = f
            .get("with")
            .and_then(Json::as_object)
            .map(|m| {
                m.iter()
                    .filter_map(|(r, p)| p.as_str().map(|p| (r.clone(), base.join(p))))
                    .collect()
            })
            .unwrap_or_default();
        out.push(Job {
            file: base.join(s("file").ok_or_else(|| at("needs \"file\""))?),
            projector: s("projector")
                .ok_or_else(|| at("needs \"projector\""))?
                .to_string(),
            source,
            with,
            observed_at,
        });
    }
    Ok(out)
}
