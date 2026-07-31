//! Localization: Russian and English.
//!
//! The language is determined from LANG/LC_ALL. If it starts with "ru",
//! Russian is used; otherwise English.

use std::sync::OnceLock;

static STRINGS: OnceLock<&'static Strings> = OnceLock::new();

/// All UI strings.
pub struct Strings {
    // Tabs
    pub tab_browse: &'static str,
    pub tab_installed: &'static str,

    // Status
    pub loading: &'static str,
    pub no_root: &'static str,

    // Search
    pub search_prefix: &'static str,

    // List
    pub title_available: &'static str,
    pub title_installed: &'static str,
    pub origin_registry: &'static str,
    pub origin_github: &'static str,
    pub origin_custom: &'static str,

    // Details
    pub detail_title: &'static str,
    pub installed_yes: &'static str,
    pub installed_no: &'static str,
    pub not_installed_hint: &'static str,
    pub desc_label: &'static str,
    pub source_label: &'static str,
    pub owner_label: &'static str,
    pub status_label: &'static str,
    pub uri_label: &'static str,
    pub type_label: &'static str,
    pub packages_label: &'static str,
    pub packages_scanning: &'static str,
    pub packages_none: &'static str,
    pub packages_count: fn(usize) -> String,
    pub packages_more: fn(usize) -> String,
    pub no_data: &'static str,

    // Synchronization
    pub sync_title: &'static str,
    pub sync_start: fn(&str) -> String,
    pub sync_done: &'static str,
    pub sync_error: &'static str,

    // Help
    pub help_title: &'static str,
    pub help_nav: &'static str,
    pub help_actions: &'static str,
    pub help_down: &'static str,
    pub help_up: &'static str,
    pub help_top: &'static str,
    pub help_bottom: &'static str,
    pub help_tab: &'static str,
    pub help_esc: &'static str,
    pub help_add: &'static str,
    pub help_rm: &'static str,
    pub help_rm_purge: &'static str,
    pub help_sync: &'static str,
    pub help_sync_all: &'static str,
    pub help_refresh: &'static str,
    pub help_search: &'static str,
    pub help_help: &'static str,
    pub help_quit: &'static str,
    pub help_enter: &'static str,

    // Messages
    pub already_installed: fn(&str) -> String,
    pub no_git_source: &'static str,
    pub need_root: &'static str,
    pub added: fn(&str) -> String,
    pub removed: fn(&str) -> String,
    pub removed_with_files: fn(&str) -> String,
    pub removed_no_files: fn(&str) -> String,
    pub removed_config_only: fn(&str, &str) -> String,
    pub not_found_in_config: &'static str,
    pub cache_loaded: fn(usize) -> String,
    pub cache_error: fn(&str) -> String,
    pub cache_updating: &'static str,
    pub sync_added: fn(&str) -> String,
    pub sync_started: &'static str,
    pub sync_finished: &'static str,
    pub sync_failed: fn(&str) -> String,
    pub operation_cancelled: &'static str,
    pub pkexec_not_found: fn(&str) -> String,

    // Footer
    pub footer_keys: &'static str,

    // Remove confirmation
    pub confirm_remove: fn(&str) -> String,
    pub confirm_remove_purge: fn(&str) -> String,
    pub confirm_yes_no: &'static str,

    pub no_description: &'static str,
    pub elevation_hint: &'static str,
}

/// Initialize locale. Call at startup.
pub fn init() {
    let lang = std::env::var("LANG")
        .unwrap_or_default()
        .to_lowercase();

    let s = if lang.starts_with("ru") { &RU } else { &EN };

    STRINGS.set(s).ok();
}

/// Get the current locale. Panics if init() has not been called.
pub fn strings() -> &'static Strings {
    STRINGS.get().expect("locale::init() not called")
}
