use anyhow::{Context, Result};
use serde::Deserialize;

use super::types::{RemoteOrigin, RemoteRepo, SyncType};

/// Search for Gentoo overlays via GitHub API by `gentoo-overlay` topic.
///
/// GitHub without token allows 60 requests/hour. With token (GITHUB_TOKEN) — 5000.
pub fn search_overlays() -> Result<Vec<RemoteRepo>> {
    let client = reqwest::blocking::Client::builder()
        .user_agent("overlay-manager/0.1")
        .build()
        .context("Failed to create HTTP client")?;

    let mut all_repos = Vec::new();
    let mut page = 1;

    loop {
        let url = format!(
            "https://api.github.com/search/repositories?q=topic:gentoo-overlay&per_page=100&page={}",
            page
        );

        let response: SearchResponse = client
            .get(&url)
            .header("Accept", "application/vnd.github.v3+json")
            .send()
            .context("GitHub API request error")?
            .json()
            .context("Failed to parse GitHub API response")?;

        let count = response.items.len();
        for item in response.items {
            all_repos.push(RemoteRepo {
                name: item.name,
                description: item.description.unwrap_or_default(),
                homepage: item.html_url.clone(),
                owner: item.owner.login,
                sources: vec![(SyncType::Git, item.clone_url)],
                quality: "unknown".into(),
                status: "unofficial".into(),
                origin: RemoteOrigin::Github,
            });
        }

        // GitHub API pagination: fewer than 100 results = last page
        if count < 100 {
            break;
        }
        page += 1;

        // Guard against infinite loop
        if page > 10 {
            break;
        }
    }

    Ok(all_repos)
}

#[derive(Debug, Deserialize)]
struct SearchResponse {
    items: Vec<RepoItem>,
}

#[derive(Debug, Deserialize)]
struct RepoItem {
    name: String,
    description: Option<String>,
    html_url: String,
    clone_url: String,
    owner: Owner,
}

#[derive(Debug, Deserialize)]
struct Owner {
    login: String,
}
