# Build and Deployment Guide

This document provides detailed instructions for building, testing, and deploying the Remote HID Control System.

## Prerequisites

### System Requirements

#### All Platforms
- **Rust**: 1.70 or later with Cargo
- **Git**: For source code management
- **Make**: Build automation (optional, can use cargo directly)

#### macOS
```bash
# Install Xcode Command Line Tools
xcode-select --install

# Install Rust (if not already installed)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env
```

#### Windows
```bash
# Install Visual Studio Build Tools or Visual Studio Community
# Download from: https://visualstudio.microsoft.com/downloads/

# Install Rust (if not already installed)
# Download from: https://rustup.rs/
# Or using winget:
winget install Rustlang.Rust.MSVC
```

#### Linux (Ubuntu/Debian)
```bash
# Install build essentials
sudo apt update
sudo apt install build-essential pkg-config

# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env
```

### Development Tools (Optional)
```bash
# Additional Rust tools for development
rustup component add rustfmt clippy
cargo install cargo-audit cargo-tarpaulin cross
```

## Building from Source

### 1. Clone the Repository
```bash
git clone <repository-url>
cd remote_hid
```

### 2. Set up Development Environment
```bash
# Using Makefile (recommended)
make dev-setup

# Or manually
rustup update
rustup component add rustfmt clippy
```

### 3. Build All Components

#### Debug Build (Development)
```bash
# Using Makefile
make build-debug

# Or using Cargo directly
cargo build --workspace
```

#### Release Build (Production)
```bash
# Using Makefile
make build

# Or using Cargo directly
cargo build --release --workspace
```

### 4. Verify Build
```bash
# Run tests to verify everything works
make test

# Check binary locations
ls -la target/release/
# Should show: session-server, hid-client, commander
```

## Testing

### Unit Tests
```bash
# Run all unit tests
make test

# Test specific component
cargo test -p remote-hid-shared
cargo test -p session-server
cargo test -p hid-client
cargo test -p commander
```

### Integration Tests
```bash
# Run integration tests
make test-integration

# Manual integration test
# Terminal 1: Start server
./target/release/session-server --debug

# Terminal 2: Start HID client
./target/release/hid-client --debug --client-id test-client

# Terminal 3: Start commander
./target/release/commander --debug --target test-client
```

### Code Quality Checks
```bash
# Format code
make format

# Run linter
make lint

# Security audit
make security-audit

# Generate test coverage report
make coverage
```

## Cross-Platform Building

### Cross-Compilation Setup
```bash
# Install cross-compilation tool
cargo install cross

# Add target platforms
rustup target add x86_64-pc-windows-gnu
rustup target add x86_64-apple-darwin
rustup target add aarch64-apple-darwin
```

### Build for Multiple Platforms
```bash
# Using Makefile (requires 'cross' tool)
make cross-compile

# Manual cross-compilation
cross build --target x86_64-pc-windows-gnu --release
cross build --target x86_64-apple-darwin --release
cross build --target aarch64-apple-darwin --release
```

### Platform-Specific Notes

#### macOS (ARM64/M1/M2)
```bash
# Native build
cargo build --release

# Cross-compile for Intel macOS
cargo build --target x86_64-apple-darwin --release
```

#### Windows
```bash
# Native build (requires MSVC or MinGW)
cargo build --release

# Cross-compile from Linux/macOS
cross build --target x86_64-pc-windows-gnu --release
```

## Packaging and Distribution

### Create Release Packages
```bash
# Create platform-specific packages
make package

# This creates:
# packages/macos/remote-hid-macos-YYYYMMDD.tar.gz
# packages/windows/remote-hid-windows-YYYYMMDD.tar.gz
# packages/linux/remote-hid-linux-YYYYMMDD.tar.gz
```

### Manual Packaging
```bash
# After building, create distribution directory
mkdir -p dist/remote-hid
cp target/release/session-server dist/remote-hid/
cp target/release/hid-client dist/remote-hid/
cp target/release/commander dist/remote-hid/
cp README.md dist/remote-hid/
cp BUILD.md dist/remote-hid/

# Create archive
tar -czf remote-hid-$(date +%Y%m%d).tar.gz -C dist remote-hid
```

## Installation

### System Installation (Optional)
```bash
# Install binaries to system PATH
make install

# This installs to ~/.cargo/bin/ by default
# Ensure ~/.cargo/bin is in your PATH
```

### Manual Installation
```bash
# Copy binaries to desired location
cp target/release/session-server /usr/local/bin/
cp target/release/hid-client /usr/local/bin/
cp target/release/commander /usr/local/bin/

# Make executable (Unix-like systems)
chmod +x /usr/local/bin/session-server
chmod +x /usr/local/bin/hid-client
chmod +x /usr/local/bin/commander
```

## Deployment Configurations

### Development Deployment
```bash
# All components on localhost
# Terminal 1: Server
./session-server --host 127.0.0.1 --port 8080 --debug

# Terminal 2: HID Client
./hid-client --server ws://127.0.0.1:8080 --client-id dev-client --debug

# Terminal 3: Commander  
./commander --server ws://127.0.0.1:8080 --target dev-client --debug
```

