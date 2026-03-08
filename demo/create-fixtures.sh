#!/usr/bin/env bash
# Generate fake session JSONL data for demo recording.
# Creates a ~/.claude-like directory structure with realistic but synthetic sessions.

set -euo pipefail

FIXTURE_DIR="${1:-$(mktemp -d)/claude-demo}"
mkdir -p "$FIXTURE_DIR/projects"

echo "Creating demo fixtures in $FIXTURE_DIR"

# Helper: create a session JSONL file with simple content
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

  cat > "$file" <<JSONL
{"type":"user","sessionId":"$session_id","cwd":"$project_path","gitBranch":"$branch","timestamp":"$timestamp","version":"2.1.56","message":{"role":"user","content":"$message"},"uuid":"msg-001"}
{"type":"assistant","sessionId":"$session_id","cwd":"$project_path","gitBranch":"$branch","timestamp":"$timestamp","message":{"role":"assistant","content":[{"type":"text","text":"I can help with that. Let me look at the code."}]},"uuid":"msg-002"}
JSONL

  echo "  Created session: $(basename "$project_path") / $branch"
}

# Helper: create a session with rich multi-turn conversation
create_rich_session() {
  local project_path="$1"
  local session_id="$2"
  local branch="$3"
  local timestamp="$4"

  local encoded_dir
  encoded_dir=$(echo "$project_path" | sed 's|/|-|g')
  local session_dir="$FIXTURE_DIR/projects/$encoded_dir"
  mkdir -p "$session_dir"

  local file="$session_dir/$session_id.jsonl"

  # Use Python to write JSON safely (handles escaping)
  python3 - "$file" "$session_id" "$project_path" "$branch" "$timestamp" <<'PYTHON'
import json, sys

file_path, sid, cwd, branch, ts = sys.argv[1:6]

def user_entry(msg, uuid):
    return json.dumps({
        "type": "user", "sessionId": sid, "cwd": cwd,
        "gitBranch": branch, "timestamp": ts, "version": "2.1.56",
        "message": {"role": "user", "content": msg}, "uuid": uuid
    })

def asst_entry(text, uuid):
    return json.dumps({
        "type": "assistant", "sessionId": sid, "cwd": cwd,
        "gitBranch": branch, "timestamp": ts,
        "message": {"role": "assistant", "content": [{"type": "text", "text": text}]},
        "uuid": uuid
    })

# Build the conversation
lines = []

# Turn 1: User asks about rate limiting
lines.append(user_entry(
    "Add rate limiting middleware to the API gateway with per-client tracking",
    "msg-001"
))

# Turn 1: Rich assistant response with headers, code, table, links
response1 = """## Rate Limiting Middleware

I've implemented a **token bucket** rate limiter with per-client tracking. Here's the core implementation:

### Configuration

The rate limiter is configured via `RateLimitConfig`:

```rust
pub struct RateLimitConfig {
    pub requests_per_second: u32,
    pub burst_size: u32,
    pub client_timeout: Duration,
}

impl RateLimitConfig {
    pub fn new(rps: u32, burst: u32) -> Self {
        Self {
            requests_per_second: rps,
            burst_size: burst,
            client_timeout: Duration::from_secs(3600),
        }
    }
}
```

### Rate Limit Headers

Each response includes standard rate limit headers per RFC 6585:

| Header | Description | Example |
|--------|------------|---------|
| `X-RateLimit-Limit` | Max requests per window | `100` |
| `X-RateLimit-Remaining` | Requests left in window | `87` |
| `X-RateLimit-Reset` | Seconds until reset | `42` |
| `Retry-After` | Wait time when limited | `5` |

### Middleware Integration

The middleware extracts client identity from the `Authorization` header or falls back to the source IP. It uses a *concurrent hash map* for lock-free client tracking.

Key design decisions:
- Token bucket algorithm for smooth rate limiting
- Separate burst and sustained rate configuration
- Automatic cleanup of stale client entries
- Graceful handling via `429 Too Many Requests`

For more details see https://datatracker.ietf.org/doc/html/rfc6585 and the tower-http middleware docs at https://docs.rs/tower-http/latest/tower_http/"""

lines.append(asst_entry(response1, "msg-002"))

# Turn 2
lines.append(user_entry(
    "Can you add Redis-backed storage for distributed rate limiting across instances?",
    "msg-003"
))

response2 = """### Redis Backend

I've added a `RedisStore` that implements the `RateLimitStore` trait:

```rust
#[async_trait]
impl RateLimitStore for RedisStore {
    async fn check_rate_limit(
        &self,
        client_id: &str,
        config: &RateLimitConfig,
    ) -> Result<RateLimitResult> {
        let key = format!("ratelimit:{}", client_id);
        let mut conn = self.pool.get().await?;

        // Use Redis MULTI/EXEC for atomic check-and-update
        let (count, ttl): (u64, i64) = redis::pipe()
            .atomic()
            .incr(&key, 1)
            .expire(&key, config.window_seconds as i64)
            .query_async(&mut conn)
            .await?;

        Ok(RateLimitResult {
            allowed: count <= config.requests_per_second as u64,
            remaining: config.requests_per_second.saturating_sub(count as u32),
            reset_after: Duration::from_secs(ttl.max(0) as u64),
        })
    }
}
```

This ensures consistent rate limiting across all gateway replicas. The Redis key expires automatically, so no manual cleanup is needed."""

lines.append(asst_entry(response2, "msg-004"))

# Turn 3
lines.append(user_entry(
    "What about the performance impact? Any benchmarks?",
    "msg-005"
))

response3 = """### Performance Benchmarks

I ran benchmarks comparing in-memory vs Redis-backed rate limiting:

| Scenario | Latency (p50) | Latency (p99) | Throughput |
|----------|--------------|--------------|------------|
| No rate limiting | 1.2ms | 4.8ms | 12,400 req/s |
| In-memory store | 1.3ms | 5.1ms | 11,800 req/s |
| Redis store (local) | 1.8ms | 7.2ms | 9,200 req/s |
| Redis store (network) | 2.4ms | 12.1ms | 7,100 req/s |

The in-memory store adds *negligible* overhead (~0.1ms p50). The Redis backend adds ~0.6ms for local Redis and ~1.2ms with network latency, which is acceptable for most use cases.

**Recommendation**: Use the in-memory store for single-instance deployments and switch to Redis only when you need distributed rate limiting across multiple gateway replicas."""

lines.append(asst_entry(response3, "msg-006"))

# Turn 4
lines.append(user_entry(
    "Add unit tests for the token bucket algorithm edge cases",
    "msg-007"
))

response4 = """### Test Coverage

Added comprehensive tests for edge cases:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_burst_then_sustain() {
        let config = RateLimitConfig::new(10, 20);
        let store = InMemoryStore::new();

        // Burst: first 20 requests should pass
        for i in 0..20 {
            let result = store.check("client-1", &config).await;
            assert!(result.allowed, "request {} should be allowed", i);
        }

        // 21st request should be rate-limited
        let result = store.check("client-1", &config).await;
        assert!(!result.allowed, "should be rate-limited after burst");
        assert!(result.reset_after > Duration::ZERO);
    }

    #[tokio::test]
    async fn test_independent_clients() {
        let config = RateLimitConfig::new(1, 1);
        let store = InMemoryStore::new();

        // Exhaust client-1's limit
        store.check("client-1", &config).await;

        // client-2 should still have its own quota
        let result = store.check("client-2", &config).await;
        assert!(result.allowed);
    }
}
```

All 14 tests pass. Coverage for the rate limiting module is now at **94%**."""

lines.append(asst_entry(response4, "msg-008"))

with open(file_path, 'w') as f:
    f.write('\n'.join(lines) + '\n')
PYTHON

  echo "  Created session: $(basename "$project_path") / $branch (4 prompts, rich content)"
}

