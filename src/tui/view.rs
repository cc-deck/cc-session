use chrono::Utc;
use chrono_humanize::{Accuracy, HumanTime, Tense};
use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, Paragraph};

use crate::session::{ConversationMessage, MessageRole};

use super::{App, Mode};

/// Render the full TUI frame.
pub fn render(frame: &mut Frame, app: &mut App) {
    let area = frame.area();

    if app.mode == Mode::Conversation || app.mode == Mode::ConversationSearch {
        render_conversation(frame, app, area);
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

/// Render the session list with single-line entries.
fn render_session_list(frame: &mut Frame, app: &App, area: Rect) {
    let width = area.width as usize;
    let height = area.height as usize;

    let visible_items = height;

    let mut lines: Vec<Line> = Vec::new();

    let terms = search_terms(app);
    let term_refs: Vec<&str> = terms.iter().map(|s| s.as_str()).collect();

    let start = app.scroll_offset;
    let end = (start + visible_items).min(app.filtered_indices.len());

    for i in start..end {
        let idx = app.filtered_indices[i];
        let session = &app.sessions[idx];
        let is_selected = i == app.selected;

        let delta = Utc::now().signed_duration_since(session.timestamp);
        let time_ago = HumanTime::from(-delta).to_text_en(Accuracy::Rough, Tense::Past);
        let right = format!("{}  {}", session.project_name, time_ago);
        let right_len = right.len();

        let cursor = if is_selected { "\u{25B8} " } else { "  " };
        let cursor_len = 2;
        let max_msg_len = width.saturating_sub(cursor_len + right_len + 2);
        let msg = truncate_str(&session.first_message, max_msg_len);
        let msg_len = msg.chars().count();

        let pad = width.saturating_sub(cursor_len + msg_len + right_len);
        let padding = " ".repeat(pad);

        let msg_style = if is_selected {
            Style::default().fg(Color::White)
        } else {
            Style::default().fg(Color::Reset)
        };

        let dim = Style::default().fg(Color::DarkGray);

        let mut spans = vec![Span::styled(cursor, Style::default().fg(Color::Cyan))];
        spans.extend(highlight_terms(&msg, &term_refs, msg_style));
        spans.push(Span::raw(padding));
        spans.push(Span::styled(right, dim));

        let line = Line::from(spans);

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

/// Render the conversation viewer.
fn render_conversation(frame: &mut Frame, app: &mut App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(1), Constraint::Length(1)])
        .split(area);

    let content_area = chunks[0];
    let status_area = chunks[1];
    let width = content_area.width;
    let height = content_area.height as usize;

    // Update page_height and check if we need to re-render lines
    if let Some(conv) = &mut app.conversation {
        conv.page_height = height;

        if conv.rendered_width != width || conv.lines.is_empty() {
            let search_terms: Vec<String> = if !conv.search_query.is_empty() {
                conv.search_query.split_whitespace().map(String::from).collect()
            } else {
                conv.initial_search_terms.clone()
            };
            let term_refs: Vec<&str> = search_terms.iter().map(|s| s.as_str()).collect();
            conv.lines = pre_render_conversation(&conv.messages, width as usize, &term_refs);
            conv.rendered_width = width;

            // Build match positions
            conv.match_positions = find_match_positions(&conv.lines, &term_refs);

            // Auto-scroll to first match on initial render
            if !conv.match_positions.is_empty() && conv.scroll_offset == 0 && !conv.search_confirmed {
                conv.scroll_offset = conv.match_positions[0].saturating_sub(2);
            }
        }
    }

    // Render content lines
    if let Some(conv) = &app.conversation {
        let total_lines = conv.lines.len();
        let start = conv.scroll_offset.min(total_lines.saturating_sub(1));
        let end = (start + height).min(total_lines);

        let visible_lines: Vec<Line> = conv.lines[start..end].to_vec();
        let text = Text::from(visible_lines);

        let paragraph = Paragraph::new(text);
        frame.render_widget(paragraph, content_area);
    }

    // Render status bar
    render_conversation_status(frame, app, status_area);
}

/// Render the conversation viewer status bar.
fn render_conversation_status(frame: &mut Frame, app: &App, area: Rect) {
    let content = if let Some(conv) = &app.conversation {
        if conv.search_active {
            let match_info = if conv.search_query.is_empty() {
                String::new()
            } else if conv.match_positions.is_empty() {
                " No matches".to_string()
            } else {
                format!(" {}/{}", conv.current_match + 1, conv.match_positions.len())
            };

            Line::from(vec![
                Span::styled(
                    " / ",
                    Style::default().fg(Color::Black).bg(Color::Cyan).bold(),
                ),
                Span::styled(" ", Style::default()),
                Span::styled(&conv.search_query, Style::default().fg(Color::White)),
                Span::styled("\u{258E}", Style::default().fg(Color::Cyan)),
                Span::styled(match_info, Style::default().fg(Color::DarkGray)),
                Span::raw("  "),
                Span::styled(
                    "Enter confirm  Esc cancel",
                    Style::default().fg(Color::DarkGray),
                ),
            ])
        } else if conv.search_confirmed && !conv.match_positions.is_empty() {
            let session = &app.sessions[conv.session_idx];
            Line::from(vec![
                Span::styled(
                    " SESSION ",
                    Style::default().fg(Color::Black).bg(Color::Green).bold(),
                ),
                Span::styled(
                    format!(" {} ", session.project_name),
                    Style::default().fg(Color::Green),
                ),
                Span::styled(
                    format!("\"{}\" {}/{}", conv.search_query, conv.current_match + 1, conv.match_positions.len()),
                    Style::default().fg(Color::DarkGray),
                ),
                Span::raw("  "),
                Span::styled(
                    "n/N next/prev  / search  Enter copy & exit  Esc back",
                    Style::default().fg(Color::DarkGray),
                ),
            ])
        } else {
            let session = &app.sessions[conv.session_idx];
            Line::from(vec![
                Span::styled(
                    " SESSION ",
                    Style::default().fg(Color::Black).bg(Color::Green).bold(),
                ),
                Span::styled(
                    format!(" {} ", session.project_name),
                    Style::default().fg(Color::Green),
                ),
                Span::raw(" "),
                Span::styled(
                    "Space/b scroll  g/G top/bottom  / search  Enter copy & exit  Esc back",
                    Style::default().fg(Color::DarkGray),
                ),
            ])
        }
    } else {
        Line::from("")
    };

    let bar = Paragraph::new(content).style(Style::default().bg(Color::Rgb(20, 20, 30)));
    frame.render_widget(bar, area);
}

/// Pre-render conversation messages into terminal lines with word wrapping.
fn pre_render_conversation(
    messages: &[ConversationMessage],
    width: usize,
    search_terms: &[&str],
) -> Vec<Line<'static>> {
    let mut lines: Vec<Line<'static>> = Vec::new();
    let dim = Style::default().fg(Color::DarkGray);
    let code_style = Style::default().fg(Color::Rgb(130, 170, 200));

    if messages.is_empty() {
        lines.push(Line::from(Span::styled(
            " No messages found in this session.",
            dim,
        )));
        return lines;
    }

    for msg in messages {
        // Separator line with timestamp
        let delta = Utc::now().signed_duration_since(msg.timestamp);
        let time_ago = HumanTime::from(-delta).to_text_en(Accuracy::Rough, Tense::Past);
        let time_str = format!(" {} ", time_ago);
        let dash_count = width.saturating_sub(time_str.len());
        let separator = "\u{2500}".repeat(dash_count);
        lines.push(Line::from(vec![
            Span::styled(separator, dim),
            Span::styled(time_str, dim),
        ]));

        // Role header
        match msg.role {
            MessageRole::User => {
                lines.push(Line::from(Span::styled(
                    "\u{25B6} You",
                    Style::default().fg(Color::Cyan).bold(),
                )));
            }
            MessageRole::Assistant => {
                lines.push(Line::from(Span::styled(
                    "\u{25C0} Claude",
                    Style::default().fg(Color::Green).bold(),
                )));
            }
        }

        // Message body with word wrapping and code fence detection
        let mut in_code_fence = false;

        for text_line in msg.text.lines() {
            let trimmed = text_line.trim();
            if trimmed.starts_with("```") {
                in_code_fence = !in_code_fence;
                // Render the fence delimiter
                let wrapped = wrap_line(text_line, width);
                for wl in wrapped {
                    lines.push(Line::from(vec![Span::styled(wl, code_style)]));
                }
                continue;
            }

            if in_code_fence {
                let wrapped = wrap_line(text_line, width);
                for wl in wrapped {
                    lines.push(Line::from(vec![Span::styled(wl, code_style)]));
                }
            } else {
                let wrapped = wrap_line(text_line, width);
                let base_style = Style::default().fg(Color::Reset);
                for wl in wrapped {
                    let spans = highlight_terms(&wl, search_terms, base_style);
                    lines.push(Line::from(spans));
                }
            }
        }

        // Blank line after message
        lines.push(Line::from(""));
    }

    lines
}

/// Find line indices that contain search term matches.
fn find_match_positions(lines: &[Line<'static>], terms: &[&str]) -> Vec<usize> {
    if terms.is_empty() {
        return Vec::new();
    }
    let mut positions = Vec::new();
    for (i, line) in lines.iter().enumerate() {
        let text: String = line.spans.iter().map(|s| s.content.as_ref()).collect();
        let text_lower = text.to_lowercase();
        for term in terms {
            if !term.is_empty() && text_lower.contains(&term.to_lowercase()) {
                positions.push(i);
                break;
            }
        }
    }
    positions
}

/// Word-wrap a single line to fit within `width` characters.
fn wrap_line(line: &str, width: usize) -> Vec<String> {
    if width == 0 {
        return vec![line.to_string()];
    }
    if line.is_empty() {
        return vec![String::new()];
    }

    let chars: Vec<char> = line.chars().collect();
    if chars.len() <= width {
        return vec![line.to_string()];
    }

    let mut result = Vec::new();
    let mut pos = 0;
    while pos < chars.len() {
        let end = (pos + width).min(chars.len());
        let chunk: String = chars[pos..end].iter().collect();
        result.push(chunk);
        pos = end;
    }
    result
}

/// Render the status/help bar at the bottom.
fn render_status_bar(frame: &mut Frame, app: &App, area: Rect) {
    let content = match app.mode {
        Mode::Filtering => {
            let match_info = if app.filter_query.is_empty() {
                String::new()
            } else if app.filtered_indices.is_empty() {
                " no matches".to_string()
            } else {
                format!(" {} matches", app.filtered_indices.len())
            };

            let mode_label = if app.is_deep_search() {
                " FILTER (deep) "
            } else {
                " FILTER "
            };

            Line::from(vec![
                Span::styled(
                    mode_label,
                    Style::default()
                        .fg(Color::Black)
                        .bg(Color::Cyan)
                        .bold(),
                ),
                Span::styled(" ", Style::default()),
                Span::styled(&app.filter_query, Style::default().fg(Color::White)),
                Span::styled("\u{258E}", Style::default().fg(Color::Cyan)),
                Span::styled(match_info, Style::default().fg(Color::DarkGray)),
                Span::raw("  "),
                Span::styled(
                    "Ctrl-G deep search  Esc cancel  Enter select",
                    Style::default().fg(Color::DarkGray),
                ),
            ])
        }
        Mode::DeepSearchInput => {
            Line::from(vec![
                Span::styled(
                    " DEEP SEARCH \u{23CE} ",
                    Style::default()
                        .fg(Color::Black)
                        .bg(Color::Yellow)
                        .bold(),
                ),
                Span::styled(" ", Style::default()),
                Span::styled(&app.filter_query, Style::default().fg(Color::White)),
                Span::styled("\u{258E}", Style::default().fg(Color::Yellow)),
                Span::raw("  "),
                Span::styled(
                    "Enter search  Esc back to filter",
                    Style::default().fg(Color::DarkGray),
                ),
            ])
        }
        Mode::DeepSearching => {
            let query = app.deep_search_query.as_deref().unwrap_or("");
            Line::from(vec![
                Span::styled(
                    " DEEP SEARCH ",
                    Style::default()
                        .fg(Color::Black)
                        .bg(Color::Yellow)
                        .bold(),
                ),
                Span::styled(
                    format!(" {} ", app.spinner_char()),
                    Style::default().fg(Color::Yellow),
                ),
                Span::styled(
                    format!("Searching \"{}\"...", query),
                    Style::default().fg(Color::DarkGray),
                ),
                Span::raw("  "),
                Span::styled(
                    "Esc cancel",
                    Style::default().fg(Color::DarkGray),
                ),
            ])
        }
        Mode::Conversation | Mode::ConversationSearch => Line::from(""),
        Mode::Browsing => {
            if let Some((msg, _)) = &app.status_message {
                let mut spans = vec![Span::styled(
                    format!(" {msg}"),
                    Style::default().fg(Color::Green).bold(),
                )];
                if app.is_deep_search() {
                    spans.push(Span::raw("  "));
                    spans.push(Span::styled(
                        "Esc back",
                        Style::default().fg(Color::DarkGray),
                    ));
                }
                Line::from(spans)
            } else if app.is_deep_search() {
                let query = app
                    .deep_search_query
                    .as_deref()
                    .unwrap_or("");
                Line::from(vec![
                    Span::styled(
                        " DEEP SEARCH ",
                        Style::default()
                            .fg(Color::Black)
                            .bg(Color::Yellow)
                            .bold(),
                    ),
                    Span::styled(
                        format!(" \"{}\"", query),
                        Style::default().fg(Color::Yellow),
                    ),
                    Span::raw("  "),
                    Span::styled("/ ", Style::default().fg(Color::DarkGray)),
                    Span::styled("filter", Style::default().fg(Color::DarkGray)),
                    Span::raw("  "),
                    Span::styled("Esc ", Style::default().fg(Color::DarkGray)),
                    Span::styled("back", Style::default().fg(Color::DarkGray)),
                    Span::raw("  "),
                    Span::styled("Enter ", Style::default().fg(Color::DarkGray)),
                    Span::styled("detail", Style::default().fg(Color::DarkGray)),
                ])
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

/// Split text into spans, underlining portions that match any of the search terms.
fn highlight_terms<'a>(text: &str, terms: &[&str], base_style: Style) -> Vec<Span<'a>> {
    if terms.is_empty() {
        return vec![Span::styled(text.to_string(), base_style)];
    }

    let text_lower = text.to_lowercase();
    let len = text.len();

    let mut matched = vec![false; len];
    for term in terms {
        if term.is_empty() {
            continue;
        }
        let term_lower = term.to_lowercase();
        let mut start = 0;
        while let Some(pos) = text_lower[start..].find(&term_lower) {
            let abs_pos = start + pos;
            for i in abs_pos..abs_pos + term_lower.len() {
                matched[i] = true;
            }
            start = abs_pos + 1;
        }
    }

    let highlight_style = base_style.underlined();
    let mut spans = Vec::new();
    let mut i = 0;
    while i < len {
        let is_match = matched[i];
        let run_start = i;
        while i < len && matched[i] == is_match {
            i += 1;
        }
        let segment = &text[run_start..i];
        let style = if is_match { highlight_style } else { base_style };
        spans.push(Span::styled(segment.to_string(), style));
    }

    if spans.is_empty() {
        spans.push(Span::styled(text.to_string(), base_style));
    }

    spans
}

/// Get the active search terms for highlighting from the app state.
fn search_terms(app: &App) -> Vec<String> {
    if !app.filter_query.is_empty() {
        return app.filter_query.split_whitespace().map(String::from).collect();
    }
    if let Some(query) = &app.deep_search_query {
        return query.split_whitespace().map(String::from).collect();
    }
    Vec::new()
}
