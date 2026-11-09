#!/bin/sh
# The checks of .github/workflows/ci.yml, run on this machine. The git hooks in .githooks/ call it:
# `rust` before each commit, `wasm` and `python` before each push.
#
#   scripts/check.sh            every check
#   scripts/check.sh rust       fmt, clippy and the Rust tests, with a coverage floor
#   scripts/check.sh wasm       the wasm tests, the site and the npm package as users get it
#   scripts/check.sh python     the Python module with pytest
#
# Needs: rustup with the wasm32-unknown-unknown target and the llvm-tools-preview component, cargo-llvm-cov,
# wasm-bindgen-cli at the version in Cargo.lock, Node 22 and Python 3.12.
set -e
cd "$(dirname "$0")/.."

name=mathra

check_rust() {
  cargo fmt --all --check
  cargo clippy --workspace --exclude "$name-python" --all-targets -- -D warnings
  # the tests, measured, with Latin Modern on so its font settings run too. The floor is what tests alone
  # reach; what is left is code no input reaches (and two bugs), to remove and then raise the floor to 100
  cargo llvm-cov --workspace --exclude "$name-python" --features "$name/latin-modern" --fail-under-lines 99.74 --fail-under-functions 99.53
  # the command line without the png feature refuses .png (the workspace build turns the feature on)
  cargo clippy -p "$name-cli" --no-default-features --all-targets -- -D warnings
  cargo test -p "$name-cli" --no-default-features
}

check_wasm() {
  CARGO_TARGET_WASM32_UNKNOWN_UNKNOWN_RUNNER=wasm-bindgen-test-runner \
    cargo test -p "$name-wasm" --target wasm32-unknown-unknown
  site/build.sh
  npm/build.sh && npm/test/run.sh
}

check_python() {
  (
    cd crates/python
    [ -d .venv ] || python3 -m venv .venv
    . .venv/bin/activate
    pip install --quiet maturin pytest
    maturin develop --quiet
    python -m pytest --quiet tests
  )
}

[ $# -eq 0 ] && set -- rust wasm python
for check in "$@"; do
  case "$check" in
    rust | wasm | python) echo "== $check"; "check_$check" ;;
    *) echo "unknown check: $check (rust, wasm or python)" >&2; exit 2 ;;
  esac
done
