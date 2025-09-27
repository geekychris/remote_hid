#!/bin/bash

# Remote HID Control System Demo Script
# This script demonstrates how to run all three components of the system

echo "======================================================="
echo "Remote HID Control System Demo"
echo "======================================================="
echo ""

# Function to check if a command exists
command_exists() {
    command -v "$1" >/dev/null 2>&1
}

# Check if cargo is installed
if ! command_exists cargo; then
    echo "❌ Cargo is not installed. Please install Rust and Cargo first."
    echo "Visit: https://rustup.rs/"
    exit 1
fi

# Check if make is available
if command_exists make; then
    BUILD_CMD="make build"
    RUN_SERVER_CMD="make run-server"
else
    BUILD_CMD="cargo build --release --workspace"
    RUN_SERVER_CMD="cargo run --release -p session-server"
fi

echo "🔧 Building the system..."
echo "Running: $BUILD_CMD"
if ! $BUILD_CMD; then
    echo "❌ Build failed!"
    exit 1
fi

echo ""
echo "✅ Build successful!"
echo ""

echo "🚀 Demo Instructions:"
echo "====================="
echo ""
echo "To run the complete system, you'll need 3 terminal windows:"
echo ""
echo "Terminal 1 - Session Server:"
echo "  $RUN_SERVER_CMD"
echo "  # or directly:"
echo "  ./target/release/session-server"
echo ""
echo "Terminal 2 - HID Client (on target machine):"
echo "  ./target/release/hid-client --client-id \"demo-client\" --client-name \"Demo Machine\""
echo ""
echo "Terminal 3 - Commander (on operator machine):"
echo "  ./target/release/commander --target \"demo-client\""
echo ""
echo "📝 Usage Notes:"
echo "==============="
echo "• The HID client runs on the machine you want to control"
echo "• The Commander runs on your control machine"  
echo "• Type text in the Commander terminal to send keystrokes"
echo "• Mouse events are sent periodically for demonstration"
echo ""
echo "🔒 Security Notice:"
echo "==================="
echo "This is a demonstration implementation. For production use:"
echo "• Enable TLS/WSS encryption"
echo "• Implement proper authentication"
echo "• Use secure network configurations"
echo "• Review security considerations in README.md"
echo ""
echo "📖 For more information, see:"
echo "• README.md - Complete usage guide"
echo "• BUILD.md - Detailed build instructions"
echo "• DESIGN.md - System architecture"
echo ""

# Check if we should run an interactive demo
read -p "Would you like to start the session server now? (y/n): " -n 1 -r
echo
if [[ $REPLY =~ ^[Yy]$ ]]; then
    echo ""
    echo "🌟 Starting session server..."
    echo "Open additional terminals for HID client and Commander."
    echo "Press Ctrl+C to stop the server."
    echo ""
    exec $RUN_SERVER_CMD
fi

echo ""
echo "Demo preparation complete! Follow the instructions above to run the system."