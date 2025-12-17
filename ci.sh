#!/usr/bin/env bash
set -euo pipefail

export CARGO_TERM_COLOR=always

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$ROOT"

echo "==> Build (workspace)"
cargo build --workspace

echo "==> Test (workspace)"
cargo test --workspace

echo "==> Format check"
cargo fmt --all -- --check

echo "==> Clippy (workspace, all-targets, all-features, deny warnings)"
cargo clippy --workspace --all-targets --all-features -- -D warnings

echo "==> Coverage (tarpaulin)"
if ! command -v cargo-tarpaulin >/dev/null 2>&1; then
  echo "cargo-tarpaulin not found. Install with:"
  echo "  cargo install cargo-tarpaulin"
  exit 1
fi

rm -rf coverage
mkdir -p coverage

cargo tarpaulin --workspace --timeout 300 --out xml --output-dir coverage

echo "==> Coverage threshold check"
XML="coverage/cobertura.xml"
if [[ ! -f "$XML" ]]; then
  echo "Expected $XML but it was not produced."
  exit 1
fi

# Extract first line-rate="..."
COVERAGE="$(grep -o 'line-rate="[0-9.]*"' "$XML" | head -1 | grep -o '[0-9.]*')"
if [[ -z "${COVERAGE:-}" ]]; then
  echo "Could not extract coverage from $XML"
  exit 1
fi

# Prefer python for arithmetic (avoid bc dependency differences)
COVERAGE_PERCENT="$(python3 - <<PY
c=float("$COVERAGE")
print(int(c*100))
PY
)"

echo "Current coverage: ${COVERAGE_PERCENT}%"

MIN_THRESHOLD=20
if [[ "$COVERAGE_PERCENT" -lt "$MIN_THRESHOLD" ]]; then
  echo "Coverage ${COVERAGE_PERCENT}% is below minimum threshold of ${MIN_THRESHOLD}%"
  exit 1
else
  echo "Coverage meets minimum threshold"
fi

BASELINE_FILE="coverage/baseline.txt"
if [[ -f "$BASELINE_FILE" ]]; then
  BASELINE="$(cat "$BASELINE_FILE")"
  BASELINE_PERCENT="$(python3 - <<PY
b=float("$BASELINE")
print(int(b*100))
PY
)"
  DIFF="$(python3 - <<PY
c=float("$COVERAGE")
b=float("$BASELINE")
print((c-b)*100.0)
PY
)"
  echo "Baseline coverage: ${BASELINE_PERCENT}%"
  echo "Coverage change: ${DIFF}%"

  # Fail if coverage dropped by more than 10%
  DROP_TOO_MUCH="$(python3 - <<PY
diff=float("$DIFF")
print("yes" if diff < -10.0 else "no")
PY
)"
  if [[ "$DROP_TOO_MUCH" == "yes" ]]; then
    echo "Coverage dropped by more than 10% (from ${BASELINE_PERCENT}% to ${COVERAGE_PERCENT}%)"
    exit 1
  else
    echo "Coverage change is within acceptable range"
  fi
else
  echo "No baseline coverage found, setting current coverage as baseline"
fi

echo "$COVERAGE" > "$BASELINE_FILE"
echo "==> Done"
