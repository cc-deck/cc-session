use chrono::Utc;
use chrono_humanize::{Accuracy, HumanTime, Tense};
use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, Paragraph};

use crate::session::{ConversationMessage, MessageRole};

use super::table;
use super::{App, ContentSearchState, MatchType, Mode};

/// Render the full TUI frame.
pub fn render(frame: &mut Frame, app: &mut App) {
    let area = frame.area();

    if app.mode == Mode::Conversation || app.mode == Mode::ConversationSearch {
        render_conversation(frame, app, area);
        return;
    }

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
    let end = (start + visible_items).min(app.display_entries.len());

    for i in start..end {
        let entry = &app.display_entries[i];
        let session = app.display_session(entry);
        let is_selected = i == app.selected;

        let delta = Utc::now().signed_duration_since(session.timestamp);
        let time_ago = HumanTime::from(-delta).to_text_en(Accuracy::Rough, Tense::Past);
        let right = format!("{}  {}", session.project_name, time_ago);
        let right_len = right.len();

        let (cursor, cursor_len) = if is_selected {
            if entry.match_type == MatchType::Content {
                ("\u{25B8}\u{00B7}", 3)
            } else {
                ("\u{25B8} ", 2)
            }
        } else if entry.match_type == MatchType::Content {
            (" \u{00B7}", 2)
        } else {
            ("  ", 2)
        };

        let max_msg_len = width.saturating_sub(cursor_len + right_len + 2);
        let msg = truncate_str(&session.first_message, max_msg_len);
        let msg_len = msg.chars().count();
        let pad = width.saturating_sub(cursor_len + msg_len + right_len);
        let padding = " ".repeat(pad);

        let msg_style = if is_selected {
            Style::default().fg(Color::White)
        } else {
            Style::default().fg(app.theme.text)
        };

        let dim = Style::default().fg(app.theme.text_dim);

        let cursor_style = if entry.match_type == MatchType::Content {
            Style::default().fg(app.theme.text_dim)
        } else {
            Style::default().fg(app.theme.cursor_color)
        };

        let mut spans = vec![Span::styled(cursor, cursor_style)];
        spans.extend(highlight_terms(&msg, &term_refs, msg_style, &app.theme));
        spans.push(Span::raw(padding));
        spans.push(Span::styled(right, dim));

        let line = Line::from(spans);

        if is_selected {
            lines.push(line.patch_style(Style::default().bg(app.theme.selected_bg)));
        } else {
            lines.push(line);
        }
    }

    let text = Text::from(lines);
    let block = Block::default()
        .borders(Borders::NONE)
        .title(format!(
            " cc-session ({}/{}) ",
            app.display_entries.len(),
            app.sessions.len()
        ))
        .title_style(Style::default().fg(app.theme.cursor_color).bold());

    let paragraph = Paragraph::new(text).block(block);
    frame.render_widget(paragraph, area);
}

const MAX_CONTENT_WIDTH: u16 = 120;

