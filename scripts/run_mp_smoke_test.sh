#!/usr/bin/env bash
# Two-process headless LAN smoke test for PudgyMon (Bevy).
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

RESULT_DIR="$ROOT/.bevy"
mkdir -p "$RESULT_DIR"

HOST_RESULT="$RESULT_DIR/mp_smoke_host.result"
JOIN_RESULT="$RESULT_DIR/mp_smoke_join.result"
rm -f "$HOST_RESULT" "$JOIN_RESULT"

echo "Building smoke binary..."
cargo build --quiet --bin pudgymon_smoke

SMOKE_BIN="$ROOT/target/debug/pudgymon_smoke"
if [[ ! -f "$SMOKE_BIN" ]]; then
  SMOKE_BIN="$ROOT/target/debug/pudgymon_smoke.exe"
fi

echo "Starting host..."
MP_TEST_ROLE=host MP_TEST_PORT=7777 "$SMOKE_BIN" host --port 7777 &
HOST_PID=$!
sleep 5

echo "Starting client..."
set +e
MP_TEST_ROLE=join MP_TEST_PORT=7777 MP_TEST_ADDRESS=127.0.0.1 \
  "$SMOKE_BIN" join --address 127.0.0.1 --port 7777
JOIN_EXIT=$?
set -e

wait "$HOST_PID" || true
HOST_EXIT=$?

echo ""
echo "Host exit: $HOST_EXIT | Client exit: $JOIN_EXIT"

if [[ -f "$HOST_RESULT" ]]; then cat "$HOST_RESULT"; else echo "host result file missing"; fi
if [[ -f "$JOIN_RESULT" ]]; then cat "$JOIN_RESULT"; else echo "join result file missing"; fi

if [[ $HOST_EXIT -eq 0 && $JOIN_EXIT -eq 0 ]] \
  && [[ -f "$HOST_RESULT" ]] && grep -q "pass=true" "$HOST_RESULT" \
  && [[ -f "$JOIN_RESULT" ]] && grep -q "pass=true" "$JOIN_RESULT"; then
  echo "BEVY MULTIPLAYER SMOKE TEST: PASS"
  exit 0
fi

echo "BEVY MULTIPLAYER SMOKE TEST: FAIL"
exit 1
