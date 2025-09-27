# Remote HID Control System Makefile

.PHONY: all build test clean install dev-setup format lint doc
.DEFAULT_GOAL := help

# Variables
CARGO := cargo
RUST_LOG ?= info
TARGET_DIR := target

# Platform detection
UNAME_S := $(shell uname -s)
UNAME_M := $(shell uname -m)

ifeq ($(UNAME_S),Linux)
    PLATFORM := linux
endif
ifeq ($(UNAME_S),Darwin) 
    PLATFORM := macos
endif
ifeq ($(UNAME_S),Windows_NT)
    PLATFORM := windows
endif

## help: Show this help message
help:
	@echo 'Usage:'
	@sed -n 's/^##//p' ${MAKEFILE_LIST} | column -t -s ':' | sed -e 's/^/ /'

## build: Build all components in release mode
build:
	@echo "Building all components for $(PLATFORM)..."
	$(CARGO) build --release --workspace

## build-debug: Build all components in debug mode  
build-debug:
	@echo "Building all components in debug mode..."
	$(CARGO) build --workspace

## test: Run all tests
test:
	@echo "Running tests..."
	$(CARGO) test --workspace

## test-integration: Run integration tests
test-integration:
	@echo "Running integration tests..."
	$(CARGO) test --workspace --test integration

## clean: Clean build artifacts
clean:
	@echo "Cleaning build artifacts..."
	$(CARGO) clean
	rm -rf $(TARGET_DIR)

## format: Format code using rustfmt
format:
	@echo "Formatting code..."
	$(CARGO) fmt --all

## lint: Run clippy linter
lint:
	@echo "Running clippy..."
	$(CARGO) clippy --all-targets --all-features -- -D warnings

## doc: Generate documentation
doc:
	@echo "Generating documentation..."
	$(CARGO) doc --no-deps --open

## dev-setup: Install development dependencies
dev-setup:
	@echo "Setting up development environment..."
	rustup update
	rustup component add rustfmt clippy
	@echo "Development setup complete!"

## install: Install binaries to system
install: build
	@echo "Installing binaries..."
	$(CARGO) install --path session-server --force
	$(CARGO) install --path hid-client --force  
	$(CARGO) install --path commander --force

## run-server: Run session server in development mode
run-server:
	@echo "Starting session server..."
	cd session-server && RUST_LOG=$(RUST_LOG) $(CARGO) run

## run-hid-client: Run HID client in development mode
run-hid-client:
	@echo "Starting HID client..."
	cd hid-client && RUST_LOG=$(RUST_LOG) $(CARGO) run

## run-commander: Run commander in development mode  
run-commander:
	@echo "Starting commander..."
	cd commander && RUST_LOG=$(RUST_LOG) $(CARGO) run

## cross-compile: Build for multiple platforms (requires cross)
cross-compile:
	@echo "Cross-compiling for multiple platforms..."
	@command -v cross >/dev/null 2>&1 || { echo "Installing cross..."; cargo install cross; }
	cross build --target x86_64-pc-windows-gnu --release
	cross build --target x86_64-apple-darwin --release
	cross build --target aarch64-apple-darwin --release

## package: Create platform-specific packages
package: build
	@echo "Creating packages for $(PLATFORM)..."
	mkdir -p packages/$(PLATFORM)
	cp target/release/session-server packages/$(PLATFORM)/
	cp target/release/hid-client packages/$(PLATFORM)/
	cp target/release/commander packages/$(PLATFORM)/
	cp README.md packages/$(PLATFORM)/
	tar -czf packages/remote-hid-$(PLATFORM)-$(shell date +%Y%m%d).tar.gz -C packages $(PLATFORM)

## security-audit: Run security audit
security-audit:
	@echo "Running security audit..."
	@command -v cargo-audit >/dev/null 2>&1 || { echo "Installing cargo-audit..."; cargo install cargo-audit; }
	cargo audit

## coverage: Generate code coverage report
coverage:
	@echo "Generating code coverage..."
	@command -v cargo-tarpaulin >/dev/null 2>&1 || { echo "Installing cargo-tarpaulin..."; cargo install cargo-tarpaulin; }
	cargo tarpaulin --out Html --output-dir coverage

## bench: Run benchmarks
bench:
	@echo "Running benchmarks..."
	$(CARGO) bench --workspace