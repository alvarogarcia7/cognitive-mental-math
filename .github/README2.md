# GitHub Actions Workflows

This directory contains GitHub Actions workflows for continuous integration and continuous deployment.

## Available Workflows

- `ci.yml`: builds, tests, and checks formatting/clippy on pushes and pull requests to `master`/`main`.
- `release.yml`: when a tag is pushed on any branch, runs the normal CI pipeline on the tagged commit, then builds the binaries (`memory_practice`, `performance_stats`, `sm2_scheduler`) for `linux-amd64` and `mac-arm64`, smoke-tests them, and publishes a release.

### Binary releases

1. Tag the commit and push the tag: `git tag v1.2.3 && git push origin v1.2.3`. Release candidates use a hyphen, e.g. `v1.2.3-rc1`.
2. CI must pass on the tagged commit; otherwise no binaries are built.
3. The workflow creates a GitHub Release named after the tag with one archive per platform (`memory-practice-<target>.tar.gz`) and a `SHA256SUMS.txt`. Hyphenated tags are marked as pre-releases.
4. If the tag is a non-prerelease on `master`, the `latest` release is replaced to point at that commit with the same archives. Stable download URL:
   `https://github.com/<owner>/<repo>/releases/download/latest/memory-practice-<target>.tar.gz`

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

