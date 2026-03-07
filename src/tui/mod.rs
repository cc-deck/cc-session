pub mod input;
pub mod view;

use std::io::stdout;
use std::sync::mpsc;
use std::time::{Duration, Instant};

use crossterm::event::{self, Event};
use crossterm::execute;
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use ratatui::prelude::*;

use crate::clipboard;
use crate::discovery::{get_claude_home, load_conversation};
use crate::filter::filter_sessions;
use crate::search;
use crate::session::{ConversationMessage, Session};

use input::handle_input;

/// TUI interaction mode.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Mode {
    Browsing,
    Filtering,
    DeepSearchInput,
    DeepSearching,
    Conversation,
    ConversationSearch,
}

/// What the input handler tells the main loop to do.
pub enum Action {
    Continue,
    Quit,
    EnterConversation(usize),
    CopyCommand(String),
    BackToList,
    StartDeepSearchInput,
    DeepSearch(String),
    RestoreOriginal,
}

/// State for the conversation viewer.
pub struct ConversationState {
    pub session_idx: usize,
    pub messages: Vec<ConversationMessage>,
    pub lines: Vec<Line<'static>>,
    pub scroll_offset: usize,
    pub page_height: usize,
    pub rendered_width: u16,
    pub search_query: String,
    pub search_active: bool,
    pub search_confirmed: bool,
    pub match_positions: Vec<usize>,
    pub current_match: usize,
    pub initial_search_terms: Vec<String>,
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
    pub conversation: Option<ConversationState>,
    /// Original sessions saved before a deep search, so we can restore them.
    pub original_sessions: Option<Vec<Session>>,
    /// The query that produced the current deep search results.
    pub deep_search_query: Option<String>,
    /// Receiver for background deep search results.
    pub search_receiver: Option<mpsc::Receiver<Vec<Session>>>,
    /// Spinner frame counter for deep search progress.
    pub spinner_tick: usize,
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
            conversation: None,
            original_sessions: None,
            deep_search_query: None,
            search_receiver: None,
            spinner_tick: 0,
        }
    }

    /// Re-run the filter and update `filtered_indices`.
    pub fn apply_filter(&mut self) {
        self.filtered_indices = filter_sessions(&self.sessions, &self.filter_query);
        self.selected = 0;
        self.scroll_offset = 0;
    }

    /// Restore original sessions after a deep search.
    pub fn restore_original_sessions(&mut self) {
        if let Some(original) = self.original_sessions.take() {
            self.sessions = original;
            self.deep_search_query = None;
            self.filtered_indices = (0..self.sessions.len()).collect();
            self.selected = 0;
            self.scroll_offset = 0;
            self.filter_query.clear();
            self.mode = Mode::Browsing;
        }
    }

    /// Whether the app is currently showing deep search results.
    pub fn is_deep_search(&self) -> bool {
        self.original_sessions.is_some()
    }

    /// Move the selection cursor down, clamped to bounds.
    pub fn move_down(&mut self) {
        if !self.filtered_indices.is_empty() {
            self.selected = (self.selected + 1).min(self.filtered_indices.len() - 1);
        }
    }

    /// Move the selection cursor up, clamped to bounds.
    pub fn move_up(&mut self) {
        self.selected = self.selected.saturating_sub(1);
    }

    /// Ensure the selected item is visible by adjusting scroll_offset.
    pub fn ensure_visible(&mut self, visible_items: usize) {
        if visible_items == 0 {
            return;
        }
        if self.mode == Mode::Conversation || self.mode == Mode::ConversationSearch {
            // Conversation view manages its own scroll
            return;
        }
        if self.selected < self.scroll_offset {
            self.scroll_offset = self.selected;
        } else if self.selected >= self.scroll_offset + visible_items {
            self.scroll_offset = self.selected - visible_items + 1;
        }
    }

    /// Enter conversation viewer for a session.
    pub fn enter_conversation(&mut self, session_idx: usize) {
        let session = &self.sessions[session_idx];
        let claude_home = get_claude_home();
        let messages = load_conversation(&claude_home, session);

        // Collect search terms from filter or deep search
        let initial_search_terms = if !self.filter_query.is_empty() {
            self.filter_query.split_whitespace().map(String::from).collect()
        } else if let Some(query) = &self.deep_search_query {
            query.split_whitespace().map(String::from).collect()
        } else {
            Vec::new()
        };

        self.conversation = Some(ConversationState {
            session_idx,
            messages,
            lines: Vec::new(),
            scroll_offset: 0,
            page_height: 20,
            rendered_width: 0,
            search_query: String::new(),
            search_active: false,
            search_confirmed: false,
            match_positions: Vec::new(),
            current_match: 0,
            initial_search_terms,
        });
        self.mode = Mode::Conversation;
    }

    /// Leave conversation viewer and return to the list.
    pub fn leave_conversation(&mut self) {
        self.conversation = None;
        self.mode = Mode::Browsing;
    }

    /// Set a status message that disappears after a few seconds.
    pub fn set_status(&mut self, msg: String) {
        self.status_message = Some((msg, Instant::now()));
    }

    /// Spinner character for the current tick.
    pub fn spinner_char(&self) -> char {
        const FRAMES: &[char] = &['\u{280B}', '\u{2819}', '\u{2839}', '\u{2838}', '\u{283C}', '\u{2834}', '\u{2826}', '\u{2827}', '\u{2807}', '\u{280F}'];
        FRAMES[self.spinner_tick % FRAMES.len()]
    }

    /// Check if the background search has completed. Returns true if results were received.
    pub fn poll_search(&mut self) -> bool {
        if let Some(rx) = &self.search_receiver {
            match rx.try_recv() {
                Ok(results) => {
                    self.search_receiver = None;
                    let query = self.deep_search_query.clone().unwrap_or_default();
                    if results.is_empty() {
                        self.set_status(format!("No sessions match \"{}\"", query));
                        self.mode = Mode::Browsing;
                        self.filter_query.clear();
                    } else {
                        let count = results.len();
                        if self.original_sessions.is_none() {
                            self.original_sessions = Some(std::mem::take(&mut self.sessions));
                        }
                        self.sessions = results;
                        self.filtered_indices = (0..self.sessions.len()).collect();
                        self.selected = 0;
                        self.scroll_offset = 0;
                        self.mode = Mode::Browsing;
                        self.filter_query.clear();
                        self.set_status(format!(
                            "Deep search: {} sessions match \"{}\"",
                            count, query
                        ));
                    }
                    true
                }
                Err(mpsc::TryRecvError::Empty) => {
                    self.spinner_tick = self.spinner_tick.wrapping_add(1);
                    false
                }
                Err(mpsc::TryRecvError::Disconnected) => {
                    self.search_receiver = None;
                    self.set_status("Search failed".to_string());
                    self.mode = Mode::Browsing;
                    true
                }
            }
        } else {
            false
        }
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
        if app.mode == Mode::DeepSearching {
            app.poll_search();
        }
        terminal.draw(|frame| {
            let height = frame.area().height.saturating_sub(2) as usize;
            let visible = height;
            app.ensure_visible(visible);
            view::render(frame, &mut app);
        })?;

        if event::poll(Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                match handle_input(&mut app, key) {
                    Action::Quit => break,
                    Action::EnterConversation(idx) => {
                        app.enter_conversation(idx);
                    }
                    Action::CopyCommand(cmd) => match clipboard::copy_to_clipboard(&cmd) {
                        Ok(()) => break,
                        Err(_) => {
                            deferred_command = Some(cmd);
                            break;
                        }
                    },
                    Action::StartDeepSearchInput => {
                        app.mode = Mode::DeepSearchInput;
                        // Keep current filter_query so user can refine it
                    }
                    Action::DeepSearch(pattern) => {
                        app.deep_search_query = Some(pattern.clone());
                        app.mode = Mode::DeepSearching;
                        app.spinner_tick = 0;

                        let (tx, rx) = mpsc::channel();
                        app.search_receiver = Some(rx);

                        let claude_home = get_claude_home();
                        std::thread::spawn(move || {
                            let results = search::deep_search(&claude_home, &pattern);
                            let _ = tx.send(results);
                        });
                    }
                    Action::BackToList => {
                        app.leave_conversation();
                    }
                    Action::RestoreOriginal => {
                        app.restore_original_sessions();
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
