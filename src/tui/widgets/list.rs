use ratatui::{
    Frame,
    layout::{Constraint, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, List, ListItem, ListState, Paragraph, Wrap},
};

use crate::tui::app::App;

/// Overlay list widget with search and highlighting.
pub fn render(frame: &mut Frame, area: Rect, app: &mut App) {
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
                    let origin = app
                        .available
                        .iter()
                        .find(|r| r.name == repo.name)
                        .map(|r| r.origin)
                        .unwrap_or(crate::core::types::RemoteOrigin::Custom);
                    (repo.name.clone(), true, origin)
                }
                _ => return ListItem::new(""),
            };

            let mut style = Style::default();
            if i == app.selected {
                style = style.bg(Color::DarkGray);
            }

            let origin_char = match origin {
                crate::core::types::RemoteOrigin::GentooRegistry => "",
                crate::core::types::RemoteOrigin::Github => "",
                crate::core::types::RemoteOrigin::Custom => "",
            };

            let status = if installed {
                // Color-code by sync freshness
                let (marker, color) = match effective_view {
                    crate::tui::app::View::Browse => {
                        if let Some(inst) = app.installed.iter().find(|r| r.name == name) {
                            sync_indicator(&inst.location)
                        } else {
                            ("[✓]", Color::Green)
                        }
                    }
                    crate::tui::app::View::Installed => {
                        sync_indicator(&app.installed[idx].location)
                    }
                    _ => ("[✓]", Color::Green),
                };
                ratatui::text::Span::styled(format!(" {} ", marker), style.fg(color))
            } else {
                ratatui::text::Span::styled("    ", style)
            };

            let line = ratatui::text::Line::from(vec![
                ratatui::text::Span::styled(format!(" {} {} ", origin_char, name), style),
                status,
            ]);

            ListItem::new(line)
        })
        .collect();

    let title = if app.search_mode {
        format!(" Search: {}▌", app.search_query)
    } else {
        let display_view = effective_list_view(&app.view, &app.previous_view);
        format!(
            " {} ({})",
            match display_view {
                crate::tui::app::View::Browse => "Available",
                crate::tui::app::View::Installed => "Installed",
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
        .highlight_style(
            Style::default()
                .fg(Color::Black)
                .bg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        );

    let mut state = ListState::default()
        .with_selected(Some(app.selected))
        .with_offset(app.scroll_offset);
    frame.render_stateful_widget(list_widget, area, &mut state);
}

/// Green [✓] = < 1 day, Yellow [~] = < 7 days, Red [!] = older.
fn sync_indicator(location: &std::path::Path) -> (&'static str, Color) {
    match crate::core::utils::repo_sync_age(location) {
        Some(age) if age.as_secs() < 86400 => ("[✓]", Color::Green),
        Some(age) if age.as_secs() < 604800 => ("[~]", Color::Yellow),
        Some(_) => ("[!]", Color::Red),
        None => ("[✓]", Color::Green),
    }
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
            ListItem::new(Line::from(Span::styled(format!(" {}", pkg), style)))
        })
        .collect();

    let not_installed = app.pkg_list.len() <= 3
        && app
            .pkg_list
            .first()
            .is_some_and(|item| item == "not installed");

    let title = if app.search_mode {
        format!(
            " Search: {} {}▌ ({} {}) ",
            app.pkg_repo,
            app.search_query,
            app.pkg_list.len(),
            if app.pkg_list.is_empty() {
                "no packages"
            } else {
                ""
            }
        )
    } else if not_installed {
        // Not installed — just show the name
        format!(" Packages: {} ", app.pkg_repo)
    } else {
        format!(
            " Packages: {} ({} {}) ",
            app.pkg_repo,
            app.pkg_list.len(),
            if app.pkg_list.is_empty() {
                "no packages"
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
    let mut desc_text = if app.pkg_description.is_empty() {
        "(no description)".to_string()
    } else {
        app.pkg_description.clone()
    };

    if !app.pkg_use_flags.is_empty() {
        desc_text.push_str("\n\nUSE: ");
        desc_text.push_str(&app.pkg_use_flags);
    }

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
