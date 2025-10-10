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

# Helper function to determine platform name for GitHub Release assets
get_platform_name() {
  local platform="$1"
  local arch="$2"

  case "$platform" in
    linux)
      case "$arch" in
        x86_64) echo "linux-x86_64" ;;
        aarch64|arm64) echo "linux-aarch64" ;;
        *) return 1 ;;
      esac
      ;;
    darwin)
      case "$arch" in
        x86_64) echo "macos-x86_64" ;;
        arm64) echo "macos-aarch64" ;;
        *) return 1 ;;
      esac
      ;;
    mingw*|msys*|cygwin*)
      case "$arch" in
        x86_64) echo "windows-x86_64" ;;
        aarch64|arm64) echo "windows-aarch64" ;;
        *) return 1 ;;
      esac
      ;;
    *)
      return 1
      ;;
  esac
}

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

# Get platform name for GitHub Release
PLATFORM_NAME=$(get_platform_name "$PLATFORM" "$ARCH")
if [ -z "$PLATFORM_NAME" ]; then
  echo "Error: Unable to determine platform name for $PLATFORM/$ARCH"
  exit 1
fi

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
  REPO="${PIPELINE_KIT_REPO:-pipeline-kit/pipeline-kit}"

  # Get the latest release tag or use a specific one
  RELEASE_TAG="${PIPELINE_KIT_VERSION:-latest}"

  echo "  Repository: $REPO"
  echo "  Release: $RELEASE_TAG"
  echo "  Platform: $PLATFORM_NAME"
  echo "  Target: $TARGET_TRIPLE"

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
  mkdir -p "$ARCH_DIR"
  tar -xzf "$ARCHIVE_PATH" -C "$ARCH_DIR"

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

    mkdir -p "$ARCH_DIR"
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
