#!/usr/bin/env bash
# Record an asciinema demo of cc-session using fake data.
# Produces demo.cast in the current directory.

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJECT_DIR="$(dirname "$SCRIPT_DIR")"
CAST_FILE="$SCRIPT_DIR/demo.cast"
FIXTURE_DIR=$(mktemp -d)
TMUX_SESSION="cc-session-demo"
COLS=100
ROWS=28

# Check prerequisites
for cmd in asciinema tmux; do
  if ! command -v "$cmd" &>/dev/null; then
    echo "Error: $cmd is not installed."
    echo "Install with: brew install $cmd"
    exit 1
  fi
done

# Build release binary
echo "Building cc-session..."
cargo build --release --manifest-path "$PROJECT_DIR/Cargo.toml" 2>/dev/null
BIN="$PROJECT_DIR/target/release/cc-session"

# Generate fixtures
echo "Generating demo fixtures..."
bash "$SCRIPT_DIR/create-fixtures.sh" "$FIXTURE_DIR" > /dev/null

# Create the inner script that asciinema will record
INNER_SCRIPT=$(mktemp)
cat > "$INNER_SCRIPT" <<'INNER'
#!/usr/bin/env bash
# This script is what asciinema records.
# It shows a clean prompt, runs cc-session, then pastes the result.
export PS1='\[\033[1;32m\]$ \[\033[0m\]'
exec bash --norc --noprofile -i
INNER
chmod +x "$INNER_SCRIPT"

# Create a bashrc that sets the prompt
DEMO_BASHRC=$(mktemp)
cat > "$DEMO_BASHRC" <<'BASHRC'
PS1='\[\033[1;32m\]$ \[\033[0m\]'
BASHRC

# Clean up previous recording
rm -f "$CAST_FILE"

# Kill any existing demo session
tmux kill-session -t "$TMUX_SESSION" 2>/dev/null || true

# Create tmux session with fixed dimensions
tmux new-session -d -s "$TMUX_SESSION" -x "$COLS" -y "$ROWS"

echo "Recording demo..."

# Start asciinema recording with a clean bash shell
tmux send-keys -t "$TMUX_SESSION" \
  "asciinema rec --cols $COLS --rows $ROWS --overwrite '$CAST_FILE' -c 'bash --rcfile $DEMO_BASHRC --noprofile -i'" Enter

sleep 1.5

# Clear screen for a fresh start
tmux send-keys -t "$TMUX_SESSION" "clear" Enter
sleep 0.5

# Run cc-session
tmux send-keys -t "$TMUX_SESSION" "CLAUDE_HOME=$FIXTURE_DIR $BIN" Enter
sleep 2

# Scroll down slowly
for i in 1 2 3 4 5; do
  tmux send-keys -t "$TMUX_SESSION" j
  sleep 0.4
done

sleep 0.8

# Enter filter mode
tmux send-keys -t "$TMUX_SESSION" /
sleep 0.5

# Type search query character by character
for c in a u t h; do
  tmux send-keys -t "$TMUX_SESSION" "$c"
  sleep 0.3
done

sleep 1

# Select first match (Enter opens detail view)
tmux send-keys -t "$TMUX_SESSION" Enter
sleep 2.5

# Copy and exit (Enter on "Copy to clipboard & Exit")
tmux send-keys -t "$TMUX_SESSION" Enter
sleep 1

# Back at the shell prompt. Paste the clipboard content.
tmux send-keys -t "$TMUX_SESSION" "pbpaste" Enter
sleep 6

# Exit the recorded shell (ends asciinema recording)
tmux send-keys -t "$TMUX_SESSION" "exit" Enter
sleep 1

# Clean up
tmux kill-session -t "$TMUX_SESSION" 2>/dev/null || true
rm -rf "$FIXTURE_DIR" "$INNER_SCRIPT" "$DEMO_BASHRC"

if [ -f "$CAST_FILE" ]; then
  echo "Recording saved to $CAST_FILE"
  echo "Run ./export-gif.sh to convert to GIF"
else
  echo "Error: Recording failed, $CAST_FILE not found"
  exit 1
fi
