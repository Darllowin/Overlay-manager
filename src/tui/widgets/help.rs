use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Paragraph},
    Frame,
};

use crate::locale;

/// Hotkey help.
pub fn render(frame: &mut Frame, area: Rect) {
    let s = locale::strings();
    let lines = vec![
        Line::from(Span::styled(
            format!(" {}", s.help_title),
            Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        Line::from(key("j / ↓", s.help_down)),
        Line::from(key("k / ↑", s.help_up)),
        Line::from(key("g", s.help_top)),
        Line::from(key("G", s.help_bottom)),
        Line::from(""),
        Line::from(key("Tab", s.help_tab)),
        Line::from(key("Esc", s.help_esc)),
        Line::from(""),
        Line::from(Span::styled(
            format!(" {}", s.help_actions),
            Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        Line::from(key("a", s.help_add)),
        Line::from(key("S", s.help_sync_all)),
        Line::from(key("d", s.help_rm)),
        Line::from(key("r", s.help_refresh)),
        Line::from(""),
        Line::from(key("/", s.help_search)),
        Line::from(key("Enter", s.help_enter)),
        Line::from(key("h", s.help_help)),
        Line::from(key("q", s.help_quit)),
    ];

    let paragraph = Paragraph::new(lines)
        .block(
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
