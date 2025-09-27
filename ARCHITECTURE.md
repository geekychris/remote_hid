# Remote HID System Architecture

## Overview

The Remote HID System is a distributed application that enables remote control of keyboard and mouse input across devices. The system consists of four main components working together to capture input events on one machine (Commander) and execute them on a target machine (HID Client) via a central Session Server.

## System Components

### 1. Session Server (`session-server`)
The central WebSocket server that manages connections and routes messages between commanders and HID clients.

**Responsibilities:**
- Accept WebSocket connections from commanders and HID clients
- Manage session creation and pairing
- Route HID events from commanders to target HID clients
- Handle client registration and discovery
- Session lifecycle management

**Key Files:**
- `src/server.rs` - Main WebSocket server implementation
- `src/session.rs` - Session management logic
- `src/config.rs` - Configuration management

### 2. HID Client (`hid-client`)
Runs on the target machine that will receive and execute HID events.

**Responsibilities:**
- Connect to session server and register as available HID client
- Receive HID events via WebSocket
- Execute keyboard and mouse events on the local system
- Platform-specific HID event injection

**Key Files:**
- `src/client.rs` - WebSocket client and message handling
- `src/hid.rs` - Platform-specific HID event execution
- `src/main.rs` - Application entry point

### 3. Commander (`commander`)
Runs on the controlling machine to capture local input and send to target HID client.

**Responsibilities:**
- Capture local keyboard and mouse input
- Connect to session server and join target HID client session
- Send captured input events to session server
- Handle session management

**Key Files:**
- `src/client.rs` - WebSocket client and session management
- `src/input_capture.rs` - Platform-specific input capture
- `src/main.rs` - Application entry point

### 4. Shared Library (`shared`)
Common types, protocols, and utilities used by all components.

**Responsibilities:**
- Define message protocols and data structures
- Authentication types and utilities
- Error handling types
- Serialization/deserialization logic

**Key Files:**
- `src/protocol.rs` - Core message types and protocols
- `src/auth.rs` - Authentication types
- `src/error.rs` - Error handling

## Data Flow Architecture

```
┌─────────────┐    WebSocket     ┌─────────────────┐    WebSocket     ┌─────────────┐
│ Commander   │◄────────────────►│ Session Server  │◄────────────────►│ HID Client  │
│             │                  │                 │                  │             │
│ ┌─────────┐ │                  │ ┌─────────────┐ │                  │ ┌─────────┐ │
│ │ Input   │ │   HID Events     │ │ Session     │ │   HID Events     │ │ HID     │ │
│ │Capture  │ │─────────────────►│ │ Manager     │ │─────────────────►│ │Handler  │ │
│ └─────────┘ │                  │ └─────────────┘ │                  │ └─────────┘ │
│             │                  │                 │                  │             │
└─────────────┘                  └─────────────────┘                  └─────────────┘
```

## Input Capture Architecture

### Commander Input Capture

The Commander captures keyboard and mouse input using platform-specific low-level hooks:

#### macOS Implementation
- **Current Status**: Simplified implementation using console input
- **Production Requirements**:
  - Use `CGEventTap` for global event capture
  - Handle accessibility permissions (`AXIsProcessTrusted`)
  - Filter out self-generated events to prevent loops

**Third-party Libraries:**
- `core-graphics` - Core Graphics framework bindings for event creation
- `core-foundation` - Core Foundation utilities
- `cocoa` - Cocoa framework bindings
- `objc` - Objective-C runtime bindings

**Key APIs Used:**
- `CGEventTapCreate` - Create system-wide event tap
- `CGEventTapEnable` - Enable/disable event capture
- `CGEventGetType` - Get event type (mouse move, key press, etc.)
- `CGEventGetLocation` - Get mouse coordinates
- `CGEventGetIntegerValueField` - Extract key codes and button states

#### Windows Implementation
- **Current Status**: Simplified implementation using console input
- **Production Requirements**:
  - Use `SetWindowsHookEx` with `WH_MOUSE_LL` and `WH_KEYBOARD_LL`
  - Handle low-level hook procedures
  - Implement proper cleanup and unhooking

**Third-party Libraries:**
- `windows` - Official Windows API bindings for Rust

**Key APIs Used:**
- `SetWindowsHookEx` - Install low-level hooks
- `GetMessage`/`PeekMessage` - Process hook messages
- `POINT` structures for mouse coordinates
- Virtual key codes for keyboard events

