# Default target
help:
	@echo "Available targets:"
	@echo "  make build       - Build the project"
	@echo "  make test        - Run all tests"
	@echo "  make run         - Run the application"
	@echo "  make clean       - Clean build artifacts"
	@echo "  make pre-commit  - Run pre-commit checks (build + test + fmt-check + clippy)"
	@echo "  make pre-push    - Run pre-push checks (same as pre-commit, use with -j 2 for parallel)"
	@echo "  make fmt         - Format code"
	@echo "  make clippy      - Run clippy linter"
	@echo "  make check       - Run all checks (fmt + clippy + test)"
.PHONY: help

# Build the project
build:
	@echo "Building project..."
	@cargo build --verbose
.PHONY: build

# Run all tests
test:
	@echo "Running tests..."
	@cargo test --quiet
.PHONY: test

# Run the application
run:
	@echo "Running application..."
	@cargo run
.PHONY: run

# Clean build artifacts
clean:
	@echo "Cleaning build artifacts..."
	@cargo clean
.PHONY: clean

# Pre-commit checks: compile and run tests
pre-commit: build test fmt-check clippy
	@echo "✅ Pre-commit checks passed!"
.PHONY: pre-commit

# Pre-push checks: run all checks in parallel
pre-push: build test fmt-check clippy
	@echo "✅ Pre-push checks passed!"
.PHONY: pre-push

# Format code
fmt:
	@echo "Formatting code..."
	@cargo fmt
.PHONY: fmt

# Check code formatting
fmt-check:
	@echo "Checking code formatting..."
	@cargo fmt --check
.PHONY: fmt-check

# Run clippy linter
clippy:
	@echo "Running clippy..."
	@cargo clippy -- -D warnings
.PHONY: clippy

# Run all checks
check: fmt-check clippy test
	@echo "✅ All checks passed!"
.PHONY: check

# Install git hooks
install-hooks:
	@echo "Installing git hooks..."
	@./install-hooks.sh
.PHONY: install-hooks
