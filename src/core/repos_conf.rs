use anyhow::{Context, Result};
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use super::types::{Repo, SyncType};

/// Directory with portage repository configs.
const REPOS_CONF_DIR: &str = "/etc/portage/repos.conf";

/// Name of the file where overlay-manager writes its overlays.
pub const MANAGER_CONF: &str = "overlay-manager.conf";

/// Read all installed overlays from /etc/portage/repos.conf/.
pub fn read_all() -> Result<Vec<Repo>> {
    let dir = Path::new(REPOS_CONF_DIR);
    if !dir.exists() {
        return Ok(Vec::new());
    }

    let mut repos = Vec::new();
    for entry in fs::read_dir(dir).context("Failed to read repos.conf")? {
        let path = entry?.path();
        if path.extension().map_or(false, |e| e == "conf") {
            repos.extend(parse_file(&path)?);
        }
    }
    Ok(repos)
}

/// Parse a single repos.conf file.
fn parse_file(path: &Path) -> Result<Vec<Repo>> {
    let content = fs::read_to_string(path).context("Failed to read config file")?;
    let mut repos = Vec::new();
    let mut current_section: Option<String> = None;
    let mut entries: BTreeMap<String, String> = BTreeMap::new();

    for line in content.lines() {
        let line = line.trim();

        // Skip empty lines and comments
        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        // Section start: [name]
        if line.starts_with('[') && line.ends_with(']') {
            // Save previous section
            if let Some(name) = current_section.take() {
                if let Some(repo) = build_repo(&name, &entries) {
                    repos.push(repo);
                }
                entries.clear();
            }
            current_section = Some(line[1..line.len() - 1].to_string());
        } else if let Some((key, value)) = line.split_once('=') {
            entries.insert(key.trim().to_lowercase(), value.trim().to_string());
        }
    }

    // Last section
    if let Some(name) = current_section {
        if let Some(repo) = build_repo(&name, &entries) {
            repos.push(repo);
        }
    }

    Ok(repos)
}

/// Build Repo from parsed section keys.
fn build_repo(name: &str, entries: &BTreeMap<String, String>) -> Option<Repo> {
    let location = entries.get("location")?;
    let sync_type = entries
        .get("sync-type")
        .map(|s| SyncType::from_str(s))
        .unwrap_or(SyncType::Other("unknown".into()));
    let sync_uri = entries.get("sync-uri").cloned().unwrap_or_default();
    let auto_sync = entries
        .get("auto-sync")
        .map(|s| s == "yes" || s == "true")
        .unwrap_or(false);
    let priority = entries.get("priority").and_then(|p| p.parse().ok());

    Some(Repo {
        name: name.to_string(),
        location: PathBuf::from(location),
        sync_type,
        sync_uri,
        auto_sync,
        priority,
    })
}

/// Write overlay to overlay-manager.conf file.
///
/// Appends section to end of file. Creates file if it doesn't exist.
pub fn append(repo: &Repo) -> Result<()> {
    let dir = Path::new(REPOS_CONF_DIR);
    fs::create_dir_all(dir).context("Failed to create repos.conf directory")?;

    let path = dir.join(MANAGER_CONF);
    let section = format!(
        "\n[{name}]\n\
         location = {location}\n\
         sync-type = {sync_type}\n\
         sync-uri = {sync_uri}\n\
         auto-sync = {auto_sync}\n\
         priority = {priority}\n",
        name = repo.name,
        location = repo.location.display(),
        sync_type = repo.sync_type.as_str(),
        sync_uri = repo.sync_uri,
        auto_sync = if repo.auto_sync { "yes" } else { "no" },
        priority = repo.priority.unwrap_or(50),
    );

    let mut content = if path.exists() {
        fs::read_to_string(&path).unwrap_or_default()
    } else {
        String::from("# Managed by overlay-manager\n")
    };

    content.push_str(&section);
    fs::write(&path, content).context("Failed to write repos.conf")?;
    Ok(())
}

/// Remove overlay from /etc/portage/repos.conf/ by name.
pub fn remove(name: &str) -> Result<bool> {
    remove_from(Path::new(REPOS_CONF_DIR), name)
}

