//! Manifests in the TUI: which manifest this session shows (`ui.manifest`, in the canonical state), the store
//! it reads (`db/manifests.sqlite`, the core's append-only store), the automatic CHECK, the stage, and the
//! writes keys, the palette and agents make (create, switch, add an entry, undo/restore, note, acknowledge,
//! plan). Every write goes through `core::manifest::ops`, so the CLI, the TUI and the bridge apply the same
//! guards. Nothing here reaches Xero: planning is local, and there is NO apply here (apply is CLI-only).

use crate::app::App;
use ownpurse_core::manifest::check::{self, Options, Report};
use ownpurse_core::manifest::ops::{self, Guard, Target};
use ownpurse_core::manifest::stage::{self, Status};
use ownpurse_core::manifest::{Manifest, ManifestError, Refs, Store, compile, effects};
use ownpurse_core::view::Ctx;
use std::sync::Arc;

/// The manifest's derived facts for one (manifest head, record version, as-of): effects, check, stage.
#[derive(Clone, Debug)]
pub struct Facts {
    pub diff: Result<effects::Diff, String>,
    pub report: Report,
    pub status: Status,
}

/// The facts cache key: (manifest, head, as-of, record version).
type FactsKey = (String, i64, String, i64);

/// The manifest side of the app (not in the state; derived from `ui.manifest` and the store).
#[derive(Default)]
pub struct ManifestState {
    /// The store, opened on first use (None without a config file).
    pub store: Option<Store>,
    /// `ui.manifest`, materialised (None on main, or when the manifest is gone).
    pub current: Option<Manifest>,
    /// The head revision the books were last attached at (re-attached when it moves).
    pub attached: Option<(String, i64)>,
    /// The check, effects and stage, cached per (manifest, head, as-of, record version).
    pub facts: Option<(FactsKey, Arc<Facts>)>,
}

impl App {
    /// The orgs and the accountant gate a view needs.
    pub fn manifest_ctx(&self) -> Ctx {
        Ctx {
            orgs: self.source.orgs.clone(),
            gate: self.source.accountant_gate.clone(),
        }
    }

    /// The store, opened lazily (never created by a read).
    pub fn manifest_store(&mut self) -> Option<&mut Store> {
        if self.manifests.store.is_none() {
            let p = self.source.manifests.as_ref()?;
            self.manifests.store = Store::open(&p.store).ok();
        }
        self.manifests.store.as_mut()
    }

    /// The manifest this session shows (None on main).
    pub fn manifest(&self) -> Option<&Manifest> {
        self.manifests.current.as_ref()
    }

    /// The store's head revision for the shown manifest (0 on main).
    pub fn manifest_head(&self) -> i64 {
        self.manifests.current.as_ref().map_or(0, |b| b.head_rev)
    }

    /// Re-read the store and re-attach the books when the shown manifest moved (another writer, a key, an
    /// agent). True when anything changed.
    pub fn manifest_sync(&mut self) -> bool {
        if let Some(s) = self.manifest_store() {
            let _ = s.refresh();
        }
        let name = self.ui.manifest.clone();
        let current = match (&name, self.manifests.store.as_ref()) {
            (Some(n), Some(s)) => s.manifest(n).filter(|m| !m.discarded),
            _ => None,
        };
        // a manifest that was discarded (or never existed here) shows main
        if name.is_some() && current.is_none() {
            self.ui.manifest = None;
        }
        let key = current.as_ref().map(|b| (b.name.clone(), b.head_rev));
        self.manifests.current = current;
        if key == self.manifests.attached {
            return false;
        }
        self.attach_manifest();
        true
    }

    /// Open the books' manifest side for the shown manifest (fresh caches), and rebuild the prepared views.
    pub fn attach_manifest(&mut self) {
        let ctx = self.manifest_ctx();
        let current = self.manifests.current.clone();
        if let Some(b) = self.books.as_mut()
            && let Err(e) = b.set_manifest(&ctx, current.as_ref())
        {
            self.ui.note = Some(format!("manifest view failed: {e}"));
        }
        self.manifests.attached = current.as_ref().map(|b| (b.name.clone(), b.head_rev));
        self.overview = None;
        self.ledger_view = Default::default();
        self.ic_view = Default::default();
        self.cmp_view = Default::default();
        self.dirty = true;
    }

    /// Show a manifest (or main). The CLI's default (`manifests.current`) is untouched unless `make_default`.
    pub fn switch_manifest(
        &mut self,
        name: Option<&str>,
        make_default: bool,
    ) -> Result<(), ManifestError> {
        if let Some(n) = name {
            let s = self.manifest_store().ok_or_else(|| {
                ManifestError::new(
                    "usage",
                    "Manifests need a config file (db/manifests.sqlite beside it).",
                )
            })?;
            s.refresh()?;
            s.open_manifest(n)?;
        }
        if make_default && let Some(p) = &self.source.manifests {
            p.set_current(name)?;
        }
        self.ui.manifest = name.map(str::to_string);
        self.manifest_sync();
        self.prepare();
        Ok(())
    }

