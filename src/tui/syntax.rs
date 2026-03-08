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

    /// Find a syntax definition by language string (case-insensitive).
    /// Tries: extension, name (case-insensitive), then common aliases.
    fn find_syntax(&self, language: &str) -> Option<&syntect::parsing::SyntaxReference> {
        let lang_lower = language.to_lowercase();

        // Try by extension first (e.g., "rs", "py", "js")
        if let Some(s) = self.syntax_set.find_syntax_by_extension(&lang_lower) {
            return Some(s);
        }

        // Try case-insensitive name match (e.g., "python" -> "Python", "rust" -> "Rust")
        for s in self.syntax_set.syntaxes() {
            if s.name.to_lowercase() == lang_lower {
                return Some(s);
            }
        }

        // Try common aliases
        let alias = match lang_lower.as_str() {
            "js" | "javascript" => "JavaScript",
            "ts" | "typescript" => "TypeScript",
            "py" => "Python",
            "rb" => "Ruby",
            "rs" => "Rust",
            "sh" | "bash" | "zsh" | "shell" => "Bourne Again Shell (bash)",
            "yml" => "YAML",
            "md" => "Markdown",
            "dockerfile" => "Dockerfile",
            "tf" | "hcl" => "Terraform",
            "cs" | "csharp" => "C#",
            "cpp" | "c++" => "C++",
            "objc" | "objective-c" => "Objective-C",
            "kt" | "kotlin" => "Kotlin",
            "jsx" => "JavaScript (JSX)",
            "tsx" => "TypeScript (TSX)",
            _ => return None,
        };
        self.syntax_set.find_syntax_by_name(alias)
    }

    /// Highlight code lines with the given language and theme.
    /// Returns styled Lines padded to `line_width`, or None if the language is not recognized.
    pub fn highlight_code(
        &self,
        code_lines: &[&str],
        language: &str,
        theme_name: &str,
        bg_color: RatColor,
        line_width: usize,
    ) -> Option<Vec<Line<'static>>> {
        let syntax = self.find_syntax(language)?;
        let theme = self.theme_set.themes.get(theme_name)?;
        let mut highlighter = HighlightLines::new(syntax, theme);

        let fallback_style = Style::default()
            .fg(RatColor::Rgb(130, 170, 200))
            .bg(bg_color);

        let mut lines = Vec::new();
        for code_line in code_lines {
            let mut spans: Vec<Span<'static>> = Vec::new();
            let mut line_len = 0usize;

            // syntect with load_defaults_newlines requires trailing newlines
            let line_with_nl = format!("{}\n", code_line);
            match highlighter.highlight_line(&line_with_nl, &self.syntax_set) {
                Ok(regions) => {
                    for (style, text) in regions {
                        let fg = RatColor::Rgb(
                            style.foreground.r,
                            style.foreground.g,
                            style.foreground.b,
                        );
                        line_len += text.chars().count();
                        spans.push(Span::styled(
                            text.to_string(),
                            Style::default().fg(fg).bg(bg_color),
                        ));
                    }
                }
                Err(_) => {
                    // Per-line fallback: use single color for this line
                    line_len = code_line.chars().count();
                    spans.push(Span::styled(code_line.to_string(), fallback_style));
                }
            }

            // Pad to full width so background spans the entire block
            if line_len < line_width {
                spans.push(Span::styled(
                    " ".repeat(line_width - line_len),
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
