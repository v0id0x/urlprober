#!/bin/bash
# URLProber Quick Launch Script

# Get the directory where this script is located
SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
BINARY="$SCRIPT_DIR/target/release/urlprober"

# Check if binary exists
if [ ! -f "$BINARY" ]; then
    echo "❌ Binary not found. Building..."
    cd "$SCRIPT_DIR"
    cargo build --release
    echo "✅ Build complete!"
fi

# Run the binary with all arguments
"$BINARY" "$@"
