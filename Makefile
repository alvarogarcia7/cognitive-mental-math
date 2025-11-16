# Default target
help: ## Show this help message
	@awk 'BEGIN {FS = ":.*##"; printf "Available targets:\n"} /^[a-zA-Z_-]+:.*##/ { printf "  make %-18s - %s\n", $$1, $$2 }' $(MAKEFILE_LIST)
.PHONY: help

build: ## Build the project
	@echo "Building project..."
	@cargo build --verbose
	@cargo build --tests --verbose
.PHONY: build

test: build ## Run all tests
	@echo "Running tests..."
	@cargo test --quiet
	$(MAKE) test-performance-stats
.PHONY: test

run: ## Run the application
	@echo "Running application..."
	RUST_LOG=info cargo run --bin memory_practice
.PHONY: run

run-dev-memory: build ## Run the application in test mode (in-memory database)
	RUST_LOG=debug cargo run --bin memory_practice -- --test --database :mem:
.PHONY: run-dev-memory

run-dev: build ## Run the application in test mode
	RUST_LOG=debug cargo run --bin memory_practice -- --test --db-path custom.db
.PHONY: run-dev

run-with-date: build ## Run the application with override date (use: make run-with-date DATE=2025-11-18)
	@if [ -z "$(DATE)" ]; then echo "Error: DATE parameter required (format: YYYY-MM-DD). Usage: make run-with-date DATE=2025-11-18"; exit 1; fi
	RUST_LOG=info cargo run --bin memory_practice -- --override-date $(DATE)
.PHONY: run-with-date

run-dev-with-date: build ## Run in test mode with override date (use: make run-dev-with-date DATE=2025-11-18)
	@if [ -z "$(DATE)" ]; then echo "Error: DATE parameter required (format: YYYY-MM-DD). Usage: make run-dev-with-date DATE=2025-11-18"; exit 1; fi
	RUST_LOG=debug cargo run --bin memory_practice -- --test --db-path custom.db --override-date $(DATE)
.PHONY: run-dev-with-date

clean: ## Clean build artifacts
	@echo "Cleaning build artifacts..."
	@cargo clean
.PHONY: clean

build_and_test: build test ## Build and run tests
	@echo "✅ Build and tests passed!"
.PHONY: build_and_test

pre-commit: ## Run pre-commit checks
	git hook run pre-commit
	@echo "✅ Pre-commit checks passed!"
.PHONY: pre-commit

pre-push: ## Run pre-push checks (use with -j 2 for parallel execution)
	git hook run pre-push
	@echo "✅ Pre-push checks passed!"
.PHONY: pre-push

fmt: ## Format code
	@echo "Formatting code..."
	@cargo fmt
.PHONY: fmt

fmt-check: ## Check code formatting
	cargo fmt --check
.PHONY: fmt-check

clippy: ## Run clippy linter
	cargo clippy -- -D warnings
.PHONY: clippy


clippy-fix: ## Run clippy fix
	cargo clippy --fix --allow-dirty -- -D warnings



.PHONY: clippy-fix

check: fmt-check clippy test ## Run all checks
	@echo "✅ All checks passed!"
.PHONY: check

test-performance-stats:
	cargo run --bin performance_stats -- data/sample.db
.PHONY: test-performance-stats

install-pre-commit: ## Install pre-commit hooks using pre-commit framework
	@echo "Installing pre-commit framework and hooks..."
	@uv sync
	@uv run pre-commit install --install-hooks --hook-type pre-commit --hook-type pre-push
	@echo "✅ Pre-commit hooks installed successfully!"
.PHONY: install-pre-commit

init: install-pre-commit ## Setup development environment with pre-commit
.PHONY: init

demo-scheduler: build ## Run the demo scheduler
	# repetitions, interval, ease factor
	cargo run --bin sm2_scheduler 3 10 2.5
	cargo run --bin sm2_scheduler 0 2 2.5
	cargo run --bin sm2_scheduler 0 3 2.5
	cargo run --bin sm2_scheduler 1 3 2.5
	cargo run --bin sm2_scheduler 1 1 2.5
	cargo run --bin sm2_scheduler 1 0 2.5
	cargo run --bin sm2_scheduler 2 0 2.5
	cargo run --bin sm2_scheduler 2 1 2.5
	cargo run --bin sm2_scheduler 2 3 2.5
.PHONY: demo-scheduler

performance-stats: ## Run performance statistics
	DB=custom.db $(MAKE) performance-stats-with
.PHONY: performance-stats

performance-stats-with: build ## Run performance statistics with in-memory database
	@if [ -z "$(DB)" ]; then echo "Error: DB parameter required (format: <path>). Usage: make performance-stats-with DB=custom.db"; exit 1; fi
	@cargo run --bin performance_stats -- $(DB)
.PHONY: performance-stats-with



