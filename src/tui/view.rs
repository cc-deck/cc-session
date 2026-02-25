use chrono::Utc;
use chrono_humanize::{Accuracy, HumanTime, Tense};
use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, Paragraph};

use super::{App, Mode};

/// Render the full TUI frame.
pub fn render(frame: &mut Frame, app: &App) {
    let area = frame.area();

    // Layout: main area + bottom status bar (1 line)
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(1), Constraint::Length(1)])
        .split(area);

    match app.mode {
        Mode::Detail => render_detail(frame, app, chunks[0]),
        _ => render_session_list(frame, app, chunks[0]),
    }
    render_status_bar(frame, app, chunks[1]);
}

/// Render the session list with two-line entries.
fn render_session_list(frame: &mut Frame, app: &App, area: Rect) {
    let width = area.width as usize;
    let height = area.height as usize;

    let visible_items = height; // 1 line per session now

    let mut lines: Vec<Line> = Vec::new();

    let start = app.scroll_offset;
    let end = (start + visible_items).min(app.filtered_indices.len());

    for i in start..end {
        let idx = app.filtered_indices[i];
        let session = &app.sessions[idx];
        let is_selected = i == app.selected;

        // Right side: "project · time_ago"
        let delta = Utc::now().signed_duration_since(session.timestamp);
        let time_ago = HumanTime::from(-delta).to_text_en(Accuracy::Rough, Tense::Past);
        let right = format!("{}  {}", session.project_name, time_ago);
        let right_len = right.len();

        // Left side: cursor + prompt text, truncated to fit
        let cursor = if is_selected { "▸ " } else { "  " };
        let cursor_len = 2;
        // 2 chars gap between text and right-aligned info
        let max_msg_len = width.saturating_sub(cursor_len + right_len + 2);
        let msg = truncate_str(&session.first_message, max_msg_len);
        let msg_len = msg.chars().count();

        // Padding to push right side to the edge
        let pad = width.saturating_sub(cursor_len + msg_len + right_len);
        let padding = " ".repeat(pad);

        let msg_style = if is_selected {
            Style::default().fg(Color::White)
        } else if session.project_exists {
            Style::default().fg(Color::Reset)
        } else {
            Style::default().fg(Color::DarkGray)
        };

        let dim = Style::default().fg(Color::DarkGray);

        let line = Line::from(vec![
            Span::styled(cursor, Style::default().fg(Color::Cyan)),
            Span::styled(msg, msg_style),
            Span::raw(padding),
            Span::styled(right, dim),
        ]);

        if is_selected {
            lines.push(line.patch_style(Style::default().bg(Color::Rgb(30, 30, 50))));
        } else {
            lines.push(line);
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

/// Render the detail view showing user prompts for a session.
fn render_detail(frame: &mut Frame, app: &App, area: Rect) {
    let detail = match &app.detail {
        Some(d) => d,
        None => return,
    };

    let session = &app.sessions[detail.session_idx];
    let width = area.width as usize;
    let height = area.height.saturating_sub(1) as usize; // -1 for title line

    let mut lines: Vec<Line> = Vec::new();

    if detail.prompts.is_empty() {
        lines.push(Line::from(Span::styled(
            "  No user prompts found in this session.",
            Style::default().fg(Color::DarkGray),
        )));
    } else {
        let start = detail.scroll_offset;
        let end = (start + height).min(detail.prompts.len());

        for i in start..end {
            let prompt = &detail.prompts[i];
            let is_selected = i == detail.selected;

            let delta = Utc::now().signed_duration_since(prompt.timestamp);
            let time_ago = HumanTime::from(-delta).to_text_en(Accuracy::Rough, Tense::Past);

            let max_text_len = width.saturating_sub(4);
            let text = truncate_str(&prompt.text, max_text_len);

            let cursor = if is_selected { "▸ " } else { "  " };

            let line = Line::from(vec![
                Span::styled(cursor, Style::default().fg(Color::Cyan)),
                Span::styled(
                    format!("{:>14}", time_ago),
                    Style::default().fg(Color::DarkGray),
                ),
                Span::styled("  ", Style::default()),
                Span::styled(
                    text,
                    if is_selected {
                        Style::default().fg(Color::White)
                    } else {
                        Style::default().fg(Color::Gray)
                    },
                ),
            ]);

            if is_selected {
                lines.push(line.patch_style(Style::default().bg(Color::Rgb(30, 30, 50))));
            } else {
                lines.push(line);
            }
        }
    }

    let branch = session.git_branch.as_deref().unwrap_or("");
    let title = if branch.is_empty() {
        format!(" {} - {} prompts ", session.project_name, detail.prompts.len())
    } else {
        format!(
            " {} · {} - {} prompts ",
            session.project_name,
            branch,
            detail.prompts.len()
        )
    };

    let text = Text::from(lines);
    let block = Block::default()
        .borders(Borders::NONE)
        .title(title)
        .title_style(Style::default().fg(Color::Green).bold());

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
        Mode::Detail => {
            if let Some((msg, _)) = &app.status_message {
                Line::from(vec![Span::styled(
                    format!(" {msg}"),
                    Style::default().fg(Color::Green).bold(),
                )])
            } else {
                Line::from(vec![
                    Span::styled(" Enter ", Style::default().fg(Color::DarkGray)),
                    Span::styled("copy", Style::default().fg(Color::DarkGray)),
                    Span::raw("  "),
                    Span::styled("Esc ", Style::default().fg(Color::DarkGray)),
                    Span::styled("back", Style::default().fg(Color::DarkGray)),
                ])
            }
        }
        Mode::Browsing => {
            if let Some((msg, _)) = &app.status_message {
                Line::from(vec![Span::styled(
                    format!(" {msg}"),
                    Style::default().fg(Color::Green).bold(),
                )])
            } else {
                Line::from(vec![
                    Span::styled(" / ", Style::default().fg(Color::DarkGray)),
                    Span::styled("filter", Style::default().fg(Color::DarkGray)),
                    Span::raw("  "),
                    Span::styled("Enter ", Style::default().fg(Color::DarkGray)),
                    Span::styled("detail", Style::default().fg(Color::DarkGray)),
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

/// Truncate a string to `max_len` characters, adding "..." if truncated.
fn truncate_str(s: &str, max_len: usize) -> String {
    if max_len <= 3 {
        return s.chars().take(max_len).collect();
    }
    let chars: Vec<char> = s.chars().collect();
    if chars.len() > max_len {
        format!("{}...", chars[..max_len - 3].iter().collect::<String>())
    } else {
        s.to_string()
    }
}
