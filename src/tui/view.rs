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
    use super::DetailButton;

    let detail = match &app.detail {
        Some(d) => d,
        None => return,
    };

    let session = &app.sessions[detail.session_idx];
    let inner_width = area.width.saturating_sub(4) as usize;

    // Build prompt lines
    let mut prompt_lines: Vec<Line> = Vec::new();

    if detail.prompts.is_empty() {
        prompt_lines.push(Line::from(Span::styled(
            " No user prompts found in this session.",
            Style::default().fg(Color::DarkGray),
        )));
    } else {
        for prompt in &detail.prompts {
            let delta = Utc::now().signed_duration_since(prompt.timestamp);
            let time_ago = HumanTime::from(-delta).to_text_en(Accuracy::Rough, Tense::Past);

            let max_text_len = inner_width.saturating_sub(18);
            let text = truncate_str(&prompt.text, max_text_len);

            prompt_lines.push(Line::from(vec![
                Span::styled(
                    format!(" {:>14}  ", time_ago),
                    Style::default().fg(Color::DarkGray),
                ),
                Span::styled(text, Style::default().fg(Color::Reset)),
            ]));
        }
    }

    // Box height = content + 2 (borders), capped to available space minus button area
    let box_content_height = prompt_lines.len();
    let max_box_height = area.height.saturating_sub(4) as usize; // leave 4 lines for gap + buttons
    let box_height = (box_content_height + 2).min(max_box_height).max(3) as u16;
    let button_block_height = 3_u16; // empty line + button line + shortcut line

    // Total used height
    let total_used = box_height + 1 + button_block_height; // +1 for gap between box and buttons
    let top_margin = area.height.saturating_sub(total_used) / 2;

    // Vertical layout: top margin, box, gap, buttons, bottom margin
    let outer = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(top_margin),
            Constraint::Length(box_height),
            Constraint::Length(1), // gap
            Constraint::Length(button_block_height),
            Constraint::Min(0), // bottom margin
        ])
        .split(area);

    let box_area = outer[1];
    let button_area = outer[3];

    // -- Bordered prompt list --
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

    // If content overflows, show from the bottom (newest)
    let visible = box_height.saturating_sub(2) as usize;
    if prompt_lines.len() > visible {
        let skip = prompt_lines.len() - visible;
        prompt_lines.drain(..skip);
    }

    let prompt_block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::DarkGray))
        .title(title)
        .title_style(Style::default().fg(Color::Green).bold());

    let prompt_widget = Paragraph::new(Text::from(prompt_lines)).block(prompt_block);
    frame.render_widget(prompt_widget, box_area);

    // -- Buttons (centered) --
    let copy_focused = detail.focused_button == DetailButton::CopyAndExit;

    let btn_copy_label = " Copy to clipboard & Exit ";
    let btn_back_label = " Back ";
    let btn_gap = "    ";
    let total_btn_width =
        btn_copy_label.len() + btn_back_label.len() + btn_gap.len() + 4; // +4 for brackets
    let left_pad = (area.width as usize).saturating_sub(total_btn_width) / 2;
    let pad_str = " ".repeat(left_pad);

    let dim = Style::default().fg(Color::DarkGray);

    // Button line
    let btn_line = Line::from(vec![
        Span::raw(&pad_str),
        if copy_focused {
            Span::styled(
                format!("[{btn_copy_label}]"),
                Style::default().fg(Color::Cyan).bold(),
            )
        } else {
            Span::styled(format!(" {btn_copy_label} "), dim)
        },
        Span::raw(btn_gap),
        if !copy_focused {
            Span::styled(
                format!("[{btn_back_label}]"),
                Style::default().fg(Color::Cyan).bold(),
            )
        } else {
            Span::styled(format!(" {btn_back_label} "), dim)
        },
    ]);

    // Shortcut line (centered under buttons)
    let sc_copy = "Enter";
    let sc_back = "Esc";
    // Align shortcuts roughly under button centers
    let copy_center = left_pad + 1 + btn_copy_label.len() / 2;
    let back_center =
        left_pad + btn_copy_label.len() + 2 + btn_gap.len() + 1 + btn_back_label.len() / 2;
    let sc_copy_pad = copy_center.saturating_sub(sc_copy.len() / 2);
    let sc_gap = back_center.saturating_sub(sc_copy_pad + sc_copy.len() + sc_back.len() / 2);

    let shortcut_line = Line::from(vec![
        Span::styled(" ".repeat(sc_copy_pad), dim),
        Span::styled(sc_copy, dim),
        Span::styled(" ".repeat(sc_gap), dim),
        Span::styled(sc_back, dim),
    ]);

    let tab_hint = Line::from(vec![
        Span::raw(&pad_str),
        Span::styled("Tab to switch", Style::default().fg(Color::Rgb(60, 60, 70))),
    ]);

    let buttons = Paragraph::new(Text::from(vec![btn_line, shortcut_line, tab_hint]));
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
