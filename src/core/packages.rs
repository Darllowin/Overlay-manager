use anyhow::{Context, Result};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

pub type PackageMap = HashMap<String, Vec<String>>;

pub fn scan() -> Result<PackageMap> {
    let pkg_root = Path::new("/var/db/pkg");
    if !pkg_root.exists() {
        return Ok(HashMap::new());
    }

    let mut map: PackageMap = HashMap::new();

    for cat_entry in fs::read_dir(pkg_root).context("Failed to read /var/db/pkg")? {
        let cat_path = cat_entry?.path();
        if !cat_path.is_dir() {
            continue;
        }

        for pkg_entry in fs::read_dir(&cat_path)? {
            let pkg_path = pkg_entry?.path();
            if !pkg_path.is_dir() {
                continue;
            }

            let repo_file = pkg_path.join("repository");
            let repo_name = match fs::read_to_string(&repo_file) {
                Ok(s) => s.trim().to_string(),
                Err(_) => continue,
            };

            let pkg_name = match pkg_path.file_name() {
                Some(n) => n.to_string_lossy().to_string(),
                None => continue,
            };

            let cat_name = cat_path
                .file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_default();

            let atom = format!("{}/{}", cat_name, pkg_name);
            map.entry(repo_name).or_default().push(atom);
        }
    }

    Ok(map)
}

pub fn scan_overlay(repo_path: &Path) -> Result<Vec<String>> {
    let mut pkgs = Vec::new();
    if !repo_path.is_dir() {
        return Ok(pkgs);
    }

    for cat_entry in fs::read_dir(repo_path)? {
        let cat_path = cat_entry?.path();
        if !cat_path.is_dir() {
            continue;
        }
        let cat_name = cat_path.file_name().unwrap_or_default().to_string_lossy();
        if cat_name == "metadata" || cat_name == "profiles" || cat_name.starts_with('.') {
            continue;
        }

        for pkg_entry in fs::read_dir(&cat_path)? {
            let pkg_path = pkg_entry?.path();
            if !pkg_path.is_dir() {
                continue;
            }
            let pkg_name = pkg_path.file_name().unwrap_or_default().to_string_lossy();

            let mut latest_ver = String::new();
            if let Ok(entries) = fs::read_dir(&pkg_path) {
                for e in entries.filter_map(|e| e.ok()) {
                    let fname = e.file_name().to_string_lossy().to_string();
                    if let Some(ver) = fname.strip_suffix(".ebuild") {
                        if let Some(v) = ver.strip_prefix(&format!("{}-", pkg_name)) {
                            if v > latest_ver.as_str() {
                                latest_ver = v.to_string();
                            }
                        }
                    }
                }
            }

            if latest_ver.is_empty() {
                pkgs.push(format!("{}/{}", cat_name, pkg_name));
            } else {
                pkgs.push(format!("{}/{}-{}", cat_name, pkg_name, latest_ver));
            }
        }
    }

    pkgs.sort();
    Ok(pkgs)
}

fn split_pkg_atom(pkg: &str) -> Option<(&str, &str)> {
    let (cat, name_ver) = pkg.split_once('/')?;
    let name = name_ver
        .rsplit_once('-')
        .filter(|(_, ver)| ver.chars().next().map_or(false, |c| c.is_ascii_digit()))
        .map(|(name, _)| name)
        .unwrap_or(name_ver);
    Some((cat, name))
}

pub fn read_description(repo_path: &Path, pkg: &str) -> String {
    let Some((cat, name)) = split_pkg_atom(pkg) else {
        return String::new();
    };
    let pkg_dir = repo_path.join(cat).join(name);

    let meta_path = pkg_dir.join("metadata.xml");
    if let Ok(xml) = fs::read_to_string(&meta_path) {
        if let Some(desc) = extract_xml_tag(&xml, "longdescription") {
            if !desc.is_empty() {
                return desc;
            }
        }
    }

    if let Ok(entries) = fs::read_dir(&pkg_dir) {
        for entry in entries.filter_map(|e| e.ok()) {
            if entry.file_name().to_string_lossy().ends_with(".ebuild") {
                if let Ok(content) = fs::read_to_string(entry.path()) {
                    if let Some(desc) = extract_ebuild_var(&content, "DESCRIPTION") {
                        if !desc.is_empty() {
                            return desc;
                        }
                    }
                }
                break;
            }
        }
    }

    String::new()
}

pub fn read_use_flags(repo_path: &Path, pkg: &str) -> String {
    let Some((cat, name)) = split_pkg_atom(pkg) else {
        return String::new();
    };
    let pkg_dir = repo_path.join(cat).join(name);

    if let Ok(entries) = fs::read_dir(&pkg_dir) {
        for entry in entries.filter_map(|e| e.ok()) {
            if entry.file_name().to_string_lossy().ends_with(".ebuild") {
                if let Ok(content) = fs::read_to_string(entry.path()) {
                    if let Some(flags) = extract_ebuild_var(&content, "IUSE") {
                        if !flags.is_empty() {
                            return flags;
                        }
                    }
                }
                break;
            }
        }
    }

    String::new()
}

fn extract_ebuild_var(content: &str, var: &str) -> Option<String> {
    for line in content.lines() {
        let line = line.trim();
        if let Some(rest) = line.strip_prefix(var).and_then(|s| s.strip_prefix('=')) {
            let val = rest.trim().trim_matches('"');
            if !val.is_empty() {
                return Some(val.to_string());
            }
        }
    }
    None
}

fn extract_xml_tag(xml: &str, tag: &str) -> Option<String> {
    let start_tag = format!("<{}", tag);
    let end_tag = format!("</{}>", tag);

    let start = xml.find(&start_tag)?;
    let content_start = xml[start..].find('>')? + 1;
    let start = start + content_start;

    let end = xml[start..].find(&end_tag)?;
    let text = xml[start..start + end].trim();
    if text.is_empty() {
        None
    } else {
        Some(text.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scan_doesnt_panic() {
        let map = scan();
        assert!(map.is_ok());
    }
}
