# Pipeline Kit

AI agent pipeline orchestration CLI built with Rust and interactive TUI.

## 🚀 Quick Start

### Option 1: Direct Rust Binary Execution

```bash
# Build the project
cd pipeline-kit-rs
cargo build --release

# Run the TUI application
cargo run --release --bin pipeline
```

### Option 2: Via npm (Recommended for distribution)

```bash
# Install dependencies and build
cd pipeline-kit-cli
npm install

# Run the CLI
npx pipeline-kit
# or if installed globally:
npm install -g .
pipeline-kit
```

## 📁 Project Structure

```
pipeline-kit/
├── pipeline-kit-rs/          # Rust workspace
│   ├── crates/
│   │   ├── cli/              # Binary entry point
│   │   ├── core/             # Business logic (PipelineEngine, AgentManager, StateManager)
│   │   ├── protocol/         # Shared data models (Process, Event, Op, etc.)
│   │   ├── protocol-ts/      # TypeScript type generation
│   │   └── tui/              # Interactive terminal UI
│   └── Cargo.toml
└── pipeline-kit-cli/         # npm wrapper package
    ├── bin/pipeline-kit.js   # Node.js launcher
    └── package.json
```

## 🎯 Features

- **Pipeline Orchestration**: Execute AI agents sequentially with state management
- **Interactive TUI**: Real-time process monitoring with dashboard and detail views
- **Slash Commands**: `/start`, `/pause`, `/resume`, `/kill`, `/list` with autocomplete
- **Event-Driven Architecture**: Async communication between core engine and UI
- **Cross-Platform**: Supports macOS, Linux, Windows (x64 and ARM64)

## ⚙️ Configuration

Create a `.pipeline-kit/` directory in your working directory with the following structure:

```
.pipeline-kit/
├── config.toml              # Global configuration
├── agents/                  # Agent definitions
│   └── example-agent.md     # Agent with YAML frontmatter + system prompt
└── pipelines/               # Pipeline definitions
    └── example.yaml         # Pipeline YAML file
```

### Example: `config.toml`

```toml
git = true
```

### Example: `agents/developer.md`

```markdown
---
name: developer
description: A helpful coding assistant
model: claude-sonnet-4.5
color: blue
---

You are a helpful coding assistant. Help the user with their programming tasks.
```

### Example: `pipelines/code-review.yaml`

```yaml
name: code-review
master:
  model: claude-sonnet-4.5
  system-prompt: |
    You are a master orchestrator for code review pipelines.
  process:
    - developer
    - HUMAN_REVIEW
sub-agents:
  - developer
```

## 🎮 TUI Controls

- **↑/↓**: Navigate between processes
- **j/k**: Scroll detail view up/down
- **PageUp/PageDown**: Fast scroll in detail view
- **Tab**: Autocomplete slash commands
- **Enter**: Execute command
- **Esc**: Clear input
- **q** or **Ctrl+C**: Quit

## 🧪 Slash Commands

- `/start <pipeline>` - Start a new pipeline
- `/pause <process-id>` - Pause a running process
- `/resume <process-id>` - Resume a paused process
- `/kill <process-id>` - Kill a process
- `/list` - List all processes

## 🧩 Architecture

### Core Components

1. **pk-protocol**: Shared data structures and IPC definitions
2. **pk-core**:
   - `ConfigLoader`: Loads YAML/TOML/Markdown configurations
   - `AgentManager`: Manages agent adapters (currently MockAgent)
   - `PipelineEngine`: Executes pipeline steps sequentially
   - `StateManager`: Manages all active processes
3. **pk-tui**: ratatui-based terminal UI with widgets (Dashboard, DetailView, CommandComposer)
4. **pk-cli**: Binary entry point that wires everything together

### Communication Flow

```
TUI (pk-tui)  <--[Op/Event channels]-->  Core (pk-core)
     │                                         │
     │                                         ├─> StateManager
     │                                         ├─> PipelineEngine
     │                                         └─> AgentManager
     └─> Widgets (Dashboard, DetailView, CommandComposer)
```

## 🛠️ Development

### Prerequisites

- Rust 1.70+ (with `edition = "2021"`)
- Node.js 16+ (for npm wrapper)
- pnpm (for monorepo management)

### Build from Source

```bash
# Build Rust workspace
cd pipeline-kit-rs
cargo build --release

# Run tests
cargo test --workspace

# Install npm wrapper
cd ../pipeline-kit-cli
npm install
```

### Running Tests

```bash
# All tests
cd pipeline-kit-rs
cargo test --workspace

# Specific crate tests
cargo test --package pk-core
cargo test --package pk-tui
cargo test --package pk-protocol
```

## 📝 Testing Strategy

All components follow TDD (Test-Driven Development):
- **Unit tests**: Every public function has corresponding tests
- **Integration tests**: End-to-end pipeline execution tests
- **TUI snapshot tests**: Using `ratatui::backend::TestBackend`

Total test coverage: 100+ tests across all crates.

## 📦 Distribution

The npm package automatically detects your platform and downloads the appropriate binary:

- `darwin-x64` (Intel Mac)
- `darwin-arm64` (Apple Silicon)
- `linux-x64` (x86_64 Linux)
- `linux-arm64` (ARM Linux)
- `win32-x64` (Windows x64)

## 🤝 Contributing

This project was built following strict coding conventions:
- All Rust crates prefixed with `pk-`
- Error handling: `thiserror` for libraries, `anyhow` for binaries
- Async runtime: `tokio` exclusively
- TDD methodology: RED/GREEN/REFACTOR for all features

## 📄 License

[Add your license here]

## 🙏 Credits

Built with reference to the excellent [codex-cli](https://github.com/openai/codex) architecture.
