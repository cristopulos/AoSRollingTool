#!/bin/bash
set -e

cd "$(dirname "$0")"

if ! command -v cargo &> /dev/null; then
    echo "Error: cargo not found. Please install Rust: https://rustup.rs/"
    exit 1
fi

MODE="${1:-release}"

if [ "$MODE" = "release" ]; then
    echo "Building and running AoS4 Combat Roller (release mode)..."
    cargo run --release
elif [ "$MODE" = "debug" ]; then
    echo "Building and running AoS4 Combat Roller (debug mode)..."
    cargo run
else
    echo "Usage: $0 [release|debug]"
    echo "  release  - Build and run optimized version (default)"
    echo "  debug    - Build and run debug version"
    exit 1
fi