### Input Event Pipeline

1. **Capture**: Platform-specific hooks capture raw input events
2. **Convert**: Raw events converted to common `InputEvent` enum
3. **Filter**: Events filtered to avoid capturing self-generated events
4. **Serialize**: Events converted to `HidEvent` messages
5. **Transmit**: Messages sent via WebSocket to session server
6. **Route**: Session server routes to target HID client
7. **Execute**: HID client executes events on target system

## HID Event Execution Architecture

### HID Client Event Execution

The HID Client receives HID events and executes them on the local system:

#### macOS Implementation
**Third-party Libraries:**
- `core-graphics` - Core Graphics framework for event injection

**Key APIs Used:**
- `CGEventSource` - Create event source for injection
- `CGEvent::new_mouse_event` - Create mouse events
- `CGEvent::new_keyboard_event` - Create keyboard events
- `CGEvent::post` - Inject events into system

**Event Types Supported:**
- Mouse movement (absolute/relative)
- Mouse clicks (left, right, middle, X1, X2)
- Mouse scrolling
- Keyboard key presses/releases
- Modifier key handling

#### Windows Implementation
**Third-party Libraries:**
- `windows` - Official Windows API bindings

**Key APIs Used:**
- `SendInput` - Inject input events into system
- `INPUT` structures for mouse and keyboard events
- `MOUSEINPUT` for mouse event data
- `KEYBDINPUT` for keyboard event data

**Event Types Supported:**
- Mouse movement (absolute/relative coordinates)
- Mouse clicks with proper button mapping
- Mouse wheel scrolling (vertical/horizontal)
- Keyboard events with virtual key codes
- Modifier key states

## Communication Protocol

### WebSocket Message Structure

All communication uses JSON-serialized messages with the following structure:

```rust
struct Message {
    message_type: MessageType,  // Auth, HidEvent, SessionControl, Status
    session_id: Option<Uuid>,   // Session identifier (if applicable)
    timestamp: DateTime<Utc>,   // Message timestamp
    payload: MessagePayload,    // Actual message content
}
```

### Message Types

1. **Authentication Messages** (`MessageType::Auth`)
   - Login requests and responses
   - Token refresh
   - Logout notifications

2. **HID Event Messages** (`MessageType::HidEvent`)
   - Mouse movement, clicks, scrolling
   - Keyboard key presses and releases
   - Modifier key states

3. **Session Control Messages** (`MessageType::SessionControl`)
   - Session creation (HID Client registration)
   - Session joining (Commander connection)
   - Client listing and discovery
   - Session termination

4. **Status Messages** (`MessageType::Status`)
   - Heartbeat/keepalive
   - Connection status
   - Error notifications

### Session Management Flow

1. **HID Client Registration:**
   ```
   HID Client → Session Server: CreateSession { client_id, client_name }
   Session Server: Registers client, creates session entry
   ```

2. **Commander Connection:**
   ```
   Commander → Session Server: JoinSession { target_client_id }
   Session Server: Validates target exists, establishes session
   ```

3. **Event Forwarding:**
   ```
   Commander → Session Server: HidEvent { ... }
   Session Server → HID Client: HidEvent { ... }
   HID Client: Executes event locally
   ```

## Security Considerations

### Current Limitations
- No authentication implementation (placeholder only)
- No encryption beyond WebSocket TLS
- No input validation or rate limiting
- No session access control

### Production Requirements
- JWT-based authentication
- Rate limiting for input events
- Session access control and permissions
- Input validation and sanitization
- Audit logging for security events

## Performance Characteristics

### Latency Sources
1. **Input Capture Latency**: Platform hook processing time
2. **Network Latency**: WebSocket communication delay
3. **Serialization Overhead**: JSON encoding/decoding
4. **Event Injection Latency**: Platform API call time

### Optimization Strategies
- Binary message format instead of JSON
- Event batching for high-frequency events
- Compression for large message payloads
- Connection pooling and keepalive

## Platform Support

### Supported Platforms
- **macOS**: Full support planned (simplified implementation current)
- **Windows**: Full support planned (simplified implementation current)
- **Linux**: Not implemented (could use X11/Wayland APIs)

### Platform-Specific Considerations

