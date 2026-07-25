//! Exclusive store writer ownership (DEF-020).
//!
//! Uses an OS advisory exclusive lock on `store-info/writer.lock` plus an
//! in-process path registry so two writer handles cannot open the same store
//! from one process either. Lock file text is diagnostic only and is never
//! trusted in place of the OS lock.

use crate::error::StoreError;
use crate::layout::StorePaths;
use std::collections::HashSet;
use std::fs::{File, OpenOptions};
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::sync::{Mutex, OnceLock};
use std::time::{SystemTime, UNIX_EPOCH};

/// Diagnostic-only lock file name under `store-info/`.
pub const WRITER_LOCK_FILE: &str = "writer.lock";

fn held_paths() -> &'static Mutex<HashSet<PathBuf>> {
    static HELD: OnceLock<Mutex<HashSet<PathBuf>>> = OnceLock::new();
    HELD.get_or_init(|| Mutex::new(HashSet::new()))
}

/// Held exclusive writer ownership for one store root.
#[derive(Debug)]
pub struct WriterLock {
    /// Canonical store root used for the in-process registry.
    root: PathBuf,
    /// Open lock file whose OS exclusive lock is held for the handle lifetime.
    file: File,
    path: PathBuf,
}

impl WriterLock {
    /// Acquire exclusive writer ownership for `paths` before any segment open.
    pub fn acquire(paths: &StorePaths) -> Result<Self, StoreError> {
        let root = canonicalize_root(&paths.root)?;
        {
            let mut held = held_paths()
                .lock()
                .unwrap_or_else(|e| e.into_inner());
            if held.contains(&root) {
                return Err(StoreError::WriterLockHeld(format!(
                    "another writer handle is already open for {} (in-process)",
                    root.display()
                )));
            }
            held.insert(root.clone());
        }

        let lock_path = paths.store_info().join(WRITER_LOCK_FILE);
        if let Some(parent) = lock_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let acquired = (|| -> Result<(File, PathBuf), StoreError> {
            let mut file = OpenOptions::new()
                .read(true)
                .write(true)
                .create(true)
                .truncate(false)
                .open(&lock_path)?;
            try_lock_exclusive(&file).map_err(|e| {
                // flock sets EAGAIN/EWOULDBLOCK when another holder exists.
                let busy = e.kind() == io::ErrorKind::WouldBlock
                    || e.raw_os_error() == Some(11) // Linux EAGAIN
                    || e.raw_os_error() == Some(35); // macOS EAGAIN
                if busy {
                    StoreError::WriterLockHeld(format!(
                        "another process holds the exclusive lock on {}",
                        lock_path.display()
                    ))
                } else {
                    StoreError::Io(e)
                }
            })?;
            write_diagnostic(&mut file)?;
            Ok((file, lock_path.clone()))
        })();

        match acquired {
            Ok((file, path)) => Ok(Self { root, file, path }),
            Err(e) => {
                // Roll back in-process reservation on failure.
                let mut held = held_paths()
                    .lock()
                    .unwrap_or_else(|err| err.into_inner());
                held.remove(&root);
                Err(e)
            }
        }
    }

    /// Path of the diagnostic lock file.
    pub fn path(&self) -> &Path {
        &self.path
    }
}

impl Drop for WriterLock {
    fn drop(&mut self) {
        let _ = unlock_exclusive(&self.file);
        let mut held = held_paths()
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        held.remove(&self.root);
    }
}

fn canonicalize_root(root: &Path) -> Result<PathBuf, StoreError> {
    // Prefer absolute/canonical paths so relative open paths still collide.
    match std::fs::canonicalize(root) {
        Ok(p) => Ok(p),
        Err(_) => {
            // Store may not exist yet (create path). Use absolute of parent + name.
            let abs = if root.is_absolute() {
                root.to_path_buf()
            } else {
                std::env::current_dir()
                    .map(|cwd| cwd.join(root))
                    .unwrap_or_else(|_| root.to_path_buf())
            };
            Ok(abs)
        }
    }
}

fn write_diagnostic(file: &mut File) -> Result<(), StoreError> {
    let pid = std::process::id();
    let ns = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    // Truncate and rewrite diagnostics; OS lock remains authoritative.
    file.set_len(0)?;
    writeln!(
        file,
        "dingo-writer-lock\npid={pid}\nacquired_ns={ns}\nnote=diagnostic-only; OS exclusive lock is authoritative\n"
    )?;
    let _ = file.sync_all();
    Ok(())
}

#[cfg(unix)]
fn try_lock_exclusive(file: &File) -> io::Result<()> {
    use std::os::unix::io::AsRawFd;
    // flock(2): LOCK_EX | LOCK_NB
    const LOCK_EX: i32 = 2;
    const LOCK_NB: i32 = 4;
    let rc = unsafe { flock(file.as_raw_fd(), LOCK_EX | LOCK_NB) };
    if rc == 0 {
        Ok(())
    } else {
        Err(io::Error::last_os_error())
    }
}

#[cfg(unix)]
fn unlock_exclusive(file: &File) -> io::Result<()> {
    use std::os::unix::io::AsRawFd;
    const LOCK_UN: i32 = 8;
    let rc = unsafe { flock(file.as_raw_fd(), LOCK_UN) };
    if rc == 0 {
        Ok(())
    } else {
        Err(io::Error::last_os_error())
    }
}

#[cfg(unix)]
extern "C" {
    fn flock(fd: i32, operation: i32) -> i32;
}

#[cfg(not(unix))]
fn try_lock_exclusive(file: &File) -> io::Result<()> {
    // Best-effort: keep the file open; in-process registry still applies.
    // Windows exclusive share modes can be added when a Windows CI target lands.
    let _ = file;
    Ok(())
}

#[cfg(not(unix))]
fn unlock_exclusive(_file: &File) -> io::Result<()> {
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn second_in_process_writer_fails() {
        let dir = tempdir().unwrap();
        let root = dir.path().join("s");
        std::fs::create_dir_all(root.join("store-info")).unwrap();
        let paths = StorePaths::new(&root);
        let _a = WriterLock::acquire(&paths).unwrap();
        let err = WriterLock::acquire(&paths).unwrap_err();
        assert!(matches!(err, StoreError::WriterLockHeld(_)), "{err:?}");
    }

    #[test]
    fn drop_releases_for_reopen() {
        let dir = tempdir().unwrap();
        let root = dir.path().join("s");
        std::fs::create_dir_all(root.join("store-info")).unwrap();
        let paths = StorePaths::new(&root);
        {
            let _a = WriterLock::acquire(&paths).unwrap();
        }
        let _b = WriterLock::acquire(&paths).unwrap();
    }
}
