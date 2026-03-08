use ratatui::prelude::*;

use crate::theme::Theme;

/// Check if a line looks like the start of a markdown table row.
pub fn is_table_line(line: &str) -> bool {
    let trimmed = line.trim();
    trimmed.starts_with('|') && trimmed.ends_with('|') && trimmed.len() > 2
}

/// Check if a line is a table separator row (e.g., |---|---|).
fn is_separator_row(line: &str) -> bool {
    let trimmed = line.trim().trim_matches('|');
    if trimmed.is_empty() {
        return false;
    }
    trimmed
        .split('|')
        .all(|cell| cell.trim().chars().all(|c| c == '-' || c == ':' || c == ' '))
}

/// Parse cells from a table row.
fn parse_cells(line: &str) -> Vec<String> {
    let trimmed = line.trim();
    let inner = trimmed
        .strip_prefix('|')
        .unwrap_or(trimmed)
        .strip_suffix('|')
        .unwrap_or(trimmed);
    inner.split('|').map(|cell| cell.trim().to_string()).collect()
}

/// A parsed markdown table.
struct Table {
    headers: Vec<String>,
    rows: Vec<Vec<String>>,
    num_cols: usize,
}

/// Parse a block of table lines into a Table struct.
fn parse_table(lines: &[&str]) -> Option<Table> {
    if lines.len() < 2 {
        return None;
    }

    let mut headers = Vec::new();
    let mut rows = Vec::new();
    let mut has_header = false;

    for (i, line) in lines.iter().enumerate() {
        if is_separator_row(line) {
            has_header = true;
            continue;
        }
        let cells = parse_cells(line);
        if cells.is_empty() {
            continue;
        }
        if (!has_header && i == 0) || (has_header && headers.is_empty()) {
            headers = cells;
        } else {
            rows.push(cells);
        }
    }

    if headers.is_empty() && rows.is_empty() {
        return None;
    }

    let num_cols = headers
        .len()
        .max(rows.iter().map(|r| r.len()).max().unwrap_or(0));

    Some(Table {
        headers,
        rows,
        num_cols,
    })
}

/// Compute column widths that fit within max_width, using word wrapping for overflow.
fn compute_col_widths(table: &Table, max_width: usize) -> Vec<usize> {
    let num_cols = table.num_cols;
    let border_overhead = num_cols + 1; // │ between and around columns
    let padding_overhead = num_cols * 2; // 1 space padding on each side
    let available = max_width.saturating_sub(border_overhead + padding_overhead);

    // Start with natural widths (max cell content length per column)
    let mut widths = vec![0usize; num_cols];
    for (i, h) in table.headers.iter().enumerate() {
        if i < num_cols {
            widths[i] = widths[i].max(h.chars().count());
        }
    }
    for row in &table.rows {
        for (i, cell) in row.iter().enumerate() {
            if i < num_cols {
                widths[i] = widths[i].max(cell.chars().count());
            }
        }
    }

    // Ensure minimum width of 3 per column
    for w in &mut widths {
        *w = (*w).max(3);
    }

    let total: usize = widths.iter().sum();
    if total <= available {
        return widths;
    }

    // Shrink columns proportionally but keep minimum of 6
    let ratio = available as f64 / total as f64;
    let mut shrunk: Vec<usize> = widths
        .iter()
        .map(|&w| ((w as f64 * ratio) as usize).max(6))
        .collect();

    // Adjust to fit exactly (distribute remainder to widest columns)
    let shrunk_total: usize = shrunk.iter().sum();
    if shrunk_total > available {
        let excess = shrunk_total - available;
        // Remove excess from widest columns first
        let mut indices: Vec<usize> = (0..num_cols).collect();
        indices.sort_by(|&a, &b| shrunk[b].cmp(&shrunk[a]));
        for (taken, &idx) in indices.iter().enumerate() {
            if taken >= excess {
                break;
            }
            if shrunk[idx] > 6 {
                shrunk[idx] -= 1;
            }
        }
    }

    shrunk
}

/// Word-wrap text to fit within a given width.
fn wrap_cell_text(text: &str, width: usize) -> Vec<String> {
    if width == 0 {
        return vec![text.to_string()];
    }
    let chars: Vec<char> = text.chars().collect();
    if chars.len() <= width {
        return vec![text.to_string()];
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

        // Find word boundary
        for j in (pos..end).rev() {
            if chars[j] == ' ' {
                break_at = j + 1;
                break;
            }
        }

        if break_at == end {
            break_at = end; // Force break
        }

        let chunk: String = chars[pos..break_at].iter().collect();
        result.push(chunk.trim_end().to_string());
        pos = break_at;

        while pos < chars.len() && chars[pos] == ' ' {
            pos += 1;
        }
    }

    if result.is_empty() {
        result.push(String::new());
    }

    result
}

