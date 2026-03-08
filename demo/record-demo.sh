#!/usr/bin/env bash
# Record an asciinema demo of cc-session using fake data.
# Produces demo.cast in the current directory.
#
# Demo flow:
#   1. Launch cc-session, show initial session list
#   2. Type to filter (seamless search: "api")
#   3. Select the rich session, Enter to open conversation
#   4. Scroll through rich content (code, tables, links, headers)
#   5. Press / to search within conversation, type "redis"
#   6. Navigate matches with n
#   7. Escape to clear search, Enter to copy+exit
#   8. Show pasted clipboard command on the CLI

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
CAST_FILE="$SCRIPT_DIR/demo.cast"
FIXTURE_DIR=$(mktemp -d)
TMUX_SESSION="cc-session-demo"
COLS=110
ROWS=32

# Check prerequisites
for cmd in asciinema tmux cc-session; do
  if ! command -v "$cmd" &>/dev/null; then
    echo "Error: $cmd is not installed."
    [ "$cmd" = "cc-session" ] && echo "Install with: brew install rhuss/tap/cc-session"
    [ "$cmd" = "asciinema" ] && echo "Install with: brew install asciinema"
    [ "$cmd" = "tmux" ] && echo "Install with: brew install tmux"
    exit 1
  fi
done

# Generate fixtures
echo "Generating demo fixtures..."
bash "$SCRIPT_DIR/create-fixtures.sh" "$FIXTURE_DIR" > /dev/null

# Clean up previous recording
rm -f "$CAST_FILE"

# Kill any existing demo session
tmux kill-session -t "$TMUX_SESSION" 2>/dev/null || true

# Create a bashrc with clean colored prompt and CLAUDE_HOME set
DEMO_BASHRC=$(mktemp)
cat > "$DEMO_BASHRC" <<BASHRC
PS1='\[\033[1;32m\]\$ \[\033[0m\]'
export CLAUDE_HOME=$FIXTURE_DIR
BASHRC

# Create tmux session with fixed dimensions
tmux new-session -d -s "$TMUX_SESSION" -x "$COLS" -y "$ROWS"

sleep 0.3

echo "Recording demo..."

# Start asciinema recording with a clean bash shell (v3 compatible)
# Using -c ensures the recording ends cleanly when the inner shell exits
tmux send-keys -t "$TMUX_SESSION" \
  "asciinema rec -f asciicast-v2 --overwrite --window-size ${COLS}x${ROWS} -c 'bash --rcfile $DEMO_BASHRC --noprofile -i' '$CAST_FILE'" Enter

sleep 2

# Clear screen for a fresh start
tmux send-keys -t "$TMUX_SESSION" "clear" Enter
sleep 0.5

# ── Scene 1: Launch cc-session, show the initial list ──
tmux send-keys -t "$TMUX_SESSION" "cc-session" Enter
sleep 3

# Scroll down slowly with arrow keys to show the full list
for i in 1 2 3 4 5 6 7; do
  tmux send-keys -t "$TMUX_SESSION" Down
  sleep 0.35
done
sleep 1

# Scroll back up
for i in 1 2 3 4 5 6 7; do
  tmux send-keys -t "$TMUX_SESSION" Up
  sleep 0.25
done
sleep 0.8

# ── Scene 2: Type to filter (seamless search) ──
# Just start typing: "api" to narrow down to api-gateway sessions
for c in a p i; do
  tmux send-keys -t "$TMUX_SESSION" "$c"
  sleep 0.4
done
sleep 1.5

# ── Scene 3: Select the rich session and open conversation ──
# The api-gateway rate limiting session should be the top result
tmux send-keys -t "$TMUX_SESSION" Enter
sleep 2

# ── Scene 4: Scroll through rich conversation content ──
# Scroll down slowly to show headers, code blocks, tables, links
for i in $(seq 1 20); do
  tmux send-keys -t "$TMUX_SESSION" Down
  sleep 0.25
done
sleep 1.5

# Continue scrolling to show more content (table, code)
for i in $(seq 1 25); do
  tmux send-keys -t "$TMUX_SESSION" Down
  sleep 0.25
done
sleep 1.5

# Keep scrolling to show the performance table and tests
for i in $(seq 1 30); do
  tmux send-keys -t "$TMUX_SESSION" Down
  sleep 0.2
done
sleep 1.5

# ── Scene 5: Search within conversation ──
# Press / to activate conversation search
tmux send-keys -t "$TMUX_SESSION" /
sleep 0.5

# Type search term "redis" character by character
for c in r e d i s; do
  tmux send-keys -t "$TMUX_SESSION" "$c"
  sleep 0.3
done
sleep 1.5

# Confirm search with Enter
tmux send-keys -t "$TMUX_SESSION" Enter
sleep 1

# Navigate to next match with n
tmux send-keys -t "$TMUX_SESSION" n
sleep 1
tmux send-keys -t "$TMUX_SESSION" n
sleep 1

# ── Scene 6: Clear search and copy session ──
# Escape to clear search highlights
tmux send-keys -t "$TMUX_SESSION" Escape
sleep 0.8

# Enter to copy the resume command and exit
tmux send-keys -t "$TMUX_SESSION" Enter
sleep 1.5

# ── Scene 7: Show the pasted clipboard on the CLI ──
# Simulate paste by injecting clipboard content
tmux set-buffer -b demo "$(pbpaste)" 2>/dev/null
tmux send-keys -t "$TMUX_SESSION" ""  # ensure prompt is ready
sleep 0.3
tmux paste-buffer -t "$TMUX_SESSION" -b demo 2>/dev/null
sleep 4

# Exit the asciinema recording
tmux send-keys -t "$TMUX_SESSION" C-c
sleep 0.3
tmux send-keys -t "$TMUX_SESSION" "exit" Enter
sleep 1

# Clean up
tmux kill-session -t "$TMUX_SESSION" 2>/dev/null || true
rm -rf "$FIXTURE_DIR" "$DEMO_BASHRC"

if [ -f "$CAST_FILE" ]; then
  echo "Recording saved to $CAST_FILE"
  echo "Run ./export-gif.sh to convert to GIF"
else
  echo "Error: Recording failed, $CAST_FILE not found"
  exit 1
fi
