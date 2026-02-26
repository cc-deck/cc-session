#!/usr/bin/env bash
# Generate fake session JSONL data for demo recording.
# Creates a ~/.claude-like directory structure with realistic but synthetic sessions.

set -euo pipefail

FIXTURE_DIR="${1:-$(mktemp -d)/claude-demo}"
mkdir -p "$FIXTURE_DIR/projects"

echo "Creating demo fixtures in $FIXTURE_DIR"

# Helper: create a session JSONL file
create_session() {
  local project_path="$1"
  local session_id="$2"
  local branch="$3"
  local message="$4"
  local timestamp="$5"

  local encoded_dir
  encoded_dir=$(echo "$project_path" | sed 's|/|-|g')
  local session_dir="$FIXTURE_DIR/projects/$encoded_dir"
  mkdir -p "$session_dir"

  local file="$session_dir/$session_id.jsonl"

  # First user message
  cat > "$file" <<JSONL
{"type":"user","sessionId":"$session_id","cwd":"$project_path","gitBranch":"$branch","timestamp":"$timestamp","version":"2.1.56","message":{"role":"user","content":"$message"},"uuid":"msg-001"}
{"type":"assistant","sessionId":"$session_id","cwd":"$project_path","gitBranch":"$branch","timestamp":"$timestamp","message":{"role":"assistant","content":[{"type":"text","text":"I can help with that. Let me look at the code."}]},"uuid":"msg-002"}
JSONL

  echo "  Created session: $(basename "$project_path") / $branch"
}

# Generate timestamps relative to now
now=$(date -u +%s)
h() { date -u -r $(( now - $1 * 3600 )) +"%Y-%m-%dT%H:%M:%S.000Z" 2>/dev/null || date -u -d "@$(( now - $1 * 3600 ))" +"%Y-%m-%dT%H:%M:%S.000Z"; }

# Project 1: api-gateway (3 sessions)
create_session "/home/dev/projects/api-gateway" "a1b2c3d4-1111-4000-8000-000000000001" "main" \
  "Add rate limiting middleware to the API gateway" "$(h 2)"
create_session "/home/dev/projects/api-gateway" "a1b2c3d4-1111-4000-8000-000000000002" "feat-cors" \
  "Configure CORS headers for the frontend domain" "$(h 26)"
create_session "/home/dev/projects/api-gateway" "a1b2c3d4-1111-4000-8000-000000000003" "fix-timeout" \
  "Debug the 504 timeout errors on the health endpoint" "$(h 72)"

# Project 2: auth-service (3 sessions)
create_session "/home/dev/projects/auth-service" "b2c3d4e5-2222-4000-8000-000000000001" "main" \
  "Implement JWT token refresh with sliding expiration" "$(h 1)"
create_session "/home/dev/projects/auth-service" "b2c3d4e5-2222-4000-8000-000000000002" "feat-oauth" \
  "Add GitHub OAuth2 provider for social login" "$(h 48)"
create_session "/home/dev/projects/auth-service" "b2c3d4e5-2222-4000-8000-000000000003" "fix-session" \
  "Fix session invalidation on password change" "$(h 168)"

# Project 3: dashboard (3 sessions)
create_session "/home/dev/projects/dashboard" "c3d4e5f6-3333-4000-8000-000000000001" "main" \
  "Build the real-time metrics dashboard with WebSocket updates" "$(h 5)"
create_session "/home/dev/projects/dashboard" "c3d4e5f6-3333-4000-8000-000000000002" "feat-charts" \
  "Add interactive time-series charts for request latency" "$(h 96)"
create_session "/home/dev/projects/dashboard" "c3d4e5f6-3333-4000-8000-000000000003" "redesign" \
  "Redesign the navigation sidebar with collapsible sections" "$(h 240)"

# Project 4: data-pipeline (3 sessions)
create_session "/home/dev/projects/data-pipeline" "d4e5f6a7-4444-4000-8000-000000000001" "main" \
  "Optimize the ETL pipeline for large CSV imports" "$(h 8)"
create_session "/home/dev/projects/data-pipeline" "d4e5f6a7-4444-4000-8000-000000000002" "feat-parquet" \
  "Add Parquet file support to the data ingestion layer" "$(h 120)"
create_session "/home/dev/projects/data-pipeline" "d4e5f6a7-4444-4000-8000-000000000003" "fix-dedup" \
  "Fix duplicate record detection in the merge step" "$(h 336)"

# Project 5: docs-site (3 sessions)
create_session "/home/dev/projects/docs-site" "e5f6a7b8-5555-4000-8000-000000000001" "main" \
  "Add search functionality to the documentation site" "$(h 12)"
create_session "/home/dev/projects/docs-site" "e5f6a7b8-5555-4000-8000-000000000002" "feat-versioning" \
  "Implement versioned documentation with dropdown selector" "$(h 200)"
create_session "/home/dev/projects/docs-site" "e5f6a7b8-5555-4000-8000-000000000003" "fix-links" \
  "Audit and fix broken cross-reference links across all pages" "$(h 360)"

echo ""
echo "Created 15 sessions across 5 projects in $FIXTURE_DIR"
echo "Use: CLAUDE_HOME=$FIXTURE_DIR cc-session"
