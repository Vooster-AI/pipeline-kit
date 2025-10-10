# Pull Request

## Summary

<!-- Provide a brief description of the changes in this PR -->

## Type of Change

<!-- Mark the relevant option with an "x" -->

- [ ] Bug fix (non-breaking change which fixes an issue)
- [ ] New feature (non-breaking change which adds functionality)
- [ ] Breaking change (fix or feature that would cause existing functionality to not work as expected)
- [ ] Documentation update
- [ ] Refactoring (no functional changes)
- [ ] Performance improvement
- [ ] Test coverage improvement

## Changes Made

<!-- List the specific changes made in this PR -->

-
-
-

## Test Plan

<!-- Describe how you tested these changes -->

- [ ] Added/updated unit tests
- [ ] Added/updated integration tests
- [ ] Manually tested locally
- [ ] CI passes

## Checklist

### Code Quality

- [ ] My code follows the project's coding conventions (see CLAUDE.md)
- [ ] I have performed a self-review of my own code
- [ ] I have commented my code, particularly in hard-to-understand areas
- [ ] All crate names use `pk-` prefix (if adding new crates)
- [ ] Appropriate error handling (`thiserror` for libs, `anyhow` for bins)
- [ ] No dead code (or properly documented with `#[allow(dead_code)]` and rationale)

### Testing

- [ ] I have added tests that prove my fix is effective or that my feature works
- [ ] New and existing unit tests pass locally: `cargo nextest run --all-features` (or `cargo test`)
- [ ] TUI changes have snapshot tests (if applicable)
- [ ] E2E tests updated if pipeline behavior changed

### Documentation

- [ ] I have made corresponding changes to the documentation
- [ ] Public APIs documented with rustdoc comments
- [ ] README updated if user-facing features changed

### Verification (Local Checks Before Pushing)

- [ ] Code formatted: `cd pipeline-kit-rs && cargo fmt --all --check`
- [ ] No clippy warnings: `cd pipeline-kit-rs && cargo clippy --all-targets --all-features -- -D warnings`
- [ ] Build succeeds: `cd pipeline-kit-rs && cargo build --all-targets --all-features`
- [ ] TypeScript types regenerated (if protocol changes): `cd pipeline-kit-rs && cargo test --package pk-protocol --lib -- --nocapture`

### CI/CD (Will be verified by GitHub Actions)

- [ ] All CI workflows pass (rust-ci, coverage, security)
- [ ] Code coverage maintained or improved
- [ ] No security vulnerabilities detected
- [ ] Build succeeds on all target platforms

### Dependencies

- [ ] Any dependent changes have been merged and published
- [ ] No new dependencies with incompatible licenses (GPL-3.0, AGPL-3.0)

## Related Issues

<!-- Link any related issues here using #issue_number -->

Closes #

## Additional Notes

<!-- Add any additional notes or context about the PR here -->
