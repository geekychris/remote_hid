# Remote HID Control System

A cross-platform remote Human Interface Device (HID) control system that allows remote control of keyboard and mouse input across Windows and macOS machines through a secure client-server architecture.

## Overview

This system enables remote control of keyboard and mouse input across Windows and macOS machines through three main components:

- **Session Server** - Central broker that manages connections and forwards messages
- **HID Client** - Runs on target machine, receives and executes HID commands
- **Commander** - Operator interface that captures local input and sends remote commands

## Architecture

```
[Commander] <---> [Session Server] <---> [HID Client]
   (Operator)         (Broker)           (Target Machine)
```

## Features

### ✅ Implemented
- Cross-platform support (Windows, macOS)
- WebSocket-based communication
- JSON protocol for message exchange
- Mouse control (movement, clicks, scrolling)
- Keyboard control (key presses with modifiers)
- Session management
- Basic authentication framework
- Comprehensive build system
- Unit and integration tests

### 🚧 For Production Enhancement
- Full JWT authentication implementation
- Global input capture hooks
- TLS/SSL encryption
- User management database
- GUI applications
- Advanced session management
- Logging and audit trails
- Performance optimization

## Quick Start

### Prerequisites

- Rust (1.70+) with Cargo
- Platform-specific dependencies:
  - **macOS**: Xcode Command Line Tools
  - **Windows**: Visual Studio Build Tools or MSVC

### Building Native Binaries

```bash
# Clone and build all components
git clone <repository>
cd remote_hid

# Build all components in release mode (optimized)
cargo build --workspace --release

# Build individual components
cargo build --bin session-server --release
cargo build --bin hid-client --release
cargo build --bin commander --release

# Debug builds (faster compilation, includes debug symbols)
cargo build --workspace

# Check build without compilation (fast syntax check)
cargo check --workspace
```

#### Native Binary Locations

After building, binaries are located in:
```
target/release/
├── session-server      # Session management server
├── hid-client         # Target machine client
└── commander          # Control machine client
```

#### Cross-Platform Building

```bash
# List available targets
rustup target list

# Add Windows target (from macOS/Linux)
rustup target add x86_64-pc-windows-gnu

# Add macOS target (from Linux/Windows)
rustup target add x86_64-apple-darwin

# Build for specific target
cargo build --target x86_64-pc-windows-gnu --release
```

### Running the System

1. **Start the Session Server:**
   ```bash
   # Development mode
   cargo run --bin session-server
   
   # Production mode (optimized binary)
   ./target/release/session-server
   
   # With custom configuration
   cargo run --bin session-server -- --host 0.0.0.0 --port 8081 --debug
   ```

2. **Start HID Client on target machine:**
   ```bash
   # Development mode
   cargo run --bin hid-client -- --server ws://127.0.0.1:8080 --client-id "my-machine"
   
   # Production mode
   ./target/release/hid-client --server ws://127.0.0.1:8080 --client-id "my-machine" --client-name "Office Computer"
   ```

3. **Start Commander to control remote machine:**
   ```bash
   # Development mode
   cargo run --bin commander -- --server ws://127.0.0.1:8080 --target "my-machine"
   
   # Production mode
   ./target/release/commander --server ws://127.0.0.1:8080 --target "my-machine"
   ```

## Usage Examples

### Basic Remote Control Session

1. **Server Setup:**
   ```bash
   # Start server on default port 8080
   ./target/release/session-server
   
   # Or with custom configuration
   ./target/release/session-server --host 0.0.0.0 --port 9000
   ```

2. **Target Machine Setup:**
   ```bash
   # Connect HID client to server
   ./target/release/hid-client --server ws://192.168.1.100:8080 --client-id "office-pc"
   ```

3. **Control from Commander:**
   ```bash
   # Connect and start controlling
   ./target/release/commander --server ws://192.168.1.100:8080 --target "office-pc"
   ```

### Configuration

The session server supports configuration via TOML file:

```toml
[server]
host = "127.0.0.1"
port = 8080
max_connections = 1000

[auth]
jwt_secret = "your-secret-key"
token_expiry_hours = 24

[session]
max_sessions = 100
session_timeout_mins = 60
```

## Development

### Project Structure

