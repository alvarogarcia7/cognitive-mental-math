# GitHub Actions Workflows

This directory contains GitHub Actions workflows for continuous integration and continuous deployment.

## Available Workflows

### CI Workflow (`workflows/ci.yml`)

Runs on every push and pull request to the `master` or `main` branches.

**Steps:**
1. **Checkout code** - Gets the latest code from the repository
2. **Install Rust toolchain** - Sets up the stable Rust compiler
3. **Cache dependencies** - Caches cargo registry, git index, and target directory for faster builds
4. **Install system dependencies** - Installs required GUI libraries (GTK, X11, XCB)
5. **Build project** - Compiles the project with `cargo build --verbose`
6. **Run tests** - Executes all tests with `cargo test --verbose` (38 tests)
7. **Check formatting** - Verifies code formatting with `cargo fmt --check` (optional)
8. **Run clippy** - Performs linting with `cargo clippy` (optional)

**Test Coverage:**
- 21 unit tests
- 11 end-to-end tests
- 6 integration tests
- **Total: 38 tests**

## Viewing Workflow Results

After pushing code or creating a pull request, you can view the workflow results:

1. Go to the repository on GitHub
2. Click on the "Actions" tab
3. Select the workflow run you want to view
4. Expand each step to see detailed logs

## Workflow Badges

You can add a badge to your README.md to show the CI status:

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

## Troubleshooting

If the CI workflow fails:

1. Check the workflow logs in the Actions tab
2. Run the failing command locally to reproduce the issue
3. Fix the issue and push again
4. The workflow will automatically run on the new push
