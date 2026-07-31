use ratatui::{
    layout::{Constraint, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, List, ListItem, ListState, Paragraph, Wrap},
    Frame,
};

use crate::locale;
use crate::tui::app::App;

/// Overlay list widget with search and highlighting.
pub fn render(frame: &mut Frame, area: Rect, app: &mut App) {
    let s = locale::strings();
    let visible = area.height.saturating_sub(2) as usize;
    app.clamp_scroll(visible);

    let list = app.current_list().to_vec();

    let items: Vec<ListItem> = list
        .iter()
        .enumerate()
        .map(|(i, &idx)| {
            let effective_view = effective_list_view(&app.view, &app.previous_view);

            let (name, installed, origin) = match effective_view {
                crate::tui::app::View::Browse => {
                    let repo = &app.available[idx];
                    let installed = app.installed.iter().any(|r| r.name == repo.name);
                    (repo.name.clone(), installed, repo.origin)
                }
                crate::tui::app::View::Installed => {
                    let repo = &app.installed[idx];
                    (repo.name.clone(), true, crate::core::types::RemoteOrigin::Custom)
                }
                _ => return ListItem::new(""),
            };

            let mut style = Style::default();
            if i == app.selected {
                style = style.bg(Color::DarkGray);
            }

            let origin_char = match origin {
                crate::core::types::RemoteOrigin::GentooRegistry => s.origin_registry,
                crate::core::types::RemoteOrigin::Github => s.origin_github,
                crate::core::types::RemoteOrigin::Custom => s.origin_custom,
            };

            let status = if installed { "[✓]" } else { "   " };

            let line = ratatui::text::Line::from(vec![
                ratatui::text::Span::styled(
                    format!(" {} {} {} ", origin_char, status, name),
                    style,
                ),
            ]);

            ListItem::new(line)
        })
        .collect();

    let title = if app.search_mode {
        format!(" {}: {}▌", s.search_prefix, app.search_query)
    } else {
        let display_view = effective_list_view(&app.view, &app.previous_view);
        format!(
            " {} ({})",
            match display_view {
                crate::tui::app::View::Browse => s.title_available,
                crate::tui::app::View::Installed => s.title_installed,
                _ => "",
            },
            list.len()
        )
    };

    let list_widget = List::new(items)
        .block(
            ratatui::widgets::Block::bordered()
                .title(title)
                .border_style(Style::default().fg(Color::DarkGray)),
        )
        .highlight_style(Style::default().fg(Color::Black).bg(Color::Cyan).add_modifier(Modifier::BOLD));

    let mut state = ListState::default()
        .with_selected(Some(app.selected))
        .with_offset(app.scroll_offset);
    frame.render_stateful_widget(list_widget, area, &mut state);
}

/// If the current view is Help or Confirm, show list from previous_view.
fn effective_list_view<'a>(
    view: &'a crate::tui::app::View,
    previous: &'a crate::tui::app::View,
) -> &'a crate::tui::app::View {
    match view {
        crate::tui::app::View::Help | crate::tui::app::View::Confirm => previous,
        other => other,
    }
}

/// Full-screen overlay package list.
pub fn render_packages(frame: &mut Frame, area: Rect, app: &mut App) {
    let s = crate::locale::strings();

    let items: Vec<ListItem> = app
        .pkg_list
        .iter()
        .enumerate()
        .map(|(i, pkg)| {
            let style = if i == app.pkg_selected {
                Style::default().bg(Color::DarkGray)
            } else {
                Style::default()
            };
            ListItem::new(Line::from(Span::styled(
                format!(" {}", pkg),
                style,
            )))
        })
        .collect();

    let not_installed = app.pkg_list.len() <= 3
        && app
            .pkg_list
            .first()
            .map_or(false, |item| item == s.installed_no);

    let title = if app.search_mode {
        format!(
            " {}: {} {}▌ ({} {}) ",
            s.search_prefix,
            app.pkg_repo,
            app.search_query,
            app.pkg_list.len(),
            if app.pkg_list.is_empty() {
                s.packages_none
            } else {
                ""
            }
        )
    } else if not_installed {
        // Not installed — just show the name
        format!(" {}: {} ", s.packages_label, app.pkg_repo)
    } else {
        format!(
            " {}: {} ({} {}) ",
            s.packages_label,
            app.pkg_repo,
            app.pkg_list.len(),
            if app.pkg_list.is_empty() {
                s.packages_none
            } else {
                ""
            }
        )
    };

    let list_widget = List::new(items)
        .block(
            Block::bordered()
                .title(title)
                .border_style(Style::default().fg(Color::DarkGray)),
        )
        .highlight_style(
            Style::default()
                .fg(Color::Black)
                .bg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        );

    let [list_area, desc_area] =
        Layout::horizontal([Constraint::Percentage(50), Constraint::Percentage(50)]).areas(area);

    let mut state = ListState::default().with_selected(Some(app.pkg_selected));
    frame.render_stateful_widget(list_widget, list_area, &mut state);

    // Right panel: package description
    let desc_text = if app.pkg_description.is_empty() {
        crate::locale::strings().no_description.to_string()
    } else {
        app.pkg_description.clone()
    };

    let desc_title = app
        .pkg_list
        .get(app.pkg_selected)
        .map(|p| format!(" {} ", p))
        .unwrap_or_else(|| " — ".into());

    let desc = Paragraph::new(desc_text)
        .block(
            Block::bordered()
                .title(desc_title)
                .border_style(Style::default().fg(Color::DarkGray)),
        )
        .wrap(Wrap { trim: false });

    frame.render_widget(desc, desc_area);
}
