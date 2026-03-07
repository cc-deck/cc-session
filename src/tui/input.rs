use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use super::{Action, App, Mode};

/// Handle a key event and return the resulting action.
pub fn handle_input(app: &mut App, key: KeyEvent) -> Action {
    // Ctrl-C always quits
    if key.code == KeyCode::Char('c') && key.modifiers.contains(KeyModifiers::CONTROL) {
        return Action::Quit;
    }

    match app.mode {
        Mode::Browsing => handle_browse(app, key),
        Mode::Filtering => handle_filter(app, key),
        Mode::DeepSearchInput => handle_deep_search_input(app, key),
        Mode::DeepSearching => handle_deep_searching(app, key),
        Mode::Detail => handle_detail(app, key),
    }
}

fn handle_browse(app: &mut App, key: KeyEvent) -> Action {
    match key.code {
        KeyCode::Char('q') => {
            if app.is_deep_search() {
                Action::RestoreOriginal
            } else {
                Action::Quit
            }
        }
        KeyCode::Esc => {
            if app.is_deep_search() {
                // Go back to deep search input so user can refine the query
                app.filter_query = app.deep_search_query.clone().unwrap_or_default();
                app.mode = Mode::DeepSearchInput;
                Action::Continue
            } else {
                Action::Quit
            }
        }
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
            Action::Continue
        }
        KeyCode::Enter => {
            if let Some(&idx) = app.filtered_indices.get(app.selected) {
                Action::EnterDetail(idx)
            } else {
                Action::Continue
            }
        }
        _ => Action::Continue,
    }
}

fn handle_filter(app: &mut App, key: KeyEvent) -> Action {
    // Ctrl-G: switch to deep search input mode
    if key.code == KeyCode::Char('g') && key.modifiers.contains(KeyModifiers::CONTROL) {
        return Action::StartDeepSearchInput;
    }

    match key.code {
        KeyCode::Esc => {
            app.mode = Mode::Browsing;
            app.filter_query.clear();
            app.apply_filter();
            Action::Continue
        }
        KeyCode::Enter => {
            if let Some(&idx) = app.filtered_indices.get(app.selected) {
                app.mode = Mode::Browsing;
                Action::EnterDetail(idx)
            } else {
                Action::Continue
            }
        }
        KeyCode::Backspace => {
            app.filter_query.pop();
            app.apply_filter();
            Action::Continue
        }
        KeyCode::Char(c) => {
            app.filter_query.push(c);
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

fn handle_deep_search_input(app: &mut App, key: KeyEvent) -> Action {
    match key.code {
        KeyCode::Esc => {
            if app.is_deep_search() {
                // We came from deep search results, restore original
                Action::RestoreOriginal
            } else {
                // We came from normal filter mode, go back there
                app.mode = Mode::Filtering;
                Action::Continue
            }
        }
        KeyCode::Enter => {
            if !app.filter_query.is_empty() {
                Action::DeepSearch(app.filter_query.clone())
            } else {
                Action::Continue
            }
        }
        KeyCode::Backspace => {
            app.filter_query.pop();
            Action::Continue
        }
        KeyCode::Char(c) => {
            app.filter_query.push(c);
            Action::Continue
        }
        _ => Action::Continue,
    }
}

fn handle_deep_searching(_app: &mut App, key: KeyEvent) -> Action {
    // Only allow Esc to cancel during search
    match key.code {
        KeyCode::Esc => {
            // Drop the receiver to abandon results; restore browsing
            _app.search_receiver = None;
            _app.deep_search_query = None;
            _app.mode = Mode::Browsing;
            _app.filter_query.clear();
            Action::Continue
        }
        _ => Action::Continue,
    }
}

fn handle_detail(app: &mut App, key: KeyEvent) -> Action {
    use super::DetailButton;

    match key.code {
        KeyCode::Esc | KeyCode::Char('q') => Action::BackToList,
        KeyCode::Tab | KeyCode::BackTab | KeyCode::Left | KeyCode::Right => {
            if let Some(detail) = &mut app.detail {
                detail.focused_button = match detail.focused_button {
                    DetailButton::CopyAndExit => DetailButton::Back,
                    DetailButton::Back => DetailButton::CopyAndExit,
                };
            }
            Action::Continue
        }
        KeyCode::Enter => {
            if let Some(detail) = &app.detail {
                match detail.focused_button {
                    DetailButton::CopyAndExit => {
                        let session = &app.sessions[detail.session_idx];
                        let cmd = session.resume_command();
                        Action::CopyCommand(cmd)
                    }
                    DetailButton::Back => Action::BackToList,
                }
            } else {
                Action::Continue
            }
        }
        _ => Action::Continue,
    }
}
