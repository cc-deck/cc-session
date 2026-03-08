use ratatui::prelude::*;

use crate::theme::Theme;

/// A parsed markdown table.
struct Table {
    headers: Vec<String>,
    rows: Vec<Vec<String>>,
    col_widths: Vec<usize>,
}

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
    // Remove leading and trailing pipes
    let inner = trimmed
        .strip_prefix('|')
        .unwrap_or(trimmed)
        .strip_suffix('|')
        .unwrap_or(trimmed);
    inner.split('|').map(|cell| cell.trim().to_string()).collect()
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
        if !has_header && i == 0 {
            headers = cells;
        } else if has_header && headers.is_empty() {
            // Shouldn't happen, but handle gracefully
            headers = cells;
        } else {
            rows.push(cells);
        }
    }

    if headers.is_empty() && rows.is_empty() {
        return None;
    }

    // If no separator was found, treat first row as header anyway
    if !has_header && !rows.is_empty() {
        // headers is already set from first line
    }

    let num_cols = headers.len().max(rows.iter().map(|r| r.len()).max().unwrap_or(0));

    // Compute column widths
    let mut col_widths = vec![0usize; num_cols];
    for (i, h) in headers.iter().enumerate() {
        if i < col_widths.len() {
            col_widths[i] = col_widths[i].max(h.chars().count());
        }
    }
    for row in &rows {
        for (i, cell) in row.iter().enumerate() {
            if i < col_widths.len() {
                col_widths[i] = col_widths[i].max(cell.chars().count());
            }
        }
    }

    // Ensure minimum width of 1
    for w in &mut col_widths {
        if *w == 0 {
            *w = 1;
        }
    }

    Some(Table {
        headers,
        rows,
        col_widths,
    })
}

/// Render a parsed table as styled Lines with box-drawing characters.
pub fn render_table_lines(
    table_lines: &[&str],
    max_width: usize,
    theme: &Theme,
) -> Option<Vec<Line<'static>>> {
    let table = parse_table(table_lines)?;

    // Check if table fits; truncate cells if needed
    let border_chars = table.col_widths.len() + 1; // |col|col| = n+1 pipes
    let padding = table.col_widths.len() * 2; // 1 space padding on each side of cell
    let content_width: usize = table.col_widths.iter().sum();
    let total_width = border_chars + padding + content_width;

    let col_widths = if total_width > max_width && max_width > border_chars + padding {
        // Shrink columns proportionally
        let available = max_width - border_chars - padding;
        let ratio = available as f64 / content_width as f64;
        table
            .col_widths
            .iter()
            .map(|&w| (w as f64 * ratio).max(1.0) as usize)
            .collect::<Vec<_>>()
    } else {
        table.col_widths.clone()
    };

    let border_style = Style::default().fg(theme.table_border);
    let mut lines = Vec::new();

    // Top border: ┌─┬─┐
    lines.push(build_border_line(&col_widths, '┌', '┬', '┐', border_style));

    // Header row
    if !table.headers.is_empty() {
        lines.push(build_data_line(
            &table.headers,
            &col_widths,
            theme.table_header,
            border_style,
        ));
        // Header separator: ├─┼─┤
        lines.push(build_border_line(&col_widths, '├', '┼', '┤', border_style));
    }

    // Data rows
    for row in &table.rows {
        lines.push(build_data_line(
            row,
            &col_widths,
            Style::default().fg(theme.text),
            border_style,
        ));
    }

    // Bottom border: └─┴─┘
    lines.push(build_border_line(&col_widths, '└', '┴', '┘', border_style));

    Some(lines)
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
        parts.push("\u{2500}".repeat(w + 2)); // +2 for padding
        if i < col_widths.len() - 1 {
            parts.push(mid.to_string());
        }
    }
    parts.push(right.to_string());
    Line::from(Span::styled(parts.join(""), style))
}

/// Build a data line with cell contents.
fn build_data_line(
    cells: &[String],
    col_widths: &[usize],
    cell_style: Style,
    border_style: Style,
) -> Line<'static> {
    let mut spans: Vec<Span<'static>> = Vec::new();
    spans.push(Span::styled("\u{2502}", border_style));

    for (i, width) in col_widths.iter().enumerate() {
        let content = cells.get(i).map(|s| s.as_str()).unwrap_or("");
        let chars: Vec<char> = content.chars().collect();
        let display = if chars.len() > *width {
            if *width > 3 {
                format!("{}...", chars[..*width - 3].iter().collect::<String>())
            } else {
                chars[..*width].iter().collect()
            }
        } else {
            let padding = " ".repeat(width - chars.len());
            format!("{}{}", content, padding)
        };
        spans.push(Span::styled(format!(" {} ", display), cell_style));
        spans.push(Span::styled("\u{2502}", border_style));
    }

    Line::from(spans)
}
