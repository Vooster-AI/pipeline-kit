# Pipeline Kit

AI agent pipeline orchestration CLI with interactive TUI, built with Rust for performance and distributed via npm for convenience.

[![npm version](https://badge.fury.io/js/pipeline-kit.svg)](https://www.npmjs.com/package/pipeline-kit)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![CI](https://github.com/Vooster-AI/pipeline-kit/actions/workflows/rust-ci.yml/badge.svg)](https://github.com/Vooster-AI/pipeline-kit/actions/workflows/rust-ci.yml)
[![codecov](https://codecov.io/gh/Vooster-AI/pipeline-kit/branch/main/graph/badge.svg)](https://codecov.io/gh/Vooster-AI/pipeline-kit)
[![Security Audit](https://github.com/Vooster-AI/pipeline-kit/actions/workflows/security.yml/badge.svg)](https://github.com/Vooster-AI/pipeline-kit/actions/workflows/security.yml)

## Features

- **Multi-Agent Pipelines**: Orchestrate multiple AI agents (Claude, Gemini, Cursor) in sequential workflows
- **Interactive TUI**: Real-time process monitoring with dashboard and detail views built with ratatui
- **Slash Commands**: Execute commands with autocomplete (`/start`, `/pause`, `/resume`, `/kill`, `/list`)
- **Event-Driven**: Async communication between core engine and UI for responsive interactions
- **Cross-Platform**: Native binaries for macOS, Linux, Windows (x64 and ARM64)
- **Git Integration**: Optional git commit automation after pipeline completion
- **Human Review Points**: Pause pipelines for manual review before continuing

## Quick Start

### Installation

```bash
# Install globally via npm
npm install -g pipeline-kit

# Or use directly with npx
npx pipeline-kit
```

### First Run

1. **Create configuration directory**:
```bash
mkdir -p .pipeline-kit/{agents,pipelines}
```

2. **Configure an agent** (`.pipeline-kit/agents/developer.md`):
```markdown
---
name: developer
description: A helpful coding assistant
model: claude-sonnet-4.5
color: blue
---

You are a helpful coding assistant. Your goal is to help users with their programming tasks efficiently and accurately.
```

3. **Create a pipeline** (`.pipeline-kit/pipelines/code-review.yaml`):
```yaml
name: code-review
master:
  model: claude-sonnet-4.5
  system-prompt: |
    You are the master orchestrator for a code review pipeline.
    Coordinate between agents to ensure high-quality code.
  process:
    - developer
    - reviewer
    - HUMAN_REVIEW
sub-agents:
  - developer
  - reviewer
```

4. **Set up API keys**:
```bash
# For Claude
export ANTHROPIC_API_KEY=your_api_key

# For Gemini
export GEMINI_API_KEY=your_api_key

# For Cursor
export CURSOR_API_KEY=your_api_key
```

5. **Launch the TUI**:
```bash
pipeline-kit
```

6. **Start a pipeline**:
Type `/start code-review` in the command input and press Enter.

## Configuration

### Directory Structure

```
your-project/
├── .pipeline-kit/
│   ├── config.toml              # Global settings
│   ├── agents/                  # Agent definitions
│   │   ├── developer.md
│   │   ├── reviewer.md
│   │   └── researcher.md
│   └── pipelines/               # Pipeline workflows
│       ├── code-review.yaml
│       ├── feature-dev.yaml
│       └── bug-fix.yaml
```

### Global Configuration (`config.toml`)

```toml
# Enable git integration
git = true

# Default timeout for agent execution (seconds)
timeout = 300
```

### Agent Configuration

Agents are defined in Markdown files with YAML frontmatter:

```markdown
---
name: agent-name
description: Brief description
model: claude-sonnet-4.5  # or gemini-1.5-pro, cursor-default
color: blue               # UI color: blue, green, yellow, red, etc.
---

System prompt for the agent goes here.
You can use multiple paragraphs and markdown formatting.

Key responsibilities:
- Task 1
- Task 2
```

**Supported Models**:
- **Claude**: `claude-sonnet-4.5`, `claude-opus-4`, `claude-haiku-4`
- **Gemini**: `gemini-1.5-pro`, `gemini-1.5-flash`
- **Cursor**: `cursor-default`

### Pipeline Configuration

Pipelines are defined in YAML:

```yaml
name: pipeline-name

# Master agent coordinates the workflow
master:
  model: claude-sonnet-4.5
  system-prompt: |
    Your role as the master orchestrator.
    Coordinate between sub-agents to achieve the goal.

  # Sequential process steps
  process:
    - researcher      # First sub-agent
    - developer       # Second sub-agent
    - HUMAN_REVIEW    # Pause for manual review
    - reviewer        # Final sub-agent

# List of sub-agents used in this pipeline
sub-agents:
  - researcher
  - developer
  - reviewer

# Optional: Required reference files
required-reference-file: false
```

**Special Keywords**:
- `HUMAN_REVIEW`: Pauses the pipeline for manual review. Resume with `/resume <process-id>`

## Usage

### TUI Mode (Interactive)

Launch the interactive terminal UI:

```bash
pipeline-kit
```

**Keyboard Controls**:
- `↑/↓` or `j/k`: Navigate between processes
- `Tab`: Autocomplete slash commands
- `Enter`: Execute command
- `Esc`: Clear input
- `PageUp/PageDown`: Fast scroll in detail view
- `q` or `Ctrl+C`: Quit

### CLI Mode (Non-Interactive)

Execute pipelines directly from the command line:

```bash
# Start a pipeline
pipeline-kit start <pipeline-name>

# List all pipelines
pipeline-kit list

# Show pipeline status
pipeline-kit status <process-id>
```

### Slash Commands

Available commands in TUI mode:

| Command | Description | Example |
|---------|-------------|---------|
| `/start <name>` | Start a new pipeline | `/start code-review` |
| `/pause <id>` | Pause a running process | `/pause a1b2c3d4` |
| `/resume <id>` | Resume a paused process | `/resume a1b2c3d4` |
| `/kill <id>` | Kill a running process | `/kill a1b2c3d4` |
| `/list` | List all active processes | `/list` |
| `/detail <id>` | Show process details | `/detail a1b2c3d4` |

**Tip**: Use `Tab` for command autocomplete and process ID suggestions.

## Supported Agents

Pipeline Kit supports multiple AI providers through adapters:

### Claude (Anthropic)

```bash
export ANTHROPIC_API_KEY=your_api_key
```

**Models**:
- `claude-sonnet-4.5` (recommended)
- `claude-opus-4`
- `claude-haiku-4`

**Features**: Streaming responses, tool calling, vision support

### Gemini (Google)

```bash
export GEMINI_API_KEY=your_api_key
```

**Models**:
- `gemini-1.5-pro`
- `gemini-1.5-flash`

**Features**: Streaming responses, multimodal input

### Cursor

```bash
export CURSOR_API_KEY=your_api_key
```

**Models**:
- `cursor-default`

**Features**: Code-focused assistant

## Example Workflows

### Code Review Pipeline

```yaml
name: code-review
master:
  model: claude-sonnet-4.5
  system-prompt: |
    Orchestrate a thorough code review process.
    Ensure code quality, testing, and documentation.
  process:
    - developer       # Implements the feature
    - reviewer        # Reviews the code
    - HUMAN_REVIEW    # Manual approval
sub-agents:
  - developer
  - reviewer
```

### Feature Development Pipeline

```yaml
name: feature-dev
master:
  model: claude-sonnet-4.5
  system-prompt: |
    Guide the development of a new feature from planning to implementation.
  process:
    - researcher      # Research requirements
    - architect       # Design the solution
    - developer       # Implement the feature
    - tester          # Write tests
    - HUMAN_REVIEW    # Final review
sub-agents:
  - researcher
  - architect
  - developer
  - tester
```

### Bug Fix Pipeline

```yaml
name: bug-fix
master:
  model: claude-sonnet-4.5
  system-prompt: |
    Coordinate debugging and fixing process.
  process:
    - debugger        # Identify the issue
    - developer       # Implement the fix
    - tester          # Verify the fix
sub-agents:
  - debugger
  - developer
  - tester
```

## Architecture

Pipeline Kit uses a monorepo architecture with clear separation of concerns:

```
┌─────────────────┐
│   User (CLI)    │
└────────┬────────┘
         │
         ▼
┌─────────────────────┐
│  TypeScript Wrapper │  (Platform detection & binary launcher)
│  pipeline-kit-cli   │
└────────┬────────────┘
         │
         ▼
┌──────────────────────────────────────────┐
│         Rust Binary (pipeline-kit-rs)    │
├──────────────────────────────────────────┤
│  ┌────────┐           ┌──────────────┐  │
│  │  TUI   │ ◄────────►│    Core      │  │
│  │(ratatui)│  Op/Event │  (Business)  │  │
│  └────────┘           └───┬──────────┘  │
│                           │              │
│                  ┌────────┼────────┐     │
│                  │        │        │     │
│              ┌───▼──┐ ┌───▼────┐ ┌▼────┐│
│              │State │ │Pipeline│ │Agent││
│              │Mgr   │ │Engine  │ │Mgr  ││
│              └──────┘ └────────┘ └─────┘│
└──────────────────────────────────────────┘
```

**Key Components**:
- **pk-protocol**: Shared data structures and IPC definitions
- **pk-core**: Business logic (StateManager, PipelineEngine, AgentManager)
- **pk-tui**: Interactive terminal UI with widgets
- **pk-cli**: Binary entry point that wires everything together

## Development

Want to contribute? See [CONTRIBUTING.md](CONTRIBUTING.md) for:
- Development setup
- Project structure details
- Testing strategy
- Coding conventions
- PR submission guidelines

### Quick Development Setup

```bash
# Clone repository
git clone https://github.com/Vooster-AI/pipeline-kit.git
cd pipeline-kit

# Build Rust workspace
cd pipeline-kit-rs
cargo build --release

# Run tests
cargo test --workspace

# Run the TUI
cargo run --release --bin pipeline
```

## Troubleshooting

### Binary Not Found

If you get "binary not found" error:

```bash
# Check vendor directory
ls -la node_modules/pipeline-kit/vendor/

# Reinstall to download binaries
npm install -g pipeline-kit --force
```

### API Key Issues

Ensure environment variables are set:

```bash
# Check if API keys are set
echo $ANTHROPIC_API_KEY
echo $GEMINI_API_KEY
echo $CURSOR_API_KEY

# Set in your shell profile (~/.bashrc, ~/.zshrc)
export ANTHROPIC_API_KEY=your_key_here
```

### Configuration Not Found

Pipeline Kit looks for `.pipeline-kit/` in:
1. Current directory
2. Parent directories (up to repository root)
3. Home directory (`~/.pipeline-kit/`)

Verify your configuration directory exists:

```bash
ls -la .pipeline-kit/
```

## Platform Support

| Platform | Architecture | Binary Name | Status |
|----------|--------------|-------------|--------|
| macOS | Intel (x64) | macos-x64 | ✅ Supported |
| macOS | Apple Silicon (ARM64) | macos-arm64 | ✅ Supported |
| Linux | x86_64 | linux-x64 | ✅ Supported |
| Linux | ARM64 | linux-arm64 | ✅ Supported |
| Windows | x64 | windows-x64 | ✅ Supported |
| Windows | ARM64 | windows-arm64 | ⚠️ Experimental |

## License

MIT License - see [LICENSE](LICENSE) file for details.

## Credits

Built with reference to the excellent [codex-cli](https://github.com/openai/codex) architecture.

## Links

- [npm package](https://www.npmjs.com/package/pipeline-kit)
- [GitHub repository](https://github.com/Vooster-AI/pipeline-kit)
- [Issue tracker](https://github.com/Vooster-AI/pipeline-kit/issues)
- [Contributing guidelines](CONTRIBUTING.md)

---

Made with ❤️ by the Vooster AI team
