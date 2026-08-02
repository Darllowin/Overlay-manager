use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::mpsc;
use std::time::Instant;

use super::actions::Action;
use crate::core::{
    packages,
    repos_conf,
    sources::SourceSet,
    sync::{self, SyncEvent},
    types::{RemoteRepo, Repo},
};
use crate::locale;

/// Main application state (Model).
pub struct App {
    /// Current view/tab.
    pub view: View,
    /// Previous view — for returning from Help.
    pub previous_view: View,
    /// Index of the selected item in the filtered list.
    pub selected: usize,
    /// Scroll offset of the list (first visible row).
    pub scroll_offset: usize,
    /// Item indices after search/filtering.
    ///
    /// IMPORTANT: indices point to `available` when view=Browse,
    /// and to `installed` when view=Installed.
    pub filtered: Vec<usize>,

    /// Installed overlays (from repos.conf).
    pub installed: Vec<Repo>,
    /// Overlays available for installation (from cache).
    pub available: Vec<RemoteRepo>,

    /// Map: repository name → list of installed packages.
    pub packages: HashMap<String, Vec<String>>,

    /// Search query string.
    pub search_query: String,
    /// Whether search input mode is active.
    pub search_mode: bool,

    /// Channel for receiving events from background sync.
    sync_rx: Option<mpsc::Receiver<SyncEvent>>,
    /// Accumulated sync output.
    pub sync_output: Vec<String>,
    /// Name of the overlay being synced.
    pub sync_repo: Option<String>,

    /// Temporary notification.
    pub message: Option<Message>,
    /// Whether cache is loading in background.
    pub loading: bool,
    /// Whether /var/db/pkg/ scan is complete.
    pub packages_ready: bool,
    /// Whether there is write access to /etc/portage/repos.conf/.
    pub is_root: bool,

    /// Channel for background cache refresh (RefreshCache).
    cache_rx: Option<mpsc::Receiver<Result<Vec<RemoteRepo>, String>>>,
    /// Channel for receiving /var/db/pkg/ scan result.
    packages_rx: Option<mpsc::Receiver<HashMap<String, Vec<String>>>>,
    /// Awaiting operation confirmation.
    pub confirm: Option<ConfirmAction>,
    /// Frame counter for spinner animation.
    pub spinner_frame: usize,
    /// Name of the repository whose packages we're viewing (View::Packages).
    pub pkg_repo: String,
    /// Package list for viewing (View::Packages).
    pub pkg_list: Vec<String>,
    /// Full package list (unfiltered) for restoration.
    pkg_list_full: Vec<String>,
    /// Selected package in Packages mode.
    pub pkg_selected: usize,
    /// Description of the selected package (lazy loaded from metadata.xml).
    pub pkg_description: String,
    /// USE flags of the selected package.
    pub pkg_use_flags: String,
    /// Path to repository for loading descriptions.
    pkg_repo_path: PathBuf,
}

/// User message with auto-dismiss.
pub struct Message {
    pub text: String,
    pub level: MessageLevel,
    pub expires_at: Instant,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MessageLevel {
    Info,
    Success,
    Error,
}

/// Navigation state.
#[derive(Debug, Clone, PartialEq)]
pub enum View {
    Browse,
    Installed,
    Help,
    Syncing,
    Confirm,
    /// Viewing overlay packages.
    Packages(String),
}

/// Operation requiring confirmation.
#[derive(Debug, Clone)]
pub struct ConfirmAction {
    pub repo_name: String,
}

impl App {
    pub fn new() -> Self {
        let installed = repos_conf::read_all().unwrap_or_default();
        let available = SourceSet::load_cached()
            .map(|s| s.repos)
            .unwrap_or_default();

        let filtered: Vec<usize> = (0..available.len()).collect();
        let loading = available.is_empty();

        // Background scan of /var/db/pkg/
        let (pkg_tx, pkg_rx) = mpsc::channel();
        std::thread::spawn(move || {
            let result = packages::scan().unwrap_or_default();
            pkg_tx.send(result).ok();
        });

        Self {
            view: View::Browse,
            previous_view: View::Browse,
            selected: 0,
            scroll_offset: 0,
            filtered,
            installed,
            available,
            packages: HashMap::new(),
            search_query: String::new(),
            search_mode: false,
            sync_rx: None,
            sync_output: Vec::new(),
            sync_repo: None,
            message: None,
            loading,
            packages_ready: false,
            is_root: is_writable(),
            cache_rx: None,
            packages_rx: Some(pkg_rx),
            confirm: None,
            spinner_frame: 0,
            pkg_repo: String::new(),
            pkg_list: Vec::new(),
            pkg_list_full: Vec::new(),
            pkg_selected: 0,
            pkg_description: String::new(),
            pkg_use_flags: String::new(),
            pkg_repo_path: PathBuf::new(),
        }
    }

