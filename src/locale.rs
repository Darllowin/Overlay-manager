use std::sync::OnceLock;

static STRINGS: OnceLock<&'static Strings> = OnceLock::new();

/// All UI strings.
#[allow(dead_code)]
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

const RU: Strings = Strings {
    tab_browse: "Browse",
    tab_installed: "Installed",
    loading: "⟳ загрузка...",
    no_root: "(без root)",
    search_prefix: "Поиск",
    title_available: "Доступные",
    title_installed: "Установленные",
    origin_registry: "[R]",
    origin_github: "[G]",
    origin_custom: "[C]",
    detail_title: "Детали",
    installed_yes: "✓ установлен",
    installed_no: "не установлен",
    not_installed_hint: "Нажмите 'a' чтобы установить и увидеть пакеты.",
    desc_label: "Описание:",
    source_label: "Источник:",
    owner_label: "Владелец:",
    status_label: "Статус:",
    uri_label: "URI:",
    type_label: "Тип:",
    packages_label: "Пакеты:",
    packages_scanning: "(сканирование...)",
    packages_none: "нет пакетов",
    packages_count: |n| format!("  {} шт.", n),
    packages_more: |n| format!("  ... и ещё {}", n),
    no_data: "Нет данных",
    sync_title: "Синхронизация",
    sync_start: |name| format!("▶ Синхронизация {}...", name),
    sync_done: "✓ Готово",
    sync_error: "✗",
    help_title: "Управление",
    help_nav: "Навигация",
    help_actions: "Действия",
    help_down: "вниз",
    help_up: "вверх",
    help_top: "в начало списка",
    help_bottom: "в конец списка",
    help_tab: "переключить Browse / Installed",
    help_esc: "выйти из поиска / закрыть справку",
    help_add: "добавить + синхронизировать (или только sync если уже установлен)",
    help_rm: "удалить оверлей (конфиг + файлы)",
    help_rm_purge: "удалить оверлей (конфиг + файлы)",
    help_sync: "синхронизировать оверлей",
    help_sync_all: "синхронизировать все оверлеи (emaint sync -a)",
    help_refresh: "обновить кэш (repositories.xml + GitHub)",
    help_search: "поиск по имени",
    help_help: "эта справка",
    help_quit: "выход",
    help_enter: "показать все пакеты оверлея",
    already_installed: |name| format!("{} уже установлен", name),
    no_git_source: "Оверлей без git-источника",
    need_root: "Нужны права root",
    added: |name| format!("{} добавлен", name),
    removed: |name| format!("{} удалён", name),
    removed_with_files: |name| format!("{} удалён (с файлами)", name),
    removed_no_files: |name| format!("{} удалён (файлы не найдены)", name),
    removed_config_only: |name, err| {
        format!("{} удалён из конфига, но ошибка очистки: {}", name, err)
    },
    not_found_in_config: "Оверлей не найден в конфиге",
    cache_loaded: |n| format!("Загружено {} оверлеев", n),
    cache_error: |e| format!("Ошибка загрузки: {}", e),
    cache_updating: "Обновление кэша...",
    sync_added: |name| format!("{} добавлен, синхронизация...", name),
    sync_started: "Синхронизация запущена",
    sync_finished: "Синхронизация завершена",
    sync_failed: |e| format!("Ошибка синхронизации: {}", e),
    operation_cancelled: "Операция отменена или ошибка",
    pkexec_not_found: |e| format!("pkexec не найден: {}", e),
    footer_keys: " a:add+sync  S:sync all  d:rm  /:search  r:refresh  h:help  q:quit ",
    confirm_remove: |name| format!("Удалить {}?", name),
    confirm_remove_purge: |name| format!("Удалить {} со всеми файлами?", name),
    confirm_yes_no: "y — да, n — нет",
    no_description: "(нет описания)",
    elevation_hint: "Нужны права root. Установите doas или sudo и запустите:\n  doas overlay-manager",
};

const EN: Strings = Strings {
    tab_browse: "Browse",
    tab_installed: "Installed",
    loading: "⟳ loading...",
    no_root: "(no root)",
    search_prefix: "Search",
    title_available: "Available",
    title_installed: "Installed",
    origin_registry: "[R]",
    origin_github: "[G]",
    origin_custom: "[C]",
    detail_title: "Details",
    installed_yes: "✓ installed",
    installed_no: "not installed",
    not_installed_hint: "Press 'a' to install and see packages.",
    desc_label: "Description:",
    source_label: "Source:",
    owner_label: "Owner:",
    status_label: "Status:",
    uri_label: "URI:",
    type_label: "Type:",
    packages_label: "Packages:",
    packages_scanning: "(scanning...)",
    packages_none: "no packages",
    packages_count: |n| format!("  {} pkg(s)", n),
    packages_more: |n| format!("  ... and {} more", n),
    no_data: "No data",
    sync_title: "Sync",
    sync_start: |name| format!("▶ Syncing {}...", name),
    sync_done: "✓ Done",
    sync_error: "✗",
    help_title: "Help",
    help_nav: "Navigation",
    help_actions: "Actions",
    help_down: "down",
    help_up: "up",
    help_top: "top of list",
    help_bottom: "bottom of list",
    help_tab: "switch Browse / Installed",
    help_esc: "exit search / close help",
    help_add: "add + sync (or just sync if already installed)",
    help_rm: "remove overlay (config + files)",
    help_rm_purge: "remove overlay (config + files)",
    help_sync: "sync overlay",
    help_sync_all: "sync all overlays (emaint sync -a)",
    help_refresh: "refresh cache (repositories.xml + GitHub)",
    help_search: "search by name",
    help_help: "this help",
    help_quit: "quit",
    help_enter: "show all overlay packages",
    already_installed: |name| format!("{} already installed", name),
    no_git_source: "Overlay has no git source",
    need_root: "Root privileges required",
    added: |name| format!("{} added", name),
    removed: |name| format!("{} removed", name),
    removed_with_files: |name| format!("{} removed (with files)", name),
    removed_no_files: |name| format!("{} removed (files not found)", name),
    removed_config_only: |name, err| {
        format!("{} removed from config, purge error: {}", name, err)
    },
    not_found_in_config: "Overlay not found in config",
    cache_loaded: |n| format!("Loaded {} overlays", n),
    cache_error: |e| format!("Load error: {}", e),
    cache_updating: "Updating cache...",
    sync_added: |name| format!("{} added, syncing...", name),
    sync_started: "Sync started",
    sync_finished: "Sync complete",
    sync_failed: |e| format!("Sync error: {}", e),
    operation_cancelled: "Operation cancelled or failed",
    pkexec_not_found: |e| format!("pkexec not found: {}", e),
    footer_keys: " a:add+sync  S:sync all  d:rm  /:search  r:refresh  h:help  q:quit ",
    confirm_remove: |name| format!("Remove {}?", name),
    confirm_remove_purge: |name| format!("Remove {} with all files?", name),
    confirm_yes_no: "y — yes, n — no",
    no_description: "(no description)",
    elevation_hint: "Root privileges required. Install doas or sudo and run:\n  doas overlay-manager",
};

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
