//! The drawer: evidence files, stored byte for byte under their SHA-256. A file is written once, to a
//! temporary name and then renamed, so a reader never sees half a file; storing the same bytes again is a
//! no-op. Bytes leave the drawer only through `redact`.

use ownpurse_facts::Hash;
use ownpurse_facts::encode::sha256;
use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct Drawer {
    root: PathBuf,
}

impl Drawer {
    pub fn new(root: impl Into<PathBuf>) -> Drawer {
        Drawer { root: root.into() }
    }

    pub fn root(&self) -> &Path {
        &self.root
    }

    /// `<root>/sha256/ab/cdef…`
    pub fn path(&self, h: &Hash) -> PathBuf {
        let hex = h.hex();
        self.root.join("sha256").join(&hex[..2]).join(&hex[2..])
    }

    /// Store bytes; returns their hash and whether they were new.
    pub fn put(&self, bytes: &[u8]) -> io::Result<(Hash, bool)> {
        let h = sha256(bytes);
        let path = self.path(&h);
        if path.exists() {
            return Ok((h, false));
        }
        let dir = path.parent().unwrap_or(&self.root);
        fs::create_dir_all(dir)?;
        let tmp = dir.join(format!(".{}.tmp", &h.hex()[2..]));
        {
            let mut f = fs::File::create(&tmp)?;
            f.write_all(bytes)?;
            f.sync_all()?;
        }
        fs::rename(&tmp, &path)?;
        Ok((h, true))
    }

    pub fn get(&self, h: &Hash) -> io::Result<Vec<u8>> {
        fs::read(self.path(h))
    }

    pub fn has(&self, h: &Hash) -> bool {
        self.path(h).exists()
    }

    /// Remove a file's bytes. The caller records why, as a fact; the hash stays in the log.
    pub fn remove(&self, h: &Hash) -> io::Result<()> {
        match fs::remove_file(self.path(h)) {
            Err(e) if e.kind() == io::ErrorKind::NotFound => Ok(()),
            other => other,
        }
    }
}
