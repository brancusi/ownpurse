//! Changes other processes make to the data this TUI shows: a CLI `sync` (or an agent) appending to the
//! record, a CLI `classify suggest`/`set --direct` appending to the classification store. The OS file events
//! on their folders (no timer, no polling; the minute tick re-checks in case an event was dropped) wake the
//! loop; the loop compares the revisions on disk with the ones it shows and, only when they moved, refreshes
//! through the normal reload path with the view kept and says what arrived (`12 new suggestions`,
//! `record updated · 3 new versions`). An event that changes nothing (the TUI's own write, a checkpoint)
//! draws nothing (V13).

use std::path::{Path, PathBuf};

/// The revisions on disk: the record's last chain seq and the store's last event id.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct DiskRevs {
    pub record: Option<i64>,
    pub store: Option<i64>,
    /// The manifest store's last event id (None: no store).
    pub manifests: Option<i64>,
}

/// A read-only handle on either database (the core's read-only opener; never creates a file).
fn open_ro(p: &Path) -> Option<ownpurse_core::Record> {
    ownpurse_core::Record::open(p).ok()
}

/// Read both revisions (two indexed MAX queries; read-only, nothing written).
pub fn read(record: &Path, store: Option<&Path>) -> DiskRevs {
    read_all(record, store, None)
}

/// Read the record's, the classification store's and the manifest store's revisions.
pub fn read_all(record: &Path, store: Option<&Path>, manifests: Option<&Path>) -> DiskRevs {
    let max = |p: &Path, sql: &str| -> Option<i64> {
        open_ro(p)?
            .conn()
            .query_row(sql, [], |r| r.get::<_, Option<i64>>(0))
            .ok()
            .flatten()
            .or(Some(0))
    };
    DiskRevs {
        record: max(record, "SELECT MAX(seq) FROM chain"),
        store: store.and_then(|s| max(s, "SELECT MAX(id) FROM events")),
        manifests: manifests
            .filter(|m| m.exists())
            .and_then(|m| max(m, "SELECT MAX(id) FROM events")),
    }
}

/// What arrived, in words: `12 new suggestions`, `labels updated · 5 events`, `record updated · 3 new versions`.
pub fn describe(record: &Path, store: Option<&Path>, from: DiskRevs, to: DiskRevs) -> String {
    let mut parts = Vec::new();
    if let (Some(a), Some(b)) = (from.record, to.record)
        && b != a
    {
        let n = open_ro(record)
            .and_then(|c| {
                c.conn().query_row(
                    "SELECT COUNT(*) FROM chain WHERE seq > ?1 AND seq <= ?2 AND tbl = 'versions'",
                    [a, b],
                    |r| r.get::<_, i64>(0),
                )
                .ok()
            })
            .unwrap_or(0);
        // write attempts that landed in that range (Phase 8): `write: 3 ops landed · XA:<package>`
        let landed: Vec<String> = open_ro(record)
            .and_then(|c| {
                let mut st = c
                    .conn()
                    .prepare(
                        "SELECT e.op_id FROM chain c JOIN op_events e ON c.tbl = 'op_events' AND e.id = CAST(c.row_id AS INTEGER) \
                         WHERE c.seq > ?1 AND c.seq <= ?2 AND e.state = 'confirmed'",
                    )
                    .ok()?;
                let rows: Vec<String> = st
                    .query_map([a, b], |r| r.get::<_, String>(0))
                    .ok()?
                    .flatten()
                    .collect();
                Some(rows)
            })
            .unwrap_or_default();
        if !landed.is_empty() {
            let pkgs = open_ro(record)
                .map(|r| ownpurse_core::write::view::packages(&r))
                .unwrap_or_default();
            let mut ids: Vec<String> = landed
                .iter()
                .filter_map(|k| k.split(':').next())
                .filter_map(|h| {
                    pkgs.iter()
                        .find(|p| p.hash.starts_with(h))
                        .map(|p| format!("XA:{}", p.id))
                })
                .collect();
            ids.sort();
            ids.dedup();
            let n = landed.len();
            parts.push(format!(
                "write: {n} op{} landed{}",
                if n == 1 { "" } else { "s" },
                if ids.is_empty() {
                    String::new()
                } else {
                    format!(" · {}", ids.join(", "))
                }
            ));
        }
        parts.push(match n {
            0 => "record updated".to_string(),
            1 => "record updated · 1 new version".to_string(),
            n => format!("record updated · {n} new versions"),
        });
    }
    if let (Some(s), Some(a), Some(b)) = (store, from.store, to.store)
        && b != a
    {
        let (sugg, all) = open_ro(s)
            .and_then(|c| {
                c.conn()
                    .query_row(
                        "SELECT COUNT(*) FILTER (WHERE kind = 'suggestion'), COUNT(*) FROM events \
                     WHERE id > ?1 AND id <= ?2",
                        [a, b],
                        |r| Ok((r.get::<_, i64>(0)?, r.get::<_, i64>(1)?)),
                    )
                    .ok()
            })
            .unwrap_or((0, b - a));
        parts.push(if sugg == all && all > 0 {
            match all {
                1 => "1 new suggestion".to_string(),
                n => format!("{n} new suggestions"),
            }
        } else {
            match all {
                1 => "labels updated · 1 event".to_string(),
                n => format!("labels updated · {n} events"),
            }
        });
    }
    parts.join(" · ")
}

