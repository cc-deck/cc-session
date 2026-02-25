use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use super::{Action, App, Mode};

/// Handle a key event and return the resulting action.
pub fn handle_input(app: &mut App, key: KeyEvent) -> Action {
    match app.mode {
        Mode::Browsing => handle_browse(app, key),
        Mode::Filtering => handle_filter(app, key),
    }
}

fn handle_browse(app: &mut App, key: KeyEvent) -> Action {
    match key.code {
        KeyCode::Char('q') => Action::Quit,
        KeyCode::Esc => Action::Quit,
        KeyCode::Char('j') | KeyCode::Down => {
            app.move_down();
            // Recalculate visible items (approx, we don't know terminal height here)
            // ensure_visible is called with a generous estimate
            app.ensure_visible(20);
            Action::Continue
        }
        KeyCode::Char('k') | KeyCode::Up => {
            app.move_up();
            app.ensure_visible(20);
            Action::Continue
        }
        KeyCode::Char('/') => {
            app.mode = Mode::Filtering;
            app.filter_query.clear();
            Action::Continue
        }
        KeyCode::Enter => {
            if let Some(&idx) = app.filtered_indices.get(app.selected) {
                Action::Select(idx)
            } else {
                Action::Continue
            }
        }
        _ => Action::Continue,
    }
}

fn handle_filter(app: &mut App, key: KeyEvent) -> Action {
    // Ctrl-C always quits
    if key.code == KeyCode::Char('c') && key.modifiers.contains(KeyModifiers::CONTROL) {
        return Action::Quit;
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
                Action::Select(idx)
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
            app.ensure_visible(20);
            Action::Continue
        }
        KeyCode::Up => {
            app.move_up();
            app.ensure_visible(20);
            Action::Continue
        }
        _ => Action::Continue,
    }
}
