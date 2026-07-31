/// Actions the user can initiate.
///
/// Intermediate layer between key presses and application state updates.
/// Simplifies testing and allows remapping keys without changing logic.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Action {
    MoveUp,
    MoveDown,
    MoveTop,
    MoveBottom,

    /// Open the full package list of the selected overlay.
    OpenPackages,
    /// Close panel / exit search / close Help.
    Close,

    /// Add overlay + sync (or just sync if already installed).
    AddOrSync,
    /// Sync all installed overlays (emaint sync -a).
    SyncAll,
    /// Remove overlay from repos.conf and delete files.
    RemoveRepo,

    /// Switch to Browse tab.
    TabBrowse,
    /// Switch to Installed tab.
    TabInstalled,

    /// Enter search mode.
    SearchStart,
    /// Character in the search string.
    SearchChar(char),
    /// Remove last character from search string.
    SearchBackspace,

    /// Refresh remote overlays cache.
    RefreshCache,
    /// Show/hide help.
    ToggleHelp,
    /// Confirm operation (y).
    ConfirmYes,

    /// Exit the application.
    Quit,

    /// Do nothing.
    Noop,
}