    /// The actor of this session's writes: `human`, or `agent:<client>` while an agent request runs.
    pub fn manifest_actor(&self, agent: bool) -> String {
        if agent {
            format!(
                "agent:{}",
                self.agent_client.clone().unwrap_or_else(|| "agent".into())
            )
        } else {
            "human".into()
        }
    }

    fn store_mut(&mut self) -> Result<&mut Store, ManifestError> {
        self.manifest_store();
        self.manifests
            .store
            .as_mut()
            .ok_or_else(|| ManifestError::new("usage", "Manifests need a config file."))
    }

    fn shown(&self) -> Result<String, ManifestError> {
        self.ui.manifest.clone().ok_or_else(|| {
            ManifestError::new("usage", "You are on main: changes go into a manifest.")
        })
    }

    fn after_write(&mut self) {
        self.manifest_sync();
        self.prepare();
    }

    /// Create a manifest and show it.
    pub fn manifest_create(
        &mut self,
        name: &str,
        why: &str,
        actor: &str,
    ) -> Result<i64, ManifestError> {
        let ctx = self.manifest_ctx();
        self.store_mut()?;
        let books = self
            .books
            .as_ref()
            .ok_or_else(|| ManifestError::new("record", "no record"))?;
        let store = self.manifests.store.as_mut().expect("opened above");
        let id = ops::create(store, books.record(), &ctx, actor, name, why)?;
        self.switch_manifest(Some(name), false)?;
        Ok(id)
    }

    /// Add (or combine onto its document's entry) the entry that makes `after`.
    pub fn manifest_put(
        &mut self,
        t: &Target,
        after: &serde_json::Value,
        why: &str,
        replaces: Option<i64>,
        actor: &str,
        guard: &Guard,
    ) -> Result<i64, ManifestError> {
        let name = self.shown()?;
        let ctx = self.manifest_ctx();
        self.store_mut()?;
        let books = self
            .books
            .as_ref()
            .ok_or_else(|| ManifestError::new("record", "no record"))?;
        let store = self.manifests.store.as_mut().expect("opened above");
        let id = ops::add(
            store,
            books.record(),
            &ctx,
            &name,
            actor,
            t,
            after,
            why,
            &Refs::default(),
            replaces,
            guard,
        )?;
        self.after_write();
        Ok(id)
    }

    /// The document as the shown manifest sees it now (normalised).
    pub fn manifest_doc(&self, t: &Target) -> Result<Option<serde_json::Value>, ManifestError> {
        let ctx = self.manifest_ctx();
        let books = self
            .books
            .as_ref()
            .ok_or_else(|| ManifestError::new("record", "no record"))?;
        ops::current(&books.view(), &ctx, t)
    }

    /// Undo (or restore) an entry on the shown manifest.
    pub fn manifest_undo(
        &mut self,
        seq: i64,
        undone: bool,
        actor: &str,
        guard: &Guard,
    ) -> Result<i64, ManifestError> {
        let name = self.shown()?;
        let id = ops::undo(self.store_mut()?, &name, actor, seq, undone, guard)?;
        self.after_write();
        Ok(id)
    }

    /// Note an entry (accountant reference and/or a note).
    pub fn manifest_annotate(
        &mut self,
        seq: i64,
        accountant: Option<&str>,
        note: Option<&str>,
        actor: &str,
        guard: &Guard,
    ) -> Result<i64, ManifestError> {
        let name = self.shown()?;
        let id = ops::annotate(
            self.store_mut()?,
            &name,
            actor,
            seq,
            accountant,
            note,
            guard,
        )?;
        self.after_write();
        Ok(id)
    }

    /// Acknowledge a warning (or withdraw the acknowledgement).
    #[allow(clippy::too_many_arguments)]
    pub fn manifest_ack(
        &mut self,
        seq: i64,
        code: &str,
        path: &str,
        acked: bool,
        actor: &str,
        guard: &Guard,
    ) -> Result<i64, ManifestError> {
        let name = self.shown()?;
        let id = ops::ack(
            self.store_mut()?,
            &name,
            actor,
            seq,
            code,
            path,
            acked,
            guard,
        )?;
        self.after_write();
        Ok(id)
    }

    /// Discard a manifest (the shown one returns to main).
    pub fn manifest_discard(
        &mut self,
        name: &str,
        actor: &str,
        guard: &Guard,
    ) -> Result<i64, ManifestError> {
        let id = ops::discard(self.store_mut()?, name, actor, guard)?;
        if self.ui.manifest.as_deref() == Some(name) {
            self.ui.manifest = None;
        }
        self.after_write();
        Ok(id)
    }