    /// Handle an action and return `true` if the app should continue.
    pub fn handle(&mut self, action: Action) -> bool {
        if self.view == View::Syncing {
            if action == Action::Quit {
                return false;
            }
            self.handle_sync_tick();
            return true;
        }

        // Confirmation mode
        if self.view == View::Confirm {
            match action {
                Action::ConfirmYes => {
                    let confirm = self.confirm.take().unwrap();
                    self.do_remove(&confirm.repo_name);
                }
                Action::Close | Action::Quit => {
                    self.confirm = None;
                    self.view = View::Installed;
                }
                _ => {}
            }
            return true;
        }

        // Package view mode: navigation + search
        if let View::Packages(_) = &self.view {
            if self.search_mode {
                return match action {
                    Action::SearchChar(c) => {
                        self.search_query.push(c);
                        self.apply_pkg_filter();
                        true
                    }
                    Action::SearchBackspace => {
                        self.search_query.pop();
                        self.apply_pkg_filter();
                        true
                    }
                    Action::Close => {
                        self.search_mode = false;
                        self.search_query.clear();
                        self.apply_pkg_filter();
                        true
                    }
                    Action::MoveUp => {
                        self.pkg_selected = self.pkg_selected.saturating_sub(1);
                        self.load_pkg_description();
                        true
                    }
                    Action::MoveDown => {
                        if self.pkg_selected + 1 < self.pkg_list.len() {
                            self.pkg_selected += 1;
                            self.load_pkg_description();
                        }
                        true
                    }
                    Action::Quit => {
                        self.search_mode = false;
                        self.search_query.clear();
                        self.view = self.previous_view.clone();
                        true
                    }
                    _ => true,
                };
            }

            match action {
                Action::MoveUp => {
                    self.pkg_selected = self.pkg_selected.saturating_sub(1);
                    self.load_pkg_description();
                }
                Action::MoveDown => {
                    if self.pkg_selected + 1 < self.pkg_list.len() {
                        self.pkg_selected += 1;
                        self.load_pkg_description();
                    }
                }
                Action::MoveTop => {
                    self.pkg_selected = 0;
                    self.load_pkg_description();
                }
                Action::MoveBottom => {
                    if !self.pkg_list.is_empty() {
                        self.pkg_selected = self.pkg_list.len() - 1;
                        self.load_pkg_description();
                    }
                }
                Action::SearchStart => {
                    self.search_mode = true;
                    self.search_query.clear();
                }
                Action::Close | Action::Quit => {
                    self.search_mode = false;
                    self.search_query.clear();
                    self.view = self.previous_view.clone();
                }
                _ => {}
            }
            return true;
        }

        match action {
            Action::Quit => return false,

            Action::MoveUp => {
                self.selected = self.selected.saturating_sub(1);
            }
            Action::MoveDown => {
                if !self.current_list().is_empty() && self.selected + 1 < self.current_list().len()
                {
                    self.selected += 1;
                }
            }
            Action::MoveTop => self.selected = 0,
            Action::MoveBottom => {
                let len = self.current_list().len();
                if len > 0 {
                    self.selected = len - 1;
                }
            }

            Action::Close => self.close(),
            Action::OpenPackages => self.open_packages(),

            Action::AddOrSync => self.add_or_sync(),
            Action::SyncAll => self.sync_all(),
            Action::RemoveRepo => self.remove_repo(),

            Action::TabBrowse => self.switch_to(View::Browse),
            Action::TabInstalled => self.switch_to(View::Installed),

            Action::SearchStart => {
                if matches!(self.view, View::Browse | View::Installed | View::Packages(_)) {
                    self.search_mode = true;
                    self.search_query.clear();
                }
            }
            Action::SearchChar(c) => {
                if self.search_mode {
                    self.search_query.push(c);
                    self.apply_filter();
                }
            }
            Action::SearchBackspace => {
                if self.search_mode {
                    self.search_query.pop();
                    self.apply_filter();
                }
            }

            Action::RefreshCache => self.refresh_cache(),
            Action::ToggleHelp => {
                if self.view == View::Help {
                    self.view = self.previous_view.clone();
                } else {
                    self.previous_view = self.view.clone();
                    self.view = View::Help;
                }
            }
            Action::Noop => {}
            // ConfirmYes is handled in the Confirm block above
            Action::ConfirmYes => unreachable!(),
        }
        true
    }

