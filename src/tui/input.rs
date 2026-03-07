use std::sync::atomic::Ordering;
use std::time::Instant;

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use super::{Action, App, ContentSearchState, Mode};

/// Handle a key event and return the resulting action.
pub fn handle_input(app: &mut App, key: KeyEvent) -> Action {
    // Ctrl-C always quits
    if key.code == KeyCode::Char('c') && key.modifiers.contains(KeyModifiers::CONTROL) {
        return Action::Quit;
    }

    match app.mode {
        Mode::Browsing => handle_browse(app, key),
        Mode::Filtering => handle_filter(app, key),
        Mode::Conversation => handle_conversation(app, key),
        Mode::ConversationSearch => handle_conversation_search(app, key),
    }
}

fn handle_browse(app: &mut App, key: KeyEvent) -> Action {
    match key.code {
        KeyCode::Char('q') | KeyCode::Esc => Action::Quit,
        KeyCode::Char('j') | KeyCode::Down => {
            app.move_down();
            Action::Continue
        }
        KeyCode::Char('k') | KeyCode::Up => {
            app.move_up();
            Action::Continue
        }
        KeyCode::Char('/') => {
            app.mode = Mode::Filtering;
            app.filter_query.clear();
            app.content_results.clear();
            app.content_search_state = ContentSearchState::Idle;
            app.rebuild_display_entries();
            Action::Continue
        }
        KeyCode::Enter => {
            if app.selected < app.display_entries.len() {
                Action::EnterConversation(app.selected)
            } else {
                Action::Continue
            }
        }
        _ => Action::Continue,
    }
}

fn handle_filter(app: &mut App, key: KeyEvent) -> Action {
    match key.code {
        KeyCode::Esc => {
            app.cancel_content_search();
            app.filter_query.clear();
            app.apply_filter();
            app.mode = Mode::Browsing;
            Action::Continue
        }
        KeyCode::Enter => {
            if app.selected < app.display_entries.len() {
                Action::EnterConversation(app.selected)
            } else {
                Action::Continue
            }
        }
        KeyCode::Backspace => {
            app.filter_query.pop();
            app.cancel_flag.store(true, Ordering::Relaxed);
            app.search_receiver = None;
            app.content_results.clear();
            if app.filter_query.is_empty() {
                app.content_search_state = ContentSearchState::Idle;
                app.last_keystroke = None;
            } else {
                app.content_search_state = ContentSearchState::Debouncing;
                app.last_keystroke = Some(Instant::now());
            }
            app.apply_filter();
            Action::Continue
        }
        KeyCode::Char(c) => {
            app.filter_query.push(c);
            app.cancel_flag.store(true, Ordering::Relaxed);
            app.search_receiver = None;
            app.content_results.clear();
            app.content_search_state = ContentSearchState::Debouncing;
            app.last_keystroke = Some(Instant::now());
            app.apply_filter();
            Action::Continue
        }
        KeyCode::Down => {
            app.move_down();
            Action::Continue
        }
        KeyCode::Up => {
            app.move_up();
            Action::Continue
        }
        _ => Action::Continue,
    }
}

