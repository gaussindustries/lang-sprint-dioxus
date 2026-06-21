//! Persistence: an append-only JSONL log of evidence. The model is re-derived
//! by folding it. No Dioxus here — just the filesystem.

use std::fs;
use std::io::Write;
use std::path::Path;

use super::evidence::Evidence;

/// Load every evidence line; malformed lines are skipped, a missing file is empty.
pub fn load_log(path: &Path) -> Vec<Evidence> {
    match fs::read_to_string(path) {
        Ok(s) => s
            .lines()
            .filter(|l| !l.trim().is_empty())
            .filter_map(|l| serde_json::from_str::<Evidence>(l).ok())
            .collect(),
        Err(_) => Vec::new(),
    }
}

/// Append one evidence record as a JSON line, creating parent dirs as needed.
pub fn append(path: &Path, e: &Evidence) -> std::io::Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let line = serde_json::to_string(e)
        .map_err(|err| std::io::Error::new(std::io::ErrorKind::Other, err))?;
    let mut f = fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)?;
    writeln!(f, "{line}")?;
    Ok(())
}
