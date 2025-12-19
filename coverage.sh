#!/usr/bin/env bash
set -euo pipefail

COV_DIR="coverage"
LCOV_FILE="${COV_DIR}/lcov.info"
HTML_BASE_DIR="${COV_DIR}"          # IMPORTANT: base dir
HTML_INDEX="${COV_DIR}/html/index.html"

# Portable ignore regex (works with / and \)
IGNORE_REGEX='([\\/]\.cargo[\\/]registry[\\/]|[\\/]target[\\/]|[\\/]rustc[\\/])'

echo "Cleaning old coverage output..."
rm -rf "${COV_DIR}" 2>/dev/null || true
mkdir -p "${COV_DIR}"

echo "Checking cargo-llvm-cov..."
if ! command -v cargo-llvm-cov >/dev/null 2>&1; then
  echo "ERROR: cargo-llvm-cov not installed"
  echo "Install with: cargo install cargo-llvm-cov"
  exit 1
fi

echo "Checking Rust LLVM tools..."
SYSROOT="$(rustc --print sysroot)"
HOST="$(rustc -vV | sed -n 's/^host: //p')"
LLVM_BIN="${SYSROOT}/lib/rustlib/${HOST}/bin"
if [ ! -x "${LLVM_BIN}/llvm-cov" ] || [ ! -x "${LLVM_BIN}/llvm-profdata" ]; then
  echo "ERROR: llvm-cov / llvm-profdata not found in Rust toolchain"
  echo "Install with: rustup component add llvm-tools-preview"
  exit 1
fi

echo "Cleaning old llvm-cov state..."
cargo llvm-cov clean --workspace

echo "Running tests once and collecting coverage (workspace, all targets)..."
cargo llvm-cov \
  --workspace \
  --all-targets \
  --no-report

echo "Generating LCOV report..."
cargo llvm-cov report \
  --lcov \
  --output-path "${LCOV_FILE}" \
  --ignore-filename-regex "${IGNORE_REGEX}"

echo "Generating HTML report..."
# IMPORTANT: output-dir is the base directory; HTML goes under <output-dir>/html
cargo llvm-cov report \
  --html \
  --output-dir "${HTML_BASE_DIR}" \
  --ignore-filename-regex "${IGNORE_REGEX}"

if [ ! -f "${LCOV_FILE}" ]; then
  echo "ERROR: LCOV report not generated: ${LCOV_FILE}"
  exit 1
fi

if [ ! -f "${HTML_INDEX}" ]; then
  echo "ERROR: HTML report not generated: ${HTML_INDEX}"
  echo "If you see 'Finished report saved to ${COV_DIR}/html/html', you set --output-dir too deep."
  exit 1
fi

echo "Done."
echo "Outputs:"
echo "  ${LCOV_FILE}"
echo "  ${HTML_INDEX}"
