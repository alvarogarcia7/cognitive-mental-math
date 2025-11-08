# Default target
help: ## Show this help message
	@awk 'BEGIN {FS = ":.*##"; printf "Available targets:\n"} /^[a-zA-Z_-]+:.*##/ { printf "  make %-18s - %s\n", $$1, $$2 }' $(MAKEFILE_LIST)
.PHONY: help

build: ## Build the project
	@echo "Building project..."
	@cargo build --verbose
.PHONY: build

test: build ## Run all tests
	@echo "Running tests..."
	@cargo test --quiet
.PHONY: test

run: ## Run the application
	@echo "Running application..."
	@cargo run
.PHONY: run

run-dev-memory: build ## Run the application in test mode (in-memory database)
	cargo run -- --test --database :mem:
.PHONY: run-dev-memory

run-dev: build ## Run the application in test mode
	cargo run -- --test --db-path custom.db
.PHONY: run-dev

clean: ## Clean build artifacts
	@echo "Cleaning build artifacts..."
	@cargo clean
.PHONY: clean

build_and_test: build test ## Build and run tests
	@echo "✅ Build and tests passed!"
.PHONY: build_and_test

pre-commit: fmt-check build_and_test  clippy ## Run pre-commit checks
	@echo "✅ Pre-commit checks passed!"
.PHONY: pre-commit

pre-push: fmt-check build_and_test clippy ## Run pre-push checks (use with -j 2 for parallel execution)
	@echo "✅ Pre-push checks passed!"
.PHONY: pre-push

fmt: ## Format code
	@echo "Formatting code..."
	@cargo fmt
.PHONY: fmt

fmt-check: ## Check code formatting
	@echo "Checking code formatting..."
	@cargo fmt --check
.PHONY: fmt-check

clippy: ## Run clippy linter
	@echo "Running clippy..."
	@cargo clippy -- -D warnings
.PHONY: clippy


clippy-fix: ## Run clippy fix
	@echo "Running clippy fix..."
	@cargo clippy --fix
.PHONY: clippy-fix

check: fmt-check clippy test ## Run all checks
	@echo "✅ All checks passed!"
.PHONY: check

install-hooks: ## Install git hooks
	@echo "Installing git hooks..."
	@./install-hooks.sh
.PHONY: install-hooks
