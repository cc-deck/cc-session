use std::io::{self, BufRead, Write};

use chrono::Utc;
use chrono_humanize::{Accuracy, HumanTime, Tense};

use crate::filter::fuzzy_filter;
use crate::session::Session;

/// Run the scriptable selection mode with fuzzy filtering.
///
/// Returns an exit code (0 = success, 1 = no match).
pub fn run_scriptable(sessions: &[Session], query: &str) -> i32 {
    let matches: Vec<&Session> = if query.is_empty() {
        sessions.iter().collect()
    } else {
        fuzzy_filter(sessions, query)
            .into_iter()
            .map(|(idx, _)| &sessions[idx])
            .collect()
    };

    present_selection(&matches, query)
}

/// Run the scriptable selection mode with pre-filtered results (e.g. from deep search).
///
/// Returns an exit code (0 = success, 1 = no match).
pub fn run_scriptable_prefiltered(sessions: &[Session], label: &str) -> i32 {
    let refs: Vec<&Session> = sessions.iter().collect();
    present_selection(&refs, label)
}

/// Shared selection logic: single match prints command, multiple shows menu.
fn present_selection(sessions: &[&Session], label: &str) -> i32 {
    if sessions.is_empty() {
        eprintln!("No sessions found matching \"{label}\"");
        return 1;
    }

    if sessions.len() == 1 {
        println!("{}", sessions[0].resume_command());
        return 0;
    }

    let total = sessions.len();
    let display_count = total.min(10);

    if total <= 10 {
        eprintln!("  {total} sessions match \"{label}\":\n");
    } else {
        eprintln!("  {total} sessions match \"{label}\" (showing top 10):\n");
    }

    for (i, session) in sessions.iter().take(display_count).enumerate() {
        render_menu_entry(i + 1, session);
    }

    eprint!("\n  Select [1-{display_count}]: ");
    io::stderr().flush().ok();

    if let Some(choice) = read_selection(display_count) {
        println!("{}", sessions[choice - 1].resume_command());
        0
    } else {
        eprintln!("Invalid selection");
        1
    }
}

fn render_menu_entry(num: usize, session: &Session) {
    let branch = session.git_branch.as_deref().unwrap_or("");
    let delta = Utc::now().signed_duration_since(session.timestamp);
    let time_ago = HumanTime::from(-delta).to_text_en(Accuracy::Rough, Tense::Past);

    let mut line1 = format!("  {num:<3}{}", session.project_name);
    if !branch.is_empty() {
        line1.push_str(&format!(" · {branch}"));
    }
    line1.push_str(&format!(" · {time_ago}"));
    eprintln!("{line1}");

    let msg: String = session.first_message.chars().take(70).collect();
    eprintln!("     \"{msg}\"");
}

fn read_selection(max: usize) -> Option<usize> {
    let stdin = io::stdin();
    let line = stdin.lock().lines().next()?.ok()?;
    let n: usize = line.trim().parse().ok()?;
    if n >= 1 && n <= max {
        Some(n)
    } else {
        None
    }
}
