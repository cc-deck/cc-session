use ratatui::prelude::*;
use ratatui::style::Color as RatColor;
use syntect::easy::HighlightLines;
use syntect::highlighting::ThemeSet;
use syntect::parsing::SyntaxSet;

/// Syntax highlighter using syntect for code block rendering.
pub struct SyntaxHighlighter {
    syntax_set: SyntaxSet,
    theme_set: ThemeSet,
}

impl Default for SyntaxHighlighter {
    fn default() -> Self {
        Self::new()
    }
}

impl SyntaxHighlighter {
    /// Create a new highlighter with default syntax definitions and themes.
    pub fn new() -> Self {
        Self {
            syntax_set: SyntaxSet::load_defaults_newlines(),
            theme_set: ThemeSet::load_defaults(),
        }
    }

    /// Highlight code lines with the given language and theme.
    /// Returns styled Lines, or None if the language is not recognized.
    pub fn highlight_code(
        &self,
        code_lines: &[&str],
        language: &str,
        theme_name: &str,
        bg_color: RatColor,
    ) -> Option<Vec<Line<'static>>> {
        // Try to find syntax by extension, then by name
        let syntax = self
            .syntax_set
            .find_syntax_by_extension(language)
            .or_else(|| self.syntax_set.find_syntax_by_name(language))
            .or_else(|| {
                // Try common aliases
                let alias = match language {
                    "js" => "JavaScript",
                    "ts" => "TypeScript",
                    "py" => "Python",
                    "rb" => "Ruby",
                    "rs" => "Rust",
                    "sh" | "bash" | "zsh" => "Bourne Again Shell (bash)",
                    "yml" => "YAML",
                    "md" => "Markdown",
                    "dockerfile" => "Dockerfile",
                    "tf" => "Terraform",
                    _ => return None,
                };
                self.syntax_set.find_syntax_by_name(alias)
            })?;

        let theme = self.theme_set.themes.get(theme_name)?;
        let mut highlighter = HighlightLines::new(syntax, theme);

        let mut lines = Vec::new();
        for code_line in code_lines {
            let regions = highlighter
                .highlight_line(code_line, &self.syntax_set)
                .ok()?;

            let mut spans: Vec<Span<'static>> = Vec::new();
            for (style, text) in regions {
                let fg = RatColor::Rgb(style.foreground.r, style.foreground.g, style.foreground.b);
                spans.push(Span::styled(
                    text.to_string(),
                    Style::default().fg(fg).bg(bg_color),
                ));
            }

            if spans.is_empty() {
                spans.push(Span::styled(
                    code_line.to_string(),
                    Style::default().bg(bg_color),
                ));
            }

            lines.push(Line::from(spans));
        }

        Some(lines)
    }
}

/// Extract the language tag from a code fence opening line.
/// Handles formats like: ```rust, ```rust,ignore, ```python
pub fn extract_language(fence_line: &str) -> Option<String> {
    let trimmed = fence_line.trim();
    let after_backticks = trimmed.trim_start_matches('`');
    if after_backticks.is_empty() {
        return None;
    }
    // Take the first word (split on whitespace or comma)
    let lang = after_backticks
        .split(|c: char| c.is_whitespace() || c == ',')
        .next()?
        .to_lowercase();
    if lang.is_empty() {
        None
    } else {
        Some(lang)
    }
}