    /// Tick — called on each iteration of the event loop.
    pub fn tick(&mut self) {
        self.handle_sync_tick();
        self.handle_cache_tick();
        self.handle_packages_tick();
        self.expire_messages();
        self.spinner_frame = self.spinner_frame.wrapping_add(1);
    }

    /// Current display list (indices into available or installed).
    pub fn current_list(&self) -> &[usize] {
        &self.filtered
    }

    /// Adjust scroll_offset so selected is visible at the given area height.
    pub fn clamp_scroll(&mut self, visible_height: usize) {
        if visible_height == 0 || self.filtered.is_empty() {
            self.scroll_offset = 0;
            return;
        }
        if self.selected < self.scroll_offset {
            self.scroll_offset = self.selected;
        } else if self.selected >= self.scroll_offset + visible_height {
            self.scroll_offset = self.selected.saturating_sub(visible_height - 1);
        }
        // Don't let it scroll past the needed range
        let max_offset = self.filtered.len().saturating_sub(visible_height);
        if self.scroll_offset > max_offset {
            self.scroll_offset = max_offset;
        }
    }

    /// Selected item from available. Returns None if view is not Browse.
    pub fn selected_remote(&self) -> Option<&RemoteRepo> {
        if !matches!(self.view, View::Browse) {
            return None;
        }
        let idx = *self.filtered.get(self.selected)?;
        self.available.get(idx)
    }

    /// Selected item from installed. Returns None if view is not Installed.
    pub fn selected_installed(&self) -> Option<&Repo> {
        if !matches!(self.view, View::Installed) {
            return None;
        }
        let idx = *self.filtered.get(self.selected)?;
        self.installed.get(idx)
    }

    /// Show a temporary message.
    pub fn show_message(&mut self, text: &str, level: MessageLevel) {
        self.message = Some(Message {
            text: text.to_string(),
            level,
            expires_at: Instant::now() + std::time::Duration::from_secs(4),
        });
    }

    /// Show an error with human-readable text.
    pub fn show_error(&mut self, error: anyhow::Error) {
        let s = locale::strings();
        // Permission denied → user-friendly message
        let msg = if format!("{}", error).contains("Permission denied") {
            s.need_root.to_string()
        } else {
            error.to_string()
        };
        self.show_message(&msg, MessageLevel::Error);
    }

    // ─── private methods ───

    fn switch_to(&mut self, new_view: View) {
        self.previous_view = self.view.clone();
        self.view = new_view;
        self.apply_filter();
    }

    fn close(&mut self) {
        if self.search_mode {
            self.search_mode = false;
            self.search_query.clear();
            self.apply_filter();
        } else if self.view == View::Help {
            self.view = self.previous_view.clone();
        }
    }

