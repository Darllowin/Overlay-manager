use ratatui::{
    Frame,
    layout::Rect,
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Paragraph, Wrap},
};
use std::path::Path;

use crate::tui::app::App;

/// Detail panel — always shows info about the selected item.
pub fn render(frame: &mut Frame, area: Rect, app: &App) {
    let mut lines: Vec<Line> = Vec::new();

    match &app.view {
        crate::tui::app::View::Browse => {
            if let Some(repo) = app.selected_remote() {
                let installed = app.installed.iter().any(|r| r.name == repo.name);
                let status = if installed {
                    Span::styled(
                        " ✓ installed ".to_string(),
                        Style::default().fg(Color::Green),
                    )
                } else {
                    Span::styled(
                        " not installed ".to_string(),
                        Style::default().fg(Color::Yellow),
                    )
                };
                lines.push(Line::from(vec![
                    Span::styled(&repo.name, Style::default().fg(Color::Cyan)),
                    status,
                ]));
                lines.push(Line::from(""));
                if !repo.description.is_empty() {
                    lines.push(Line::from(Span::styled(
                        "Description:",
                        Style::default().fg(Color::Gray),
                    )));
                    lines.push(Line::from(repo.description.clone()));
                    lines.push(Line::from(""));
                }
                lines.push(Line::from(Span::styled(
                    "Source:",
                    Style::default().fg(Color::Gray),
                )));
                lines.push(Line::from(repo.homepage.clone()));
                lines.push(Line::from(""));
                lines.push(Line::from(Span::styled(
                    "Owner:",
                    Style::default().fg(Color::Gray),
                )));
                lines.push(Line::from(repo.owner.clone()));
                lines.push(Line::from(""));
                lines.push(Line::from(Span::styled(
                    format!("Status: {} / {}", repo.status, repo.quality),
                    Style::default().fg(Color::DarkGray),
                )));

                add_packages_section(&mut lines, app, &repo.name);

                if installed
                    && let Some(inst) = app.installed.iter().find(|r| r.name == repo.name) {
                        add_repo_info(&mut lines, &inst.location);
                    }
            } else {
                lines.push(Line::from("No data"));
            }
        }
        crate::tui::app::View::Installed => {
            if let Some(repo) = app.selected_installed() {
                lines.push(Line::from(Span::styled(
                    &repo.name,
                    Style::default().fg(Color::Cyan),
                )));
                lines.push(Line::from(""));
                lines.push(Line::from(Span::styled(
                    "URI:",
                    Style::default().fg(Color::Gray),
                )));
                lines.push(Line::from(repo.sync_uri.clone()));
                lines.push(Line::from(""));
                lines.push(Line::from(Span::styled(
                    format!(
                        "Type: {}  Status: {}",
                        repo.sync_type.as_str(),
                        repo.priority.unwrap_or(50)
                    ),
                    Style::default().fg(Color::DarkGray),
                )));

                add_packages_section(&mut lines, app, &repo.name);
                add_repo_info(&mut lines, &repo.location);
            } else {
                lines.push(Line::from("No data"));
            }
        }
        _ => {
            lines.push(Line::from(""));
        }
    }

    let paragraph = Paragraph::new(lines)
        .block(
            Block::bordered()
                .title(" Details ")
                .border_style(Style::default().fg(Color::DarkGray)),
        )
        .wrap(Wrap { trim: false });

    frame.render_widget(paragraph, area);
}

fn add_repo_info(lines: &mut Vec<Line>, location: &Path) {
    use crate::core::utils;

    let size = utils::repo_disk_usage(location);
    let sync = utils::repo_last_sync(location);

    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled(
        format!("Size: {}  Synced: {}", size, sync),
        Style::default().fg(Color::DarkGray),
    )));
}

fn add_packages_section(lines: &mut Vec<Line>, app: &App, repo_name: &str) {
    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled(
        "Packages:",
        Style::default().fg(Color::Gray),
    )));

    if !app.packages_ready {
        lines.push(Line::from(Span::styled(
            "  (scanning...)".to_string(),
            Style::default().fg(Color::DarkGray),
        )));
        return;
    }

    match app.packages.get(repo_name) {
        Some(list) if !list.is_empty() => {
            lines.push(Line::from(Span::styled(
                format!("  {} pkg(s)", list.len()),
                Style::default().fg(Color::DarkGray),
            )));

            for pkg in list.iter().take(20) {
                lines.push(Line::from(Span::styled(
                    format!("  {}", pkg),
                    Style::default(),
                )));
            }
            if list.len() > 20 {
                lines.push(Line::from(Span::styled(
                    format!("  ... and {} more", list.len() - 20),
                    Style::default().fg(Color::DarkGray),
                )));
            }
        }
        _ => {
            lines.push(Line::from(Span::styled(
                "  no packages".to_string(),
                Style::default().fg(Color::DarkGray),
            )));
        }
    }
}
