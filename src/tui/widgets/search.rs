use ratatui::{
    layout::Rect,
    style::{Color, Style},
    widgets::{Block, Paragraph},
    Frame,
};

use crate::tui::app::App;

/// Search bar (standalone widget, alternatively — embedded in list.rs).
pub fn render(frame: &mut Frame, area: Rect, app: &App) {
    let text = if app.search_mode {
        format!("Search: {}▌", app.search_query)
    } else {
        "Press / to search".into()
    };

    let style = if app.search_mode {
        Style::default().fg(Color::Yellow)
    } else {
        Style::default().fg(Color::DarkGray)
    };

    let paragraph = Paragraph::new(text)
        .block(Block::bordered().title(" Search ").border_style(style));

    frame.render_widget(paragraph, area);
}
