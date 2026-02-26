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
# The main session gets a realistic multi-prompt conversation (shown in detail view)
AUTH_DIR="$FIXTURE_DIR/projects/-home-dev-projects-auth-service"
mkdir -p "$AUTH_DIR"
AUTH_ID="b2c3d4e5-2222-4000-8000-000000000001"
cat > "$AUTH_DIR/$AUTH_ID.jsonl" <<JSONL
{"type":"user","sessionId":"$AUTH_ID","cwd":"/home/dev/projects/auth-service","gitBranch":"main","timestamp":"$(h 1)","version":"2.1.56","message":{"role":"user","content":"Implement JWT token refresh with sliding expiration"},"uuid":"msg-001"}
{"type":"assistant","sessionId":"$AUTH_ID","cwd":"/home/dev/projects/auth-service","gitBranch":"main","timestamp":"$(h 1)","message":{"role":"assistant","content":[{"type":"text","text":"I'll implement the JWT refresh flow. Let me start with the token service."}]},"uuid":"msg-002"}
{"type":"user","sessionId":"$AUTH_ID","cwd":"/home/dev/projects/auth-service","gitBranch":"main","timestamp":"$(h 1)","message":{"role":"user","content":"The refresh token should be stored in an HTTP-only cookie, not localStorage"},"uuid":"msg-003"}
{"type":"assistant","sessionId":"$AUTH_ID","cwd":"/home/dev/projects/auth-service","gitBranch":"main","timestamp":"$(h 1)","message":{"role":"assistant","content":[{"type":"text","text":"Good point. I'll update the implementation to use HTTP-only cookies."}]},"uuid":"msg-004"}
{"type":"user","sessionId":"$AUTH_ID","cwd":"/home/dev/projects/auth-service","gitBranch":"main","timestamp":"$(h 1)","message":{"role":"user","content":"Can you add token rotation? Each refresh should invalidate the old token"},"uuid":"msg-005"}
{"type":"assistant","sessionId":"$AUTH_ID","cwd":"/home/dev/projects/auth-service","gitBranch":"main","timestamp":"$(h 1)","message":{"role":"assistant","content":[{"type":"text","text":"I'll add a token family tracking mechanism for rotation."}]},"uuid":"msg-006"}
{"type":"user","sessionId":"$AUTH_ID","cwd":"/home/dev/projects/auth-service","gitBranch":"main","timestamp":"$(h 1)","message":{"role":"user","content":"What about the race condition when multiple tabs refresh at the same time?"},"uuid":"msg-007"}
{"type":"assistant","sessionId":"$AUTH_ID","cwd":"/home/dev/projects/auth-service","gitBranch":"main","timestamp":"$(h 1)","message":{"role":"assistant","content":[{"type":"text","text":"We can add a grace period for the old token after rotation."}]},"uuid":"msg-008"}
{"type":"user","sessionId":"$AUTH_ID","cwd":"/home/dev/projects/auth-service","gitBranch":"main","timestamp":"$(h 1)","message":{"role":"user","content":"Add rate limiting to the refresh endpoint, max 10 refreshes per minute per user"},"uuid":"msg-009"}
{"type":"assistant","sessionId":"$AUTH_ID","cwd":"/home/dev/projects/auth-service","gitBranch":"main","timestamp":"$(h 1)","message":{"role":"assistant","content":[{"type":"text","text":"Done. I've added a sliding window rate limiter."}]},"uuid":"msg-010"}
{"type":"user","sessionId":"$AUTH_ID","cwd":"/home/dev/projects/auth-service","gitBranch":"main","timestamp":"$(h 1)","message":{"role":"user","content":"The tests are failing because the mock clock isn't advancing, can you fix that?"},"uuid":"msg-011"}
{"type":"assistant","sessionId":"$AUTH_ID","cwd":"/home/dev/projects/auth-service","gitBranch":"main","timestamp":"$(h 1)","message":{"role":"assistant","content":[{"type":"text","text":"I see the issue. The test clock needs to be injected via the constructor."}]},"uuid":"msg-012"}
{"type":"user","sessionId":"$AUTH_ID","cwd":"/home/dev/projects/auth-service","gitBranch":"main","timestamp":"$(h 1)","message":{"role":"user","content":"Please also add a /revoke endpoint that invalidates all tokens for a user"},"uuid":"msg-013"}
{"type":"assistant","sessionId":"$AUTH_ID","cwd":"/home/dev/projects/auth-service","gitBranch":"main","timestamp":"$(h 1)","message":{"role":"assistant","content":[{"type":"text","text":"I'll add the revoke endpoint with proper cascade invalidation."}]},"uuid":"msg-014"}
{"type":"user","sessionId":"$AUTH_ID","cwd":"/home/dev/projects/auth-service","gitBranch":"main","timestamp":"$(h 1)","message":{"role":"user","content":"Let's write integration tests for the full refresh flow including cookie handling"},"uuid":"msg-015"}
{"type":"assistant","sessionId":"$AUTH_ID","cwd":"/home/dev/projects/auth-service","gitBranch":"main","timestamp":"$(h 1)","message":{"role":"assistant","content":[{"type":"text","text":"I'll create a test suite that exercises the complete lifecycle."}]},"uuid":"msg-016"}
{"type":"user","sessionId":"$AUTH_ID","cwd":"/home/dev/projects/auth-service","gitBranch":"main","timestamp":"$(h 1)","message":{"role":"user","content":"Can we also log token refresh events for the audit trail?"},"uuid":"msg-017"}
{"type":"assistant","sessionId":"$AUTH_ID","cwd":"/home/dev/projects/auth-service","gitBranch":"main","timestamp":"$(h 1)","message":{"role":"assistant","content":[{"type":"text","text":"Added structured logging for all token lifecycle events."}]},"uuid":"msg-018"}
{"type":"user","sessionId":"$AUTH_ID","cwd":"/home/dev/projects/auth-service","gitBranch":"main","timestamp":"$(h 1)","message":{"role":"user","content":"One more thing: add a config option to set the refresh token TTL, default 30 days"},"uuid":"msg-019"}
JSONL
echo "  Created session: auth-service / main (10 prompts)"

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
