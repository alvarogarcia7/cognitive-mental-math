# Git Hooks

This directory contains legacy git hooks for the project. The project now uses the `pre-commit` framework for managing hooks.

## Available Hooks (Legacy)

### pre-commit
- Automatically runs `cargo test --quiet` before each commit
- Prevents commits if any tests fail
- Ensures code quality by requiring all tests to pass

### pre-push
- Runs pre-push checks via Makefile with parallel execution

## Installation (New Method - Using pre-commit Framework)

To install hooks using the `pre-commit` framework, run from the repository root:

```bash
make install-pre-commit
```

This will:
1. Install the `pre-commit` Python package using `uv`
2. Install pre-commit and pre-push hooks defined in `.pre-commit-config.yaml`

## Installation (Legacy Method)

To install these hooks using the legacy installation script, run the installation script from the repository root:

```bash
./install-hooks.sh
```

This will copy all hooks from the `hooks/` directory to `.git/hooks/` and make them executable.

## Manual Installation (Legacy)

If you prefer to install manually:

```bash
cp hooks/* .git/hooks/
chmod +x .git/hooks/*
```

## Pre-commit Framework Configuration

The project uses the `pre-commit` framework which is configured in `.pre-commit-config.yaml`. This framework provides:

- **Dependency Management**: Uses `uv` for Python dependency management
- **Hook Stages**: Separate hooks for `commit` and `push` stages
- **Commit Stage Hooks**:
  - `make fmt`: Format code using `cargo fmt`
  - `make test`: Run tests
- **Push Stage Hooks**:
  - `make fmt-check`: Check code formatting
  - `make test`: Run tests

## Notes

- Git hooks are local to each repository clone and are not automatically installed when cloning.
- Each developer needs to run `make install-pre-commit` after cloning the repository.
- The `pre-commit` framework is recommended for new projects and provides better maintainability.
- Legacy hooks are still available for backward compatibility.
