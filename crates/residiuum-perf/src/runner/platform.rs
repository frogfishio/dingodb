//! Platform adapters: macOS, Linux, portable fallback (plan §5).

use super::RunnerError;
use std::path::{Path, PathBuf};
use std::process::Command;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlatformAdapter {
    MacOs,
    Linux,
    Portable,
}

impl PlatformAdapter {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::MacOs => "macos",
            Self::Linux => "linux",
            Self::Portable => "portable",
        }
    }
}

pub fn detect_adapter() -> PlatformAdapter {
    match std::env::consts::OS {
        "macos" => PlatformAdapter::MacOs,
        "linux" => PlatformAdapter::Linux,
        _ => PlatformAdapter::Portable,
    }
}

/// True for paths that look like raw devices / character or block nodes under /dev.
pub fn is_block_device_path(path: &Path) -> bool {
    let s = path.to_string_lossy();
    if s.starts_with("/dev/") {
        return true;
    }
    // Windows-style device paths (portable rejection).
    if s.starts_with(r"\\.\") || s.starts_with("//./") {
        return true;
    }
    false
}

/// Free bytes on the filesystem containing `path` (via `df -Pk`).
pub fn free_space_bytes(path: &Path) -> Result<u64, RunnerError> {
    let out = Command::new("df")
        .args(["-Pk"])
        .arg(path)
        .output()
        .map_err(|e| RunnerError::Preflight(format!("df -Pk failed: {e}")))?;
    if !out.status.success() {
        return Err(RunnerError::Preflight(format!(
            "df -Pk exit {}: {}",
            out.status,
            String::from_utf8_lossy(&out.stderr)
        )));
    }
    parse_df_available_kib(&String::from_utf8_lossy(&out.stdout))
        .map(|kib| kib.saturating_mul(1024))
}

/// Free inodes when `df -Pi` is available; `None` if unavailable.
pub fn free_space_inodes(path: &Path) -> Result<Option<u64>, RunnerError> {
    let out = Command::new("df").args(["-Pi"]).arg(path).output();
    let out = match out {
        Ok(o) if o.status.success() => o,
        _ => return Ok(None),
    };
    Ok(parse_df_ifree(&String::from_utf8_lossy(&out.stdout)))
}

/// Mount points from `df -Pk` (for mount-root rejection).
pub fn mount_points() -> Result<Vec<PathBuf>, RunnerError> {
    let out = Command::new("df")
        .args(["-Pk"])
        .output()
        .map_err(|e| RunnerError::Preflight(format!("df -Pk failed: {e}")))?;
    if !out.status.success() {
        return Err(RunnerError::Preflight("df -Pk failed".into()));
    }
    Ok(parse_df_mounts(&String::from_utf8_lossy(&out.stdout)))
}

fn parse_df_available_kib(stdout: &str) -> Result<u64, RunnerError> {
    // Filesystem 1024-blocks Used Available Capacity Mounted on
    let mut lines = stdout.lines().filter(|l| !l.trim().is_empty());
    let _header = lines.next();
    let data = lines
        .next()
        .ok_or_else(|| RunnerError::Preflight("df produced no data line".into()))?;
    let cols: Vec<&str> = data.split_whitespace().collect();
    // Available is column index 3 (0-based) on POSIX df -Pk.
    let avail = cols
        .get(3)
        .ok_or_else(|| RunnerError::Preflight(format!("df line missing Available: {data}")))?;
    avail
        .parse::<u64>()
        .map_err(|e| RunnerError::Preflight(format!("df Available parse: {e}")))
}

fn parse_df_ifree(stdout: &str) -> Option<u64> {
    let mut lines = stdout.lines().filter(|l| !l.trim().is_empty());
    let _header = lines.next()?;
    let data = lines.next()?;
    let cols: Vec<&str> = data.split_whitespace().collect();
    // macOS df -Pi: Filesystem 512-blocks Used Available Capacity iused ifree %iused Mounted on
    // Linux df -Pi: Filesystem Inodes IUsed IFree IUse% Mounted on
    // Prefer a column named ifree/IFree by header when possible.
    let header = stdout.lines().next().unwrap_or("");
    let hcols: Vec<&str> = header.split_whitespace().collect();
    if let Some(idx) = hcols.iter().position(|h| {
        let l = h.to_ascii_lowercase();
        l == "ifree" || l == "ifree%" // not percent
    }) {
        // exact ifree
        if let Some(v) = cols.get(idx).and_then(|s| s.parse().ok()) {
            return Some(v);
        }
    }
    if let Some(idx) = hcols
        .iter()
        .position(|h| h.eq_ignore_ascii_case("ifree") || h.eq_ignore_ascii_case("IFree"))
    {
        return cols.get(idx).and_then(|s| s.parse().ok());
    }
    // Linux: IFree often index 3
    if hcols.iter().any(|h| h.eq_ignore_ascii_case("IFree")) {
        if let Some(idx) = hcols.iter().position(|h| h.eq_ignore_ascii_case("IFree")) {
            return cols.get(idx).and_then(|s| s.parse().ok());
        }
    }
    // Fallback: macOS style index 6 (ifree)
    cols.get(6).and_then(|s| s.parse().ok())
}

fn parse_df_mounts(stdout: &str) -> Vec<PathBuf> {
    let mut out = Vec::new();
    for (i, line) in stdout.lines().enumerate() {
        if i == 0 || line.trim().is_empty() {
            continue;
        }
        let cols: Vec<&str> = line.split_whitespace().collect();
        if let Some(mnt) = cols.last() {
            if mnt.starts_with('/') {
                out.push(PathBuf::from(mnt));
            }
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn df_available_parses() {
        let sample = "\
Filesystem     1024-blocks      Used Available Capacity Mounted on
/dev/disk3s1s1   239362496  10989408  22121440    34%    /
";
        assert_eq!(parse_df_available_kib(sample).unwrap(), 22121440);
    }

    #[test]
    fn block_device_paths() {
        assert!(is_block_device_path(Path::new("/dev/disk0")));
        assert!(is_block_device_path(Path::new("/dev/null")));
        assert!(!is_block_device_path(Path::new("/var/tmp/work")));
    }

    #[test]
    fn free_space_on_tmp_is_positive() {
        let tmp = tempfile::tempdir().unwrap();
        let free = free_space_bytes(tmp.path()).unwrap();
        assert!(free > 0);
    }

    #[test]
    fn adapter_detects_host_os() {
        let a = detect_adapter();
        match std::env::consts::OS {
            "macos" => assert_eq!(a, PlatformAdapter::MacOs),
            "linux" => assert_eq!(a, PlatformAdapter::Linux),
            _ => assert_eq!(a, PlatformAdapter::Portable),
        }
    }
}
