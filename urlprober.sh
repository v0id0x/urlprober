#!/bin/bash
# URLProber Quick Launch Script

# Get the directory where this script is located
SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
cd "$SCRIPT_DIR"

BINARY="$SCRIPT_DIR/target/release/urlprober"
TMP_BINARY="/tmp/target/release/urlprober"

echo "🔄 Checking and building URLProber..."
# Attempt normal cargo build (suppress expected WSL errors like permission denied)
cargo build --release > /dev/null 2>&1

# Check if binary exists in the normal target path
if [ -f "$BINARY" ]; then
    RUN_BIN="$BINARY"
else
    echo "⚠️ Standard build failed (likely WSL filesystem locks). Using /tmp fallback build..."
    CARGO_TARGET_DIR=/tmp/target cargo build --release
    if [ -f "$TMP_BINARY" ]; then
        cp "$TMP_BINARY" ./urlprober
        RUN_BIN="./urlprober"
    else
        echo "❌ Build completely failed."
        exit 1
    fi
fi

# Run the binary with all arguments
"$RUN_BIN" "$@"
