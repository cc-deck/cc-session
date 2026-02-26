mod clipboard;
mod discovery;
mod filter;
mod scriptable;
mod search;
mod session;
mod shell_setup;
mod tui;

use clap::Parser;

use discovery::{apply_filters, discover_sessions, get_claude_home};

/// Fast CLI tool for finding and resuming Claude Code sessions.
#[derive(Parser, Debug)]
#[command(name = "cc-session", version, about)]
struct Cli {
    /// Scriptable select mode, with optional initial query
    #[arg(short, long)]
    select: Option<Option<String>>,

    /// Search (grep) inside session content
    #[arg(short, long)]
    grep: Option<String>,

    /// Only show sessions newer than duration (e.g. 7d, 2w, 1m)
    #[arg(long)]
    since: Option<String>,

    /// Show at most N sessions
    #[arg(long)]
    last: Option<usize>,

    /// Quick mode: print the top match directly (no menu, no clipboard). Use with -s or -g.
    #[arg(short = 'q', long = "quick")]
    quick: bool,

    /// Print shell function definitions, or install them with --install
    #[arg(long = "shell-setup")]
    shell_setup: bool,

    /// When used with --shell-setup, append functions to your shell rc file
    #[arg(long = "install", requires = "shell_setup")]
    install: bool,
}

/// Parse a human-friendly duration string into a chrono::Duration.
///
/// Supported suffixes: `d` (days), `w` (weeks), `m` (30-day months).
fn parse_duration(s: &str) -> Result<chrono::Duration, String> {
    let s = s.trim();
    if s.is_empty() {
        return Err("empty duration string".to_string());
    }

    let (num_str, suffix) = s.split_at(s.len() - 1);
    let num: i64 = num_str
        .parse()
        .map_err(|_| format!("invalid number in duration: {num_str:?}"))?;

    match suffix {
        "d" => Ok(chrono::Duration::days(num)),
        "w" => Ok(chrono::Duration::weeks(num)),
        "m" => Ok(chrono::Duration::days(num * 30)),
        other => Err(format!(
            "unknown duration suffix: {other:?} (expected d, w, or m)"
        )),
    }
}

fn main() {
    let cli = Cli::parse();

    // Handle --shell-setup before anything else
    if cli.shell_setup {
        if cli.install {
            shell_setup::install();
        } else {
            shell_setup::print_definitions();
        }
        return;
    }

    let claude_home = get_claude_home();
    let projects_dir = claude_home.join("projects");

    if !projects_dir.is_dir() {
        eprintln!(
            "No Claude projects directory found at {}",
            projects_dir.display()
        );
        std::process::exit(2);
    }

    let sessions = discover_sessions(&claude_home);

    // Apply --since filter
    let since_duration = cli.since.map(|s| {
        parse_duration(&s).unwrap_or_else(|e| {
            eprintln!("Invalid --since value: {e}");
            std::process::exit(1);
        })
    });

    let sessions = apply_filters(sessions, since_duration, cli.last);

    // Dispatch to the appropriate mode
    if let Some(query) = cli.select {
        let q = query.unwrap_or_default();
        let code = scriptable::run_scriptable(&sessions, &q, cli.quick);
        std::process::exit(code);
    }

    if let Some(pattern) = cli.grep {
        let results = search::deep_search(&claude_home, &pattern);
        let code = scriptable::run_scriptable_prefiltered(&results, &pattern, cli.quick);
        std::process::exit(code);
    }

    // Default: interactive TUI
    if let Err(e) = tui::run(sessions) {
        eprintln!("TUI error: {e}");
        std::process::exit(1);
    }
}