/// Render the conversation viewer.
fn render_conversation(frame: &mut Frame, app: &mut App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(1), Constraint::Length(1)])
        .split(area);

    let full_content_area = chunks[0];
    let status_area = chunks[1];

    let terminal_width = full_content_area.width;
    let content_width = terminal_width.min(MAX_CONTENT_WIDTH);
    let left_margin = (terminal_width.saturating_sub(content_width)) / 2;

    let content_area = Rect {
        x: full_content_area.x + left_margin,
        y: full_content_area.y,
        width: content_width,
        height: full_content_area.height,
    };

    let height = content_area.height as usize;

    if let Some(conv) = &mut app.conversation {
        conv.page_height = height;

        if conv.rendered_width != content_width || conv.lines.is_empty() {
            let search_terms: Vec<String> = if !conv.search_query.is_empty() {
                conv.search_query
                    .split_whitespace()
                    .map(String::from)
                    .collect()
            } else {
                conv.initial_search_terms.clone()
            };
            let term_refs: Vec<&str> = search_terms.iter().map(|s| s.as_str()).collect();
            conv.lines = pre_render_conversation(
                &conv.messages,
                content_width as usize,
                &term_refs,
                &app.theme,
                &app.syntax_highlighter,
            );
            conv.rendered_width = content_width;

            conv.match_positions = find_match_positions(&conv.lines, &term_refs);

            if !conv.match_positions.is_empty()
                && conv.scroll_offset == 0
                && !conv.search_confirmed
            {
                let match_line = conv.match_positions[0];
                let max = conv.lines.len().saturating_sub(height);
                conv.scroll_offset = match_line.saturating_sub(height / 2).min(max);
            }
        }
    }

    if let Some(conv) = &app.conversation {
        let total_lines = conv.lines.len();
        let start = conv.scroll_offset.min(total_lines.saturating_sub(1));
        let end = (start + height).min(total_lines);

        let visible_lines: Vec<Line> = conv.lines[start..end].to_vec();
        let text = Text::from(visible_lines);

        let paragraph = Paragraph::new(text);
        frame.render_widget(paragraph, content_area);
    }

    render_conversation_status(frame, app, status_area);
}

/// Render the conversation viewer status bar.
fn render_conversation_status(frame: &mut Frame, app: &App, area: Rect) {
    let content = if let Some(conv) = &app.conversation {
        let dim = Style::default().fg(app.theme.text_dim);
        if conv.search_active {
            let match_info = if conv.search_query.is_empty() {
                String::new()
            } else if conv.match_positions.is_empty() {
                " No matches".to_string()
            } else {
                format!(
                    " {}/{}",
                    conv.current_match + 1,
                    conv.match_positions.len()
                )
            };

            // Show search query with "selected" appearance when replacing
            let query_style = if conv.search_replacing {
                Style::default().fg(Color::White).bg(app.theme.status_label_bg)
            } else {
                Style::default().fg(Color::White)
            };

            Line::from(vec![
                Span::styled(
                    " / ",
                    Style::default()
                        .fg(app.theme.status_label_fg)
                        .bg(app.theme.status_label_bg)
                        .bold(),
                ),
                Span::styled(" ", Style::default()),
                Span::styled(&conv.search_query, query_style),
                Span::styled("\u{258E}", Style::default().fg(app.theme.status_label_bg)),
                Span::styled(match_info, dim),
                Span::raw("  "),
                Span::styled("Enter confirm  Esc cancel", dim),
            ])
        } else if conv.search_confirmed && !conv.match_positions.is_empty() {
            let project_label = format_project_label(&conv.session);
            Line::from(vec![
                Span::styled(
                    format!(" {} ", project_label),
                    Style::default().fg(Color::Green).bold(),
                ),
                Span::styled(
                    format!(
                        "\"{}\" {}/{}",
                        conv.search_query,
                        conv.current_match + 1,
                        conv.match_positions.len()
                    ),
                    dim,
                ),
                Span::raw("  "),
                Span::styled(
                    "n/N next/prev  / search  Enter copy & exit  Esc back",
                    dim,
                ),
            ])
        } else {
            let project_label = format_project_label(&conv.session);
            Line::from(vec![
                Span::styled(
                    format!(" {} ", project_label),
                    Style::default().fg(Color::Green).bold(),
                ),
                Span::raw(" "),
                Span::styled(
                    "Space/b scroll  g/G top/bottom  / search  Enter copy & exit  Esc back",
                    dim,
                ),
            ])
        }
    } else {
        Line::from("")
    };

    let bar =
        Paragraph::new(content).style(Style::default().bg(app.theme.status_bar_bg));
    frame.render_widget(bar, area);
}

