use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

use super::types::{RemoteOrigin, RemoteRepo, SyncType};

/// Load custom overlays from ~/.config/overlay-manager/custom.toml.
pub fn load() -> Result<Vec<RemoteRepo>> {
    let path = custom_path();
    if !path.exists() {
        return Ok(Vec::new());
    }

    let content = fs::read_to_string(&path).context("Failed to read custom.toml")?;
    let entries: Vec<CustomEntry> =
        toml::from_str(&content).context("Failed to parse custom.toml")?;

    Ok(entries
        .into_iter()
        .map(|e| RemoteRepo {
            name: e.name,
            description: e.description.unwrap_or_default(),
            homepage: e.url.clone(),
            owner: "user".into(),
            sources: vec![(SyncType::Git, e.url)],
            quality: "unknown".into(),
            status: "unofficial".into(),
            origin: RemoteOrigin::Custom,
        })
        .collect())
}

/// Save overlay to custom.toml (append to end of list).
pub fn save(repo: &RemoteRepo) -> Result<()> {
    let mut existing = load().unwrap_or_default();

    // Don't duplicate
    if existing.iter().any(|r| r.name == repo.name) {
        return Ok(());
    }

    existing.push(repo.clone());

    let entries: Vec<CustomEntry> = existing
        .iter()
        .map(|r| CustomEntry {
            name: r.name.clone(),
            url: r.sources.first().map(|(_, u)| u.clone()).unwrap_or_default(),
            description: if r.description.is_empty() {
                None
            } else {
                Some(r.description.clone())
            },
        })
        .collect();

    let toml = toml::to_string_pretty(&entries).context("Failed to serialize custom.toml")?;
    let dir = custom_path().parent().unwrap().to_path_buf();
    fs::create_dir_all(&dir).ok();
    fs::write(custom_path(), toml).context("Failed to write custom.toml")?;

    Ok(())
}

#[derive(Debug, Serialize, Deserialize)]
struct CustomEntry {
    name: String,
    url: String,
    description: Option<String>,
}

fn custom_path() -> PathBuf {
    dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("/tmp"))
        .join("overlay-manager")
        .join("custom.toml")
}
