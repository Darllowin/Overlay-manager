use std::path::Path;
use std::process::Command;

/// Get disk usage of an overlay directory (e.g., "111M").
pub fn repo_disk_usage(path: &Path) -> String {
    Command::new("du")
        .args(["-sh", "--"])
        .arg(path)
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map(|s| s.split_whitespace().next().unwrap_or("?").to_string())
        .unwrap_or_else(|| "?".to_string())
}

/// Get the time elapsed since last sync (None = never synced).
pub fn repo_sync_age(path: &Path) -> Option<std::time::Duration> {
    let fetch_head = path.join(".git").join("FETCH_HEAD");
    let meta = std::fs::metadata(&fetch_head).ok()?;
    meta.modified().ok()?.elapsed().ok()
}

/// Get the age of the cache file as a human-readable string.
pub fn cache_age() -> String {
    let path = super::remote::json_cache_path();
    let meta = match std::fs::metadata(&path) {
        Ok(m) => m,
        Err(_) => return "never".to_string(),
    };

    let Ok(modified) = meta.modified() else {
        return "—".to_string();
    };
    let Ok(duration) = modified.elapsed() else {
        return "—".to_string();
    };

    format_age(duration)
}

fn format_age(duration: std::time::Duration) -> String {
    let secs = duration.as_secs();
    if secs < 60 {
        format!("{}s ago", secs)
    } else if secs < 3600 {
        format!("{}m ago", secs / 60)
    } else if secs < 86400 {
        format!("{}h ago", secs / 3600)
    } else {
        format!("{}d ago", secs / 86400)
    }
}

/// Get the last sync date from .git/FETCH_HEAD as a human-readable string.
pub fn repo_last_sync(path: &Path) -> String {
    let fetch_head = path.join(".git").join("FETCH_HEAD");
    let meta = match std::fs::metadata(&fetch_head) {
        Ok(m) => m,
        Err(_) => return "—".to_string(),
    };

    let Ok(modified) = meta.modified() else {
        return "—".to_string();
    };

    let Ok(duration) = modified.elapsed() else {
        return "—".to_string();
    };

    format_age(duration)
}
