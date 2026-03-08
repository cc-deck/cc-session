use std::time::Duration;

use ratatui::style::{Color, Style};

/// Centralized color theme for all visual elements.
#[allow(dead_code)]
pub struct Theme {
    pub name: &'static str,
    // Role headers
    pub user_header_bg: Color,
    pub user_header_fg: Color,
    pub assistant_header_bg: Color,
    pub assistant_header_fg: Color,
    // Message bodies
    pub user_message_bg: Color,
    // Code blocks
    pub code_block_bg: Color,
    // Text
    pub text: Color,
    pub text_dim: Color,
    pub heading: Color,
    pub heading_bg: Color,
    pub separator: Color,
    // Tables
    pub table_border: Color,
    pub table_header: Style,
    // Search
    pub search_highlight_bg: Color,
    // Status bar
    pub status_bar_bg: Color,
    pub status_label_bg: Color,
    pub status_label_fg: Color,
    // Session list
    pub selected_bg: Color,
    pub cursor_color: Color,
    // Syntect theme name
    pub syntect_theme: &'static str,
}

impl Theme {
    /// Dark theme optimized for dark terminal backgrounds.
    pub fn dark() -> Self {
        Self {
            name: "dark",
            user_header_bg: Color::Rgb(25, 45, 55),
            user_header_fg: Color::Rgb(100, 200, 220),
            assistant_header_bg: Color::Rgb(45, 40, 25),
            assistant_header_fg: Color::Rgb(220, 190, 100),
            user_message_bg: Color::Rgb(25, 30, 40),
            code_block_bg: Color::Rgb(20, 25, 35),
            text: Color::Reset,
            text_dim: Color::DarkGray,
            heading: Color::Green,
            heading_bg: Color::Rgb(20, 35, 25),
            separator: Color::DarkGray,
            table_border: Color::DarkGray,
            table_header: Style::default().fg(Color::White).add_modifier(ratatui::style::Modifier::BOLD),
            search_highlight_bg: Color::Rgb(100, 80, 0),
            status_bar_bg: Color::Rgb(20, 20, 30),
            status_label_bg: Color::Cyan,
            status_label_fg: Color::Black,
            selected_bg: Color::Rgb(30, 30, 50),
            cursor_color: Color::Cyan,
            syntect_theme: "base16-eighties.dark",
        }
    }

    /// Light theme optimized for light terminal backgrounds.
    pub fn light() -> Self {
        Self {
            name: "light",
            user_header_bg: Color::Rgb(215, 235, 245),
            user_header_fg: Color::Rgb(0, 100, 130),
            assistant_header_bg: Color::Rgb(245, 235, 215),
            assistant_header_fg: Color::Rgb(130, 100, 0),
            user_message_bg: Color::Rgb(235, 240, 248),
            code_block_bg: Color::Rgb(245, 247, 250),
            text: Color::Black,
            text_dim: Color::DarkGray,
            heading: Color::Rgb(0, 120, 0),
            heading_bg: Color::Rgb(230, 245, 232),
            separator: Color::Rgb(180, 180, 180),
            table_border: Color::Rgb(150, 150, 150),
            table_header: Style::default().fg(Color::Black).add_modifier(ratatui::style::Modifier::BOLD),
            search_highlight_bg: Color::Rgb(255, 230, 150),
            status_bar_bg: Color::Rgb(230, 230, 235),
            status_label_bg: Color::Rgb(0, 130, 150),
            status_label_fg: Color::White,
            selected_bg: Color::Rgb(210, 220, 240),
            cursor_color: Color::Rgb(0, 130, 150),
            syntect_theme: "InspiredGitHub",
        }
    }

    /// Detect terminal background and return the appropriate theme.
    /// Falls back to dark theme if detection fails or times out.
    pub fn detect() -> Self {
        let timeout = Duration::from_millis(100);
        match termbg::theme(timeout) {
            Ok(termbg::Theme::Light) => Self::light(),
            Ok(termbg::Theme::Dark) | Err(_) => Self::dark(),
        }
    }
}