    fn open_packages(&mut self) {
        // Reset search on transition
        self.search_mode = false;
        self.search_query.clear();

        let repo_name = match &self.view {
            View::Browse => {
                let idx = match self.filtered.get(self.selected) {
                    Some(i) => *i,
                    None => return,
                };
                match self.available.get(idx) {
                    Some(r) => r.name.clone(),
                    None => return,
                }
            }
            View::Installed => {
                let idx = match self.filtered.get(self.selected) {
                    Some(i) => *i,
                    None => return,
                };
                match self.installed.get(idx) {
                    Some(r) => r.name.clone(),
                    None => return,
                }
            }
            _ => return,
        };

        // Find overlay in installed (regardless of current tab)
        if let Some(repo) = self.installed.iter().find(|r| r.name == repo_name) {
            let pkgs = crate::core::packages::scan_overlay(&repo.location).unwrap_or_default();
            self.pkg_repo = repo_name;
            self.pkg_repo_path = repo.location.clone();
            self.pkg_list = pkgs.clone();
            self.pkg_list_full = pkgs;
        } else {
            let s = locale::strings();
            self.pkg_repo = repo_name;
            let hint = vec![
                s.installed_no.to_string(),
                String::new(),
                s.not_installed_hint.to_string(),
            ];
            self.pkg_list = hint.clone();
            self.pkg_list_full = hint;
        }

        self.pkg_selected = 0;
        self.pkg_description = String::new();
        self.previous_view = self.view.clone();
        self.view = View::Packages(self.pkg_repo.clone());
        self.load_pkg_description();
    }

    fn add_or_sync(&mut self) {
        let s = locale::strings();

        let (repo_name, is_installed, remote_repo) = match &self.view {
            View::Browse => {
                let idx = match self.filtered.get(self.selected) {
                    Some(i) => *i,
                    None => return,
                };
                let repo = match self.available.get(idx) {
                    Some(r) => r.clone(),
                    None => return,
                };
                let installed = self.installed.iter().any(|r| r.name == repo.name);
                (repo.name.clone(), installed, Some(repo))
            }
            View::Installed => {
                let idx = match self.filtered.get(self.selected) {
                    Some(i) => *i,
                    None => return,
                };
                let name = match self.installed.get(idx) {
                    Some(r) => r.name.clone(),
                    None => return,
                };
                (name, true, None)
            }
            _ => return,
        };

        if !self.is_root {
            self.show_message(s.need_root, MessageLevel::Error);
            return;
        }

        if !is_installed {
            if let Some(repo) = &remote_repo {
                match self.install_repo(repo) {
                    Ok(()) => self.show_message(
                        &(s.sync_added)(&repo_name),
                        MessageLevel::Info,
                    ),
                    Err(e) => {
                        self.show_error(e);
                        return;
                    }
                }
            }
        }

        self.sync_output.clear();
        self.sync_repo = Some(repo_name.clone());
        self.sync_rx = Some(sync::sync_repo(repo_name));
        self.view = View::Syncing;
    }

    fn install_repo(&mut self, repo: &RemoteRepo) -> anyhow::Result<()> {
        let s = locale::strings();
        if self.installed.iter().any(|r| r.name == repo.name) {
            anyhow::bail!("{}", (s.already_installed)(&repo.name));
        }

        let source = repo
            .sources
            .iter()
            .find(|(t, _)| matches!(t, crate::core::types::SyncType::Git))
            .map(|(_, u)| u.clone())
            .ok_or_else(|| anyhow::anyhow!("{}", s.no_git_source))?;

        let new_repo = Repo {
            name: repo.name.clone(),
            location: std::path::PathBuf::from(format!("/var/db/repos/{}", repo.name)),
            sync_type: crate::core::types::SyncType::Git,
            sync_uri: source,
            auto_sync: true,
            priority: Some(50),
        };

        repos_conf::append(&new_repo)?;
        self.installed.push(new_repo);
        Ok(())
    }

