# Remote HID Control System Design

## Overview

This system enables remote control of keyboard and mouse input across Windows and macOS machines through a client-server architecture with three main components:

1. **HID Client** - Runs on target machine, receives and executes HID commands
2. **Commander** - Operator interface that captures local input and sends commands 
3. **Session Server** - Central server that manages connections and forwards messages

## Architecture

```
[Commander] <---> [Session Server] <---> [HID Client]
   (Operator)         (Broker)           (Target Machine)
```

## Communication Protocol

### Transport Layer
- **Protocol**: WebSocket over TLS for secure, bidirectional communication
- **Authentication**: JWT tokens with username/password authentication
- **Session Management**: UUID-based session identification

### Message Format
JSON-based protocol with the following message types:

```json
{
  "type": "auth|hid_event|session_control",
  "session_id": "uuid",
  "timestamp": "iso8601",
  "payload": {}
}
```

### Message Types

#### Authentication Messages
- `auth_request`: Client authentication with credentials
- `auth_response`: Server response with JWT token
- `auth_refresh`: Token refresh requests

#### HID Event Messages
- `mouse_move`: Relative/absolute mouse movement
- `mouse_click`: Mouse button press/release
- `mouse_scroll`: Mouse wheel events
- `key_press`: Keyboard key press
- `key_release`: Keyboard key release

#### Session Control Messages
- `session_create`: Create new control session
- `session_join`: Join existing session as commander
- `session_list`: List available HID clients
- `session_end`: Terminate session

## Technology Stack

### Programming Language: Rust
**Rationale**: 
- Excellent cross-platform support
- Memory safety without garbage collection
- Strong async/networking ecosystem
- Good FFI capabilities for system APIs

### Key Libraries

#### Networking & WebSockets
- `tokio`: Async runtime
- `tokio-tungstenite`: WebSocket implementation
- `rustls`: TLS support

#### Authentication & Security
- `jsonwebtoken`: JWT token handling
- `bcrypt`: Password hashing
- `uuid`: Session ID generation

#### HID Platform APIs
- `windows-rs`: Windows API bindings
- `core-foundation` + `core-graphics`: macOS APIs
- Custom FFI wrappers for low-level HID access

#### Serialization
- `serde`: JSON serialization/deserialization
- `serde_json`: JSON handling

#### Testing
- `tokio-test`: Async testing utilities
- `mockall`: Mock object generation

## Component Details

### Session Server
- **Role**: Central broker for all communications
- **Features**:
  - Client authentication and authorization
  - Session management and routing
  - Message forwarding between clients
  - Connection health monitoring
  - Logging and audit trail

### HID Client
- **Role**: Execute HID commands on target machine
- **Features**:
  - Cross-platform HID API abstraction
  - Secure connection to session server
  - Command validation and sanitization
  - Platform-specific input injection

### Commander
- **Role**: Capture and forward operator input
- **Features**:
  - Local HID event capture
  - UI for session management
  - Real-time command transmission
  - Connection status monitoring

## Security Considerations

1. **Authentication**: Strong password policies, JWT with expiration
2. **Authorization**: Role-based access control for sessions
3. **Encryption**: TLS encryption for all communications
4. **Input Validation**: Sanitize all HID commands before execution
5. **Audit Logging**: Comprehensive logging of all actions
6. **Rate Limiting**: Prevent command flooding

## Cross-Platform Considerations

### Windows HID APIs
- `SendInput()`: Low-level input injection
- Windows message handling
- User Account Control (UAC) considerations

### macOS HID APIs  
- `CGEventCreateMouseEvent()`: Mouse events
- `CGEventCreateKeyboardEvent()`: Keyboard events
- Accessibility permissions requirements
- Quartz Event Services

## Build System

- **Primary**: Cargo for Rust dependency management
- **Cross-compilation**: Target-specific builds for Windows/macOS
- **CI/CD**: GitHub Actions for automated testing and builds
- **Packaging**: Platform-specific installers/packages

## Testing Strategy

1. **Unit Tests**: Component-level testing
2. **Integration Tests**: Inter-component communication
3. **Platform Tests**: OS-specific HID functionality
4. **Security Tests**: Authentication and authorization
5. **Performance Tests**: Latency and throughput measurement

## Deployment

### Session Server
- Containerized deployment (Docker)
- Cloud hosting with load balancing
- Database for user management (SQLite/PostgreSQL)

### Client Applications
- Standalone executables
- Platform-specific installation packages
- Auto-update mechanisms