/// Render inline markdown (bold, italic, inline code) within a cell,
/// then pad to the target width.
fn render_cell_spans(text: &str, width: usize, base_style: Style) -> Vec<Span<'static>> {
    let bold_style = base_style.bold();
    let italic_style = base_style.italic();
    let code_style = Style::default().fg(Color::Rgb(130, 170, 200));

    let mut spans: Vec<Span<'static>> = Vec::new();
    let chars: Vec<char> = text.chars().collect();
    let len = chars.len();
    let mut i = 0;
    let mut current = String::new();
    let mut char_count = 0;

    while i < len {
        // Bold: **text**
        if i + 1 < len && chars[i] == '*' && chars[i + 1] == '*' {
            if !current.is_empty() {
                char_count += current.chars().count();
                spans.push(Span::styled(std::mem::take(&mut current), base_style));
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
            char_count += bold_text.chars().count();
            spans.push(Span::styled(bold_text, bold_style));
            continue;
        }

        // Inline code: `text`
        if chars[i] == '`' {
            if !current.is_empty() {
                char_count += current.chars().count();
                spans.push(Span::styled(std::mem::take(&mut current), base_style));
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
            char_count += code_text.chars().count();
            spans.push(Span::styled(code_text, code_style));
            continue;
        }

        // Italic: *text*
        if chars[i] == '*' && (i + 1 >= len || chars[i + 1] != '*') {
            if !current.is_empty() {
                char_count += current.chars().count();
                spans.push(Span::styled(std::mem::take(&mut current), base_style));
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
                spans.push(Span::styled("*", base_style));
                char_count += 1;
            } else {
                char_count += italic_text.chars().count();
                spans.push(Span::styled(italic_text, italic_style));
            }
            continue;
        }

        current.push(chars[i]);
        i += 1;
    }

    if !current.is_empty() {
        char_count += current.chars().count();
        spans.push(Span::styled(current, base_style));
    }

    // Pad to width
    if char_count < width {
        spans.push(Span::styled(
            " ".repeat(width - char_count),
            base_style,
        ));
    }

    spans
}

/// Render a parsed table as styled Lines with box-drawing characters.
/// Cells that exceed column width are word-wrapped into multi-line rows.
/// Multi-line rows get horizontal separators between them.
pub fn render_table_lines(
    table_lines: &[&str],
    max_width: usize,
    theme: &Theme,
) -> Option<Vec<Line<'static>>> {
    let table = parse_table(table_lines)?;
    let col_widths = compute_col_widths(&table, max_width);
    let border_style = Style::default().fg(theme.table_border);
    let header_style = theme.table_header;
    let cell_style = Style::default().fg(theme.text);

    let mut lines = Vec::new();

    // Top border: ┌─┬─┐
    lines.push(build_border_line(&col_widths, '┌', '┬', '┐', border_style));

    // Header row (with wrapping)
    if !table.headers.is_empty() {
        let wrapped = wrap_row(&table.headers, &col_widths, table.num_cols);
        render_wrapped_row(&wrapped, &col_widths, header_style, border_style, &mut lines);
        // Header separator: ├─┼─┤ (always present after header)
        lines.push(build_border_line(&col_widths, '├', '┼', '┤', border_style));
    }

    // Data rows
    let has_multiline = table.rows.iter().any(|row| {
        row.iter()
            .enumerate()
            .any(|(i, cell)| i < col_widths.len() && cell.chars().count() > col_widths[i])
    });

    for (row_idx, row) in table.rows.iter().enumerate() {
        let wrapped = wrap_row(row, &col_widths, table.num_cols);
        render_wrapped_row(&wrapped, &col_widths, cell_style, border_style, &mut lines);

        // Add row separator if there are multi-line cells (except after last row)
        if has_multiline && row_idx < table.rows.len() - 1 {
            lines.push(build_border_line(&col_widths, '├', '┼', '┤', border_style));
        }
    }

    // Bottom border: └─┴─┘
    lines.push(build_border_line(&col_widths, '└', '┴', '┘', border_style));

    Some(lines)
}

/// Wrap all cells in a row, returning wrapped lines per column.
fn wrap_row(cells: &[String], col_widths: &[usize], num_cols: usize) -> Vec<Vec<String>> {
    let mut wrapped_cols: Vec<Vec<String>> = Vec::new();
    for i in 0..num_cols {
        let content = cells.get(i).map(|s| s.as_str()).unwrap_or("");
        let width = col_widths.get(i).copied().unwrap_or(3);
        wrapped_cols.push(wrap_cell_text(content, width));
    }
    wrapped_cols
}

/// Render a wrapped row (potentially multi-line) into output lines.
fn render_wrapped_row(
    wrapped_cols: &[Vec<String>],
    col_widths: &[usize],
    cell_style: Style,
    border_style: Style,
    output: &mut Vec<Line<'static>>,
) {
    let max_lines = wrapped_cols.iter().map(|c| c.len()).max().unwrap_or(1);

    for line_idx in 0..max_lines {
        let mut spans: Vec<Span<'static>> = Vec::new();
        spans.push(Span::styled("\u{2502}", border_style));

        for (col_idx, col_width) in col_widths.iter().enumerate() {
            let cell_text = wrapped_cols
                .get(col_idx)
                .and_then(|lines| lines.get(line_idx))
                .map(|s| s.as_str())
                .unwrap_or("");

            spans.push(Span::styled(" ", cell_style));
            spans.extend(render_cell_spans(cell_text, *col_width, cell_style));
            spans.push(Span::styled(" ", cell_style));
            spans.push(Span::styled("\u{2502}", border_style));
        }

        output.push(Line::from(spans));
    }
}

/// Build a border line (top, separator, or bottom).
fn build_border_line(
    col_widths: &[usize],
    left: char,
    mid: char,
    right: char,
    style: Style,
) -> Line<'static> {
    let mut parts = vec![left.to_string()];
    for (i, &w) in col_widths.iter().enumerate() {
        parts.push("\u{2500}".repeat(w + 2)); // +2 for cell padding
        if i < col_widths.len() - 1 {
            parts.push(mid.to_string());
        }
    }
    parts.push(right.to_string());
    Line::from(Span::styled(parts.join(""), style))
}
