.PHONY: help build test clean run pre-commit fmt clippy check

# Default target
help:
	@echo "Available targets:"
	@echo "  make build       - Build the project"
	@echo "  make test        - Run all tests"
	@echo "  make run         - Run the application"
	@echo "  make clean       - Clean build artifacts"
	@echo "  make pre-commit  - Run pre-commit checks (build + test)"
	@echo "  make fmt         - Format code"
	@echo "  make clippy      - Run clippy linter"
	@echo "  make check       - Run all checks (fmt + clippy + test)"

# Build the project
build:
	@echo "Building project..."
	@cargo build --verbose

# Run all tests
test:
	@echo "Running tests..."
	@cargo test --quiet

# Run the application
run:
	@echo "Running application..."
	@cargo run

# Clean build artifacts
clean:
	@echo "Cleaning build artifacts..."
	@cargo clean

# Pre-commit checks: compile and run tests
pre-commit: build test
	@echo "✅ Pre-commit checks passed!"

# Format code
fmt:
	@echo "Formatting code..."
	@cargo fmt

# Check code formatting
fmt-check:
	@echo "Checking code formatting..."
	@cargo fmt --check

# Run clippy linter
clippy:
	@echo "Running clippy..."
	@cargo clippy -- -D warnings

# Run all checks
check: fmt-check clippy test
	@echo "✅ All checks passed!"

# Install git hooks
install-hooks:
	@echo "Installing git hooks..."
	@./install-hooks.sh