```
remote_hid/
├── shared/           # Common protocol and utilities
├── session-server/   # Central message broker
├── hid-client/       # Target machine agent
├── commander/        # Operator control interface
├── tests/           # Integration tests
├── Cargo.toml        # Workspace configuration
├── ARCHITECTURE.md   # Detailed architecture docs
└── README.md         # This file
```

### Building for Different Platforms

```bash
# Build for current platform
cargo build --workspace --release

# Cross-compile for multiple platforms
cargo build --target x86_64-pc-windows-gnu --release
cargo build --target x86_64-apple-darwin --release
cargo build --target x86_64-unknown-linux-gnu --release

# Create optimized distribution binaries
cargo build --workspace --release --profile release

# Strip debug symbols for smaller binaries (Unix systems)
strip target/release/session-server
strip target/release/hid-client  
strip target/release/commander
```

#### Binary Sizes (Approximate)

- **session-server**: ~8-12 MB
- **hid-client**: ~6-10 MB  
- **commander**: ~6-10 MB

#### Installation

```bash
# Install to system PATH
cargo install --path session-server
cargo install --path hid-client
cargo install --path commander

# Or copy binaries manually
cp target/release/{session-server,hid-client,commander} /usr/local/bin/
```

### Running Tests

#### Complete Test Suite (73 Tests Total)

```bash
# Run all tests across the entire workspace
cargo test --workspace

# Run tests with output for debugging
cargo test --workspace -- --nocapture

# Run tests in release mode for performance
cargo test --workspace --release
```

#### Component-Specific Tests

```bash
# Test individual components
cargo test --package commander          # 13 tests
cargo test --package hid-client         # 15 tests  
cargo test --package session-server     # 17 tests
cargo test --package remote-hid-shared  # 19 tests

# Test with specific pattern
cargo test --package commander -- char_to_keycode
cargo test input_capture
```

#### Integration Tests (9 Tests)

```bash
# Run comprehensive integration tests
cargo test --package integration-tests

# Run specific integration test
cargo test --package integration-tests -- test_full_message_protocol_compatibility
cargo test --package integration-tests -- test_complex_key_combination_scenario
```

#### Test Categories

The test suite includes:

- **Unit Tests** (64 tests):
  - Commander: 13 tests - Input capture and conversion
  - HID Client: 15 tests - Event processing and execution  
  - Session Server: 17 tests - Session management and config
  - Shared Library: 19 tests - Protocol and message validation

- **Integration Tests** (9 tests):
  - Protocol compatibility testing
  - Complex interaction scenarios (typing, mouse operations)
  - Session lifecycle testing
  - Error handling and edge case validation
  - Performance and concurrency testing

#### Expected Test Output

```
running 73 tests across workspace...

Commander Tests:         13 passed ✅
HID Client Tests:        15 passed ✅  
Session Server Tests:    17 passed ✅
Shared Library Tests:    19 passed ✅
Integration Tests:        9 passed ✅

Total: 73 passed, 0 failed
```

### Code Quality & Development Tools

```bash
# Format code
cargo fmt --workspace

# Run linter
cargo clippy --workspace

# Security audit
cargo audit

# Check for outdated dependencies
cargo outdated

# Clean build artifacts
cargo clean

# Generate and open documentation
cargo doc --workspace --open
```

#### Development Workflow

```bash
# Quick development cycle
cargo check --workspace           # Fast syntax checking
cargo build --workspace           # Build all components
cargo test --workspace           # Run all tests
cargo clippy --workspace         # Lint code
cargo fmt --workspace           # Format code

# Continuous development with file watching (requires cargo-watch)
cargo install cargo-watch
cargo watch -x "build --workspace" -x "test --workspace"
```

## Platform-Specific Notes

### macOS

- **Permissions Required**: Accessibility permissions for input injection
- **Setup**: Grant accessibility access in System Preferences → Security & Privacy → Privacy → Accessibility
- **APIs Used**: Core Graphics Event Services (CGEvent*)

### Windows

- **Permissions Required**: Usually runs without special permissions
- **UAC Considerations**: May need elevated privileges for certain applications
- **APIs Used**: Windows Input API (SendInput, Windows message system)

## Security Considerations

⚠️ **Important Security Notes:**

