#!/usr/bin/env bash
# Convert asciinema recording to GIF using agg.
# Produces demo.gif in demo/ and copies to docs/ for README.

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJECT_DIR="$(dirname "$SCRIPT_DIR")"
CAST_FILE="$SCRIPT_DIR/demo.cast"
GIF_FILE="$SCRIPT_DIR/demo.gif"
DOCS_GIF="$PROJECT_DIR/docs/demo.gif"

# Check prerequisites
if ! command -v agg &>/dev/null; then
  echo "Error: agg is not installed."
  echo "Install with: brew install agg"
  exit 1
fi

if [ ! -f "$CAST_FILE" ]; then
  echo "Error: $CAST_FILE not found. Run ./record-demo.sh first."
  exit 1
fi

echo "Converting recording to GIF..."
agg \
  --font-size 14 \
  --speed 1.2 \
  --theme monokai \
  "$CAST_FILE" "$GIF_FILE"

# Copy to docs/ for README embedding
mkdir -p "$PROJECT_DIR/docs"
cp "$GIF_FILE" "$DOCS_GIF"

echo "GIF saved to:"
echo "  $GIF_FILE"
echo "  $DOCS_GIF (for README)"
echo ""
echo "Size: $(du -h "$GIF_FILE" | cut -f1)"
