use ratatui::{
    layout::{Constraint, Layout, Rect},
    Frame,
};

use super::app::App;
use super::widgets::{detail, help, list, log};
use crate::locale;

/// Render the entire interface.
pub fn render(frame: &mut Frame, app: &mut App) {
    let area = frame.area();

    // Main vertical layout: header, body, footer
    let [header_area, body_area, footer_area] = Layout::vertical([
        Constraint::Length(1),
        Constraint::Fill(1),
        Constraint::Length(1),
    ])
    .areas(area);

    render_header(frame, header_area, app);

    match app.view {
        super::app::View::Syncing => {
            log::render(frame, body_area, app);
        }
        super::app::View::Help => {
            help::render(frame, body_area);
        }
        super::app::View::Confirm => {
            render_body(frame, body_area, app);
            render_confirm_popup(frame, app);
        }
        super::app::View::Packages(_) => {
            list::render_packages(frame, body_area, app);
        }
        _ => {
            render_body(frame, body_area, app);
        }
    }

    render_footer(frame, footer_area, app);
}

/// Status bar: tabs + cache status.
fn render_header(frame: &mut Frame, area: Rect, app: &App) {
    use ratatui::{
        style::{Color, Modifier, Style},
        text::Line,
        widgets::Paragraph,
    };

    let tab_style = |active: bool| -> Style {
        if active {
            Style::default()
                .fg(Color::Black)
                .bg(Color::Cyan)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::Gray)
        }
    };

    let s = locale::strings();
    let spinner_chars = ['⠋', '⠙', '⠹', '⠸', '⠼', '⠴', '⠦', '⠧', '⠇', '⠏'];
    let cache_status = if app.loading {
        let ch = spinner_chars[app.spinner_frame % spinner_chars.len()];
        format!(" {} {}", ch, s.loading)
    } else if !app.is_root {
        s.no_root.to_string()
    } else {
        let age = crate::core::utils::cache_age();
        if age == "never" {
            String::new()
        } else {
            format!(" cache: {}", age)
        }
    };

    let text = Line::from(vec![
        ratatui::text::Span::styled(
            format!(" {} ", s.tab_browse),
            tab_style(matches!(app.view, super::app::View::Browse)),
        ),
        ratatui::text::Span::styled(
            format!(" {} ", s.tab_installed),
            tab_style(matches!(app.view, super::app::View::Installed)),
        ),
        ratatui::text::Span::styled(
            format!(" overlay-manager{}", cache_status),
            Style::default().fg(Color::DarkGray),
        ),
    ]);

    frame.render_widget(Paragraph::new(text), area);
}

/// Main section: list + detail panel.
fn render_body(frame: &mut Frame, area: Rect, app: &mut App) {
    let [list_area, detail_area] =
        Layout::horizontal([Constraint::Percentage(55), Constraint::Percentage(45)]).areas(area);

    list::render(frame, list_area, app);
    detail::render(frame, detail_area, app);
}

/// Help bar + messages.
fn render_footer(frame: &mut Frame, area: Rect, app: &App) {
    use ratatui::{
        style::{Color, Style},
        text::{Line, Span},
        widgets::Paragraph,
    };

    if let Some(msg) = &app.message {
        let color = match msg.level {
            super::app::MessageLevel::Info => Color::Cyan,
            super::app::MessageLevel::Success => Color::Green,
            super::app::MessageLevel::Error => Color::Red,
        };
        let text = Line::from(Span::styled(&msg.text, Style::default().fg(color)));
        frame.render_widget(Paragraph::new(text), area);
    } else {
        let text = Line::from(Span::styled(
            locale::strings().footer_keys,
            Style::default().fg(Color::DarkGray),
        ));
        frame.render_widget(Paragraph::new(text), area);
    }
}

/// Removal confirmation popup.
fn render_confirm_popup(frame: &mut Frame, app: &App) {
    use ratatui::{
        style::{Color, Style},
        text::{Line, Span},
        widgets::{Block, Clear, Paragraph},
    };

    let s = locale::strings();
    let confirm = match &app.confirm {
        Some(c) => c,
        None => return,
    };

    let msg = (s.confirm_remove)(&confirm.repo_name);

    let lines = vec![
        Line::from(Span::styled(
            msg,
            Style::default().fg(Color::Yellow),
        )),
        Line::from(""),
        Line::from(Span::styled(
            s.confirm_yes_no,
            Style::default().fg(Color::DarkGray),
        )),
    ];

    let area = frame.area();
    let popup_width = 50u16;
    let popup_height = 5u16;
    let x = (area.width.saturating_sub(popup_width)) / 2;
    let y = (area.height.saturating_sub(popup_height)) / 2;
    let popup_area = ratatui::layout::Rect::new(x, y, popup_width, popup_height);

    let paragraph = Paragraph::new(lines)
        .block(
            Block::bordered()
                .title(" ? ")
                .border_style(Style::default().fg(Color::Red)),
        )
        .centered();

    frame.render_widget(Clear, popup_area);
    frame.render_widget(paragraph, popup_area);
}
