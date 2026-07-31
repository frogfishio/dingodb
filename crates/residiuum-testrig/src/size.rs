//! On-disk size helpers and human size parsing (`1G`, `256M`, raw bytes).

use std::fs;
use std::io;
use std::path::Path;

/// Recursively sum file lengths under `root` (follows neither symlinks as new roots).
pub fn dir_size_bytes(root: &Path) -> io::Result<u64> {
    let mut total = 0u64;
    if !root.exists() {
        return Ok(0);
    }
    let meta = fs::symlink_metadata(root)?;
    if meta.file_type().is_file() {
        return Ok(meta.len());
    }
    if !meta.file_type().is_dir() {
        return Ok(0);
    }
    let mut stack = vec![root.to_path_buf()];
    while let Some(dir) = stack.pop() {
        let entries = match fs::read_dir(&dir) {
            Ok(e) => e,
            Err(e) if e.kind() == io::ErrorKind::NotFound => continue,
            Err(e) => return Err(e),
        };
        for ent in entries {
            let ent = ent?;
            let ft = ent.file_type()?;
            if ft.is_dir() {
                stack.push(ent.path());
            } else if ft.is_file() {
                total = total.saturating_add(ent.metadata()?.len());
            }
        }
    }
    Ok(total)
}

/// Parse sizes like `1073741824`, `1G`, `1GB`, `256M`, `512k`.
pub fn parse_size(s: &str) -> Result<u64, String> {
    let s = s.trim();
    if s.is_empty() {
        return Err("empty size".into());
    }
    let lower = s.to_ascii_lowercase();
    let (num, mult) = if let Some(rest) = lower.strip_suffix("gb") {
        (rest, 1024u64 * 1024 * 1024)
    } else if let Some(rest) = lower.strip_suffix('g') {
        (rest, 1024u64 * 1024 * 1024)
    } else if let Some(rest) = lower.strip_suffix("mb") {
        (rest, 1024u64 * 1024)
    } else if let Some(rest) = lower.strip_suffix('m') {
        (rest, 1024u64 * 1024)
    } else if let Some(rest) = lower.strip_suffix("kb") {
        (rest, 1024u64)
    } else if let Some(rest) = lower.strip_suffix('k') {
        (rest, 1024u64)
    } else if let Some(rest) = lower.strip_suffix('b') {
        (rest, 1u64)
    } else {
        (lower.as_str(), 1u64)
    };
    let num = num.trim();
    if num.is_empty() {
        return Err(format!("missing number in size `{s}`"));
    }
    let n: f64 = num
        .parse()
        .map_err(|_| format!("invalid size number in `{s}`"))?;
    if !n.is_finite() || n < 0.0 {
        return Err(format!("size out of range: `{s}`"));
    }
    let bytes = (n * mult as f64).round();
    if bytes > u64::MAX as f64 {
        return Err(format!("size too large: `{s}`"));
    }
    Ok(bytes as u64)
}

/// Format bytes for human logs.
pub fn format_bytes(n: u64) -> String {
    const K: f64 = 1024.0;
    let n = n as f64;
    if n >= K * K * K {
        format!("{:.2} GiB", n / (K * K * K))
    } else if n >= K * K {
        format!("{:.2} MiB", n / (K * K))
    } else if n >= K {
        format!("{:.2} KiB", n / K)
    } else {
        format!("{n:.0} B")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_common_sizes() {
        assert_eq!(parse_size("1024").unwrap(), 1024);
        assert_eq!(parse_size("1k").unwrap(), 1024);
        assert_eq!(parse_size("1M").unwrap(), 1024 * 1024);
        assert_eq!(parse_size("1G").unwrap(), 1024 * 1024 * 1024);
        assert_eq!(parse_size("1.5M").unwrap(), (1.5 * 1024.0 * 1024.0) as u64);
    }
}
