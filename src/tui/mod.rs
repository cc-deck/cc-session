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
use crossterm::event::{self, DisableMouseCapture, EnableMouseCapture, Event, MouseEventKind};
use crossterm::execute;
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use ratatui::prelude::*;

use crate::clipboard;
use crate::discovery::{get_claude_home, load_conversation};
use crate::filter::filter_sessions;
use crate::search::{self, SearchResult};
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
#[allow(dead_code)]
pub struct DisplayEntry {
    pub match_type: MatchType,
    pub source: DisplaySource,
    pub timestamp: DateTime<Utc>,
}

/// A project group header in the grouped view.
#[derive(Debug, Clone)]
pub struct ProjectGroup {
    pub project_name: String,
    pub latest_timestamp: DateTime<Utc>,
    pub session_count: usize,
}

/// An item in the display list: either a session entry or a project group header.
#[derive(Debug, Clone)]
pub enum DisplayItem {
    Session(DisplayEntry),
    Header(ProjectGroup),
}

/// What the input handler tells the main loop to do.
pub enum Action {
    Continue,
    Quit,
    EnterConversation(usize),
    CopyCommand(String),
    ExecCommand { cmd: String, cwd: String },
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
    /// Cursor position within search_query for editing
    pub search_cursor: usize,
    pub match_positions: Vec<usize>,
    pub current_match: usize,
    pub initial_search_terms: Vec<String>,
}

/// Application state for the TUI.
pub struct App {
    pub sessions: Vec<Session>,
    pub filtered_indices: Vec<usize>,
    pub display_items: Vec<DisplayItem>,
    pub selected: usize,
    pub scroll_offset: usize,
    pub mode: Mode,
    pub filter_query: String,
    /// Whether the filter UI indicator is shown (activated by / or typing)
    pub filter_active: bool,
    pub status_message: Option<(String, Instant)>,
    pub conversation: Option<ConversationState>,
    /// Content-only search results from background search.
    pub content_results: Vec<SearchResult>,
    /// Current phase of content search.
    pub content_search_state: ContentSearchState,
    /// When the last filter keystroke occurred, for debounce.
    pub last_keystroke: Option<Instant>,
    /// Flag to cancel in-progress content search.
    pub cancel_flag: Arc<AtomicBool>,
    /// Receiver for background content search results.
    pub search_receiver: Option<mpsc::Receiver<Vec<SearchResult>>>,
    /// Spinner frame counter.
    pub spinner_tick: usize,
    /// Pre-built file-path-to-session index for fast content search.
    pub session_index: Arc<HashMap<PathBuf, Session>>,
    /// Active color theme.
    pub theme: Theme,
    /// Syntax highlighter for code blocks.
    pub syntax_highlighter: syntax::SyntaxHighlighter,
    /// Whether the grouped (by project) view is active.
    pub grouped_mode: bool,
    /// Set of project names whose groups are currently expanded.
    pub expanded_projects: HashSet<String>,
}

impl App {
    pub fn new(sessions: Vec<Session>, session_index: HashMap<PathBuf, Session>, theme: Theme) -> Self {
        let filtered_indices: Vec<usize> = (0..sessions.len()).collect();
        let display_items: Vec<DisplayItem> = filtered_indices
            .iter()
            .map(|&idx| DisplayItem::Session(DisplayEntry {
                match_type: MatchType::Metadata,
                source: DisplaySource::Sessions(idx),
                timestamp: sessions[idx].timestamp,
            }))
            .collect();
        Self {
            sessions,
            filtered_indices,
            display_items,
            selected: 0,
            scroll_offset: 0,
            mode: Mode::Browsing,
            filter_query: String::new(),
            filter_active: false,
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
            grouped_mode: false,
            expanded_projects: HashSet::new(),
        }
    }

    /// Re-run the metadata filter and rebuild display items.
    pub fn apply_filter(&mut self) {
        self.filtered_indices = filter_sessions(&self.sessions, &self.filter_query);
        self.rebuild_display_items();
        self.selected = 0;
        self.scroll_offset = 0;
    }

