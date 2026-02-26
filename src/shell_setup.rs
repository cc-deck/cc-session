use std::env;
use std::fs;
use std::io::{self, Write};
use std::path::PathBuf;

const MARKER_BEGIN: &str = "# >>> cc-session shell functions >>>";
const MARKER_END: &str = "# <<< cc-session shell functions <<<";

fn shell_functions() -> String {
    format!(
        r#"
{MARKER_BEGIN}
# Deep search for a session and resume it
ccs() {{
  local cmd
  cmd=$(cc-session -g "$1" -q 2>/dev/null)
  if [ $? -ne 0 ] || [ -z "$cmd" ]; then
    echo "No session found for: $1" >&2
    return 1
  fi
  eval "$cmd"
}}

# Fuzzy search for a session and resume it
ccf() {{
  local cmd
  cmd=$(cc-session -s "$1" -q 2>/dev/null)
  if [ $? -ne 0 ] || [ -z "$cmd" ]; then
    echo "No session found for: $1" >&2
    return 1
  fi
  eval "$cmd"
}}
{MARKER_END}
"#
    )
    .trim_start()
    .to_string()
}

/// Print the shell function definitions to stdout.
pub fn print_definitions() {
    println!("{}", shell_functions());
    eprintln!("# Paste the above into your shell rc file, or run:");
    eprintln!("#   cc-session --shell-setup --install");
}

/// Detect the shell rc file and install the functions.
pub fn install() {
    let rc_path = detect_rc_file();

    let rc_path = match rc_path {
        Some(p) => p,
        None => {
            eprintln!("Could not detect shell rc file.");
            eprintln!("Add the following to your shell config manually:\n");
            println!("{}", shell_functions());
            std::process::exit(1);
        }
    };

    let content = fs::read_to_string(&rc_path).unwrap_or_default();

    // Check if already installed
    if content.contains(MARKER_BEGIN) {
        // Replace existing block
        let before = content
            .split(MARKER_BEGIN)
            .next()
            .unwrap_or(&content)
            .trim_end();
        let after = content
            .split(MARKER_END)
            .nth(1)
            .unwrap_or("")
            .trim_start();

        let mut new_content = before.to_string();
        new_content.push('\n');
        new_content.push('\n');
        new_content.push_str(&shell_functions());
        if !after.is_empty() {
            new_content.push('\n');
            new_content.push_str(after);
        }
        new_content.push('\n');

        fs::write(&rc_path, new_content).unwrap_or_else(|e| {
            eprintln!("Failed to write {}: {e}", rc_path.display());
            std::process::exit(1);
        });
        eprintln!("Updated shell functions in {}", rc_path.display());
    } else {
        // Append
        let mut file = fs::OpenOptions::new()
            .append(true)
            .open(&rc_path)
            .unwrap_or_else(|e| {
                eprintln!("Failed to open {}: {e}", rc_path.display());
                std::process::exit(1);
            });

        writeln!(file).ok();
        write!(file, "{}", shell_functions()).unwrap_or_else(|e| {
            eprintln!("Failed to write to {}: {e}", rc_path.display());
            std::process::exit(1);
        });
        writeln!(file).ok();

        eprintln!("Added shell functions to {}", rc_path.display());
    }

    eprintln!("Restart your shell or run:  source {}", rc_path.display());
    eprintln!();
    eprintln!("Available functions:");
    eprintln!("  ccs <pattern>   Deep search (grep) and resume top match");
    eprintln!("  ccf <query>     Fuzzy search and resume top match");
}

/// Detect the user's shell rc file.
fn detect_rc_file() -> Option<PathBuf> {
    let home = dirs::home_dir()?;
    let shell = env::var("SHELL").unwrap_or_default();

    if shell.contains("zsh") {
        let zdotdir = env::var("ZDOTDIR")
            .map(PathBuf::from)
            .unwrap_or_else(|_| home.clone());
        let zshrc = zdotdir.join(".zshrc");
        if zshrc.exists() {
            return Some(zshrc);
        }
    }

    if shell.contains("bash") {
        let bashrc = home.join(".bashrc");
        if bashrc.exists() {
            return Some(bashrc);
        }
        let profile = home.join(".bash_profile");
        if profile.exists() {
            return Some(profile);
        }
    }

    // Fallback: try common files
    for name in [".zshrc", ".bashrc", ".bash_profile"] {
        let path = home.join(name);
        if path.exists() {
            // Confirm with user
            eprint!("Found {}. Use this file? [Y/n] ", path.display());
            io::stderr().flush().ok();
            let mut answer = String::new();
            io::stdin().read_line(&mut answer).ok();
            let answer = answer.trim().to_lowercase();
            if answer.is_empty() || answer == "y" || answer == "yes" {
                return Some(path);
            }
        }
    }

    None
}
