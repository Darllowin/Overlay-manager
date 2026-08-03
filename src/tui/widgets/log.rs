use ratatui::{
    Frame,
    layout::Rect,
    style::{Color, Style},
    widgets::{Block, Paragraph},
};

use crate::tui::app::App;

/// Sync log panel with animated spinner.
pub fn render(frame: &mut Frame, area: Rect, app: &App) {
    let spinner = ['⠋', '⠙', '⠹', '⠸', '⠼', '⠴', '⠦', '⠧', '⠇', '⠏'];
    let ch = spinner[app.spinner_frame % spinner.len()];

    let title = match &app.sync_repo {
        Some(name) => format!(" {} Sync: {} ", ch, name),
        None => format!(" {} Sync ", ch),
    };

    let text = if app.sync_output.is_empty() {
        format!("▶ Syncing {}...", app.sync_repo.as_deref().unwrap_or("")).to_string()
    } else {
        let max_lines = area.height.saturating_sub(2) as usize;
        let start = app.sync_output.len().saturating_sub(max_lines);
        app.sync_output[start..].join("\n")
    };

    let paragraph = Paragraph::new(text).block(
        Block::bordered()
            .title(title)
            .border_style(Style::default().fg(Color::Yellow)),
    );

    frame.render_widget(paragraph, area);
}