    /// PLAN, locally: compile the checked manifest into the write package, seal it, write it with its review
    /// export under `db/packages/`, and record `manifest.planned`. Refused while the check has errors or
    /// unacknowledged warnings. (The record learns about the package when it is approved or run, from the
    /// CLI; the TUI never links the api crate, and never approves or applies.)
    pub fn manifest_plan(&mut self, actor: &str) -> Result<(String, String), ManifestError> {
        let name = self.shown()?;
        let m = self
            .manifest()
            .cloned()
            .ok_or_else(|| ManifestError::new("usage", "no manifest"))?;
        self.prepare_manifest();
        let facts = self
            .manifest_facts()
            .ok_or_else(|| ManifestError::new("record", "no record"))?;
        let ctx = self.manifest_ctx();
        let cfg = ownpurse_core::write::plan::PlanConfig {
            profile: self.source.profile.clone(),
            orgs: self.source.orgs.clone(),
            gates: self.source.accountant_gate.clone(),
            history_notes: false,
        };
        let paths = self
            .source
            .manifests
            .clone()
            .ok_or_else(|| ManifestError::new("usage", "Manifests need a config file."))?;
        let now = ownpurse_core::classify::store::now_iso();
        let books = self
            .books
            .as_ref()
            .ok_or_else(|| ManifestError::new("record", "no record"))?;
        let pkg = compile::compile(books.record(), &ctx, &cfg, &m, &facts.report, &now)?;
        let sealed = ownpurse_core::write::seal::seal(pkg)
            .map_err(|e| ManifestError::new("plan", e.message))?;
        let hash = sealed.seal.clone().unwrap_or_default();
        std::fs::create_dir_all(&paths.packages).map_err(|e| {
            ManifestError::new("store", format!("{}: {e}", paths.packages.display()))
        })?;
        let path = paths.packages.join(format!("{}.package.json", sealed.id));
        ownpurse_core::write::seal::save(&path, &sealed)
            .map_err(|e| ManifestError::new("store", e.message))?;
        let stem = path.with_file_name(format!("{}-{}", sealed.id, &hash[..12]));
        if let Ok(pdf) = ownpurse_core::write::export::pdf(&sealed) {
            let _ = std::fs::write(stem.with_extension("pdf"), pdf);
        }
        if let Ok(csv) = ownpurse_core::write::export::csv(&sealed) {
            let _ = std::fs::write(stem.with_extension("csv"), csv);
        }
        ops::planned(
            self.store_mut()?,
            &name,
            actor,
            &path.display().to_string(),
            &hash,
            m.content_rev,
        )?;
        self.after_write();
        Ok((path.display().to_string(), hash))
    }

    /// The manifest side of the books (conflicts, applied entries, posted ones), or None on main.
    pub fn manifest_overlay(&self) -> Option<&ownpurse_core::view::Overlay> {
        self.books.as_ref().and_then(|b| b.overlay())
    }

    /// Compute (or reuse) the shown manifest's effects, check and stage.
    pub fn prepare_manifest(&mut self) {
        let Some(m) = self.manifest().cloned() else {
            self.manifests.facts = None;
            return;
        };
        let key = (
            m.name.clone(),
            m.head_rev,
            self.ui.as_of.clone(),
            self.model.record_version,
        );
        if self
            .manifests
            .facts
            .as_ref()
            .is_some_and(|(k, _)| *k == key)
        {
            return;
        }
        let ctx = self.manifest_ctx();
        let ic = serde_json::json!({
            "parties": self.source.ic_config.get("parties").cloned().unwrap_or_default(),
            "intercompany": self.source.ic_config.get("intercompany").cloned().unwrap_or_default(),
        });
        let Some(books) = self.books.as_ref() else {
            return;
        };
        let rec = books.record();
        let diff = effects::diff(
            rec,
            &ctx,
            &m,
            &self.ui.as_of,
            Some(&ic),
            &self.source.zero_accounts,
        )
        .map_err(|e| e.message);
        let opts = Options {
            as_of: self.ui.as_of.clone(),
            ic_config: Some(ic),
            zero_accounts: self.source.zero_accounts.clone(),
            line_limit: self.source.journal_line_limit,
        };
        let report = check::check(rec, &ctx, &m, &opts, diff.as_ref().ok()).unwrap_or_default();
        let status = stage::status(rec, &m, &report, books.overlay());
        self.manifests.facts = Some((
            key,
            Arc::new(Facts {
                diff,
                report,
                status,
            }),
        ));
    }

    /// The shown manifest's facts (prepared in `update`).
    pub fn manifest_facts(&self) -> Option<Arc<Facts>> {
        self.manifests.facts.as_ref().map(|(_, f)| f.clone())
    }

    /// The header chip: `(name, the stage's short form, error)` (the core's `Stage::short`, so the chip and
    /// the stage strip always agree).
    pub fn manifest_chip(&self) -> Option<(String, String, bool)> {
        let m = self.manifest()?;
        let n = m.active().count();
        let (s, e) = match self.manifest_facts() {
            Some(f) => f
                .status
                .stage
                .short(n, f.report.errors() + f.report.unacked()),
            None => (crate::layout::plural_changes(n), false),
        };
        Some((m.name.clone(), s, e))
    }

    /// The commands the planned / approved panel shows (the core's one builder, as the CLI prints them).
    pub fn manifest_commands(&self) -> Option<compile::Commands> {
        let m = self.manifest()?;
        let p = m.plan()?;
        Some(compile::commands(&m.name, &p.hash))
    }
}
