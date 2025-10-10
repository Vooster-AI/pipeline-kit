#!/usr/bin/env bash
# Install Pipeline Kit native binary
# This script is called as a postinstall hook by npm.

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
CLI_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
VENDOR_DIR="$CLI_ROOT/vendor"

# Detect platform and architecture
PLATFORM="$(uname -s | tr '[:upper:]' '[:lower:]')"
ARCH="$(uname -m)"

# Map to Rust target triple
case "$PLATFORM" in
  linux)
    case "$ARCH" in
      x86_64)
        TARGET_TRIPLE="x86_64-unknown-linux-musl"
        ;;
      aarch64|arm64)
        TARGET_TRIPLE="aarch64-unknown-linux-musl"
        ;;
      *)
        echo "Unsupported architecture: $ARCH on $PLATFORM"
        exit 1
        ;;
    esac
    BINARY_NAME="pipeline"
    ;;
  darwin)
    case "$ARCH" in
      x86_64)
        TARGET_TRIPLE="x86_64-apple-darwin"
        ;;
      arm64)
        TARGET_TRIPLE="aarch64-apple-darwin"
        ;;
      *)
        echo "Unsupported architecture: $ARCH on $PLATFORM"
        exit 1
        ;;
    esac
    BINARY_NAME="pipeline"
    ;;
  mingw*|msys*|cygwin*)
    case "$ARCH" in
      x86_64)
        TARGET_TRIPLE="x86_64-pc-windows-msvc"
        ;;
      aarch64|arm64)
        TARGET_TRIPLE="aarch64-pc-windows-msvc"
        ;;
      *)
        echo "Unsupported architecture: $ARCH on $PLATFORM"
        exit 1
        ;;
    esac
    BINARY_NAME="pipeline.exe"
    ;;
  *)
    echo "Unsupported platform: $PLATFORM"
    exit 1
    ;;
esac

ARCH_DIR="$VENDOR_DIR/$TARGET_TRIPLE/pipeline-kit"
BINARY_DEST="$ARCH_DIR/$BINARY_NAME"

# Development mode: copy from local Rust build if it exists
RUST_BUILD_DIR="$CLI_ROOT/../pipeline-kit-rs/target/release"
SOURCE_BINARY="$RUST_BUILD_DIR/$BINARY_NAME"

if [ -f "$SOURCE_BINARY" ]; then
  echo "Installing Pipeline Kit binary from local build..."
  echo "  Source: $SOURCE_BINARY"
  echo "  Destination: $BINARY_DEST"

  mkdir -p "$ARCH_DIR"
  cp "$SOURCE_BINARY" "$BINARY_DEST"
  chmod +x "$BINARY_DEST"

  echo "Binary installed successfully."
else
  # In production, this would download from GitHub releases
  # For now, we just provide a helpful message
  echo "Warning: Pipeline Kit binary not found at $SOURCE_BINARY"
  echo ""
  echo "For development, build the Rust binary first:"
  echo "  cd pipeline-kit-rs"
  echo "  cargo build --release"
  echo ""
  echo "For production installation, binaries will be downloaded from GitHub releases."
  echo "Installation will continue, but the binary will not be available until built."
  # Don't exit with error to allow npm install to complete
fi