/// A note about other processes' changes (it shows whole; key hints drop first).
pub fn is_note(n: &str) -> bool {
    n.starts_with("record updated")
        || n.starts_with("write: ")
        || n.starts_with("config reloaded")
        || n.contains("labels updated")
        || n.contains("new suggestion")
}

/// The files whose writes matter: each database and its WAL (never `-shm`, which readers touch too).
fn names_for(p: &Path) -> Vec<std::ffi::OsString> {
    let Some(n) = p.file_name() else {
        return Vec::new();
    };
    let mut wal = n.to_os_string();
    wal.push("-wal");
    let mut journal = n.to_os_string();
    journal.push("-journal");
    vec![n.to_os_string(), wal, journal]
}

/// Watch the folders of `files` (non-recursive; one watcher) and call `changed` on a write to one of them or
/// its WAL. Keep the returned watcher alive for as long as events are wanted.
pub fn watch(
    files: &[PathBuf],
    changed: impl Fn() + Send + 'static,
) -> Option<notify::RecommendedWatcher> {
    use notify::{EventKind, RecursiveMode, Watcher};
    let names: Vec<std::ffi::OsString> = files.iter().flat_map(|f| names_for(f)).collect();
    let mut dirs: Vec<PathBuf> = files
        .iter()
        .filter_map(|f| f.parent().map(Path::to_path_buf))
        .collect();
    dirs.sort();
    dirs.dedup();
    let mut w = notify::recommended_watcher(move |res: notify::Result<notify::Event>| {
        let Ok(ev) = res else { return };
        if matches!(ev.kind, EventKind::Access(_)) {
            return;
        }
        if ev
            .paths
            .iter()
            .any(|p| p.file_name().is_some_and(|n| names.iter().any(|m| m == n)))
        {
            changed();
        }
    })
    .ok()?;
    let mut any = false;
    for d in &dirs {
        any |= w.watch(d, RecursiveMode::NonRecursive).is_ok();
    }
    any.then_some(w)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_names_are_the_database_and_its_wal_never_shm() {
        let n = names_for(Path::new("/x/db/classify.sqlite"));
        let s: Vec<String> = n.iter().map(|x| x.to_string_lossy().into_owned()).collect();
        assert_eq!(
            s,
            [
                "classify.sqlite",
                "classify.sqlite-wal",
                "classify.sqlite-journal"
            ]
        );
    }

    #[test]
    fn missing_files_read_as_unknown() {
        let r = read(Path::new("/nonexistent/record.sqlite"), None);
        assert_eq!(r, DiskRevs::default());
    }
}
