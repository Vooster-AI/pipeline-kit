# Pipeline Kit CLI Scripts

This directory contains scripts for building and installing the Pipeline Kit native binary.

## Scripts

### install_native_deps.sh

Installs the Pipeline Kit native binary from either local build (development) or GitHub Releases (production).

#### Usage

**Development Mode (default):**
```bash
# Copies binary from local Rust build if available
npm install
# or
bash scripts/install_native_deps.sh
```

**Production Mode:**
```bash
# Downloads binary from GitHub Releases
NODE_ENV=production npm install
# or
bash scripts/install_native_deps.sh --production
```

#### Environment Variables

- `NODE_ENV=production`: Triggers production mode (downloads from GitHub Releases)
- `PIPELINE_KIT_REPO`: Override the default repository (default: `pipeline-kit/pipeline-kit`)
- `PIPELINE_KIT_VERSION`: Specify a release tag (default: `latest`)

#### Examples

Download from a specific release:
```bash
PIPELINE_KIT_VERSION=v1.0.0 NODE_ENV=production npm install
```

Download from a custom repository:
```bash
PIPELINE_KIT_REPO=myorg/pipeline-kit NODE_ENV=production npm install
```

#### Requirements

**Development Mode:**
- Rust toolchain
- Built binary at `../pipeline-kit-rs/target/release/pipeline`

**Production Mode:**
- [GitHub CLI (gh)](https://cli.github.com/) - required for downloading releases
- Published release with platform-specific binaries

#### Binary Archive Format

Production mode expects GitHub Release assets in the following format:
- Archive name: `pipeline-kit-{platform}.tar.gz`
- Platforms: `linux-x86_64`, `linux-aarch64`, `macos-x86_64`, `macos-aarch64`, `windows-x86_64`, `windows-aarch64`
- Archive structure: Binary at root level named `pipeline` (or `pipeline.exe` for Windows)

Optional checksum files:
- Format: `pipeline-kit-{platform}.tar.gz.sha256`
- Content: Standard sha256sum format (hash followed by filename)

#### Supported Platforms

| Platform | Architecture | Target Triple | Binary Name |
|----------|-------------|---------------|-------------|
| Linux | x86_64 | x86_64-unknown-linux-musl | pipeline |
| Linux | aarch64 | aarch64-unknown-linux-musl | pipeline |
| macOS | x86_64 | x86_64-apple-darwin | pipeline |
| macOS | arm64 | aarch64-apple-darwin | pipeline |
| Windows | x86_64 | x86_64-pc-windows-msvc | pipeline.exe |
| Windows | aarch64 | aarch64-pc-windows-msvc | pipeline.exe |

### test_install_native_deps.sh

Test script for validating the install_native_deps.sh functionality.

#### Usage

```bash
bash scripts/test_install_native_deps.sh
```

This script performs the following tests:
1. Platform name detection for all supported platforms
2. Development mode installation (if local build exists)
3. Production mode structure validation
4. Error handling validation
5. Integration test (requires gh CLI and published releases)

## Setting Up GitHub Actions for Production

To enable production mode downloads, you need to:

1. Create a GitHub Actions workflow that builds binaries for all platforms
2. Create archives in the expected format:
   ```bash
   # For each platform
   tar -czf pipeline-kit-{platform}.tar.gz pipeline
   sha256sum pipeline-kit-{platform}.tar.gz > pipeline-kit-{platform}.tar.gz.sha256
   ```
3. Publish a GitHub Release with these archives as assets

See the main repository's `.github/workflows/` directory for example workflows.

## Troubleshooting

### Error: gh CLI is not installed

Install the GitHub CLI:
```bash
# macOS
brew install gh

# Linux
# See https://github.com/cli/cli/blob/trunk/docs/install_linux.md

# Windows
# See https://github.com/cli/cli#windows
```

### Error: Failed to download from latest release

This usually means no releases have been published yet. Either:
1. Build locally for development: `cd pipeline-kit-rs && cargo build --release`
2. Publish a release with binaries using GitHub Actions

### Binary not found in archive

The archive structure might be incorrect. Ensure the binary is at the root level of the tar.gz file:
```bash
tar -tzf pipeline-kit-macos-aarch64.tar.gz
# Should show: pipeline
```

## Development Workflow

For local development:
1. Build the Rust binary: `cd pipeline-kit-rs && cargo build --release`
2. Run `npm install` in the pipeline-kit-cli directory
3. The script will automatically copy the binary from the local build

For production releases:
1. Set up GitHub Actions to build for all platforms
2. Publish a release with the binaries
3. Users can then install with: `NODE_ENV=production npm install pipeline-kit`
