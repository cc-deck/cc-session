use chrono::Utc;
use chrono_humanize::{Accuracy, HumanTime, Tense};
use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, Paragraph};

use super::{App, Mode};

/// Render the full TUI frame.
pub fn render(frame: &mut Frame, app: &App) {
    let area = frame.area();

    // Layout: main list area + bottom status bar (1 line)
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(1), Constraint::Length(1)])
        .split(area);

    render_session_list(frame, app, chunks[0]);
    render_status_bar(frame, app, chunks[1]);
}

/// Render the session list with two-line entries.
fn render_session_list(frame: &mut Frame, app: &App, area: Rect) {
    let width = area.width as usize;
    let height = area.height as usize;

    // Each session takes 2 lines
    let visible_items = height / 2;
    // We can't mutate app here, so the caller must have called ensure_visible

    let mut lines: Vec<Line> = Vec::new();

    let start = app.scroll_offset;
    let end = (start + visible_items).min(app.filtered_indices.len());

    for i in start..end {
        let idx = app.filtered_indices[i];
        let session = &app.sessions[idx];
        let is_selected = i == app.selected;

        // Line 1: project · branch · time
        let branch = session.git_branch.as_deref().unwrap_or("");
        let delta = Utc::now().signed_duration_since(session.timestamp);
        let time_ago = HumanTime::from(-delta).to_text_en(Accuracy::Rough, Tense::Past);

        let mut line1_spans = vec![
            Span::styled(
                if is_selected { "▸ " } else { "  " },
                Style::default().fg(Color::Cyan),
            ),
            Span::styled(
                &session.project_name,
                if session.project_exists {
                    Style::default().fg(Color::Green).bold()
                } else {
                    Style::default().fg(Color::DarkGray).bold()
                },
            ),
        ];

        if !branch.is_empty() {
            line1_spans.push(Span::raw(" · "));
            line1_spans.push(Span::styled(branch, Style::default().fg(Color::Yellow)));
        }

        line1_spans.push(Span::raw(" · "));
        line1_spans.push(Span::styled(time_ago, Style::default().fg(Color::DarkGray)));

        let line1 = Line::from(line1_spans);

        // Line 2: indented first message
        let max_msg_len = width.saturating_sub(6);
        let msg = if session.first_message.len() > max_msg_len && max_msg_len > 3 {
            format!("{}...", &session.first_message[..max_msg_len - 3])
        } else {
            session.first_message.clone()
        };

        let msg_style = if session.project_exists {
            Style::default().fg(Color::Gray)
        } else {
            Style::default().fg(Color::DarkGray)
        };

        let line2 = Line::from(vec![
            Span::raw("    "),
            Span::styled(format!("\"{msg}\""), msg_style),
        ]);

        if is_selected {
            lines.push(line1.patch_style(Style::default().bg(Color::Rgb(30, 30, 50))));
            lines.push(line2.patch_style(Style::default().bg(Color::Rgb(30, 30, 50))));
        } else {
            lines.push(line1);
            lines.push(line2);
        }
    }

    let text = Text::from(lines);
    let block = Block::default()
        .borders(Borders::NONE)
        .title(format!(
            " cc-session ({}/{}) ",
            app.filtered_indices.len(),
            app.sessions.len()
        ))
        .title_style(Style::default().fg(Color::Cyan).bold());

    let paragraph = Paragraph::new(text).block(block);
    frame.render_widget(paragraph, area);
}

/// Render the status/help bar at the bottom.
fn render_status_bar(frame: &mut Frame, app: &App, area: Rect) {
    let content = match app.mode {
        Mode::Filtering => {
            let match_info = if app.filtered_indices.is_empty() {
                " No matches".to_string()
            } else {
                format!(" {} matches", app.filtered_indices.len())
            };

            Line::from(vec![
                Span::styled(" / ", Style::default().fg(Color::Cyan).bold()),
                Span::styled(&app.filter_query, Style::default().fg(Color::White)),
                Span::styled("▎", Style::default().fg(Color::Cyan)),
                Span::styled(match_info, Style::default().fg(Color::DarkGray)),
                Span::raw("  "),
                Span::styled(
                    "Esc clear  Enter select",
                    Style::default().fg(Color::DarkGray),
                ),
            ])
        }
        Mode::Browsing => {
            if let Some((msg, _)) = &app.status_message {
                Line::from(vec![
                    Span::styled(
                        format!(" {msg}"),
                        Style::default().fg(Color::Green).bold(),
                    ),
                ])
            } else {
                Line::from(vec![
                    Span::styled(" / ", Style::default().fg(Color::DarkGray)),
                    Span::styled("filter", Style::default().fg(Color::DarkGray)),
                    Span::raw("  "),
                    Span::styled("Enter ", Style::default().fg(Color::DarkGray)),
                    Span::styled("copy", Style::default().fg(Color::DarkGray)),
                    Span::raw("  "),
                    Span::styled("q ", Style::default().fg(Color::DarkGray)),
                    Span::styled("quit", Style::default().fg(Color::DarkGray)),
                ])
            }
        }
    };

    let bar = Paragraph::new(content).style(Style::default().bg(Color::Rgb(20, 20, 30)));
    frame.render_widget(bar, area);
}
