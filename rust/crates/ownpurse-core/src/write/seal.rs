//! The package seal: sha256 over the canonical JSON (`canon::canonical`: sorted keys, compact, exact number
//! digits) of the package without its `seal` field. Any change to any operation, precondition, expected
//! effect, recovery, exclusion or checklist line is a new hash, and needs a new approval.

use super::{PACKAGE_V, Package, WriteError, exit, sha256_hex};
use crate::canon::canonical;
use std::path::Path;

/// The canonical text that is hashed.
pub fn canonical_text(p: &Package) -> Result<String, WriteError> {
    let mut v = serde_json::to_value(p)
        .map_err(|e| WriteError::new("usage", exit::USAGE, e.to_string()))?;
    if let Some(o) = v.as_object_mut() {
        o.remove("seal");
    }
    Ok(canonical(&v))
}

pub fn hash(p: &Package) -> Result<String, WriteError> {
    Ok(sha256_hex(canonical_text(p)?.as_bytes()))
}

/// Seal (or re-seal) a package: the hash of its content now.
pub fn seal(mut p: Package) -> Result<Package, WriteError> {
    if p.v != PACKAGE_V {
        return Err(WriteError::new(
            "usage",
            exit::USAGE,
            format!("package format v{} (this build reads v{PACKAGE_V})", p.v),
        ));
    }
    if p.ops.is_empty() {
        return Err(WriteError::new(
            "usage",
            exit::USAGE,
            "the package has no operations",
        ));
    }
    p.seal = Some(hash(&p)?);
    Ok(p)
}

/// The sealed hash, recomputed and compared: refuses an unsealed or edited package.
pub fn verify(p: &Package) -> Result<String, WriteError> {
    let Some(sealed) = &p.seal else {
        return Err(WriteError::new(
            "not_sealed",
            exit::NOT_APPROVED,
            "the package is not sealed",
        )
        .help("run `ownpurse write seal <package>` and review what it prints"));
    };
    let now = hash(p)?;
    if &now != sealed {
        return Err(WriteError::new(
            "hash_mismatch",
            exit::NOT_APPROVED,
            format!(
                "the package changed after sealing: sealed {} but its content hashes to {}",
                &sealed[..12.min(sealed.len())],
                &now[..12]
            ),
        )
        .help("a changed package is a new version: seal it again, and it needs a new approval"));
    }
    Ok(now)
}

pub fn load(path: &Path) -> Result<Package, WriteError> {
    let text = std::fs::read_to_string(path)
        .map_err(|e| WriteError::new("io", exit::RUNTIME, format!("{}: {e}", path.display())))?;
    serde_json::from_str(&text).map_err(|e| {
        WriteError::new(
            "usage",
            exit::USAGE,
            format!("{}: not a package ({e})", path.display()),
        )
    })
}

/// Write a package as pretty JSON (the file's bytes are not what is hashed; the canonical form is).
pub fn save(path: &Path, p: &Package) -> Result<(), WriteError> {
    let text = serde_json::to_string_pretty(p)
        .map_err(|e| WriteError::new("usage", exit::USAGE, e.to_string()))?;
    std::fs::write(path, text + "\n")
        .map_err(|e| WriteError::new("io", exit::RUNTIME, format!("{}: {e}", path.display())))
}
