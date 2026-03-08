pub mod input;
pub mod syntax;
pub mod table;
pub mod view;

use std::collections::{HashMap, HashSet};
use std::io::stdout;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{mpsc, Arc};
use std::time::{Duration, Instant};

use chrono::{DateTime, Utc};
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
use crate::theme::Theme;

use input::handle_input;

/// TUI interaction mode.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Mode {
    Browsing,
    Conversation,
    ConversationSearch,
}

/// Phase of the background content search.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ContentSearchState {
    Idle,
    Debouncing,
    Searching,
    Complete,
}

/// How a session matched the search query.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MatchType {
    Metadata,
    Content,
    Both,
}

/// Reference to a session in either the main sessions list or content results.
#[derive(Debug, Clone)]
pub enum DisplaySource {
    Sessions(usize),
    Content(usize),
}

/// A single entry in the merged search results display.
#[derive(Debug, Clone)]
pub struct DisplayEntry {
    pub match_type: MatchType,
    pub source: DisplaySource,
    pub timestamp: DateTime<Utc>,
}

/// What the input handler tells the main loop to do.
pub enum Action {
    Continue,
    Quit,
    EnterConversation(usize),
    CopyCommand(String),
    BackToList,
}

/// State for the conversation viewer.
pub struct ConversationState {
    pub session: Session,
    pub messages: Vec<ConversationMessage>,
    pub lines: Vec<Line<'static>>,
    pub scroll_offset: usize,
    pub page_height: usize,
    pub rendered_width: u16,
    pub search_query: String,
    pub search_active: bool,
    pub search_confirmed: bool,
    /// First keystroke in search replaces the pre-filled text
    pub search_replacing: bool,
    pub match_positions: Vec<usize>,
    pub current_match: usize,
    pub initial_search_terms: Vec<String>,
}

/// Application state for the TUI.
pub struct App {
    pub sessions: Vec<Session>,
    pub filtered_indices: Vec<usize>,
    pub display_entries: Vec<DisplayEntry>,
    pub selected: usize,
    pub scroll_offset: usize,
    pub mode: Mode,
    pub filter_query: String,
    pub status_message: Option<(String, Instant)>,
    pub conversation: Option<ConversationState>,
    /// Content-only search results from background search.
    pub content_results: Vec<Session>,
    /// Current phase of content search.
    pub content_search_state: ContentSearchState,
    /// When the last filter keystroke occurred, for debounce.
    pub last_keystroke: Option<Instant>,
    /// Flag to cancel in-progress content search.
    pub cancel_flag: Arc<AtomicBool>,
    /// Receiver for background content search results.
    pub search_receiver: Option<mpsc::Receiver<Vec<Session>>>,
    /// Spinner frame counter.
    pub spinner_tick: usize,
    /// Pre-built file-path-to-session index for fast content search.
    pub session_index: Arc<HashMap<PathBuf, Session>>,
    /// Active color theme.
    pub theme: Theme,
    /// Syntax highlighter for code blocks.
    pub syntax_highlighter: syntax::SyntaxHighlighter,
}

impl App {
    pub fn new(sessions: Vec<Session>, session_index: HashMap<PathBuf, Session>, theme: Theme) -> Self {
        let filtered_indices: Vec<usize> = (0..sessions.len()).collect();
        let display_entries: Vec<DisplayEntry> = filtered_indices
            .iter()
            .map(|&idx| DisplayEntry {
                match_type: MatchType::Metadata,
                source: DisplaySource::Sessions(idx),
                timestamp: sessions[idx].timestamp,
            })
            .collect();
        Self {
            sessions,
            filtered_indices,
            display_entries,
            selected: 0,
            scroll_offset: 0,
            mode: Mode::Browsing,
            filter_query: String::new(),
            status_message: None,
            conversation: None,
            content_results: Vec::new(),
            content_search_state: ContentSearchState::Idle,
            last_keystroke: None,
            cancel_flag: Arc::new(AtomicBool::new(false)),
            search_receiver: None,
            spinner_tick: 0,
            session_index: Arc::new(session_index),
            theme,
            syntax_highlighter: syntax::SyntaxHighlighter::new(),
        }
    }

    /// Re-run the metadata filter and rebuild display entries.
    pub fn apply_filter(&mut self) {
        self.filtered_indices = filter_sessions(&self.sessions, &self.filter_query);
        self.rebuild_display_entries();
        self.selected = 0;
        self.scroll_offset = 0;
    }

    /// Build merged display entries from metadata matches and content results.
    pub fn rebuild_display_entries(&mut self) {
        let content_ids: HashSet<&str> = self
            .content_results
            .iter()
            .map(|s| s.id.as_str())
            .collect();
        let metadata_ids: HashSet<&str> = self
            .filtered_indices
            .iter()
            .map(|&idx| self.sessions[idx].id.as_str())
            .collect();

        let mut entries = Vec::new();

        for &idx in &self.filtered_indices {
            let session = &self.sessions[idx];
            let match_type = if content_ids.contains(session.id.as_str()) {
                MatchType::Both
            } else {
                MatchType::Metadata
            };
            entries.push(DisplayEntry {
                match_type,
                source: DisplaySource::Sessions(idx),
                timestamp: session.timestamp,
            });
        }

        for (i, session) in self.content_results.iter().enumerate() {
            if !metadata_ids.contains(session.id.as_str()) {
                entries.push(DisplayEntry {
                    match_type: MatchType::Content,
                    source: DisplaySource::Content(i),
                    timestamp: session.timestamp,
                });
            }
        }

        entries.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
        self.display_entries = entries;
    }

