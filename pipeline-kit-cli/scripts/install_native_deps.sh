#!/usr/bin/env bash
# Install Pipeline Kit native binary
# This script is called as a postinstall hook by npm.

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
CLI_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
VENDOR_DIR="$CLI_ROOT/vendor"

# Determine the mode: production or development
# Production mode is triggered by NODE_ENV=production or --production flag
MODE="development"
if [ "${NODE_ENV:-}" = "production" ] || [ "${1:-}" = "--production" ]; then
  MODE="production"
fi

# Helper function to determine user-friendly platform name
# This maps OS and architecture to a simple, user-friendly format
# Used for both GitHub Release assets and local vendor directory structure
get_platform_name() {
  local os=$(uname -s)
  local arch=$(uname -m)

  case "$os-$arch" in
    Darwin-x86_64) echo "macos-x64" ;;
    Darwin-arm64) echo "macos-arm64" ;;
    Linux-x86_64) echo "linux-x64" ;;
    Linux-aarch64) echo "linux-arm64" ;;
    MINGW*-x86_64|MSYS*-x86_64|CYGWIN*-x86_64) echo "windows-x64" ;;
    MINGW*-aarch64|MSYS*-aarch64|CYGWIN*-aarch64) echo "windows-arm64" ;;
    *) echo "unsupported" ;;
  esac
}

# Get user-friendly platform name
PLATFORM_NAME=$(get_platform_name)
if [ "$PLATFORM_NAME" = "unsupported" ]; then
  echo "Error: Unsupported platform: $(uname -s) ($(uname -m))"
  echo ""
  echo "Supported platforms:"
  echo "  - macOS x64 (Intel)"
  echo "  - macOS ARM64 (Apple Silicon)"
  echo "  - Linux x64"
  echo "  - Linux ARM64"
  echo "  - Windows x64"
  echo "  - Windows ARM64"
  echo ""
  exit 1
fi

# Determine binary name based on platform
case "$PLATFORM_NAME" in
  windows-*)
    BINARY_NAME="pipeline.exe"
    ;;
  *)
    BINARY_NAME="pipeline"
    ;;
esac

# Use user-friendly platform name for directory structure
PLATFORM_DIR="$VENDOR_DIR/$PLATFORM_NAME/pipeline-kit"
BINARY_DEST="$PLATFORM_DIR/$BINARY_NAME"