    fn sync_all(&mut self) {
        if !self.is_root {
            self.show_message(locale::strings().need_root, MessageLevel::Error);
            return;
        }

        self.sync_output.clear();
        self.sync_repo = Some("all".into());
        self.sync_rx = Some(sync::sync_all());
        self.view = View::Syncing;
    }

    fn remove_repo(&mut self) {
        if !matches!(self.view, View::Installed) {
            return;
        }
        let idx = match self.filtered.get(self.selected) {
            Some(i) => *i,
            None => return,
        };
        let name = match self.installed.get(idx) {
            Some(r) => r.name.clone(),
            None => return,
        };

        self.confirm = Some(ConfirmAction { repo_name: name });
        self.view = View::Confirm;
    }

    fn do_remove(&mut self, name: &str) {
        let s = locale::strings();
        self.view = View::Installed;

        if !self.is_root {
            self.show_message(s.need_root, MessageLevel::Error);
            return;
        }

        match repos_conf::remove(name) {
            Ok(true) => {
                self.installed.retain(|r| r.name != name);
                match repos_conf::purge_files(name) {
                    Ok(true) => self.show_message(
                        &(s.removed_with_files)(name),
                        MessageLevel::Success,
                    ),
                    Ok(false) => self.show_message(
                        &(s.removed_no_files)(name),
                        MessageLevel::Success,
                    ),
                    Err(e) => self.show_error(e),
                }
                self.switch_to(View::Installed);
            }
            Ok(false) => {
                self.show_message(s.not_found_in_config, MessageLevel::Info);
            }
            Err(e) => self.show_error(e),
        }
    }

    // ── background cache loading ──

    /// Start background cache refresh.
    pub fn start_background_cache(&mut self) {
        if !self.loading {
            return;
        }
        let (tx, rx) = mpsc::channel();
        self.cache_rx = Some(rx);
        std::thread::spawn(move || {
            let result = SourceSet::build()
                .map(|s| s.repos)
                .map_err(|e| e.to_string());
            tx.send(result).ok();
        });
    }

    fn refresh_cache(&mut self) {
        let s = locale::strings();
        let (tx, rx) = mpsc::channel();
        self.cache_rx = Some(rx);
        self.loading = true;
        self.show_message(s.cache_updating, MessageLevel::Info);
        std::thread::spawn(move || {
            let result = SourceSet::build()
                .map(|s| s.repos)
                .map_err(|e| e.to_string());
            tx.send(result).ok();
        });
    }

    fn handle_cache_tick(&mut self) {
        let rx = match self.cache_rx.take() {
            Some(rx) => rx,
            None => return,
        };
        match rx.try_recv() {
            Ok(Ok(repos)) => {
                let s = locale::strings();
                self.available = repos;
                self.loading = false;
                if matches!(self.view, View::Browse) {
                    self.apply_filter();
                }
                self.show_message(
                    &(s.cache_loaded)(self.available.len()),
                    MessageLevel::Success,
                );
            }
            Ok(Err(e)) => {
                let s = locale::strings();
                self.loading = false;
                self.show_message(&(s.cache_error)(&e), MessageLevel::Error);
            }
            Err(mpsc::TryRecvError::Empty) => {
                self.cache_rx = Some(rx); // not ready yet
            }
            Err(mpsc::TryRecvError::Disconnected) => {
                self.loading = false;
            }
        }
    }

    /// Receive the result of background /var/db/pkg/ scan.
    fn handle_packages_tick(&mut self) {
        let rx = match self.packages_rx.take() {
            Some(rx) => rx,
            None => return,
        };
        match rx.try_recv() {
            Ok(map) => {
                self.packages = map;
                self.packages_ready = true;
            }
            Err(mpsc::TryRecvError::Empty) => {
                self.packages_rx = Some(rx);
            }
            Err(mpsc::TryRecvError::Disconnected) => {
                self.packages_ready = true;
            }
        }
    }