1. **Network Security**: Current implementation uses plain WebSocket. For production, implement TLS/WSS.
2. **Authentication**: Basic framework provided. Implement proper JWT validation for production.
3. **Input Validation**: All HID commands are sanitized before execution.
4. **Access Control**: Consider implementing role-based access control.
5. **Audit Logging**: Enable comprehensive logging for security monitoring.

## Protocol Documentation

### Message Format

All communication uses JSON messages with this structure:

```json
{
  "message_type": "HidEvent|Auth|SessionControl|Status",
  "session_id": "uuid-optional",
  "timestamp": "2023-12-01T10:00:00Z",
  "payload": { /* message-specific data */ }
}
```

### HID Event Types

#### Mouse Events
```json
// Mouse movement
{
  "event_type": "MouseMove",
  "x": 100,
  "y": 200,
  "absolute": true
}

// Mouse click
{
  "event_type": "MouseClick",
  "button": "Left|Right|Middle",
  "pressed": true,
  "x": 100,
  "y": 200
}

// Mouse scroll
{
  "event_type": "MouseScroll",
  "delta_x": 0,
  "delta_y": 3
}
```

#### Keyboard Events
```json
{
  "event_type": "KeyEvent",
  "key": "A|Space|Enter|...",
  "pressed": true,
  "modifiers": {
    "shift": false,
    "control": true,
    "alt": false,
    "super_key": false
  }
}
```

## Troubleshooting

### Common Issues

1. **Connection Refused**
   - Ensure session server is running
   - Check firewall settings
   - Verify correct server URL and port

2. **HID Events Not Working (macOS)**
   - Grant Accessibility permissions
   - Run with `sudo` if necessary (not recommended)

3. **HID Events Not Working (Windows)**
   - Run as Administrator if targeting elevated applications
   - Check Windows Defender settings

4. **Input Capture Not Working**
   - Current implementation uses simplified console input
   - For production, implement proper global hooks

### Debug Mode & Troubleshooting

Run any component with debug logging:

```bash
# Enable debug logging with environment variable
RUST_LOG=debug ./target/release/session-server
RUST_LOG=debug cargo run --bin session-server

# Or use built-in debug flags
./target/release/session-server --debug
cargo run --bin session-server -- --debug

# Component-specific debugging
RUST_LOG=session_server=debug,remote_hid_shared=info ./target/release/session-server
```

#### Build Troubleshooting

```bash
# Clean build artifacts if having issues
cargo clean

# Update Rust toolchain
rustup update

# Check for common issues
cargo check --workspace
cargo clippy --workspace

# Verbose build output
cargo build --workspace --verbose
```

### Log Analysis

All components use structured logging with tracing. Key log levels:
- `ERROR`: Critical issues requiring attention
- `WARN`: Important notices (e.g., permission issues)
- `INFO`: General operational information
- `DEBUG`: Detailed execution traces

## Contributing

### Code Style
- Use `cargo fmt` for formatting
- Run `cargo clippy` for linting
- Follow existing patterns in the codebase

### Testing
- Add unit tests for new functionality
- Include integration tests for protocol changes
- Test cross-platform compatibility

### Documentation
- Update README for new features
- Document protocol changes in DESIGN.md
- Include inline code documentation

## Performance Considerations

- **Latency**: WebSocket provides low-latency communication
- **Throughput**: Current implementation can handle typical HID event rates
- **Memory**: Async/await design minimizes memory footprint
- **CPU**: Platform-specific HID APIs are efficient

For high-frequency applications (gaming, real-time control), consider:
- UDP-based transport for lowest latency
- Event batching and compression
- Local input prediction

## License

This project is provided as-is for educational and development purposes. 
Please ensure compliance with local laws and regulations regarding remote access software.

## Future Enhancements

### Short Term
- [ ] Complete JWT authentication
- [ ] TLS/WSS encryption
- [ ] GUI applications
- [ ] Global input hooks

### Long Term
- [ ] Multi-session support
- [ ] File transfer capabilities
- [ ] Screen sharing integration
- [ ] Mobile client support
- [ ] Cloud deployment guides

## Support

For issues and questions:
1. Check the troubleshooting section
2. Review the DESIGN.md for detailed architecture
3. Enable debug logging to diagnose issues
4. Check platform-specific requirements

---

**⚠️ Security Warning**: This is a demonstration implementation. For production use, implement proper authentication, encryption, and security auditing.