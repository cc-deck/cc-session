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
        Mode::Detail => handle_detail(app, key),
    }
}

fn handle_browse(app: &mut App, key: KeyEvent) -> Action {
    match key.code {
        KeyCode::Char('q') => Action::Quit,
        KeyCode::Esc => Action::Quit,
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
    // Ctrl-G: deep search
    if key.code == KeyCode::Char('g') && key.modifiers.contains(KeyModifiers::CONTROL) {
        if !app.filter_query.is_empty() {
            return Action::DeepSearch(app.filter_query.clone());
        }
        return Action::Continue;
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

fn handle_detail(app: &App, key: KeyEvent) -> Action {
    match key.code {
        KeyCode::Esc | KeyCode::Char('q') => Action::BackToList,
        KeyCode::Enter => {
            if let Some(detail) = &app.detail {
                let session = &app.sessions[detail.session_idx];
                let cmd = session.resume_command();
                Action::CopyCommand(cmd)
            } else {
                Action::Continue
            }
        }
        _ => Action::Continue,
    }
}