fn handle_conversation(app: &mut App, key: KeyEvent) -> Action {
    match key.code {
        KeyCode::Esc | KeyCode::Char('q') => Action::BackToList,
        KeyCode::Char(' ') => {
            if let Some(conv) = &mut app.conversation {
                let max = conv.lines.len().saturating_sub(conv.page_height);
                conv.scroll_offset = (conv.scroll_offset + conv.page_height).min(max);
            }
            Action::Continue
        }
        KeyCode::Char('b') => {
            if let Some(conv) = &mut app.conversation {
                conv.scroll_offset = conv.scroll_offset.saturating_sub(conv.page_height);
            }
            Action::Continue
        }
        KeyCode::Char('g') => {
            if let Some(conv) = &mut app.conversation {
                conv.scroll_offset = 0;
            }
            Action::Continue
        }
        KeyCode::Char('G') => {
            if let Some(conv) = &mut app.conversation {
                let max = conv.lines.len().saturating_sub(conv.page_height);
                conv.scroll_offset = max;
            }
            Action::Continue
        }
        KeyCode::PageDown => {
            if let Some(conv) = &mut app.conversation {
                let max = conv.lines.len().saturating_sub(conv.page_height);
                conv.scroll_offset = (conv.scroll_offset + conv.page_height).min(max);
            }
            Action::Continue
        }
        KeyCode::PageUp => {
            if let Some(conv) = &mut app.conversation {
                conv.scroll_offset = conv.scroll_offset.saturating_sub(conv.page_height);
            }
            Action::Continue
        }
        KeyCode::Char('j') | KeyCode::Down => {
            if let Some(conv) = &mut app.conversation {
                let max = conv.lines.len().saturating_sub(conv.page_height);
                conv.scroll_offset = (conv.scroll_offset + 1).min(max);
            }
            Action::Continue
        }
        KeyCode::Char('k') | KeyCode::Up => {
            if let Some(conv) = &mut app.conversation {
                conv.scroll_offset = conv.scroll_offset.saturating_sub(1);
            }
            Action::Continue
        }
        KeyCode::Char('n') => {
            jump_to_next_match(app);
            Action::Continue
        }
        KeyCode::Char('N') => {
            jump_to_prev_match(app);
            Action::Continue
        }
        KeyCode::Char('/') => {
            if let Some(conv) = &mut app.conversation {
                conv.search_active = true;
                conv.search_query.clear();
                conv.search_confirmed = false;
            }
            app.mode = Mode::ConversationSearch;
            Action::Continue
        }
        KeyCode::Enter => {
            if let Some(conv) = &app.conversation {
                let cmd = conv.session.resume_command();
                Action::CopyCommand(cmd)
            } else {
                Action::Continue
            }
        }
        _ => Action::Continue,
    }
}

fn handle_conversation_search(app: &mut App, key: KeyEvent) -> Action {
    match key.code {
        KeyCode::Esc => {
            if let Some(conv) = &mut app.conversation {
                conv.search_active = false;
                conv.search_query.clear();
                conv.search_confirmed = false;
                conv.rendered_width = 0; // force re-render
            }
            app.mode = Mode::Conversation;
            Action::Continue
        }
        KeyCode::Enter => {
            if let Some(conv) = &mut app.conversation {
                conv.search_active = false;
                if !conv.search_query.is_empty() && !conv.match_positions.is_empty() {
                    conv.search_confirmed = true;
                    conv.current_match = 0;
                    let max = conv.lines.len().saturating_sub(conv.page_height);
                    conv.scroll_offset = conv.match_positions[0]
                        .saturating_sub(conv.page_height / 2)
                        .min(max);
                }
            }
            app.mode = Mode::Conversation;
            Action::Continue
        }
        KeyCode::Backspace => {
            if let Some(conv) = &mut app.conversation {
                conv.search_query.pop();
                conv.rendered_width = 0; // force re-render
            }
            Action::Continue
        }
        KeyCode::Char(c) => {
            if let Some(conv) = &mut app.conversation {
                conv.search_query.push(c);
                conv.rendered_width = 0; // force re-render
            }
            Action::Continue
        }
        _ => Action::Continue,
    }
}

fn jump_to_next_match(app: &mut App) {
    if let Some(conv) = &mut app.conversation {
        if conv.match_positions.is_empty() {
            return;
        }
        conv.current_match = (conv.current_match + 1) % conv.match_positions.len();
        let max = conv.lines.len().saturating_sub(conv.page_height);
        conv.scroll_offset = conv.match_positions[conv.current_match]
            .saturating_sub(conv.page_height / 2)
            .min(max);
    }
}

fn jump_to_prev_match(app: &mut App) {
    if let Some(conv) = &mut app.conversation {
        if conv.match_positions.is_empty() {
            return;
        }
        if conv.current_match == 0 {
            conv.current_match = conv.match_positions.len() - 1;
        } else {
            conv.current_match -= 1;
        }
        let max = conv.lines.len().saturating_sub(conv.page_height);
        conv.scroll_offset = conv.match_positions[conv.current_match]
            .saturating_sub(conv.page_height / 2)
            .min(max);
    }
}