# Generate timestamps relative to now
now=$(date -u +%s)
h() { date -u -r $(( now - $1 * 3600 )) +"%Y-%m-%dT%H:%M:%S.000Z" 2>/dev/null || date -u -d "@$(( now - $1 * 3600 ))" +"%Y-%m-%dT%H:%M:%S.000Z"; }

# Project 1: api-gateway (rich session + 2 simple)
create_rich_session "/home/dev/projects/api-gateway" "a1b2c3d4-1111-4000-8000-000000000001" "feat-ratelimit" "$(h 1)"
create_session "/home/dev/projects/api-gateway" "a1b2c3d4-1111-4000-8000-000000000002" "feat-cors" \
  "Configure CORS headers for the frontend domain" "$(h 26)"
create_session "/home/dev/projects/api-gateway" "a1b2c3d4-1111-4000-8000-000000000003" "fix-timeout" \
  "Debug the 504 timeout errors on the health endpoint" "$(h 72)"

# Project 2: auth-service (3 sessions)
AUTH_DIR="$FIXTURE_DIR/projects/-home-dev-projects-auth-service"
mkdir -p "$AUTH_DIR"
AUTH_ID="b2c3d4e5-2222-4000-8000-000000000001"
cat > "$AUTH_DIR/$AUTH_ID.jsonl" <<JSONL
{"type":"user","sessionId":"$AUTH_ID","cwd":"/home/dev/projects/auth-service","gitBranch":"main","timestamp":"$(h 2)","version":"2.1.56","message":{"role":"user","content":"Implement JWT token refresh with sliding expiration"},"uuid":"msg-001"}
{"type":"assistant","sessionId":"$AUTH_ID","cwd":"/home/dev/projects/auth-service","gitBranch":"main","timestamp":"$(h 2)","message":{"role":"assistant","content":[{"type":"text","text":"I'll implement the JWT refresh flow with sliding window expiration."}]},"uuid":"msg-002"}
{"type":"user","sessionId":"$AUTH_ID","cwd":"/home/dev/projects/auth-service","gitBranch":"main","timestamp":"$(h 2)","message":{"role":"user","content":"The refresh token should be stored in an HTTP-only cookie, not localStorage"},"uuid":"msg-003"}
{"type":"assistant","sessionId":"$AUTH_ID","cwd":"/home/dev/projects/auth-service","gitBranch":"main","timestamp":"$(h 2)","message":{"role":"assistant","content":[{"type":"text","text":"Good point. I'll update the implementation to use HTTP-only cookies for the refresh token."}]},"uuid":"msg-004"}
{"type":"user","sessionId":"$AUTH_ID","cwd":"/home/dev/projects/auth-service","gitBranch":"main","timestamp":"$(h 2)","message":{"role":"user","content":"Can you add token rotation? Each refresh should invalidate the old token"},"uuid":"msg-005"}
{"type":"assistant","sessionId":"$AUTH_ID","cwd":"/home/dev/projects/auth-service","gitBranch":"main","timestamp":"$(h 2)","message":{"role":"assistant","content":[{"type":"text","text":"I'll add a token family tracking mechanism for secure rotation with reuse detection."}]},"uuid":"msg-006"}
{"type":"user","sessionId":"$AUTH_ID","cwd":"/home/dev/projects/auth-service","gitBranch":"main","timestamp":"$(h 2)","message":{"role":"user","content":"What about the race condition when multiple tabs refresh at the same time?"},"uuid":"msg-007"}
{"type":"assistant","sessionId":"$AUTH_ID","cwd":"/home/dev/projects/auth-service","gitBranch":"main","timestamp":"$(h 2)","message":{"role":"assistant","content":[{"type":"text","text":"We can add a short grace period for the old token after rotation to handle concurrent refresh attempts."}]},"uuid":"msg-008"}
{"type":"user","sessionId":"$AUTH_ID","cwd":"/home/dev/projects/auth-service","gitBranch":"main","timestamp":"$(h 2)","message":{"role":"user","content":"Add rate limiting to the refresh endpoint, max 10 refreshes per minute per user"},"uuid":"msg-009"}
{"type":"assistant","sessionId":"$AUTH_ID","cwd":"/home/dev/projects/auth-service","gitBranch":"main","timestamp":"$(h 2)","message":{"role":"assistant","content":[{"type":"text","text":"Done. I've added a sliding window rate limiter to the /auth/refresh endpoint."}]},"uuid":"msg-010"}
JSONL
echo "  Created session: auth-service / main (5 prompts)"

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