/// Pre-render conversation messages into terminal lines with word wrapping,
/// syntax highlighting, table rendering, and role header bars.
fn pre_render_conversation(
    messages: &[ConversationMessage],
    width: usize,
    search_terms: &[&str],
    theme: &crate::theme::Theme,
    syntax_highlighter: &super::syntax::SyntaxHighlighter,
) -> Vec<Line<'static>> {
    let mut lines: Vec<Line<'static>> = Vec::new();
    let dim = Style::default().fg(theme.text_dim);
    let fallback_code_style = Style::default()
        .fg(Color::Rgb(130, 170, 200))
        .bg(theme.code_block_bg);

    if messages.is_empty() {
        lines.push(Line::from(Span::styled(
            " No messages found in this session.",
            dim,
        )));
        return lines;
    }

    for msg in messages {
        // Full-width role header bar with integrated timestamp
        let delta = Utc::now().signed_duration_since(msg.timestamp);
        let time_ago = HumanTime::from(-delta).to_text_en(Accuracy::Rough, Tense::Past);

        let (label, header_bg, header_fg): (&str, Color, Color) = match msg.role {
            MessageRole::User => (
                " \u{25B6} You ",
                theme.user_header_bg,
                theme.user_header_fg,
            ),
            MessageRole::Assistant => (
                " \u{25C0} Claude ",
                theme.assistant_header_bg,
                theme.assistant_header_fg,
            ),
        };

        let time_str = format!(" {} ", time_ago);
        let label_len = label.chars().count();
        let time_len = time_str.chars().count();
        let pad_len = width.saturating_sub(label_len + time_len);
        let padding = " ".repeat(pad_len);

        let header_style = Style::default().fg(header_fg).bg(header_bg).bold();
        let header_dim = Style::default().fg(header_fg).bg(header_bg);
        lines.push(Line::from(vec![
            Span::styled(label, header_style),
            Span::styled(padding, Style::default().bg(header_bg)),
            Span::styled(time_str, header_dim),
        ]));

        // Determine message body background
        let msg_bg = match msg.role {
            MessageRole::User => Some(theme.user_message_bg),
            MessageRole::Assistant => None,
        };

        let base_style = if let Some(bg) = msg_bg {
            Style::default().fg(theme.text).bg(bg)
        } else {
            Style::default().fg(theme.text)
        };
        let heading_style = Style::default().fg(theme.heading).bold();

        // Collect message lines for table detection
        let text_lines: Vec<&str> = msg.text.lines().collect();
        let mut i = 0;
        let mut in_code_fence = false;
        let mut code_lang: Option<String> = None;
        let mut code_buffer: Vec<String> = Vec::new();

        while i < text_lines.len() {
            let text_line = text_lines[i];
            let trimmed = text_line.trim();

            // Code fence handling
            if trimmed.starts_with("```") {
                if in_code_fence {
                    // Closing fence: render buffered code
                    let code_refs: Vec<&str> =
                        code_buffer.iter().map(|s| s.as_str()).collect();
                    let highlighted = code_lang.as_ref().and_then(|lang| {
                        syntax_highlighter.highlight_code(
                            &code_refs,
                            lang,
                            theme.syntect_theme,
                            theme.code_block_bg,
                            width,
                        )
                    });

                    if let Some(hl_lines) = highlighted {
                        lines.extend(hl_lines);
                    } else {
                        // Fallback: single-color code, padded to full width
                        for cl in &code_buffer {
                            let wrapped = wrap_line(cl, width);
                            for wl in wrapped {
                                let char_len = wl.chars().count();
                                let pad = " ".repeat(width.saturating_sub(char_len));
                                lines.push(Line::from(vec![Span::styled(
                                    format!("{}{}", wl, pad),
                                    fallback_code_style,
                                )]));
                            }
                        }
                    }

                    code_buffer.clear();
                    code_lang = None;
                    in_code_fence = false;
                } else {
                    // Opening fence: extract language
                    in_code_fence = true;
                    code_lang =
                        super::syntax::extract_language(trimmed);
                }
                i += 1;
                continue;
            }

            if in_code_fence {
                code_buffer.push(text_line.to_string());
                i += 1;
                continue;
            }

            // Table detection: collect consecutive pipe lines
            if table::is_table_line(trimmed) {
                let mut table_lines: Vec<&str> = vec![text_line];
                let mut j = i + 1;
                while j < text_lines.len() && table::is_table_line(text_lines[j].trim()) {
                    table_lines.push(text_lines[j]);
                    j += 1;
                }

                if table_lines.len() >= 2 {
                    let table_refs: Vec<&str> = table_lines.to_vec();
                    if let Some(rendered) =
                        table::render_table_lines(&table_refs, width, theme)
                    {
                        // Apply message background to table lines if user message
                        for tl in rendered {
                            if let Some(bg) = msg_bg {
                                lines.push(tl.patch_style(Style::default().bg(bg)));
                            } else {
                                lines.push(tl);
                            }
                        }
                        i = j;
                        continue;
                    }
                }
                // Fall through to normal rendering if table parsing failed
            }

            // Markdown heading
            if trimmed.starts_with('#') {
                let level = trimmed.chars().take_while(|&c| c == '#').count();
                let heading_text = trimmed[level..].trim_start();
                let prefix = "\u{2500}".repeat(level.min(3));
                let wrapped =
                    wrap_line(heading_text, width.saturating_sub(prefix.len() + 1));
                let heading_with_bg = heading_style.bg(theme.heading_bg);
                for (idx, wl) in wrapped.into_iter().enumerate() {
                    let mut spans = Vec::new();
                    if idx == 0 {
                        spans.push(Span::styled(
                            format!("{} ", prefix),
                            Style::default().fg(theme.text_dim).bg(theme.heading_bg),
                        ));
                    }
                    spans.extend(render_markdown_inline(
                        &wl,
                        heading_with_bg,
                        search_terms,
                        theme,
                    ));
                    // Pad to full width for consistent background
                    let char_count: usize = spans.iter().map(|s| s.content.chars().count()).sum();
                    if char_count < width {
                        spans.push(Span::styled(
                            " ".repeat(width - char_count),
                            Style::default().bg(theme.heading_bg),
                        ));
                    }
                    lines.push(Line::from(spans));
                }
                i += 1;
                continue;
            }

            // Empty line
            if trimmed.is_empty() {
                if let Some(bg) = msg_bg {
                    let pad = " ".repeat(width);
                    lines.push(Line::from(Span::styled(
                        pad,
                        Style::default().bg(bg),
                    )));
                } else {
                    lines.push(Line::from(""));
                }
                i += 1;
                continue;
            }

            // Normal text with markdown inline rendering
            let wrapped = wrap_line(text_line, width);
            for wl in wrapped {
                let spans =
                    render_markdown_inline(&wl, base_style, search_terms, theme);
                let line = Line::from(spans);
                if let Some(bg) = msg_bg {
                    if !has_bg_set(&line) {
                        lines.push(
                            line.patch_style(Style::default().bg(bg)),
                        );
                    } else {
                        lines.push(line);
                    }
                } else {
                    lines.push(line);
                }
            }
            i += 1;
        }

        // Handle unclosed code fence
        if in_code_fence {
            for cl in &code_buffer {
                let wrapped = wrap_line(cl, width);
                for wl in wrapped {
                    lines.push(Line::from(vec![Span::styled(wl, fallback_code_style)]));
                }
            }
        }

        // Blank line after message
        lines.push(Line::from(""));
    }

    lines
}

