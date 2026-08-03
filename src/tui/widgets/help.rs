use ratatui::{
    Frame,
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Paragraph},
};

/// Hotkey help.
pub fn render(frame: &mut Frame, area: Rect) {
    let lines = vec![
        Line::from(Span::styled(
            " Help".to_string(),
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        key("j / ↓", "down"),
        key("k / ↑", "up"),
        key("g", "top of list"),
        key("G", "bottom of list"),
        Line::from(""),
        key("Tab", "switch Browse / Installed"),
        key("Esc", "exit search / close help"),
        Line::from(""),
        Line::from(Span::styled(
            " Actions".to_string(),
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        key("a", "add + sync (or just sync if already installed)"),
        key("S", "sync all overlays (emaint sync -a)"),
        key("d", "remove overlay (config + files)"),
        key("r", "refresh cache (repositories.xml + GitHub)"),
        Line::from(""),
        key("/", "search by name"),
        key("Enter", "show all overlay packages"),
        key("h", "this help"),
        key("q", "quit"),
    ];

    let paragraph = Paragraph::new(lines).block(
        Block::bordered()
            .title(" Help ")
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
