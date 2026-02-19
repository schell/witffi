#!/usr/bin/env bash
#
# Build the EIP-681 Swift example.
#
# This script:
#   1. Builds the Rust FFI library (eip681-ffi) in release mode
#   2. Generates Swift bindings using the witffi CLI
#   3. Copies C headers into the Swift package
#   4. Builds the Swift package
#
# Prerequisites: Rust toolchain (cargo) and Swift toolchain (swift) installed.
#
# Usage:
#   ./build.sh          # Build everything
#   ./build.sh test     # Build and run tests
#   ./build.sh run      # Build and run the example CLI

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
ROOT_DIR="$SCRIPT_DIR/../.."

# ---- Step 1: Build Rust FFI library ----

echo "==> Building Rust FFI library (release)..."
cargo build --release -p eip681-ffi --manifest-path "$ROOT_DIR/Cargo.toml"

# ---- Step 2: Generate Swift bindings ----

echo "==> Generating Swift bindings..."
cargo run --release -p witffi-cli --manifest-path "$ROOT_DIR/Cargo.toml" -- \
    generate \
    --wit "$ROOT_DIR/wit/eip681.wit" \
    --lang swift \
    --output "$SCRIPT_DIR/Sources/Eip681" \
    --c-prefix zcash_eip681

# ---- Step 3: Copy C headers ----

echo "==> Copying C headers..."
INCLUDE_DIR="$SCRIPT_DIR/Sources/CZcashEip681/include"
mkdir -p "$INCLUDE_DIR"
cp "$ROOT_DIR/crates/witffi-types/witffi_types.h" "$INCLUDE_DIR/"
cp "$ROOT_DIR/examples/eip681-ffi/ffi.h" "$INCLUDE_DIR/"

# ---- Step 4: Build / test / run ----

cd "$SCRIPT_DIR"

case "${1:-build}" in
    build)
        echo "==> Building Swift package..."
        swift build
        echo "==> Done! Run with: swift run eip681-example"
        ;;
    test)
        echo "==> Running Swift tests..."
        swift test
        ;;
    run)
        echo "==> Building and running example..."
        swift run eip681-example
        ;;
    *)
        echo "Usage: $0 [build|test|run]"
        exit 1
        ;;
esac
