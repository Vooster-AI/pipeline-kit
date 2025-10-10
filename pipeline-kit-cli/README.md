# pipeline-kit

AI agent pipeline orchestration CLI

## Overview

`pipeline-kit` is a high-performance CLI tool for orchestrating AI agent pipelines. This npm package serves as a cross-platform distribution wrapper that automatically downloads and executes the appropriate native Rust binary for your operating system and architecture.

### Architecture

- **Core Engine**: Written in Rust for maximum performance and reliability
- **npm Wrapper**: Provides seamless installation and execution across platforms
- **Cross-Platform**: Automatically detects and runs the correct native binary (macOS, Linux, Windows)

## Installation

### npm (Recommended)

```bash
npm install -g pipeline-kit
```

### pnpm

```bash
pnpm add -g pipeline-kit
```

### yarn

```bash
yarn global add pipeline-kit
```

## Quick Start

After installation, the `pipeline-kit` command will be available globally:

```bash
# Initialize a new project
pipeline-kit init

# Run a pipeline
pipeline-kit run my-pipeline

# Interactive mode with TUI
pipeline-kit
```

## Features

- **High Performance**: Rust-powered core engine for fast pipeline execution
- **Interactive TUI**: Built-in terminal user interface for real-time monitoring
- **Agent Abstraction**: Pluggable adapter pattern for various AI providers (Claude, Gemini, etc.)
- **Pipeline Orchestration**: Define complex multi-step AI workflows with YAML
- **State Management**: Robust state tracking and process management
- **Cross-Platform**: Works on macOS, Linux, and Windows

## Project Structure

This package is part of a monorepo:

```
pipeline-kit/
├── pipeline-kit-cli/      # This npm package (TypeScript wrapper)
│   ├── bin/               # CLI entry point
│   ├── lib/               # TypeScript source
│   ├── scripts/           # Installation scripts
│   └── package.json
│
└── pipeline-kit-rs/       # Rust core engine
    ├── crates/cli/        # Binary entry point
    ├── crates/core/       # Pipeline engine & agent manager
    ├── crates/tui/        # Interactive terminal UI
    └── crates/protocol/   # Shared data structures
```

## How It Works

When you install `pipeline-kit` via npm:

1. The `postinstall` script detects your OS and architecture
2. Downloads the pre-built Rust binary from GitHub releases
3. Stores it in the `vendor/` directory
4. The CLI wrapper executes the native binary with your arguments

This approach provides:
- **Fast execution** (native Rust performance)
- **Easy distribution** (via npm ecosystem)
- **Automatic updates** (standard npm workflow)

## Development

### Prerequisites

- Node.js >= 16
- Rust toolchain (for building the core engine)

### Local Development

```bash
# Clone the repository
git clone https://github.com/Vooster-AI/pipeline-kit.git
cd pipeline-kit

# Install npm dependencies
cd pipeline-kit-cli
npm install

# Build the Rust core (from project root)
cd ../pipeline-kit-rs
cargo build --release

# The built binary will be in pipeline-kit-rs/target/release/
```

### Testing

```bash
# Run TypeScript tests
npm test

# Run tests with coverage
npm run test:coverage

# Type checking
npm run type-check
```

## Configuration

Pipeline and agent configurations are stored in the `.pipeline-kit/` directory:

```
.pipeline-kit/
├── pipelines/    # YAML pipeline definitions
└── agents/       # Markdown agent configurations
```

See the [main repository](https://github.com/Vooster-AI/pipeline-kit) for detailed configuration documentation.

## Contributing

This package is part of a larger monorepo. For contribution guidelines, please refer to the main repository.

**Important**: Business logic should NOT be added to this npm wrapper. All core functionality must be implemented in the Rust crates (`pipeline-kit-rs/`).

## License

MIT

## Links

- [GitHub Repository](https://github.com/Vooster-AI/pipeline-kit)
- [Issue Tracker](https://github.com/Vooster-AI/pipeline-kit/issues)
- [Documentation](https://github.com/Vooster-AI/pipeline-kit#readme)

## Support

For questions and support:
- Open an issue on [GitHub](https://github.com/Vooster-AI/pipeline-kit/issues)
- Check the documentation in the main repository

---

**Note**: This is the npm distribution package. For detailed architecture and development guidelines, see the main [pipeline-kit repository](https://github.com/Vooster-AI/pipeline-kit).
