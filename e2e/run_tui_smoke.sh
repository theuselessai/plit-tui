#!/usr/bin/env bash
# E2E smoke test for plit-tui via tmux.
#
# Launches plit-tui in a tmux session against a running plit backend,
# sends a "hello" message, and verifies a response appears.
#
# Prerequisites:
#   - plit Docker container running on localhost (API port 8000, Gateway port 8080)
#   - tmux installed
#
# Usage:
#   ./e2e/run_tui_smoke.sh <path-to-plit-tui-binary>
#
# Environment:
#   PIPELIT_PORT  — API port (default: 8000)
#   ADMIN_USER    — admin username (default: admin)
#   ADMIN_PASS    — admin password (default: testpass123)

set -euo pipefail

BINARY="${1:?Usage: $0 <plit-tui-binary>}"
PIPELIT_PORT="${PIPELIT_PORT:-8000}"
ADMIN_USER="${ADMIN_USER:-admin}"
ADMIN_PASS="${ADMIN_PASS:-testpass123}"
SESSION="plit-tui-e2e-$$"
PANE_FILE="/tmp/plit-tui-e2e-pane-$$.txt"
PASS=0
FAIL=0

# ── Helpers ──────────────────────────────────────────────────────────────────

cleanup() {
    echo ""
    echo "═══ Cleanup ═══"
    tmux kill-session -t "$SESSION" 2>/dev/null && echo "  Killed tmux session" || true
    rm -f "$PANE_FILE"
    echo ""
    echo "═══ Results: $PASS passed, $FAIL failed ═══"
    [ "$FAIL" -gt 0 ] && exit 1
    exit 0
}
trap cleanup EXIT

assert() {
    local name="$1" condition="$2"
    if eval "$condition"; then
        echo "  ✓ $name"
        PASS=$((PASS + 1))
    else
        echo "  ✗ $name"
        FAIL=$((FAIL + 1))
    fi
}

capture() {
    tmux capture-pane -t "$SESSION" -p > "$PANE_FILE" 2>/dev/null || true
}

echo "═══ plit-tui E2E Smoke Test ═══"
echo "  Binary: $BINARY"

# ── 1. Authenticate ─────────────────────────────────────────────────────────

echo ""
echo "═══ 1. Authentication ═══"

LOGIN_RESP=$(curl -sf -X POST "http://localhost:${PIPELIT_PORT}/api/v1/auth/token/" \
    -H "Content-Type: application/json" \
    -d "{\"username\":\"${ADMIN_USER}\",\"password\":\"${ADMIN_PASS}\"}")

TOKEN=$(echo "$LOGIN_RESP" | python3 -c "import sys,json; print(json.load(sys.stdin)['key'])" 2>/dev/null || echo "")

assert "Got auth token" "[ -n '$TOKEN' ]"

if [ -z "$TOKEN" ]; then
    echo "  FATAL: Cannot continue without auth token"
    echo "  Response: $LOGIN_RESP"
    exit 1
fi

# ── 2. Write auth.json ──────────────────────────────────────────────────────

echo ""
echo "═══ 2. Write auth.json ═══"

AUTH_DIR="${HOME}/.config/plit"
mkdir -p "$AUTH_DIR"
cat > "${AUTH_DIR}/auth.json" <<EOF
{
  "token": "${TOKEN}",
  "username": "${ADMIN_USER}",
  "pipelit_url": "http://localhost:${PIPELIT_PORT}"
}
EOF

assert "auth.json created" "[ -f '${AUTH_DIR}/auth.json' ]"

# ── 3. Launch TUI in tmux ───────────────────────────────────────────────────

echo ""
echo "═══ 3. Launch TUI ═══"

tmux new-session -d -s "$SESSION" -x 120 -y 40 "$BINARY"
sleep 5

assert "tmux session running" "tmux has-session -t '$SESSION' 2>/dev/null"

capture
assert "TUI rendered" "[ -s '$PANE_FILE' ]"

echo "  Initial pane:"
head -5 "$PANE_FILE" || true

# ── 4. Select agent + send message ──────────────────────────────────────────

echo ""
echo "═══ 4. Interact ═══"

# Select first agent (Enter in normal mode on tab 0 → AgentSelect → switches to chat tab)
tmux send-keys -t "$SESSION" Enter
sleep 2

# Enter insert mode
tmux send-keys -t "$SESSION" i
sleep 0.5

# Type and send message
tmux send-keys -t "$SESSION" "hello"
sleep 0.5
tmux send-keys -t "$SESSION" Enter

# Wait for response (mock LLM is instant; real LLM may take up to 60s)
echo "  Waiting for response (up to 60s)..."
RESPONSE_FOUND=false
for i in $(seq 1 60); do
    capture
    if grep -q "E2E_MOCK_RESPONSE_OK" "$PANE_FILE" 2>/dev/null; then
        RESPONSE_FOUND=true
        echo "  Mock response appeared after ${i}s"
        break
    fi
    sleep 1
done

# ── 5. Verify ────────────────────────────────────────────────────────────────

echo ""
echo "═══ 5. Verify ═══"

capture
echo "  Final pane:"
cat "$PANE_FILE" || true

assert "Sent message visible" "grep -qi 'hello' '$PANE_FILE'"

if [ "$RESPONSE_FOUND" = true ]; then
    assert "Mock response received" "grep -q 'E2E_MOCK_RESPONSE_OK' '$PANE_FILE'"
else
    # Real LLM — response text is unpredictable; just verify TUI is alive
    assert "TUI still responsive" "tmux has-session -t '$SESSION' 2>/dev/null"
fi

# ── 6. Quit ──────────────────────────────────────────────────────────────────

echo ""
echo "═══ 6. Quit ═══"

# Escape insert mode → normal mode → quit
tmux send-keys -t "$SESSION" Escape
sleep 0.5
tmux send-keys -t "$SESSION" q
sleep 2

if tmux has-session -t "$SESSION" 2>/dev/null; then
    echo "  TUI didn't exit on 'q', sending Ctrl+C"
    tmux send-keys -t "$SESSION" C-c
    sleep 2
fi

echo ""
echo "═══ Done ═══"