    /// Get the session referenced by a display entry.
    pub fn display_session(&self, entry: &DisplayEntry) -> &Session {
        match &entry.source {
            DisplaySource::Sessions(idx) => &self.sessions[*idx],
            DisplaySource::Content(idx) => &self.content_results[*idx],
        }
    }

    /// Cancel any in-progress content search and clear results.
    pub fn cancel_content_search(&mut self) {
        self.cancel_flag.store(true, Ordering::Relaxed);
        self.search_receiver = None;
        self.content_results.clear();
        self.content_search_state = ContentSearchState::Idle;
        self.last_keystroke = None;
    }

    /// Move the selection cursor down, clamped to bounds.
    pub fn move_down(&mut self) {
        if !self.display_entries.is_empty() {
            self.selected = (self.selected + 1).min(self.display_entries.len() - 1);
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
            return;
        }
        if self.selected < self.scroll_offset {
            self.scroll_offset = self.selected;
        } else if self.selected >= self.scroll_offset + visible_items {
            self.scroll_offset = self.selected - visible_items + 1;
        }
    }

    /// Enter conversation viewer for a display entry.
    pub fn enter_conversation(&mut self, display_idx: usize) {
        if display_idx >= self.display_entries.len() {
            return;
        }
        let entry = &self.display_entries[display_idx];
        let session = self.display_session(entry).clone();
        let claude_home = get_claude_home();
        let messages = load_conversation(&claude_home, &session);

        let initial_search_terms: Vec<String> = {
            let trimmed = self.filter_query.trim();
            if !trimmed.is_empty() {
                vec![trimmed.to_string()]
            } else {
                Vec::new()
            }
        };

        self.conversation = Some(ConversationState {
            session,
            messages,
            lines: Vec::new(),
            scroll_offset: 0,
            page_height: 20,
            rendered_width: 0,
            search_query: String::new(),
            search_active: false,
            search_confirmed: false,
            search_replacing: false,
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
    #[allow(dead_code)]
    pub fn set_status(&mut self, msg: String) {
        self.status_message = Some((msg, Instant::now()));
    }

    /// Spinner character for the current tick.
    pub fn spinner_char(&self) -> char {
        const FRAMES: &[char] = &[
            '\u{280B}', '\u{2819}', '\u{2839}', '\u{2838}', '\u{283C}', '\u{2834}', '\u{2826}',
            '\u{2827}', '\u{2807}', '\u{280F}',
        ];
        FRAMES[self.spinner_tick % FRAMES.len()]
    }

    /// Check if content search results have arrived.
    pub fn poll_content_search(&mut self) -> bool {
        if let Some(rx) = &self.search_receiver {
            match rx.try_recv() {
                Ok(results) => {
                    self.search_receiver = None;
                    let selected_id = self
                        .display_entries
                        .get(self.selected)
                        .map(|e| self.display_session(e).id.clone());

                    self.content_results = results;
                    self.content_search_state = ContentSearchState::Complete;
                    self.rebuild_display_entries();

                    if let Some(id) = selected_id {
                        if let Some(pos) = self
                            .display_entries
                            .iter()
                            .position(|e| self.display_session(e).id == id)
                        {
                            self.selected = pos;
                        }
                    }
                    true
                }
                Err(mpsc::TryRecvError::Empty) => {
                    self.spinner_tick = self.spinner_tick.wrapping_add(1);
                    false
                }
                Err(mpsc::TryRecvError::Disconnected) => {
                    self.search_receiver = None;
                    self.content_search_state = ContentSearchState::Complete;
                    true
                }
            }
        } else {
            false
        }
    }

    /// Check if debounce period has elapsed and start content search if so.
    pub fn check_debounce(&mut self) {
        if self.content_search_state != ContentSearchState::Debouncing {
            return;
        }
        if let Some(last) = self.last_keystroke {
            if last.elapsed() >= Duration::from_millis(300) && !self.filter_query.is_empty() {
                self.content_search_state = ContentSearchState::Searching;
                self.spinner_tick = 0;

                let cancel = Arc::new(AtomicBool::new(false));
                self.cancel_flag = Arc::clone(&cancel);

                let (tx, rx) = mpsc::channel();
                self.search_receiver = Some(rx);

                let claude_home = get_claude_home();
                let index = Arc::clone(&self.session_index);
                let pattern = self.filter_query.clone();
                std::thread::spawn(move || {
                    let results =
                        search::deep_search_indexed(&claude_home, &pattern, &index, &cancel);
                    let _ = tx.send(results);
                });
            }
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
pub fn run(sessions: Vec<Session>, theme: Theme) -> Result<(), Box<dyn std::error::Error>> {
    if sessions.is_empty() {
        eprintln!("No sessions found.");
        return Ok(());
    }

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

    let claude_home = get_claude_home();
    let session_index = search::build_session_index(&claude_home, &sessions);
    let mut app = App::new(sessions, session_index, theme);
    let mut deferred_command: Option<String> = None;

    loop {
        app.tick_status();

        if app.content_search_state == ContentSearchState::Searching {
            app.poll_content_search();
        }

        if app.mode == Mode::Browsing {
            app.check_debounce();
        }

        terminal.draw(|frame| {
            let height = frame.area().height.saturating_sub(2) as usize;
            app.ensure_visible(height);
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
                    Action::BackToList => {
                        app.leave_conversation();
                    }
                    Action::Continue => {}
                }
            }
        }
    }

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    if let Some(cmd) = deferred_command {
        println!("{cmd}");
    }

    Ok(())
}
