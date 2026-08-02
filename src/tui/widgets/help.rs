use ratatui::{
    Frame,
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Paragraph},
};

use crate::locale;

/// Hotkey help.
pub fn render(frame: &mut Frame, area: Rect) {
    let s = locale::strings();
    let lines = vec![
        Line::from(Span::styled(
            format!(" {}", s.help_title),
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        key("j / ↓", s.help_down),
        key("k / ↑", s.help_up),
        key("g", s.help_top),
        key("G", s.help_bottom),
        Line::from(""),
        key("Tab", s.help_tab),
        key("Esc", s.help_esc),
        Line::from(""),
        Line::from(Span::styled(
            format!(" {}", s.help_actions),
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        key("a", s.help_add),
        key("S", s.help_sync_all),
        key("d", s.help_rm),
        key("r", s.help_refresh),
        Line::from(""),
        key("/", s.help_search),
        key("Enter", s.help_enter),
        key("h", s.help_help),
        key("q", s.help_quit),
    ];

    let paragraph = Paragraph::new(lines).block(
        Block::bordered()
            .title(format!(" {} ", s.help_title))
            .border_style(Style::default().fg(Color::DarkGray)),
    );

    frame.render_widget(paragraph, area);
}

fn key(key: &str, desc: &str) -> Line<'static> {
    Line::from(vec![
        Span::styled(format!(" {:6}", key), Style::default().fg(Color::Yellow)),
        Span::raw(desc.to_string()),
    ])
}