#### macOS
- Requires accessibility permissions for global input capture
- Sandbox restrictions may limit functionality
- SIP (System Integrity Protection) considerations

#### Windows
- May require administrator privileges for low-level hooks
- UAC (User Account Control) considerations
- Windows Defender may flag as potentially unwanted

## Dependencies Overview

### Core Dependencies
- `tokio` - Async runtime for all network operations
- `tokio-tungstenite` - WebSocket client/server implementation
- `serde`/`serde_json` - Message serialization
- `anyhow`/`thiserror` - Error handling
- `uuid` - Session and message identifiers
- `chrono` - Timestamp handling

### Platform-Specific Dependencies

#### macOS
- `core-graphics` (0.23) - Core Graphics framework bindings
- `core-foundation` (0.9) - Core Foundation utilities
- `cocoa` (0.25) - Cocoa framework access
- `objc` (0.2) - Objective-C runtime interop

#### Windows
- `windows` (0.52) - Official Microsoft Windows API bindings
  - `Win32_Foundation` - Basic Windows types
  - `Win32_UI_Input_KeyboardAndMouse` - Input APIs
  - `Win32_UI_WindowsAndMessaging` - Window messaging
  - `Win32_System_Threading` - Threading utilities

## Building and Testing

### Building Native Binaries

```bash
# Build all components in release mode (optimized)
cargo build --workspace --release

# Build individual components
cargo build --bin session-server --release
cargo build --bin hid-client --release
cargo build --bin commander --release

# Cross-platform building
cargo build --target x86_64-pc-windows-gnu --release
cargo build --target x86_64-apple-darwin --release
```

#### Binary Locations

Built binaries are located in:
```
target/release/
├── session-server      # WebSocket server for session management
├── hid-client         # Target machine HID event executor  
└── commander          # Control machine input capturer
```

### Comprehensive Test Suite (73 Tests)

#### Running All Tests

```bash
# Run complete test suite
cargo test --workspace

# Run with detailed output
cargo test --workspace -- --nocapture
```

#### Component Test Breakdown

- **Commander Tests** (13 tests): Input capture, keycode mapping, event conversion
- **HID Client Tests** (15 tests): Event processing, message handling, coordinate validation
- **Session Server Tests** (17 tests): Session management, configuration, message routing
- **Shared Library Tests** (19 tests): Protocol validation, serialization, authentication
- **Integration Tests** (9 tests): End-to-end protocol compatibility, complex scenarios

#### Test Categories

**Unit Tests** validate individual component functionality:
```bash
cargo test --package commander          # Input capture testing
cargo test --package hid-client         # HID event execution testing
cargo test --package session-server     # Server and session management
cargo test --package remote-hid-shared  # Protocol and message validation
```

**Integration Tests** validate system-wide functionality:
```bash
cargo test --package integration-tests  # End-to-end protocol testing
```

#### Key Test Scenarios

1. **Protocol Compatibility**: Full message flow validation
2. **Complex Interactions**: Typing sequences, drag-and-drop operations
3. **Session Lifecycle**: Complete session creation to termination
4. **Error Handling**: Edge cases and error conditions
5. **Performance**: Message size limits, coordinate extremes
6. **Concurrency**: Multiple simultaneous sessions

### Development Workflow

```bash
# Quick development cycle
cargo check --workspace           # Fast syntax checking
cargo build --workspace           # Build all components
cargo test --workspace            # Run all tests
cargo clippy --workspace          # Lint code
cargo fmt --workspace             # Format code

# Continuous testing
cargo install cargo-watch
cargo watch -x "test --workspace"
```

## Future Enhancements

### Planned Features
1. **Full Input Capture**: Complete low-level hook implementations
2. **Authentication**: JWT-based authentication system
3. **Encryption**: End-to-end encryption for sensitive events
4. **Multi-Session**: Support multiple concurrent sessions
5. **Event Recording**: Record and replay input sequences
6. **Cross-Platform**: Linux support via X11/Wayland
7. **Performance Monitoring**: Latency and throughput metrics
8. **GUI Management**: Desktop application with UI

### Technical Debt
1. **Error Handling**: More granular error types and recovery
2. **Documentation**: API documentation and usage examples
3. **Configuration**: More flexible configuration options
4. **Logging**: Structured logging with proper levels
5. **Monitoring**: Health checks and observability
