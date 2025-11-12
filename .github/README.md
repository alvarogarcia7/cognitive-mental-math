# GitHub Actions Workflows

This directory contains GitHub Actions workflows for continuous integration and continuous deployment.

## Available Workflows

```markdown
![CI](https://github.com/USERNAME/REPO/actions/workflows/ci.yml/badge.svg)
```

## Local Testing

Before pushing, you can run the same checks locally:

```bash
# Build
cargo build --verbose

# Run tests
cargo test --verbose

# Check formatting
cargo fmt --check

# Run clippy
cargo clippy -- -D warnings
```

