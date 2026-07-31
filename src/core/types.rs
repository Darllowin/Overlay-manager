use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Unique repository identifier — normalized source URL.
pub type RepoId = String;

/// Overlay sync type.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum SyncType {
    Git,
    Rsync,
    Svn,
    Mercurial,
    Other(String),
}

impl SyncType {
    /// Recognize the type from a repos.conf string.
    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "git" => Self::Git,
            "rsync" => Self::Rsync,
            "svn" => Self::Svn,
            "mercurial" | "hg" => Self::Mercurial,
            other => Self::Other(other.to_string()),
        }
    }

    pub fn as_str(&self) -> &str {
        match self {
            Self::Git => "git",
            Self::Rsync => "rsync",
            Self::Svn => "svn",
            Self::Mercurial => "mercurial",
            Self::Other(s) => s.as_str(),
        }
    }
}

/// Installed overlay (from repos.conf).
#[derive(Debug, Clone)]
pub struct Repo {
    pub name: String,
    pub location: PathBuf,
    pub sync_type: SyncType,
    pub sync_uri: String,
    pub auto_sync: bool,
    pub priority: Option<u32>,
}

/// Where an overlay was sourced from for TUI display.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum RemoteOrigin {
    /// Official Gentoo registry (repositories.xml)
    GentooRegistry,
    /// Found via GitHub API
    Github,
    /// Added manually by the user
    Custom,
}

/// Overlay from the remote list (for search and adding).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RemoteRepo {
    pub name: String,
    pub description: String,
    pub homepage: String,
    pub owner: String,
    /// List of sync sources: (type, url)
    pub sources: Vec<(SyncType, String)>,
    pub quality: String,
    pub status: String,
    pub origin: RemoteOrigin,
}

impl RemoteRepo {
    /// Normalized identifier for deduplication.
    /// Takes the first git source and canonicalizes the URL.
    pub fn repo_id(&self) -> Option<RepoId> {
        self.sources
            .iter()
            .find(|(t, _)| matches!(t, SyncType::Git))
            .map(|(_, url)| canonical_repo_id(url))
    }
}

/// Canonicalize an overlay URL for comparison.
///
/// Normalizes different forms of the same git address to a single key:
///   https://github.com/foo/bar.git
///   https://github.com/foo/bar
///   git@github.com:foo/bar.git
///   git://github.com/foo/bar
///
/// All of them reduce to `github.com/foo/bar`.
pub fn canonical_repo_id(url: &str) -> RepoId {
    let s = url.trim();

    // Strip protocol prefix and user
    let s = s
        .trim_start_matches("https://")
        .trim_start_matches("http://")
        .trim_start_matches("git://")
        .trim_start_matches("git+ssh://");

    // git@github.com:user/repo.git → github.com/user/repo.git
    let s = if let Some(rest) = s.strip_prefix("git@") {
        rest.replace(':', "/")
    } else {
        s.to_string()
    };

    // ssh://git@github.com/user/repo.git → github.com/user/repo.git
    let s = s.trim_start_matches("ssh://");
    if let Some(rest) = s.strip_prefix("git@") {
        rest.replace(':', "/")
    } else {
        s.to_string()
    };

    // Strip .git suffix
    let s = s.strip_suffix(".git").unwrap_or(&s);

    s.trim_end_matches('/').to_lowercase()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn canonical_urls_match() {
        let urls = [
            "https://github.com/foo/bar.git",
            "https://github.com/foo/bar",
            "git@github.com:foo/bar.git",
            "git://github.com/foo/bar",
        ];
        let ids: Vec<_> = urls.iter().map(|u| canonical_repo_id(u)).collect();
        let first = &ids[0];
        for id in &ids {
            assert_eq!(id, first);
        }
    }

    #[test]
    fn ssh_git_url() {
        assert_eq!(
            canonical_repo_id("git+ssh://git@git.gentoo.org/repo/proj.git"),
            "git.gentoo.org/repo/proj"
        );
    }

    #[test]
    fn synctype_from_str() {
        assert!(matches!(SyncType::from_str("git"), SyncType::Git));
        assert!(matches!(SyncType::from_str("GIT"), SyncType::Git));
        assert!(matches!(SyncType::from_str("rsync"), SyncType::Rsync));
        assert!(matches!(SyncType::from_str("svn"), SyncType::Svn));
        assert!(matches!(SyncType::from_str("hg"), SyncType::Mercurial));
        assert!(matches!(SyncType::from_str("mercurial"), SyncType::Mercurial));
        assert!(matches!(SyncType::from_str("bzr"), SyncType::Other(_)));
    }

    #[test]
    fn synctype_as_str() {
        assert_eq!(SyncType::Git.as_str(), "git");
        assert_eq!(SyncType::Rsync.as_str(), "rsync");
    }

    #[test]
    fn repo_id_from_sources() {
        let repo = RemoteRepo {
            name: "test".into(),
            description: "".into(),
            homepage: "".into(),
            owner: "".into(),
            sources: vec![
                (SyncType::Git, "https://github.com/user/repo.git".into()),
                (SyncType::Git, "git@github.com:user/repo.git".into()),
            ],
            quality: "".into(),
            status: "".into(),
            origin: RemoteOrigin::GentooRegistry,
        };
        assert_eq!(repo.repo_id(), Some("github.com/user/repo".into()));
    }

    #[test]
    fn repo_id_no_git_source() {
        let repo = RemoteRepo {
            name: "test".into(),
            description: "".into(),
            homepage: "".into(),
            owner: "".into(),
            sources: vec![(SyncType::Rsync, "rsync://example.com/repo".into())],
            quality: "".into(),
            status: "".into(),
            origin: RemoteOrigin::GentooRegistry,
        };
        assert_eq!(repo.repo_id(), None);
    }
}