/// Remove section [name] from all .conf files in the specified directory.
fn remove_from(dir: &Path, name: &str) -> Result<bool> {
    if !dir.exists() {
        return Ok(false);
    }

    for entry in fs::read_dir(dir).context("Failed to read repos.conf")? {
        let path = entry?.path();
        if path.extension().map_or(true, |e| e != "conf") {
            continue;
        }

        let content = fs::read_to_string(&path)
            .with_context(|| format!("Failed to read {}", path.display()))?;

        let mut new_content = String::new();
        let mut skip = false;
        let mut found = false;

        for line in content.lines() {
            let trimmed = line.trim();

            if trimmed.starts_with('[') && trimmed.ends_with(']') {
                let section_name = &trimmed[1..trimmed.len() - 1];
                skip = section_name == name;
                if skip {
                    found = true;
                }
            }

            if !skip {
                new_content.push_str(line);
                new_content.push('\n');
            }
        }

        if found {
            fs::write(&path, new_content.trim_end())
                .with_context(|| format!("Failed to write {}", path.display()))?;
            return Ok(true);
        }
    }

    Ok(false)
}

/// Remove overlay directory from /var/db/repos/<name>.
pub fn purge_files(name: &str) -> Result<bool> {
    let path = Path::new("/var/db/repos").join(name);
    if path.exists() {
        fs::remove_dir_all(&path)
            .with_context(|| format!("Failed to remove {}", path.display()))?;
        Ok(true)
    } else {
        Ok(false)
    }
}

pub fn is_installed(name: &str) -> Result<bool> {
    let repos = read_all()?;
    Ok(repos.iter().any(|r| r.name == name))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeMap;
    use std::fs;

    #[test]
    fn parse_single_section() {
        let dir = temp_dir();
        let conf = dir.join("test.conf");
        fs::write(
            &conf,
            "[my-overlay]\n\
             location = /var/db/repos/my-overlay\n\
             sync-type = git\n\
             sync-uri = https://github.com/me/my-overlay.git\n\
             auto-sync = yes\n\
             priority = 50\n",
        )
        .unwrap();

        let repos = parse_file(&conf).unwrap();
        assert_eq!(repos.len(), 1);
        assert_eq!(repos[0].name, "my-overlay");
        assert_eq!(repos[0].sync_uri, "https://github.com/me/my-overlay.git");
        assert!(repos[0].auto_sync);
        assert_eq!(repos[0].priority, Some(50));
        assert!(matches!(repos[0].sync_type, SyncType::Git));
    }

    #[test]
    fn parse_multiple_sections() {
        let dir = temp_dir();
        let conf = dir.join("multi.conf");
        fs::write(
            &conf,
            "[foo]\n\
             location = /a\n\
             sync-type = git\n\
             sync-uri = u\n\
             \n\
             [bar]\n\
             location = /b\n\
             sync-type = rsync\n\
             sync-uri = v\n",
        )
        .unwrap();

        let repos = parse_file(&conf).unwrap();
        assert_eq!(repos.len(), 2);
        assert_eq!(repos[0].name, "foo");
        assert_eq!(repos[1].name, "bar");
    }

    #[test]
    fn build_repo_missing_location() {
        let entries = BTreeMap::from([("sync-type".into(), "git".into())]);
        assert!(build_repo("test", &entries).is_none());
    }

    #[test]
    fn build_repo_defaults() {
        let entries = BTreeMap::from([
            ("location".into(), "/var/db/repos/x".into()),
            ("sync-uri".into(), "https://example.com".into()),
        ]);
        let repo = build_repo("x", &entries).unwrap();
        assert_eq!(repo.name, "x");
        assert!(!repo.auto_sync);
        assert_eq!(repo.priority, None);
    }

    #[test]
    fn remove_from_file() {
        let dir = temp_dir();
        let conf = dir.join("test.conf");
        fs::write(
            &conf,
            "[keep]\n\
             location = /a\n\
             sync-type = git\n\
             sync-uri = u\n\
             \n\
             [remove-me]\n\
             location = /b\n\
             sync-type = git\n\
             sync-uri = v\n",
        )
        .unwrap();

        let result = remove_from(&dir, "remove-me").unwrap();
        assert!(result);

        let after = fs::read_to_string(&conf).unwrap();
        assert!(after.contains("[keep]"));
        assert!(!after.contains("[remove-me]"));
    }

    #[test]
    fn remove_nonexistent() {
        let dir = temp_dir();
        let conf = dir.join("test.conf");
        fs::write(
            &conf,
            "[keep]\nlocation = /a\nsync-type = git\nsync-uri = u\n",
        )
        .unwrap();

        let result = remove_from(&dir, "nope").unwrap();
        assert!(!result);
    }

    fn temp_dir() -> PathBuf {
        use std::sync::atomic::{AtomicUsize, Ordering};
        static COUNTER: AtomicUsize = AtomicUsize::new(0);
        let n = COUNTER.fetch_add(1, Ordering::SeqCst);
        let dir = std::env::temp_dir().join(format!("om-test-{}-{}", std::process::id(), n));
        fs::create_dir_all(&dir).ok();
        dir
    }
}
