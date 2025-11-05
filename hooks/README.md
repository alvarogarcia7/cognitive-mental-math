# Git Hooks

This directory contains git hooks for the project.

## Available Hooks

### pre-commit
- Automatically runs `cargo test --quiet` before each commit
- Prevents commits if any tests fail
- Ensures code quality by requiring all tests to pass

### pre-push
- Currently does nothing (placeholder for future use)

## Installation

To install these hooks, run the installation script from the repository root:

```bash
./install-hooks.sh
```

This will copy all hooks from the `hooks/` directory to `.git/hooks/` and make them executable.

## Manual Installation

If you prefer to install manually:

```bash
cp hooks/* .git/hooks/
chmod +x .git/hooks/*
```

## Note

Git hooks are local to each repository clone and are not automatically installed when cloning. Each developer needs to run the installation script after cloning the repository.
