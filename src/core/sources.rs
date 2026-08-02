use anyhow::{Context, Result};
use std::collections::HashMap;
use std::fs;
use std::time::SystemTime;

use super::remote::{self, json_cache_path};
use super::types::RemoteRepo;

/// Cache TTL in seconds (24 hours).
const CACHE_TTL: u64 = 24 * 60 * 60;

/// Unified overlay list from all sources with deduplication.
pub struct SourceSet {
    pub repos: Vec<RemoteRepo>,
}

impl SourceSet {
    /// Load from cache if fresh. Otherwise return None.
    pub fn load_cached() -> Option<Self> {
        let path = json_cache_path();
        if !path.exists() {
            return None;
        }

        // Check cache freshness
        if let Ok(meta) = fs::metadata(&path)
            && let Ok(modified) = meta.modified()
                && let Ok(age) = SystemTime::now().duration_since(modified)
                    && age.as_secs() > CACHE_TTL {
                        return None;
                    }

        let json = fs::read_to_string(&path).ok()?;
        let repos: Vec<RemoteRepo> = serde_json::from_str(&json).ok()?;
        Some(Self { repos })
    }

    /// Build from all sources, deduplicate and write to cache.
    pub fn build() -> Result<Self> {
        let mut all = Vec::new();

        // 1. Official Gentoo registry
        match remote::fetch_and_parse() {
            Ok(repos) => all.extend(repos),
            Err(e) => eprintln!("Warning: failed to load repositories.xml: {}", e),
        }

        // 2. GitHub API
        match super::github::search_overlays() {
            Ok(repos) => all.extend(repos),
            Err(e) => eprintln!("Warning: GitHub search failed: {}", e),
        }

        let repos = dedup(all);
        let set = Self { repos };

        set.save_cache()?;
        Ok(set)
    }

    /// Write the merged list to JSON cache.
    fn save_cache(&self) -> Result<()> {
        let dir = json_cache_path().parent().unwrap().to_path_buf();
        fs::create_dir_all(&dir).ok();

        let json =
            serde_json::to_string_pretty(&self.repos).context("Failed to serialize cache")?;
        fs::write(json_cache_path(), json).context("Failed to write cache")?;

        Ok(())
    }
}

/// Deduplication by normalized repo_id.
///
/// Priority: official registry > GitHub
/// If two overlays have the same canonical URL,
/// keep the one that appears earlier in the list (i.e. from the higher priority source).
fn dedup(repos: Vec<RemoteRepo>) -> Vec<RemoteRepo> {
    let mut seen: HashMap<String, RemoteRepo> = HashMap::new();

    for repo in repos {
        let key = match repo.repo_id() {
            Some(id) => id,
            None => repo.name.to_lowercase(),
        };

        // First encountered is highest priority (GentooRegistry > Github > Custom)
        seen.entry(key).or_insert(repo);
    }

    let mut result: Vec<_> = seen.into_values().collect();
    result.sort_by(|a, b| a.name.cmp(&b.name));
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::types::{RemoteOrigin, SyncType};

    fn make_repo(name: &str, url: &str) -> RemoteRepo {
        RemoteRepo {
            name: name.into(),
            description: "".into(),
            homepage: "".into(),
            owner: "".into(),
            sources: vec![(SyncType::Git, url.into())],
            quality: "".into(),
            status: "".into(),
            origin: RemoteOrigin::GentooRegistry,
        }
    }

    #[test]
    fn dedup_same_url() {
        let repos = vec![
            make_repo("a", "https://github.com/x/y.git"),
            make_repo("b", "https://github.com/x/y"),
        ];
        let result = dedup(repos);
        assert_eq!(result.len(), 1);
    }

    #[test]
    fn dedup_different_urls() {
        let repos = vec![
            make_repo("a", "https://github.com/x/a.git"),
            make_repo("b", "https://github.com/x/b.git"),
        ];
        let result = dedup(repos);
        assert_eq!(result.len(), 2);
    }

    #[test]
    fn dedup_keeps_first() {
        let repos = vec![
            make_repo("first", "https://github.com/x/y.git"),
            make_repo("second", "https://github.com/x/y"),
        ];
        let result = dedup(repos);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].name, "first");
    }

    #[test]
    fn dedup_sorted_result() {
        let repos = vec![make_repo("z", "https://a"), make_repo("a", "https://b")];
        let result = dedup(repos);
        assert_eq!(result[0].name, "a");
        assert_eq!(result[1].name, "z");
    }
}
