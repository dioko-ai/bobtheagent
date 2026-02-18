#!/usr/bin/env bash
set -euo pipefail

BIN_PATH="${1:-target/debug/metaagent-rust}"

if [[ ! -x "$BIN_PATH" ]]; then
  echo "Building binary at $BIN_PATH" >&2
  cargo build --quiet
fi

if command -v jq >/dev/null 2>&1; then
  JSON_QUERY_TOOL="jq"
else
  JSON_QUERY_TOOL="python3"
fi

TMP_DIR="$(mktemp -d)"
trap 'rm -rf "$TMP_DIR"' EXIT

assert_json() {
  local file="$1"
  local expr="$2"
  if [[ "$JSON_QUERY_TOOL" == "jq" ]]; then
    jq -e "$expr" "$file" >/dev/null
  else
    python3 - "$file" "$expr" <<'PY'
import json
import sys

path = sys.argv[1]
expr = sys.argv[2]
obj = json.load(open(path, "r", encoding="utf-8"))

# Minimal checks used in this script.
if expr == '.status == "ok"':
    ok = obj.get("status") == "ok"
elif expr == '.status == "err"':
    ok = obj.get("status") == "err"
elif expr == '.data | type == "array"':
    ok = isinstance(obj.get("data"), list)
elif expr == '.error.code == "invalid_request"':
    err = obj.get("error") or {}
    ok = err.get("code") == "invalid_request"
else:
    raise SystemExit(f"unsupported fallback expression: {expr}")

if not ok:
    raise SystemExit(1)
PY
  fi
}

echo "[1/3] Validate success JSON envelope"
"$BIN_PATH" --output json api capability list >"$TMP_DIR/cap_list.stdout" 2>"$TMP_DIR/cap_list.stderr"
assert_json "$TMP_DIR/cap_list.stdout" '.status == "ok"'
assert_json "$TMP_DIR/cap_list.stdout" '.data | type == "array"'
if [[ -s "$TMP_DIR/cap_list.stderr" ]]; then
  echo "Expected empty stderr for successful JSON command" >&2
  cat "$TMP_DIR/cap_list.stderr" >&2
  exit 1
fi

echo "[2/3] Validate JSON error envelope and exit code"
set +e
"$BIN_PATH" --output json api capability get --id definitely-not-real >"$TMP_DIR/cap_bad.stdout" 2>"$TMP_DIR/cap_bad.stderr"
STATUS=$?
set -e
if [[ "$STATUS" -ne 10 ]]; then
  echo "Expected exit code 10 for invalid_request, got $STATUS" >&2
  exit 1
fi
assert_json "$TMP_DIR/cap_bad.stdout" '.status == "err"'
assert_json "$TMP_DIR/cap_bad.stdout" '.error.code == "invalid_request"'

# In JSON mode, command-domain errors are emitted as JSON to stdout.
if [[ -s "$TMP_DIR/cap_bad.stderr" ]]; then
  echo "Expected empty stderr for JSON-mode domain error envelope" >&2
  cat "$TMP_DIR/cap_bad.stderr" >&2
  exit 1
fi

echo "[3/3] Validate parse-time argument errors are on stderr"
set +e
"$BIN_PATH" --definitely-unknown-flag >"$TMP_DIR/parse_err.stdout" 2>"$TMP_DIR/parse_err.stderr"
STATUS=$?
set -e
if [[ "$STATUS" -eq 0 ]]; then
  echo "Expected non-zero exit for parse-time argument failure" >&2
  exit 1
fi
if ! rg -q "Unknown argument:" "$TMP_DIR/parse_err.stderr"; then
  echo "Expected parse-time Unknown argument message on stderr" >&2
  cat "$TMP_DIR/parse_err.stderr" >&2
  exit 1
fi

echo "All scriptability checks passed."
