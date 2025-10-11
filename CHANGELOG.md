# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.1.3] - 2025-10-11

### Fixed
- npm global install failed due to missing runtime deps. Moved `axios` and `tar` to production `dependencies` and lazily loaded them in `scripts/install.js` to avoid ESM import errors in development mode.

## [0.1.2] - 2025-10-11

### Added
- TUI now wired to core StateManager and protocol (3a505aa)

### Changed
- Improved agent CLI availability tests formatting (69ef122)

### Fixed
- Production install E2E job now uses local server with URL override (33c2f50)
- CLI wrapper now properly overrides download URL via environment variables (4e9a310)

## [0.1.1] - 2025-10-11

### Added
- `init` command to bootstrap .pipeline-kit projects (6fab618)
- Qwen adapter with ACP protocol support (334c65f)
- TypeScript type-check support for CLI (5b5c185)
- Comprehensive E2E pipeline execution tests (835388d)
- `cargo nextest` support to verification guide (783474f)
- Automated release workflow with npm publishing (e7421a1)
- Pre-release check script with code formatting validation (6c26e24, 21ff983)
- MIT License (e321277)
- Scheduled agent-integration workflow (9ac711c)
- CLI `--json-events` flag to stream Events as JSONL (5d5663d)
- Allowlist for common non-ASCII symbols in asciicheck (712955b)

### Changed
- Delegated event handling to widgets (303e991 - Ticket 9.3)
- Replaced shell script with Node.js install script (62ce25f - Ticket 9.2)
- Introduced CliExecutor for agent adapters (eb0ec0d - Ticket 9.1)
- Deduplicated Rust checks and added documentation verification (9be804e)
- Added caching to cargo shear job for improved CI performance (ee2ef3b)

### Fixed
- README.md to match actual implementation (3bd0383)
- All CI checks and test timeouts (dbb0701)
- Cargo verification errors (fmt, shear, clippy) (7b6f8ce)
- Clippy warnings: moved result_large_err suppression to correct location (bda4484)
- Clippy warnings: suppressed result_large_err for ConfigError (c11425c)
- Cargo-audit configuration to ignore paste unmaintained warning (5c104bd)
- Typo in codebase (b5f81ef)
- pnpm version error (2b21d7b)

### Documentation
- Added comprehensive README for pipeline-kit-cli npm package (133a5ab)
- Added Phase 9 refactoring specifications and progress tracking (f939fd0)
- Integrated CLAUDE.md verification checklist into CI workflow (ef54493)
- Added #[allow(dead_code)] usage guidelines (feecdea)
- Added verification guide for agents (97c1532)

### Internal
- Removed outdated TODO comment in StateManager (0def68f)
- Reactivated disabled engine tests (b38e32a)
- Updated pnpm-lock file (aa86a59)
- Added codespell configuration (65ae850)
- Removed target directory from git (ded1d7e)
- Added GitHub CI settings (f451b71)

## [0.1.0] - 2025-10-11

### Added
- Initial monorepo setup with Cargo workspace structure (fc457a2)
- pk-protocol crate with core data models (b6fbeab)
- Config file loader for pk-core (ffc7ff4 - Ticket 2.1)
- Agent adapter pattern and manager (23bdf5b - Ticket 2.2)
- PipelineEngine and StateManager (22e18e1 - Ticket 3.1)
- TUI application shell and event loop (f744ed9 - Ticket 4.1)
- Process detail view widget with scrolling (e3d0ced)
- Command composer widget with slash command autocomplete (4cf2598)
- Dashboard widget with Table rendering (df7f22b - Ticket 4.2)
- TypeScript wrapper and npm packaging (8270bdf - Ticket 5.1)
- GitHub Release binary download support (a9bfd43)
- TDD tests for CLI wrapper (f41a82e)
- Complete Phase 2.1-B error handling tests (3a8937b)

### Fixed
- TUI entry point integration (e932a45 - Ticket 4.1-Fix)
- Dashboard widget integration (a5ae798 - Ticket 4.2-Fix)
- start_pipeline to return actual Process ID (e8a4daf - Ticket 3.1-Fix.1)
- resume_process_by_id logic (ff2f0bf - Ticket 3.1-Fix.2)
- kill_process task cancellation (cc0a51a - Ticket 3.1-Fix.3)
- Directory structure cleanup (5eb0ba8 - Ticket 5.1-Fix.3)

### Documentation
- Added git commit guidelines to coding conventions (a68cfe6)
- Initial spec and agent definitions (832e95e)
- Updated ticket progress tracking throughout development

[Unreleased]: https://github.com/Vooster-AI/pipeline-kit/compare/v0.1.3...HEAD
[0.1.3]: https://github.com/Vooster-AI/pipeline-kit/compare/v0.1.2...v0.1.3
[0.1.2]: https://github.com/Vooster-AI/pipeline-kit/compare/v0.1.1...v0.1.2
[0.1.1]: https://github.com/Vooster-AI/pipeline-kit/compare/v0.1.0...v0.1.1
[0.1.0]: https://github.com/Vooster-AI/pipeline-kit/releases/tag/v0.1.0
