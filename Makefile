# HiPDF - High-level PDF manipulation library
# Makefile for common development tasks

.PHONY: all build test doc clean example install check fmt clippy

# Default target
all: check

# Build the library
build:
	@cargo build

# Build in release mode
build-release:
	@cargo build --release

# Run tests
test:
	@cargo test --lib --bins --tests --examples

# Run tests with output
test-verbose:
	@cargo test --lib --bins --tests --examples -- --nocapture

# Test only the library
test-lib:
	@cargo test --lib

# Test only integration tests
test-integration:
	@cargo test --test "*"

# Generate documentation
doc:
	@cargo doc --open

# Clean build artifacts
clean:
	@cargo clean
	@rm -rf tests/outputs/*.pdf
	@echo "🏗️  Cleaned build artifacts and test outputs"

# Run example
example:
	@cargo run --bin hipdf-example

# Install the library locally
install:
	@cargo install --path .

# Check for compilation errors
check:
	@cargo check --lib --bins --tests --examples

# Format code
fmt:
	@cargo fmt

# Run clippy linter
clippy:
	@cargo clippy -- -D warnings

# Run clippy with fixes
clippy-fix:
	@cargo clippy --fix -- -D warnings

# Full development check
dev-check: fmt clippy check test

# Update dependencies
update:
	@cargo update

# Performance test
perf-test:
	@cargo test --release test_ocg_performance

# Help
help:
	@echo "HiPDF Development Commands:"
	@echo ""
	@echo "Building:"
	@echo "  build          - Build the library in debug mode"
	@echo "  build-release  - Build the library in release mode"
	@echo "  check          - Check for compilation errors"
	@echo ""
	@echo "Testing:"
	@echo "  test           - Run all tests"
	@echo "  test-verbose   - Run all tests with verbose output"
	@echo "  test-lib       - Run only library tests"
	@echo "  test-integration - Run only integration tests"
	@echo ""
	@echo "Code Quality:"
	@echo "  fmt            - Format code with rustfmt"
	@echo "  clippy         - Run clippy linter"
	@echo "  clippy-fix     - Run clippy with auto-fixes"
	@echo "  dev-check      - Run fmt, clippy, check, and test"
	@echo ""
	@echo "Documentation:"
	@echo "  doc            - Generate and open documentation"
	@echo ""
	@echo "Examples & Installation:"
	@echo "  example        - Run the OCG example"
	@echo "  install        - Install the library locally"
	@echo ""
	@echo "Maintenance:"
	@echo "  clean          - Clean build artifacts"
	@echo "  update         - Update dependencies"
	@echo "  perf-test      - Run performance tests"
