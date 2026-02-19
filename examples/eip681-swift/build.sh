#!/usr/bin/env bash
#
# Build the EIP-681 Swift example.
#
# This script:
#   1. Builds the Rust FFI library (eip681-ffi) in release mode
#   2. Generates Swift bindings + C headers using the witffi CLI
#   3. Builds the Swift package
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

# ---- Step 2: Generate Swift bindings + C headers ----

INCLUDE_DIR="$SCRIPT_DIR/Sources/CZcashEip681/include"
SWIFT_DIR="$SCRIPT_DIR/Sources/Eip681"

mkdir -p "$INCLUDE_DIR" "$SWIFT_DIR"

echo "==> Generating Swift bindings and C headers..."
cargo run --release -p witffi-cli --manifest-path "$ROOT_DIR/Cargo.toml" -- \
    generate \
    --wit "$ROOT_DIR/wit/eip681.wit" \
    --lang swift \
    --output "$INCLUDE_DIR" \
    --c-prefix zcash_eip681

# Move the Swift source out of the C headers directory.
mv "$INCLUDE_DIR/Bindings.swift" "$SWIFT_DIR/"

# ---- Step 3: Build / test / run ----

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