    /// Apply search filter to the package list.
    fn apply_pkg_filter(&mut self) {
        let query = self.search_query.to_lowercase();
        if query.is_empty() {
            self.pkg_list = self.pkg_list_full.clone();
        } else {
            self.pkg_list = self
                .pkg_list_full
                .iter()
                .filter(|pkg| pkg.to_lowercase().contains(&query))
                .cloned()
                .collect();
        }
        if self.pkg_selected >= self.pkg_list.len() {
            self.pkg_selected = self.pkg_list.len().saturating_sub(1);
        }
        self.load_pkg_description();
    }

    fn load_pkg_description(&mut self) {
        if self.pkg_list.is_empty() || self.pkg_repo_path.as_os_str().is_empty() {
            self.pkg_description = String::new();
            self.pkg_use_flags = String::new();
            return;
        }
        let pkg = match self.pkg_list.get(self.pkg_selected) {
            Some(p) => p,
            None => {
                self.pkg_description = String::new();
                self.pkg_use_flags = String::new();
                return;
            }
        };
        self.pkg_description = crate::core::packages::read_description(&self.pkg_repo_path, pkg);
        self.pkg_use_flags = crate::core::packages::read_use_flags(&self.pkg_repo_path, pkg);
    }

    // ── sync ──

    fn handle_sync_tick(&mut self) {
        let rx = match self.sync_rx.take() {
            Some(rx) => rx,
            None => return,
        };

        while let Ok(event) = rx.try_recv() {
            let s = locale::strings();
            match event {
                SyncEvent::Started(name) => {
                    self.sync_output.push((s.sync_start)(&name));
                }
                SyncEvent::Output(line) => {
                    self.sync_output.push(line);
                }
                SyncEvent::Finished(result) => {
                    match result {
                        Ok(()) => {
                            self.sync_output.push(s.sync_done.to_string());
                            self.show_message(s.sync_finished, MessageLevel::Success);
                        }
                        Err(e) => {
                            self.sync_output.push(format!("{} {}", s.sync_error, e));
                            self.show_message(
                                &(s.sync_failed)(&e),
                                MessageLevel::Error,
                            );
                        }
                    }
                    self.view = View::Browse;
                    return;
                }
            }
        }

        self.sync_rx = Some(rx);
    }

    // ── filter ──

    fn apply_filter(&mut self) {
        let list: Vec<String> = match self.view {
            View::Browse => self.available.iter().map(|r| r.name.clone()).collect(),
            View::Installed => self.installed.iter().map(|r| r.name.clone()).collect(),
            _ => return,
        };

        let query = self.search_query.to_lowercase();
        let mut indices: Vec<usize> = if query.is_empty() {
            (0..list.len()).collect()
        } else {
            list.iter()
                .enumerate()
                .filter(|(_, name)| name.to_lowercase().contains(&query))
                .map(|(i, _)| i)
                .collect()
        };

        indices.sort_by_key(|&i| {
            let name = &list[i].to_lowercase();
            if name == &query {
                0
            } else if name.starts_with(&query) {
                1
            } else {
                2
            }
        });

        self.filtered = indices;
        if self.selected >= self.filtered.len() {
            self.selected = self.filtered.len().saturating_sub(1);
        }
    }