/// Check if any span in a line already has a background set.
fn has_bg_set(line: &Line) -> bool {
    line.spans
        .iter()
        .any(|s| s.style.bg.is_some())
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
        if pos + width >= chars.len() {
            result.push(chars[pos..].iter().collect());
            break;
        }

        let end = pos + width;
        let mut break_at = end;

        for j in (pos..end).rev() {
            if chars[j] == ' ' {
                break_at = j + 1;
                break;
            }
        }

        if break_at == end && (end - pos) > width / 2 {
            break_at = end;
        }

        let chunk: String = chars[pos..break_at].iter().collect();
        result.push(chunk.trim_end().to_string());
        pos = break_at;

        while pos < chars.len() && chars[pos] == ' ' {
            pos += 1;
        }
    }

    result
}

/// Render the status/help bar at the bottom.
fn render_status_bar(frame: &mut Frame, app: &App, area: Rect) {
    let dim = Style::default().fg(app.theme.text_dim);
    let content = match app.mode {
        Mode::Conversation | Mode::ConversationSearch => Line::from(""),
        Mode::Browsing => {
            if let Some((msg, _)) = &app.status_message {
                Line::from(vec![Span::styled(
                    format!(" {msg}"),
                    Style::default().fg(Color::Green).bold(),
                )])
            } else if !app.filter_query.is_empty() {
                // Filter active: show filter text with match count
                let match_count = app.display_entries.len();
                let match_info = if match_count == 0 {
                    " no matches".to_string()
                } else {
                    match app.content_search_state {
                        ContentSearchState::Searching => {
                            format!(
                                " {} {} matches (searching content...)",
                                app.spinner_char(),
                                match_count
                            )
                        }
                        _ => format!(" {} matches", match_count),
                    }
                };

                Line::from(vec![
                    Span::styled(
                        " filter: ",
                        dim,
                    ),
                    Span::styled(
                        &app.filter_query,
                        Style::default().fg(app.theme.status_label_bg).bold(),
                    ),
                    Span::styled(match_info, dim),
                    Span::raw("  "),
                    Span::styled("Esc clear  Enter select", dim),
                ])
            } else {
                Line::from(vec![
                    Span::styled(" Enter ", dim),
                    Span::styled("detail", dim),
                    Span::raw("  "),
                    Span::styled("Esc ", dim),
                    Span::styled("quit", dim),
                    Span::raw("  "),
                    Span::styled("(type to search)", dim),
                ])
            }
        }
    };

    let bar =
        Paragraph::new(content).style(Style::default().bg(app.theme.status_bar_bg));
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

/// Render inline markdown (bold, italic, inline code) as styled spans,
/// then apply search term highlighting on top.
fn render_markdown_inline<'a>(
    text: &str,
    base_style: Style,
    search_terms: &[&str],
    theme: &crate::theme::Theme,
) -> Vec<Span<'a>> {
    let bold_style = base_style.bold();
    let italic_style = base_style.italic();
    let code_style = Style::default().fg(Color::Rgb(130, 170, 200));

    let mut spans: Vec<(String, Style)> = Vec::new();
    let chars: Vec<char> = text.chars().collect();
    let len = chars.len();
    let mut i = 0;
    let mut current = String::new();

    while i < len {
        if i + 1 < len && chars[i] == '*' && chars[i + 1] == '*' {
            if !current.is_empty() {
                spans.push((std::mem::take(&mut current), base_style));
            }
            i += 2;
            let mut bold_text = String::new();
            while i + 1 < len && !(chars[i] == '*' && chars[i + 1] == '*') {
                bold_text.push(chars[i]);
                i += 1;
            }
            if i + 1 < len {
                i += 2;
            }
            spans.push((bold_text, bold_style));
            continue;
        }

        if chars[i] == '`' {
            if !current.is_empty() {
                spans.push((std::mem::take(&mut current), base_style));
            }
            i += 1;
            let mut code_text = String::new();
            while i < len && chars[i] != '`' {
                code_text.push(chars[i]);
                i += 1;
            }
            if i < len {
                i += 1;
            }
            spans.push((code_text, code_style));
            continue;
        }

        if chars[i] == '*' && (i + 1 >= len || chars[i + 1] != '*') {
            if !current.is_empty() {
                spans.push((std::mem::take(&mut current), base_style));
            }
            i += 1;
            let mut italic_text = String::new();
            while i < len && chars[i] != '*' {
                italic_text.push(chars[i]);
                i += 1;
            }
            if i < len {
                i += 1;
            }
            if italic_text.is_empty() {
                spans.push(("*".to_string(), base_style));
            } else {
                spans.push((italic_text, italic_style));
            }
            continue;
        }

        current.push(chars[i]);
        i += 1;
    }

    if !current.is_empty() {
        spans.push((current, base_style));
    }

    // Post-process: detect URLs and style them
    let link_style = base_style
        .fg(theme.link)
        .add_modifier(Modifier::UNDERLINED);
    let processed: Vec<(String, Style)> = spans
        .into_iter()
        .flat_map(|(text, style)| split_urls(&text, style, link_style))
        .collect();

    if search_terms.is_empty() {
        return processed
            .into_iter()
            .map(|(text, style)| Span::styled(text, style))
            .collect();
    }

    let mut result: Vec<Span<'a>> = Vec::new();
    for (segment, style) in processed {
        result.extend(highlight_terms(&segment, search_terms, style, theme));
    }
    result
}

