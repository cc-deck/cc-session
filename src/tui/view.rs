use chrono::Utc;
use chrono_humanize::{Accuracy, HumanTime, Tense};
use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, Paragraph};

use super::{App, Mode};

/// Render the full TUI frame.
pub fn render(frame: &mut Frame, app: &App) {
    let area = frame.area();

    if app.mode == Mode::Detail {
        // Detail view has its own layout (no shared status bar)
        render_detail(frame, app, area);
        return;
    }

    // Layout: main area + bottom status bar (1 line)
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
        } else {
            Style::default().fg(Color::Reset)
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

    // Layout: bordered prompt list + button area (4 lines)
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(1), Constraint::Length(4)])
        .split(area);

    let list_area = chunks[0];
    let button_area = chunks[1];

    // -- Bordered prompt list --
    let inner_width = list_area.width.saturating_sub(4) as usize; // 2 for border + 2 padding
    let inner_height = list_area.height.saturating_sub(2) as usize; // 2 for top/bottom border

    let mut lines: Vec<Line> = Vec::new();

    if detail.prompts.is_empty() {
        lines.push(Line::from(Span::styled(
            " No user prompts found in this session.",
            Style::default().fg(Color::DarkGray),
        )));
    } else {
        let start = detail.scroll_offset;
        let end = (start + inner_height).min(detail.prompts.len());

        for i in start..end {
            let prompt = &detail.prompts[i];

            let delta = Utc::now().signed_duration_since(prompt.timestamp);
            let time_ago = HumanTime::from(-delta).to_text_en(Accuracy::Rough, Tense::Past);

            let max_text_len = inner_width.saturating_sub(18);
            let text = truncate_str(&prompt.text, max_text_len);

            let line = Line::from(vec![
                Span::styled(
                    format!(" {:>14}  ", time_ago),
                    Style::default().fg(Color::DarkGray),
                ),
                Span::styled(text, Style::default().fg(Color::Reset)),
            ]);

            lines.push(line);
        }
    }

    let branch = session.git_branch.as_deref().unwrap_or("");
    let title = if branch.is_empty() {
        format!(
            " {} ({} prompts) ",
            session.project_name,
            detail.prompts.len()
        )
    } else {
        format!(
            " {} · {} ({} prompts) ",
            session.project_name,
            branch,
            detail.prompts.len()
        )
    };

    let prompt_block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::DarkGray))
        .title(title)
        .title_style(Style::default().fg(Color::Green).bold());

    let prompt_list = Paragraph::new(Text::from(lines)).block(prompt_block);
    frame.render_widget(prompt_list, list_area);

    // -- Buttons --
    let dim = Style::default().fg(Color::DarkGray);
    let button_lines = vec![
        Line::from(""),
        Line::from(vec![
            Span::styled("   Copy to clipboard & Exit", Style::default().fg(Color::Cyan).bold()),
            Span::styled("          ", Style::default()),
            Span::styled("Back", Style::default().fg(Color::Reset)),
        ]),
        Line::from(vec![
            Span::styled("   Enter", dim),
            Span::styled("                            ", Style::default()),
            Span::styled("Esc", dim),
        ]),
    ];

    let buttons = Paragraph::new(Text::from(button_lines));
    frame.render_widget(buttons, button_area);
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
                    "Ctrl-G deep search  Esc clear  Enter select",
                    Style::default().fg(Color::DarkGray),
                ),
            ])
        }
        Mode::Detail => Line::from(""), // handled by render_detail
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