### Production Deployment

#### Server Configuration
```toml
# config.toml
[server]
host = "0.0.0.0"  # Listen on all interfaces
port = 8080
max_connections = 1000
heartbeat_interval_secs = 30

[auth]
jwt_secret = "your-production-secret-key-change-this"
token_expiry_hours = 8
max_failed_attempts = 3
lockout_duration_mins = 15

[session]
max_sessions = 50
session_timeout_mins = 60
cleanup_interval_secs = 300
```

#### Server Startup
```bash
# Using configuration file
./session-server --config /etc/remote-hid/config.toml

# Or with command line options
./session-server --host 0.0.0.0 --port 8080
```

#### Client Deployment
```bash
# HID Client (on target machine)
./hid-client \
  --server ws://your-server.com:8080 \
  --client-id "$(hostname)" \
  --client-name "$(hostname) - $(whoami)"

# Commander (on operator machine)
./commander \
  --server ws://your-server.com:8080 \
  --target target-hostname
```

### Docker Deployment (Session Server)
```dockerfile
# Dockerfile
FROM rust:1.70-slim as builder
WORKDIR /app
COPY . .
RUN cargo build --release -p session-server

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/target/release/session-server /usr/local/bin/
EXPOSE 8080
CMD ["session-server", "--host", "0.0.0.0", "--port", "8080"]
```

```bash
# Build and run
docker build -t remote-hid-server .
docker run -p 8080:8080 remote-hid-server
```

### Systemd Service (Linux)
```ini
# /etc/systemd/system/remote-hid-server.service
[Unit]
Description=Remote HID Session Server
After=network.target

[Service]
Type=simple
User=remote-hid
Group=remote-hid
WorkingDirectory=/opt/remote-hid
ExecStart=/opt/remote-hid/session-server --config /etc/remote-hid/config.toml
Restart=always
RestartSec=5

[Install]
WantedBy=multi-user.target
```

```bash
# Enable and start service
sudo systemctl enable remote-hid-server
sudo systemctl start remote-hid-server
sudo systemctl status remote-hid-server
```

## Performance Optimization

### Build Optimizations
```toml
# Add to Cargo.toml for production builds
[profile.release]
codegen-units = 1
lto = true
panic = "abort"
```

### Runtime Optimizations
```bash
# Set optimal environment variables
export RUST_LOG=info  # Reduce debug overhead
export TOKIO_WORKER_THREADS=4  # Adjust based on CPU cores
```

## Security Hardening

### Build Security
```bash
# Security audit
cargo audit

# Check for vulnerable dependencies
cargo audit --db ./advisory-db
```

### Deployment Security
1. **Use TLS/WSS in production**
2. **Change default JWT secret**
3. **Run with minimal privileges**
4. **Enable audit logging**
5. **Use firewall rules**
6. **Regular security updates**

## Troubleshooting Build Issues

### Common Build Errors

#### Missing System Dependencies
```bash
# macOS
xcode-select --install

# Ubuntu/Debian
sudo apt install build-essential pkg-config libssl-dev

# CentOS/RHEL
sudo yum groupinstall "Development Tools"
sudo yum install openssl-devel
```

#### Rust Version Issues
```bash
# Update Rust
rustup update stable

# Check version
rustc --version  # Should be 1.70+
```

#### Cross-Compilation Issues
```bash
# Install target
rustup target add x86_64-pc-windows-gnu

# Install cross tool
cargo install cross

# Use cross instead of cargo
cross build --target x86_64-pc-windows-gnu --release
```

### Platform-Specific Issues

#### macOS Code Signing
```bash
# For distribution, you may need to sign binaries
codesign --force --deep --sign "Developer ID Application: Your Name" target/release/hid-client
```

#### Windows Antivirus
- Some antivirus software may flag the binaries
- Add build directory to exclusions during development
- For distribution, consider code signing

## CI/CD Integration

### GitHub Actions Example
```yaml
# .github/workflows/build.yml
name: Build and Test

on: [push, pull_request]

jobs:
  test:
    runs-on: ${{ matrix.os }}
    strategy:
      matrix:
        os: [ubuntu-latest, windows-latest, macos-latest]
    
    steps:
    - uses: actions/checkout@v3
    - uses: actions-rs/toolchain@v1
      with:
        toolchain: stable
    
    - name: Run tests
      run: cargo test --all
    
    - name: Build release
      run: cargo build --release --all
```

## Performance Benchmarking

```bash
# Benchmark protocol serialization
cargo bench -p remote-hid-shared

# Profile server performance
cargo build --release
perf record ./target/release/session-server
perf report
```

## Maintenance

### Regular Updates
```bash
# Update dependencies
cargo update

# Security audit
make security-audit

# Rebuild and test
make clean build test
```

### Monitoring
- Monitor WebSocket connections
- Track HID event latency
- Log authentication attempts
- Monitor system resource usage

---

This build guide covers the complete process from source to deployment. For specific deployment scenarios or additional questions, refer to the main README.md or create an issue in the project repository.