if [ "$MODE" = "production" ]; then
  # Production mode: download from GitHub Releases
  echo "Production mode: Downloading Pipeline Kit binary from GitHub Releases..."

  # Check if gh CLI is available
  if ! command -v gh &> /dev/null; then
    echo ""
    echo "Error: gh CLI is not installed."
    echo ""
    echo "The gh CLI is required to download binaries from GitHub Releases."
    echo "Install it from: https://cli.github.com/"
    echo ""
    echo "Alternatively, for development:"
    echo "  1. Build the Rust binary: cd pipeline-kit-rs && cargo build --release"
    echo "  2. Run npm install without NODE_ENV=production"
    echo ""
    exit 1
  fi

  # Get the repository from package.json or use default
  REPO="${PIPELINE_KIT_REPO:-Vooster-AI/pipeline-kit}"

  # Get the latest release tag or use a specific one
  RELEASE_TAG="${PIPELINE_KIT_VERSION:-latest}"

  echo "  Repository: $REPO"
  echo "  Release: $RELEASE_TAG"
  echo "  Platform: $PLATFORM_NAME"

  # Create a temporary directory for downloads
  TEMP_DIR=$(mktemp -d)
  trap "rm -rf $TEMP_DIR" EXIT

  # Download the archive for this platform
  ARCHIVE_NAME="pipeline-kit-${PLATFORM_NAME}.tar.gz"

  echo "  Downloading: $ARCHIVE_NAME"

  if [ "$RELEASE_TAG" = "latest" ]; then
    gh release download --repo "$REPO" -p "$ARCHIVE_NAME" -D "$TEMP_DIR" 2>&1 || {
      echo ""
      echo "Error: Failed to download $ARCHIVE_NAME from latest release"
      echo ""
      echo "Possible reasons:"
      echo "  - No releases have been published yet"
      echo "  - The release doesn't include binaries for $PLATFORM_NAME"
      echo "  - Network connectivity issues"
      echo ""
      echo "To create a release with binaries:"
      echo "  1. Set up GitHub Actions to build binaries for all platforms"
      echo "  2. Publish a release with the built binaries"
      echo ""
      echo "For development, build locally:"
      echo "  cd pipeline-kit-rs && cargo build --release"
      echo "  Then run: npm install (without NODE_ENV=production)"
      echo ""
      exit 1
    }
  else
    gh release download "$RELEASE_TAG" --repo "$REPO" -p "$ARCHIVE_NAME" -D "$TEMP_DIR" 2>&1 || {
      echo ""
      echo "Error: Failed to download $ARCHIVE_NAME from release $RELEASE_TAG"
      echo ""
      echo "Verify that:"
      echo "  1. Release $RELEASE_TAG exists"
      echo "  2. It includes a $ARCHIVE_NAME asset"
      echo "  3. You have access to the repository"
      echo ""
      exit 1
    }
  fi

  ARCHIVE_PATH="$TEMP_DIR/$ARCHIVE_NAME"

  if [ ! -f "$ARCHIVE_PATH" ]; then
    echo "Error: Downloaded archive not found at $ARCHIVE_PATH"
    exit 1
  fi

  # Download checksum file if available (optional)
  CHECKSUM_NAME="${ARCHIVE_NAME}.sha256"
  if [ "$RELEASE_TAG" = "latest" ]; then
    gh release download --repo "$REPO" -p "$CHECKSUM_NAME" -D "$TEMP_DIR" 2>/dev/null || echo "  Note: No checksum file available"
  else
    gh release download "$RELEASE_TAG" --repo "$REPO" -p "$CHECKSUM_NAME" -D "$TEMP_DIR" 2>/dev/null || echo "  Note: No checksum file available"
  fi

  CHECKSUM_PATH="$TEMP_DIR/$CHECKSUM_NAME"

  # Verify checksum if available
  if [ -f "$CHECKSUM_PATH" ]; then
    echo "  Verifying checksum..."
    if command -v sha256sum &> /dev/null; then
      cd "$TEMP_DIR"
      if sha256sum -c "$CHECKSUM_NAME" 2>&1 | grep -q "$ARCHIVE_NAME: OK"; then
        echo "  Checksum verified successfully"
      else
        echo "Error: Checksum verification failed for $ARCHIVE_NAME"
        exit 1
      fi
      cd - > /dev/null
    elif command -v shasum &> /dev/null; then
      cd "$TEMP_DIR"
      if shasum -a 256 -c "$CHECKSUM_NAME" 2>&1 | grep -q "$ARCHIVE_NAME: OK"; then
        echo "  Checksum verified successfully"
      else
        echo "Error: Checksum verification failed for $ARCHIVE_NAME"
        exit 1
      fi
      cd - > /dev/null
    else
      echo "  Warning: sha256sum/shasum not available, skipping checksum verification"
    fi
  fi

  # Extract the archive
  echo "  Extracting archive..."
  mkdir -p "$PLATFORM_DIR"
  tar -xzf "$ARCHIVE_PATH" -C "$PLATFORM_DIR"

  # Verify the binary exists
  if [ ! -f "$BINARY_DEST" ]; then
    echo "Error: Binary not found in archive at expected location: $BINARY_DEST"
    echo "Archive contents:"
    tar -tzf "$ARCHIVE_PATH"
    exit 1
  fi

  # Make binary executable
  chmod +x "$BINARY_DEST"

  echo "  Binary installed successfully at: $BINARY_DEST"

else
  # Development mode: copy from local Rust build if it exists
  RUST_BUILD_DIR="$CLI_ROOT/../pipeline-kit-rs/target/release"
  SOURCE_BINARY="$RUST_BUILD_DIR/$BINARY_NAME"

  if [ -f "$SOURCE_BINARY" ]; then
    echo "Development mode: Installing Pipeline Kit binary from local build..."
    echo "  Source: $SOURCE_BINARY"
    echo "  Destination: $BINARY_DEST"
    echo "  Platform: $PLATFORM_NAME"

    mkdir -p "$PLATFORM_DIR"
    cp "$SOURCE_BINARY" "$BINARY_DEST"
    chmod +x "$BINARY_DEST"

    echo "Binary installed successfully."
  else
    # In development without a local build
    echo "Warning: Pipeline Kit binary not found at $SOURCE_BINARY"
    echo ""
    echo "For development, build the Rust binary first:"
    echo "  cd pipeline-kit-rs"
    echo "  cargo build --release"
    echo ""
    echo "For production installation, use: NODE_ENV=production npm install"
    echo "Installation will continue, but the binary will not be available until built."
    # Don't exit with error to allow npm install to complete
  fi
fi
