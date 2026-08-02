use anyhow::{Context, Result};
use quick_xml::Reader;
use quick_xml::events::Event;
use std::path::PathBuf;

use super::types::{RemoteOrigin, RemoteRepo, SyncType};

const REPOSITORIES_URL: &str = "https://api.gentoo.org/overlays/repositories.xml";

/// Load and parse repositories.xml, return a list of overlays.
pub fn fetch_and_parse() -> Result<Vec<RemoteRepo>> {
    let xml = reqwest::blocking::get(REPOSITORIES_URL)
        .context("Failed to download repositories.xml")?
        .text()
        .context("Failed to read response body")?;

    parse(&xml)
}

/// Event-driven XML parsing of repositories.xml.
fn parse(xml: &str) -> Result<Vec<RemoteRepo>> {
    let mut reader = Reader::from_str(xml);
    reader.config_mut().trim_text(true);

    let mut repos = Vec::new();
    let mut buf = Vec::new();

    let mut repo: Option<RemoteRepo> = None;
    let mut tag = String::new();
    let mut source_type = String::new();

    // Depth flags: which elements we are currently inside.
    let mut depth_repo: usize = 0;
    let mut depth_owner: usize = 0;
    let mut depth_source: usize = 0;

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(e)) => {
                tag = String::from_utf8_lossy(e.name().as_ref()).to_string();

                match tag.as_str() {
                    "repo" => {
                        depth_repo += 1;
                        repo = Some(empty_repo());
                        // Extract quality and status from <repo> tag attributes
                        if let Some(r) = &mut repo {
                            for attr in e.attributes().filter_map(|a| a.ok()) {
                                match attr.key.as_ref() {
                                    b"quality" => {
                                        r.quality = String::from_utf8_lossy(&attr.value).to_string()
                                    }
                                    b"status" => {
                                        r.status = String::from_utf8_lossy(&attr.value).to_string()
                                    }
                                    _ => {}
                                }
                            }
                        }
                    }
                    "owner" if depth_repo > 0 => depth_owner += 1,
                    "source" if depth_repo > 0 => {
                        depth_source += 1;
                        // Remember source type from type="git" attribute
                        source_type = e
                            .attributes()
                            .filter_map(|a| a.ok())
                            .find(|a| a.key.as_ref() == b"type")
                            .map(|a| String::from_utf8_lossy(&a.value).to_string())
                            .unwrap_or_default();
                    }
                    _ => {}
                }
            }
            Ok(Event::End(e)) => {
                tag = String::from_utf8_lossy(e.name().as_ref()).to_string();

                match tag.as_str() {
                    "repo" => {
                        depth_repo -= 1;
                        if depth_repo == 0
                            && let Some(r) = repo.take() {
                                repos.push(r);
                            }
                    }
                    "owner" => depth_owner = depth_owner.saturating_sub(1),
                    "source" => {
                        depth_source -= 1;
                        source_type.clear();
                    }
                    _ => {}
                }
            }
            Ok(Event::Text(e)) => {
                let text = e.unescape().unwrap_or_default();
                let r = match &mut repo {
                    Some(r) => r,
                    None => continue,
                };

                // Processing depends on which depth level we are at
                if depth_source > 0 {
                    // Text inside <source> is the URL
                    if !text.is_empty() {
                        r.sources
                            .push((SyncType::from_str(&source_type), text.to_string()));
                    }
                } else if depth_owner > 0 {
                    match tag.as_str() {
                        "name" => r.owner = text.to_string(),
                        "email" if r.owner.is_empty() => r.owner = text.to_string(),
                        _ => {}
                    }
                } else if depth_repo > 0 {
                    match tag.as_str() {
                        "name" => r.name = text.to_string(),
                        "description" => r.description = text.to_string(),
                        "homepage" => r.homepage = text.to_string(),
                        _ => {}
                    }
                }
            }
            Ok(Event::Eof) => break,
            Err(e) => return Err(anyhow::anyhow!("XML parsing error: {}", e)),
            _ => {}
        }
        buf.clear();
    }

    Ok(repos)
}

fn empty_repo() -> RemoteRepo {
    RemoteRepo {
        name: String::new(),
        description: String::new(),
        homepage: String::new(),
        owner: String::new(),
        sources: Vec::new(),
        quality: String::new(),
        status: String::new(),
        origin: RemoteOrigin::GentooRegistry,
    }
}

/// Path to the JSON cache of the merged list.
pub fn json_cache_path() -> PathBuf {
    dirs::cache_dir()
        .unwrap_or_else(|| PathBuf::from("/tmp"))
        .join("overlay-manager")
        .join("repos.json")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_minimal_xml() {
        let xml = r#"<?xml version="1.0"?>
<repositories version="1.0">
  <repo quality="experimental" status="unofficial">
    <name>test-overlay</name>
    <description lang="en">A test overlay</description>
    <homepage>https://github.com/test/overlay</homepage>
    <owner type="person">
      <email>test@example.com</email>
      <name>Test Owner</name>
    </owner>
    <source type="git">https://github.com/test/overlay.git</source>
  </repo>
</repositories>"#;

        let repos = parse(xml).unwrap();
        assert_eq!(repos.len(), 1);
        let r = &repos[0];
        assert_eq!(r.name, "test-overlay");
        assert_eq!(r.description, "A test overlay");
        assert_eq!(r.homepage, "https://github.com/test/overlay");
        assert_eq!(r.owner, "Test Owner");
        assert_eq!(r.quality, "experimental");
        assert_eq!(r.status, "unofficial");
        assert_eq!(r.sources.len(), 1);
        assert_eq!(r.sources[0].1, "https://github.com/test/overlay.git");
    }

    #[test]
    fn parse_multiple_repos() {
        let xml = r#"<?xml version="1.0"?>
<repositories>
  <repo><name>a</name><source type="git">u1</source></repo>
  <repo><name>b</name><source type="git">u2</source></repo>
</repositories>"#;

        let repos = parse(xml).unwrap();
        assert_eq!(repos.len(), 2);
    }

    #[test]
    fn parse_multiple_sources() {
        let xml = r#"<?xml version="1.0"?>
<repositories>
  <repo>
    <name>multi</name>
    <source type="git">https://github.com/x/y.git</source>
    <source type="git">git@github.com:x/y.git</source>
  </repo>
</repositories>"#;

        let repos = parse(xml).unwrap();
        assert_eq!(repos[0].sources.len(), 2);
    }
}
