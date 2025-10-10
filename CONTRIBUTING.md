# Contributing to Pipeline Kit

Thank you for your interest in contributing to Pipeline Kit! This document provides guidelines and instructions for contributors.

## Table of Contents

- [Project Architecture](#project-architecture)
- [Development Setup](#development-setup)
- [Project Structure](#project-structure)
- [Building and Running](#building-and-running)
- [Testing Strategy](#testing-strategy)
- [Coding Conventions](#coding-conventions)
- [Adding New Features](#adding-new-features)
- [Submitting Changes](#submitting-changes)

## Project Architecture

Pipeline Kit is a monorepo combining a high-performance **Rust core engine** with a lightweight **TypeScript wrapper** for npm distribution. The architecture follows strict separation of concerns:

- **Rust (`pipeline-kit-rs`)**: All business logic, state management, and core functionality
- **TypeScript (`pipeline-kit-cli`)**: Platform detection and binary execution wrapper only

### Communication Flow

```
User → TypeScript Wrapper → Rust Binary → TUI ↔ Core
                                          ↓
                                    ┌─────────────┐
                                    │   Core      │
                                    ├─────────────┤
                                    │ StateManager│
                                    │ PipelineEngine
                                    │ AgentManager│
                                    └─────────────┘
```

All communication between TUI and Core uses the `Op` (operations) and `Event` (state updates) protocol defined in `pk-protocol`.

## Development Setup

### Prerequisites

- **Rust**: 1.70+ with edition 2021
  ```bash
  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
  ```

- **Node.js**: 16+ and npm
  ```bash
  # macOS
  brew install node

  # Or use nvm
  curl -o- https://raw.githubusercontent.com/nvm-sh/nvm/v0.39.0/install.sh | bash
  nvm install 16
  ```

- **Just** (task runner, optional but recommended):
  ```bash
  cargo install just
  ```

### Clone and Install

```bash
# Clone the repository
git clone https://github.com/Vooster-AI/pipeline-kit.git
cd pipeline-kit

# Install Rust dependencies
cd pipeline-kit-rs
cargo build

# Install TypeScript dependencies
cd ../pipeline-kit-cli
npm install
```

## Project Structure

```
pipeline-kit/
├── pipeline-kit-rs/          # Rust Cargo Workspace
│   ├── crates/
│   │   ├── cli/              # Binary entry point (main.rs)
│   │   │   └── src/
│   │   │       └── main.rs   # clap-based CLI argument parser
│   │   │
│   │   ├── core/             # Core business logic
│   │   │   └── src/
│   │   │       ├── agents/   # Agent trait and adapters
│   │   │       │   ├── base.rs          # Agent trait definition
│   │   │       │   ├── manager.rs       # AgentManager (factory)
│   │   │       │   └── adapters/        # Claude, Gemini, etc.
│   │   │       │       ├── claude_adapter.rs
│   │   │       │       ├── gemini_adapter.rs
│   │   │       │       └── cursor_adapter.rs
│   │   │       ├── config/   # Configuration loading
│   │   │       │   └── loader.rs        # YAML/TOML/MD parsers
│   │   │       ├── engine/   # Pipeline execution engine
│   │   │       │   └── pipeline.rs      # PipelineEngine
│   │   │       └── state/    # State management
│   │   │           └── manager.rs       # StateManager
│   │   │
│   │   ├── protocol/         # Shared data structures (IPC)
│   │   │   └── src/
│   │   │       ├── op.rs     # Operations (TUI → Core)
│   │   │       ├── event.rs  # Events (Core → TUI)
│   │   │       └── models.rs # Shared types
│   │   │
│   │   ├── protocol-ts/      # TypeScript type generation
│   │   │   └── src/
│   │   │       └── lib.rs    # ts-rs bindings
│   │   │
│   │   └── tui/              # Terminal User Interface
│   │       └── src/
│   │           ├── app.rs    # Main TUI application loop
│   │           ├── widgets/  # UI components
│   │           │   ├── dashboard.rs     # Process list view
│   │           │   ├── detail.rs        # Process detail view
│   │           │   └── composer.rs      # Command input widget
│   │           └── event_handler.rs     # Keyboard/terminal events
│   │
│   ├── Cargo.toml            # Workspace manifest
│   └── justfile              # Task runner commands
│
└── pipeline-kit-cli/         # npm Wrapper Package
    ├── bin/
    │   └── pipeline-kit.js   # Platform-aware launcher
    ├── lib/
    │   └── platform.js       # Platform detection utilities
    ├── scripts/
    │   └── install_native_deps.sh  # Binary download script
    ├── test/                 # TypeScript tests
    └── package.json
```

## Building and Running

### Rust Development

```bash
cd pipeline-kit-rs

# Build the project
cargo build --release

# Run the TUI directly
cargo run --release --bin pipeline

# Run with specific command
cargo run --release --bin pipeline -- start code-review

# Run tests
cargo test --workspace

# Run tests for specific crate
cargo test --package pk-core
cargo test --package pk-tui

# Format code
cargo fmt --all

# Check for lints
cargo clippy --all-targets --all-features
```

### Using Just (Task Runner)

If you have `just` installed:

```bash
cd pipeline-kit-rs

just build         # Build release binary
just test          # Run all tests
just format        # Format all code
just lint          # Run clippy
```

### TypeScript Development

```bash
cd pipeline-kit-cli

# Install dependencies
npm install

# Run tests
npm test

# Run tests in watch mode
npm run test:watch

# Run tests with UI
npm run test:ui

# Test the launcher (requires built Rust binary)
npx pipeline-kit
```

## Testing Strategy

Pipeline Kit follows **Test-Driven Development (TDD)** with the RED/GREEN/REFACTOR cycle.

### Test Requirements

1. **Every ticket MUST start with a failing test** that defines acceptance criteria
2. **Every public function MUST have unit tests**
3. **TUI components MUST have snapshot tests** using `ratatui::backend::TestBackend`

### Test Types

#### Unit Tests (Rust)

```rust
// In pk-core/src/config/loader.rs
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_load_config_acceptance() {
        // Arrange
        let temp_dir = setup_test_config();

        // Act
        let config = load_config(temp_dir.path()).await.unwrap();

        // Assert
        assert_eq!(config.pipelines.len(), 1);
        assert_eq!(config.agents.len(), 1);
    }
}
```

#### TUI Snapshot Tests

```rust
// In pk-tui/src/widgets/dashboard.rs
#[cfg(test)]
mod tests {
    use ratatui::backend::TestBackend;

    #[test]
    fn test_dashboard_renders_processes() {
        let mut backend = TestBackend::new(80, 24);
        let mut terminal = Terminal::new(backend).unwrap();

        // Render widget and capture output
        terminal.draw(|f| {
            let widget = Dashboard::new(test_processes);
            f.render_widget(widget, f.size());
        }).unwrap();

        // Assert on rendered text
        let buffer = terminal.backend().buffer();
        assert!(buffer.content.contains("Process 1"));
    }
}
```

#### Integration Tests (TypeScript)

```javascript
// In pipeline-kit-cli/test/platform.test.js
import { describe, it, expect } from 'vitest';
import { getPlatformName } from '../lib/platform.js';

describe('Platform Detection', () => {
  it('should map darwin-arm64 to macos-arm64', () => {
    const platform = getPlatformName('darwin', 'arm64');
    expect(platform).toBe('macos-arm64');
  });
});
```

### Running Tests

```bash
# All Rust tests
cargo test --workspace

# Specific crate tests
cargo test --package pk-core

# All TypeScript tests
cd pipeline-kit-cli
npm test

# Test coverage
npm run test:coverage
```

## Coding Conventions

### Rust

1. **Crate Naming**: All crates MUST be prefixed with `pk-`
   - ✅ `pk-core`, `pk-protocol`, `pk-tui`
   - ❌ `core`, `protocol`, `tui`

2. **Error Handling**:
   - Use `thiserror` for library crates (`pk-core`, `pk-protocol`)
   - Use `anyhow` for binary crates (`pk-cli`, `pk-tui`)

3. **Async Runtime**: MUST use `tokio` exclusively

4. **Shared Types**: Any type shared between crates MUST:
   - Be defined in `pk-protocol`
   - Derive `Serialize`, `Deserialize`, `Debug`, `Clone`
   - Derive `ts_rs::TS` if exposed to TypeScript

5. **Documentation**: All public APIs MUST have doc comments

```rust
/// Loads configuration from the .pipeline-kit directory.
///
/// # Arguments
/// * `root` - The root directory to search for .pipeline-kit/
///
/// # Returns
/// * `Config` - Parsed configuration or error
pub async fn load_config(root: &Path) -> Result<Config> {
    // ...
}
```

### TypeScript

1. **No Business Logic**: TypeScript wrapper should ONLY handle:
   - Platform detection
   - Binary path resolution
   - Process spawning

2. **ES Modules**: Use `type: "module"` in package.json

3. **Testing**: All platform detection logic MUST be tested

### Git Commits

- Write commits in English
- Use present tense ("Add feature" not "Added feature")
- Follow conventional commits format:
  ```
  feat: Add Gemini agent adapter
  fix: Correct binary path resolution on Windows
  test: Add snapshot tests for dashboard widget
  docs: Update CONTRIBUTING guide
  ```

## Adding New Features

### Adding a New Agent Adapter

1. **Create adapter file**: `pk-core/src/agents/adapters/my_agent.rs`

2. **Implement Agent trait**:
```rust
use crate::agents::base::{Agent, ExecutionContext, AgentEvent, AgentError};
use async_trait::async_trait;

pub struct MyAgentAdapter {
    api_key: String,
}

#[async_trait]
impl Agent for MyAgentAdapter {
    async fn check_availability(&self) -> bool {
        // Check if API key is valid
        true
    }

    async fn execute(
        &self,
        context: &ExecutionContext,
    ) -> Result<Pin<Box<dyn Stream<Item = Result<AgentEvent, AgentError>> + Send>>, AgentError> {
        // Implement agent execution
    }
}
```

3. **Register in AgentManager**: Update `pk-core/src/agents/manager.rs`

4. **Add tests**: Write unit tests for your adapter

5. **Update documentation**: Add agent to README.md

### Adding a New TUI Widget

1. **Create widget file**: `pk-tui/src/widgets/my_widget.rs`

2. **Implement Widget trait**:
```rust
use ratatui::{
    widgets::{Widget, Block, Borders},
    buffer::Buffer,
    layout::Rect,
};

pub struct MyWidget {
    // Widget state
}

impl Widget for MyWidget {
    fn render(self, area: Rect, buf: &mut Buffer) {
        // Rendering logic
    }
}
```

3. **Add snapshot tests**: Test rendering with `TestBackend`

4. **Integrate into app**: Update `pk-tui/src/app.rs`

### Adding a New Slash Command

1. **Define operation in protocol**: `pk-protocol/src/op.rs`
```rust
pub enum Op {
    // ... existing ops
    MyNewCommand { arg: String },
}
```

2. **Handle in Core**: Update `pk-core/src/state/manager.rs`

3. **Parse in TUI**: Update slash command parser in `pk-tui/src/widgets/composer.rs`

4. **Add tests**: Test the entire flow

## Submitting Changes

### Before Submitting

1. **Run all tests**:
   ```bash
   # Rust tests
   cd pipeline-kit-rs
   cargo test --workspace

   # TypeScript tests
   cd ../pipeline-kit-cli
   npm test
   ```

2. **Format code**:
   ```bash
   cargo fmt --all
   ```

3. **Check lints**:
   ```bash
   cargo clippy --all-targets --all-features
   ```

### Pull Request Process

1. **Create a feature branch**:
   ```bash
   git checkout -b feature/my-new-feature
   ```

2. **Make your changes** following TDD

3. **Commit your changes**:
   ```bash
   git add .
   git commit -m "feat: Add my new feature"
   ```

4. **Push to GitHub**:
   ```bash
   git push origin feature/my-new-feature
   ```

5. **Create Pull Request** with:
   - Clear description of changes
   - Reference to any related issues
   - Test coverage summary
   - Screenshots (for UI changes)

### PR Review Criteria

- All tests pass
- Code follows conventions
- Documentation updated
- No unnecessary dependencies added
- TDD approach followed

## Questions?

- Open an issue: https://github.com/Vooster-AI/pipeline-kit/issues
- Check existing documentation in `docs/`
- Review `CLAUDE.md` for detailed architecture notes

Thank you for contributing to Pipeline Kit!
