mod core;
mod locale;
mod tui;

use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use std::process::Command;
use std::time::Duration;
use tui::actions::Action;
use tui::app::App;

fn main() -> anyhow::Result<()> {
    run_tui()
}

fn run_tui() -> anyhow::Result<()> {
    locale::init();

    if !is_root() {
        return elevate();
    }

    let mut terminal = ratatui::init();

    let mut app = App::new();
    app.start_background_cache();

    loop {
        terminal.draw(|frame| tui::ui::render(frame, &mut app))?;

        if event::poll(Duration::from_millis(50))? {
            match event::read()? {
                Event::Key(key) if key.kind != KeyEventKind::Release => {
                    let action = map_key(key.code, &app);
                    if !app.handle(action) {
                        break;
                    }
                }
                Event::Resize(_, _) => {}
                _ => {}
            }
        }

        app.tick();
    }

    ratatui::restore();
    Ok(())
}

fn map_key(code: KeyCode, app: &App) -> Action {
    if app.search_mode {
        return match code {
            KeyCode::Esc => Action::Close,
            KeyCode::Backspace => Action::SearchBackspace,
            KeyCode::Down | KeyCode::Char('j') => Action::MoveDown,
            KeyCode::Up | KeyCode::Char('k') => Action::MoveUp,
            KeyCode::Enter => Action::Close,
            KeyCode::Char(c) => Action::SearchChar(c),
            _ => Action::Noop,
        };
    }

    // Confirmation mode
    if app.view == tui::app::View::Confirm {
        return match code {
            KeyCode::Char('y') | KeyCode::Char('Y') => Action::ConfirmYes,
            KeyCode::Char('n') | KeyCode::Char('N') | KeyCode::Esc => Action::Close,
            _ => Action::Noop,
        };
    }

    match code {
        KeyCode::Char('q') => Action::Quit,
        KeyCode::Char('j') | KeyCode::Down => Action::MoveDown,
        KeyCode::Char('k') | KeyCode::Up => Action::MoveUp,
        KeyCode::Char('g') => Action::MoveTop,
        KeyCode::Char('G') => Action::MoveBottom,
        KeyCode::Enter => Action::OpenPackages,
        KeyCode::Esc => Action::Close,
        KeyCode::Char('a') => Action::AddOrSync,
        KeyCode::Char('S') => Action::SyncAll,
        KeyCode::Char('d') => Action::RemoveRepo,
        KeyCode::Char('r') => Action::RefreshCache,
        KeyCode::Tab => match app.view {
            tui::app::View::Browse => Action::TabInstalled,
            _ => Action::TabBrowse,
        },
        KeyCode::Char('/') => Action::SearchStart,
        KeyCode::Char('h') => Action::ToggleHelp,
        _ => Action::Noop,
    }
}

fn is_root() -> bool {
    let test = std::path::Path::new("/etc/portage/repos.conf").join(".om-test");
    match std::fs::write(&test, b"x") {
        Ok(()) => {
            std::fs::remove_file(&test).ok();
            true
        }
        Err(_) => false,
    }
}

fn elevate() -> anyhow::Result<()> {
    let exe =
        std::env::current_exe().unwrap_or_else(|_| std::path::PathBuf::from("overlay-manager"));

    for tool in &["doas", "sudo"] {
        if Command::new(tool).arg("--version").output().is_ok() {
            return Err(Command::new(tool)
                .arg(&exe)
                .status()
                .map(|_| anyhow::anyhow!(""))
                .unwrap_or_else(|e| anyhow::anyhow!("{}", e)));
        }
    }

    anyhow::bail!("{}", locale::strings().elevation_hint)
}