    fn expire_messages(&mut self) {
        if let Some(ref msg) = self.message {
            if Instant::now() >= msg.expires_at {
                self.message = None;
            }
        }
    }
}

/// Check if the repos.conf directory is writable.
fn is_writable() -> bool {
    let dir = std::path::Path::new("/etc/portage/repos.conf");
    if !dir.exists() {
        return false;
    }
    let test = dir.join(".overlay-manager-test");
    match std::fs::write(&test, b"test") {
        Ok(()) => {
            std::fs::remove_file(&test).ok();
            true
        }
        Err(_) => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::types::{RemoteOrigin, RemoteRepo, SyncType};

    fn make_remote(name: &str) -> RemoteRepo {
        RemoteRepo {
            name: name.into(),
            description: "".into(),
            homepage: "".into(),
            owner: "".into(),
            sources: vec![(SyncType::Git, "https://example.com".into())],
            quality: "".into(),
            status: "".into(),
            origin: RemoteOrigin::GentooRegistry,
        }
    }

    fn test_app() -> App {
        let available = vec![
            make_remote("alpha"),
            make_remote("beta"),
            make_remote("gamma"),
            make_remote("alphabet"),
        ];
        let filtered = (0..available.len()).collect();
        App {
            view: View::Browse,
            previous_view: View::Browse,
            selected: 0,
            scroll_offset: 0,
            filtered,
            installed: Vec::new(),
            available,
            packages: HashMap::new(),
            search_query: String::new(),
            search_mode: false,
            sync_rx: None,
            sync_output: Vec::new(),
            sync_repo: None,
            message: None,
            loading: false,
            packages_ready: false,
            cache_rx: None,
            packages_rx: None,
            is_root: true,
            confirm: None,
            spinner_frame: 0,
            pkg_repo: String::new(),
            pkg_list: Vec::new(),
            pkg_list_full: Vec::new(),
            pkg_selected: 0,
            pkg_description: String::new(),
            pkg_use_flags: String::new(),
            pkg_repo_path: PathBuf::new(),
        }
    }

    #[test]
    fn filter_no_query_shows_all() {
        let mut app = test_app();
        app.apply_filter();
        assert_eq!(app.filtered.len(), 4);
    }

    #[test]
    fn filter_exact_match_first() {
        let mut app = test_app();
        app.search_query = "beta".into();
        app.apply_filter();
        assert_eq!(app.filtered.len(), 1);
        assert_eq!(app.available[app.filtered[0]].name, "beta");
    }

    #[test]
    fn filter_prefix_matches() {
        let mut app = test_app();
        app.search_query = "alp".into();
        app.apply_filter();
        assert_eq!(app.filtered.len(), 2);
    }

    #[test]
    fn filter_no_match() {
        let mut app = test_app();
        app.search_query = "xyz".into();
        app.apply_filter();
        assert_eq!(app.filtered.len(), 0);
    }

    #[test]
    fn filter_handles_empty_list() {
        let mut app = test_app();
        app.available.clear();
        app.filtered.clear();
        app.apply_filter();
        assert_eq!(app.filtered.len(), 0);
    }

    #[test]
    fn handle_move_down() {
        let mut app = test_app();
        app.handle(Action::MoveDown);
        assert_eq!(app.selected, 1);
        app.handle(Action::MoveDown);
        assert_eq!(app.selected, 2);
    }

    #[test]
    fn handle_move_up_clamps() {
        let mut app = test_app();
        app.handle(Action::MoveUp);
        assert_eq!(app.selected, 0);
    }

    #[test]
    fn handle_move_down_clamps() {
        let mut app = test_app();
        app.selected = 3;
        app.handle(Action::MoveDown);
        assert_eq!(app.selected, 3);
    }

    #[test]
    fn handle_move_top_bottom() {
        let mut app = test_app();
        app.handle(Action::MoveBottom);
        assert_eq!(app.selected, 3);
        app.handle(Action::MoveTop);
        assert_eq!(app.selected, 0);
    }

    #[test]
    fn handle_quit_returns_false() {
        let mut app = test_app();
        assert!(!app.handle(Action::Quit));
    }

    #[test]
    fn handle_non_quit_returns_true() {
        let mut app = test_app();
        assert!(app.handle(Action::MoveDown));
    }

    #[test]
    fn view_switch_preserves_selected() {
        let mut app = test_app();
        app.selected = 2;
        app.handle(Action::TabInstalled);
        assert_eq!(app.view, View::Installed);
        app.handle(Action::TabBrowse);
        assert_eq!(app.view, View::Browse);
    }

    #[test]
    fn help_toggles() {
        let mut app = test_app();
        app.handle(Action::ToggleHelp);
        assert_eq!(app.view, View::Help);
        app.handle(Action::ToggleHelp);
        assert_eq!(app.view, View::Browse);
    }
}
