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

### Building

```bash
# Clone and build all components
git clone <repository>
cd remote_hid

# Set up development environment
make dev-setup

# Build all components
make build

# Or build in debug mode
make build-debug
```

### Running the System

1. **Start the Session Server:**
   ```bash
   make run-server
   # Or directly:
   cd session-server && cargo run
   ```

2. **Start HID Client on target machine:**
   ```bash
   make run-hid-client
   # Or with custom parameters:
   cd hid-client && cargo run -- --client-id "my-machine" --client-name "Office Computer"
   ```

3. **Start Commander to control remote machine:**
   ```bash
   make run-commander
   # Or with target specification:
   cd commander && cargo run -- --target "my-machine"
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
├── Cargo.toml        # Workspace configuration
├── Makefile          # Build automation
└── DESIGN.md         # Detailed architecture docs
```

### Building for Different Platforms

```bash
# Build for current platform
make build

# Cross-compile for multiple platforms
make cross-compile

# Create distribution packages
make package
```

### Running Tests

```bash
# Run all tests
make test

# Run integration tests
make test-integration

# Generate code coverage
make coverage
```

### Code Quality

```bash
# Format code
make format

# Run linter
make lint

# Security audit
make security-audit
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

### Debug Mode

Run any component with debug logging:

```bash
# Enable debug logging
RUST_LOG=debug ./target/release/session-server
# Or using make targets
make run-server RUST_LOG=debug
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