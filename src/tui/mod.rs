pub mod input;
pub mod view;

use std::io::stdout;
use std::time::{Duration, Instant};

use crossterm::event::{self, Event};
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use crossterm::execute;
use ratatui::prelude::*;

use crate::clipboard;
use crate::discovery::{get_claude_home, load_session_prompts};
use crate::filter::fuzzy_filter;
use crate::session::{Session, UserPrompt};

use input::handle_input;

/// TUI interaction mode.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Mode {
    Browsing,
    Filtering,
    Detail,
}

/// What the input handler tells the main loop to do.
pub enum Action {
    Continue,
    Quit,
    EnterDetail(usize),
    CopyCommand(String),
    BackToList,
}

/// State for the detail view of a single session.
pub struct DetailState {
    pub session_idx: usize,
    pub prompts: Vec<UserPrompt>,
    pub selected: usize,
    pub scroll_offset: usize,
}

/// Application state for the TUI.
pub struct App {
    pub sessions: Vec<Session>,
    pub filtered_indices: Vec<usize>,
    pub selected: usize,
    pub scroll_offset: usize,
    pub mode: Mode,
    pub filter_query: String,
    pub status_message: Option<(String, Instant)>,
    pub detail: Option<DetailState>,
}

impl App {
    pub fn new(sessions: Vec<Session>) -> Self {
        let filtered_indices: Vec<usize> = (0..sessions.len()).collect();
        Self {
            sessions,
            filtered_indices,
            selected: 0,
            scroll_offset: 0,
            mode: Mode::Browsing,
            filter_query: String::new(),
            status_message: None,
            detail: None,
        }
    }

    /// Re-run the fuzzy filter and update `filtered_indices`.
    pub fn apply_filter(&mut self) {
        if self.filter_query.is_empty() {
            self.filtered_indices = (0..self.sessions.len()).collect();
        } else {
            let matches = fuzzy_filter(&self.sessions, &self.filter_query);
            self.filtered_indices = matches.into_iter().map(|(idx, _)| idx).collect();
        }
        self.selected = 0;
        self.scroll_offset = 0;
    }

    /// Move the selection cursor down, clamped to bounds.
    pub fn move_down(&mut self) {
        let len = match &self.detail {
            Some(d) => d.prompts.len(),
            None => self.filtered_indices.len(),
        };
        if len > 0 {
            match &mut self.detail {
                Some(d) => d.selected = (d.selected + 1).min(len - 1),
                None => self.selected = (self.selected + 1).min(len - 1),
            }
        }
    }

    /// Move the selection cursor up, clamped to bounds.
    pub fn move_up(&mut self) {
        match &mut self.detail {
            Some(d) => d.selected = d.selected.saturating_sub(1),
            None => self.selected = self.selected.saturating_sub(1),
        }
    }

    /// Ensure the selected item is visible by adjusting scroll_offset.
    pub fn ensure_visible(&mut self, visible_items: usize) {
        if visible_items == 0 {
            return;
        }
        match &mut self.detail {
            Some(d) => {
                if d.selected < d.scroll_offset {
                    d.scroll_offset = d.selected;
                } else if d.selected >= d.scroll_offset + visible_items {
                    d.scroll_offset = d.selected - visible_items + 1;
                }
            }
            None => {
                if self.selected < self.scroll_offset {
                    self.scroll_offset = self.selected;
                } else if self.selected >= self.scroll_offset + visible_items {
                    self.scroll_offset = self.selected - visible_items + 1;
                }
            }
        }
    }

    /// Enter detail mode for a session.
    pub fn enter_detail(&mut self, session_idx: usize) {
        let session = &self.sessions[session_idx];
        let claude_home = get_claude_home();
        let prompts = load_session_prompts(&claude_home, session, 20);

        let selected = if prompts.is_empty() { 0 } else { prompts.len() - 1 };
        self.detail = Some(DetailState {
            session_idx,
            prompts,
            selected,
            scroll_offset: 0,
        });
        self.mode = Mode::Detail;
    }

    /// Leave detail mode and return to the list.
    pub fn leave_detail(&mut self) {
        self.detail = None;
        self.mode = Mode::Browsing;
    }

    /// Set a status message that disappears after a few seconds.
    pub fn set_status(&mut self, msg: String) {
        self.status_message = Some((msg, Instant::now()));
    }

    /// Clear expired status messages.
    pub fn tick_status(&mut self) {
        if let Some((_, when)) = &self.status_message {
            if when.elapsed() > Duration::from_secs(3) {
                self.status_message = None;
            }
        }
    }
}

/// Run the interactive TUI session picker.
pub fn run(sessions: Vec<Session>) -> Result<(), Box<dyn std::error::Error>> {
    if sessions.is_empty() {
        eprintln!("No sessions found.");
        return Ok(());
    }

    // Set up panic hook to restore terminal
    let original_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |panic_info| {
        let _ = disable_raw_mode();
        let _ = execute!(stdout(), LeaveAlternateScreen);
        original_hook(panic_info);
    }));

    enable_raw_mode()?;
    let mut stdout = stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app = App::new(sessions);
    let mut deferred_command: Option<String> = None;

    loop {
        app.tick_status();
        terminal.draw(|frame| {
            let height = frame.area().height.saturating_sub(2) as usize;
            let visible = if app.mode == Mode::Detail {
                height // 1 line per prompt in detail
            } else {
                height / 2 // 2 lines per session in list
            };
            app.ensure_visible(visible);
            view::render(frame, &app);
        })?;

        if event::poll(Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                match handle_input(&mut app, key) {
                    Action::Quit => break,
                    Action::EnterDetail(idx) => {
                        app.enter_detail(idx);
                    }
                    Action::CopyCommand(cmd) => {
                        match clipboard::copy_to_clipboard(&cmd) {
                            Ok(()) => {
                                app.set_status("Copied to clipboard!".to_string());
                            }
                            Err(_) => {
                                deferred_command = Some(cmd);
                                app.set_status(
                                    "Clipboard unavailable, will print on exit".to_string(),
                                );
                            }
                        }
                    }
                    Action::BackToList => {
                        app.leave_detail();
                    }
                    Action::Continue => {}
                }
            }
        }
    }

    // Restore terminal
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    // Print deferred command if clipboard failed
    if let Some(cmd) = deferred_command {
        println!("{cmd}");
    }

    Ok(())
}