/// Split a text segment into URL and non-URL parts.
fn split_urls(text: &str, base_style: Style, link_style: Style) -> Vec<(String, Style)> {
    const URL_PREFIXES: &[&str] = &["https://", "http://"];

    let mut parts = Vec::new();
    let mut remaining = text;

    while !remaining.is_empty() {
        // Find the earliest URL
        let mut earliest_pos = None;
        let mut earliest_prefix_len = 0;
        for prefix in URL_PREFIXES {
            if let Some(pos) = remaining.find(prefix) {
                if earliest_pos.is_none() || pos < earliest_pos.unwrap() {
                    earliest_pos = Some(pos);
                    earliest_prefix_len = prefix.len();
                }
            }
        }

        match earliest_pos {
            Some(pos) => {
                // Text before URL
                if pos > 0 {
                    parts.push((remaining[..pos].to_string(), base_style));
                }
                // Extract URL (until whitespace, closing paren/bracket, or end)
                let url_start = &remaining[pos..];
                let url_end = url_start[earliest_prefix_len..]
                    .find(|c: char| c.is_whitespace() || c == ')' || c == ']' || c == '>' || c == '"' || c == '\'')
                    .map(|p| p + earliest_prefix_len)
                    .unwrap_or(url_start.len());
                // Trim trailing punctuation that's likely not part of the URL
                let mut url = &url_start[..url_end];
                while url.ends_with('.') || url.ends_with(',') || url.ends_with(';') || url.ends_with(':') {
                    url = &url[..url.len() - 1];
                }
                parts.push((url.to_string(), link_style));
                remaining = &remaining[pos + url.len()..];
            }
            None => {
                parts.push((remaining.to_string(), base_style));
                break;
            }
        }
    }

    parts
}

/// Split text into spans, highlighting portions that match any search terms.
fn highlight_terms<'a>(
    text: &str,
    terms: &[&str],
    base_style: Style,
    theme: &crate::theme::Theme,
) -> Vec<Span<'a>> {
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
            for m in matched.iter_mut().skip(abs_pos).take(term_lower.len()) {
                *m = true;
            }
            start = abs_pos + 1;
        }
    }

    let highlight_style = base_style.bg(theme.search_highlight_bg);
    let mut spans = Vec::new();
    let mut i = 0;
    while i < len {
        let is_match = matched[i];
        let run_start = i;
        while i < len && matched[i] == is_match {
            i += 1;
        }
        let segment = &text[run_start..i];
        let style = if is_match {
            highlight_style
        } else {
            base_style
        };
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
        return app
            .filter_query
            .split_whitespace()
            .map(String::from)
            .collect();
    }
    Vec::new()
}

/// Format project label as "project_name (branch)" for the conversation status bar.
fn format_project_label(session: &crate::session::Session) -> String {
    match &session.git_branch {
        Some(branch) if !branch.is_empty() => format!("{} ({})", session.project_name, branch),
        _ => session.project_name.clone(),
    }
}