    /// Build merged display items from metadata matches and content results.
    ///
    /// In flat mode: wraps each DisplayEntry in DisplayItem::Session (existing behavior).
    /// In grouped mode: groups by project_name, sorts groups by latest timestamp,
    /// emits Header + Session items respecting expanded_projects state.
    /// When filter_query is non-empty in grouped mode, auto-expands matching groups
    /// and hides empty groups (bypasses expanded_projects set).
    pub fn rebuild_display_items(&mut self) {
        let content_ids: HashSet<&str> = self
            .content_results
            .iter()
            .map(|r| r.session.id.as_str())
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

        for (i, result) in self.content_results.iter().enumerate() {
            if !metadata_ids.contains(result.session.id.as_str()) {
                entries.push(DisplayEntry {
                    match_type: MatchType::Content,
                    source: DisplaySource::Content(i),
                    timestamp: result.session.timestamp,
                });
            }
        }

        entries.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));

        if !self.grouped_mode {
            // Flat mode: wrap each entry
            self.display_items = entries.into_iter().map(DisplayItem::Session).collect();
        } else {
            // Grouped mode: group by project_name
            let mut groups: HashMap<String, Vec<DisplayEntry>> = HashMap::new();
            for entry in entries {
                let session = self.display_session_from_entry(&entry);
                let project = session.project_name.clone();
                groups.entry(project).or_default().push(entry);
            }

            // Build sorted group list (by latest timestamp descending)
            let mut group_list: Vec<(ProjectGroup, Vec<DisplayEntry>)> = groups
                .into_iter()
                .map(|(name, sessions)| {
                    let latest = sessions.iter().map(|e| e.timestamp).max().unwrap_or_default();
                    let count = sessions.len();
                    (
                        ProjectGroup {
                            project_name: name,
                            latest_timestamp: latest,
                            session_count: count,
                        },
                        sessions,
                    )
                })
                .collect();
            group_list.sort_by(|a, b| b.0.latest_timestamp.cmp(&a.0.latest_timestamp));

            let has_filter = !self.filter_query.is_empty();
            let mut items = Vec::new();

            for (group, sessions) in group_list {
                // No filter: expanded_projects = set of expanded groups (default collapsed)
                // Filter active: auto-expand all, but expanded_projects = set of
                // manually collapsed groups (inverted semantics during filter)
                let is_expanded = if has_filter {
                    !self.expanded_projects.contains(&group.project_name)
                } else {
                    self.expanded_projects.contains(&group.project_name)
                };

                items.push(DisplayItem::Header(group));
                if is_expanded {
                    for session in sessions {
                        items.push(DisplayItem::Session(session));
                    }
                }
            }

            self.display_items = items;
        }
    }

    /// Get the session referenced by a display entry.
    pub fn display_session(&self, entry: &DisplayEntry) -> &Session {
        self.display_session_from_entry(entry)
    }

    /// Get the session referenced by a display entry (internal helper).
    fn display_session_from_entry(&self, entry: &DisplayEntry) -> &Session {
        match &entry.source {
            DisplaySource::Sessions(idx) => &self.sessions[*idx],
            DisplaySource::Content(idx) => &self.content_results[*idx].session,
        }
    }

    /// Get the snippet for a content-matched display entry, if available.
    ///
    /// Returns `Some(&MatchSnippet)` when the entry came from content search
    /// (DisplaySource::Content) and the corresponding SearchResult has a snippet.
    /// Returns `None` for metadata-only matches or when no snippet was extracted.
    pub fn snippet_for_entry(&self, entry: &DisplayEntry) -> Option<&search::MatchSnippet> {
        match &entry.source {
            DisplaySource::Content(idx) => self.content_results[*idx].snippet.as_ref(),
            DisplaySource::Sessions(idx) => {
                // For Both matches, the session is in filtered_indices but the snippet
                // is in content_results. Look up by session ID.
                let session_id = &self.sessions[*idx].id;
                self.content_results
                    .iter()
                    .find(|r| r.session.id == *session_id)
                    .and_then(|r| r.snippet.as_ref())
            }
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
        if !self.display_items.is_empty() {
            self.selected = (self.selected + 1).min(self.display_items.len() - 1);
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

    /// Enter conversation viewer for a display item.
    pub fn enter_conversation(&mut self, display_idx: usize) {
        if display_idx >= self.display_items.len() {
            return;
        }
        let entry = match &self.display_items[display_idx] {
            DisplayItem::Session(entry) => entry,
            DisplayItem::Header(_) => return, // Cannot enter conversation on a header
        };
        let session = self.display_session(entry).clone();
        let claude_home = get_claude_home();
        let messages = load_conversation(&claude_home, &session);

        let initial_search_terms = crate::filter::parse_keywords(&self.filter_query);

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
            search_cursor: 0,
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
                        .display_items
                        .get(self.selected)
                        .and_then(|item| match item {
                            DisplayItem::Session(e) => Some(self.display_session(e).id.clone()),
                            DisplayItem::Header(_) => None,
                        });

                    self.content_results = results;
                    self.content_search_state = ContentSearchState::Complete;
                    self.rebuild_display_items();

                    if let Some(id) = selected_id {
                        if let Some(pos) = self
                            .display_items
                            .iter()
                            .position(|item| match item {
                                DisplayItem::Session(e) => self.display_session(e).id == id,
                                DisplayItem::Header(_) => false,
                            })
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
pub fn run(
    sessions: Vec<Session>,
    theme: Theme,
    cd_file: Option<String>,
) -> Result<(), Box<dyn std::error::Error>> {
    if sessions.is_empty() {
        eprintln!("No sessions found.");
        return Ok(());
    }

    let original_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |panic_info| {
        let _ = disable_raw_mode();
        let _ = execute!(stdout(), LeaveAlternateScreen, DisableMouseCapture);
        original_hook(panic_info);
    }));

    enable_raw_mode()?;
    let mut stdout = stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let claude_home = get_claude_home();
    let session_index = search::build_session_index(&claude_home, &sessions);
    let mut app = App::new(sessions, session_index, theme);
    let mut deferred_command: Option<String> = None;
    let mut deferred_cwd: Option<String> = None;

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
            match event::read()? {
                Event::Key(key) => {
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
                        Action::ExecCommand { cmd, cwd } => {
                            deferred_command = Some(cmd);
                            deferred_cwd = Some(cwd);
                            break;
                        }
                        Action::BackToList => {
                            app.leave_conversation();
                        }
                        Action::Continue => {}
                    }
                }
                Event::Mouse(mouse) => {
                    match mouse.kind {
                        MouseEventKind::ScrollDown => {
                            match app.mode {
                                Mode::Browsing => {
                                    for _ in 0..3 {
                                        app.move_down();
                                    }
                                }
                                Mode::Conversation | Mode::ConversationSearch => {
                                    if let Some(conv) = &mut app.conversation {
                                        let max = conv.lines.len().saturating_sub(conv.page_height);
                                        conv.scroll_offset = (conv.scroll_offset + 3).min(max);
                                    }
                                }
                            }
                        }
                        MouseEventKind::ScrollUp => {
                            match app.mode {
                                Mode::Browsing => {
                                    for _ in 0..3 {
                                        app.move_up();
                                    }
                                }
                                Mode::Conversation | Mode::ConversationSearch => {
                                    if let Some(conv) = &mut app.conversation {
                                        conv.scroll_offset = conv.scroll_offset.saturating_sub(3);
                                    }
                                }
                            }
                        }
                        MouseEventKind::Down(crossterm::event::MouseButton::Left) if app.mode == Mode::Browsing => {
                            let clicked_row = mouse.row as usize;
                            if clicked_row >= 1 {
                                let item_idx = app.scroll_offset + clicked_row - 1;
                                if item_idx < app.display_items.len() {
                                    if item_idx == app.selected {
                                        match &app.display_items[item_idx] {
                                            DisplayItem::Header(group) => {
                                                let name = group.project_name.clone();
                                                if app.expanded_projects.contains(&name) {
                                                    app.expanded_projects.remove(&name);
                                                } else {
                                                    app.expanded_projects.insert(name);
                                                }
                                                app.rebuild_display_items();
                                            }
                                            DisplayItem::Session(_) => {
                                                app.enter_conversation(item_idx);
                                            }
                                        }
                                    } else {
                                        app.selected = item_idx;
                                    }
                                }
                            }
                        }
                        _ => {}
                    }
                }
                _ => {}
            }
        }
    }

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen, DisableMouseCapture)?;
    terminal.show_cursor()?;

    if let Some(cmd) = deferred_command {
        if let (Some(ref cd_path), Some(ref cwd)) = (&cd_file, &deferred_cwd) {
            let _ = std::fs::write(cd_path, cwd);
        }
        let shell = std::env::var("SHELL").unwrap_or_else(|_| "/bin/sh".to_string());
        let status = std::process::Command::new(&shell)
            .arg("-ic")
            .arg(&cmd)
            .status();
        match status {
            Ok(s) => std::process::exit(s.code().unwrap_or(1)),
            Err(e) => {
                eprintln!("Failed to exec: {e}");
                println!("{cmd}");
            }
        }
    }

    Ok(())
}
