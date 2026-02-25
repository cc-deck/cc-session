use std::io::{self, BufRead, Write};

use chrono::Utc;
use chrono_humanize::{Accuracy, HumanTime, Tense};

use crate::filter::fuzzy_filter;
use crate::session::Session;

/// Run the scriptable selection mode.
///
/// Returns an exit code (0 = success, 1 = no match).
pub fn run_scriptable(sessions: &[Session], query: &str) -> i32 {
    let matches = if query.is_empty() {
        // No query: use all sessions
        (0..sessions.len()).collect::<Vec<_>>()
    } else {
        fuzzy_filter(sessions, query)
            .into_iter()
            .map(|(idx, _)| idx)
            .collect()
    };

    if matches.is_empty() {
        eprintln!("No sessions found matching \"{query}\"");
        return 1;
    }

    if matches.len() == 1 {
        let session = &sessions[matches[0]];
        println!("{}", session.resume_command());
        return 0;
    }

    // Multiple matches: show slim selection menu
    let total = matches.len();
    let display_count = total.min(10);
    let showing_all = total <= 10;

    if showing_all {
        eprintln!("  {total} sessions match \"{query}\":\n");
    } else {
        eprintln!("  {total} sessions match \"{query}\" (showing top 10):\n");
    }

    for (i, &idx) in matches.iter().take(display_count).enumerate() {
        let session = &sessions[idx];
        render_menu_entry(i + 1, session);
    }

    eprint!("\n  Select [1-{display_count}]: ");
    io::stderr().flush().ok();

    if let Some(choice) = read_selection(display_count) {
        let session = &sessions[matches[choice - 1]];
        println!("{}", session.resume_command());
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
