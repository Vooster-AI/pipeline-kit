#!/usr/bin/env node

/**
 * Install Pipeline Kit native binary
 * This script is called as a postinstall hook by npm.
 *
 * It supports two modes:
 * 1. Production mode (NODE_ENV=production): Downloads binaries from GitHub Releases
 * 2. Development mode (default): Copies binaries from local Rust build
 */

import fs from 'fs';
import path from 'path';
import os from 'os';
import { fileURLToPath } from 'url';
import axios from 'axios';
import { extract as tarExtract } from 'tar';

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);

/**
 * Detects the current platform and returns platform name and binary name
 * @param {string} platform - process.platform value (e.g., 'darwin', 'linux', 'win32')
 * @param {string} arch - process.arch value (e.g., 'x64', 'arm64')
 * @returns {{ platformName: string, binaryName: string }}
 * @throws {Error} If platform is unsupported
 */
export function detectPlatform(platform, arch) {
  const platformKey = `${platform}-${arch}`;

  const platformMap = {
    'darwin-x64': 'macos-x64',
    'darwin-arm64': 'macos-arm64',
    'linux-x64': 'linux-x64',
    'linux-arm64': 'linux-arm64',
    'android-arm64': 'linux-arm64', // Android uses Linux binaries
    'win32-x64': 'windows-x64',
    'win32-arm64': 'windows-arm64'
  };

  const platformName = platformMap[platformKey];

  if (!platformName) {
    throw new Error(
      `Unsupported platform: ${platform} (${arch})\n\n` +
      'Supported platforms:\n' +
      '  - macOS x64 (Intel)\n' +
      '  - macOS ARM64 (Apple Silicon)\n' +
      '  - Linux x64\n' +
      '  - Linux ARM64\n' +
      '  - Windows x64\n' +
      '  - Windows ARM64\n'
    );
  }

  const binaryName = platform === 'win32' ? 'pipeline.exe' : 'pipeline';

  return { platformName, binaryName };
}

/**
 * Builds the GitHub release download URL
 * @param {object} options
 * @param {string} options.repo - GitHub repository (e.g., 'Vooster-AI/pipeline-kit')
 * @param {string} options.version - Release version ('latest' or specific tag like 'v0.1.0')
 * @param {string} options.platformName - Platform name (e.g., 'macos-arm64')
 * @returns {string} - Download URL
 */
export function buildDownloadUrl({ repo, version, platformName }) {
  const archiveName = `pipeline-kit-${platformName}.tar.gz`;

  if (version === 'latest') {
    return `https://github.com/${repo}/releases/latest/download/${archiveName}`;
  } else {
    return `https://github.com/${repo}/releases/download/${version}/${archiveName}`;
  }
}

/**
 * Downloads and extracts a binary from GitHub releases
 * @param {object} options
 * @param {string} options.url - Download URL
 * @param {string} options.vendorDir - Vendor directory path
 * @param {string} options.platformName - Platform name
 * @param {string} options.binaryName - Binary filename
 * @param {boolean} [options.showProgress] - Whether to show download progress
 * @returns {Promise<void>}
 */
export async function downloadAndExtract({ url, vendorDir, platformName, binaryName, showProgress = true }) {
  const platformDir = path.join(vendorDir, platformName, 'pipeline-kit');
  const binaryPath = path.join(platformDir, binaryName);

  console.log('Production mode: Downloading Pipeline Kit binary from GitHub Releases...');
  console.log(`  URL: ${url}`);
  console.log(`  Platform: ${platformName}`);

  try {
    // Download the tar.gz file
    const response = await axios.get(url, {
      responseType: 'stream',
      timeout: 120000, // 2 minutes timeout
      maxRedirects: 5
    });

    if (!response.data) {
      throw new Error('No data received from download');
    }

    // Create a temporary file for the download
    const tempDir = fs.mkdtempSync(path.join(os.tmpdir(), 'pipeline-kit-'));
    const tempFile = path.join(tempDir, `pipeline-kit-${platformName}.tar.gz`);

    // Show progress if enabled
    const totalLength = response.headers['content-length'];
    let downloadedLength = 0;

    if (showProgress && totalLength) {
      console.log(`  Downloading: 0%`);
    }

    // Write to temporary file
    const writer = fs.createWriteStream(tempFile);

    response.data.on('data', (chunk) => {
      downloadedLength += chunk.length;
      if (showProgress && totalLength) {
        const percentage = Math.round((downloadedLength / totalLength) * 100);
        process.stdout.write(`\r  Downloading: ${percentage}%`);
      }
    });

    await new Promise((resolve, reject) => {
      response.data.pipe(writer);
      writer.on('finish', resolve);
      writer.on('error', reject);
    });

    if (showProgress && totalLength) {
      console.log(''); // New line after progress
    }

    console.log('  Extracting archive...');

    // Create platform directory
    fs.mkdirSync(platformDir, { recursive: true });

    // Extract tar.gz to platform directory
    await tarExtract({
      file: tempFile,
      cwd: platformDir
    });

    // Verify binary exists
    if (!fs.existsSync(binaryPath)) {
      throw new Error(
        `Binary not found in archive at expected location: ${binaryPath}\n` +
        'Please check that the archive contains the correct structure.'
      );
    }

    // Make binary executable (Unix systems)
    if (process.platform !== 'win32') {
      fs.chmodSync(binaryPath, 0o755);
    }

    // Clean up temp file
    fs.rmSync(tempDir, { recursive: true, force: true });

    console.log(`  Binary installed successfully at: ${binaryPath}`);

  } catch (error) {
    if (error.response) {
      // HTTP error
      throw new Error(
        `Failed to download binary from ${url}\n` +
        `HTTP Status: ${error.response.status}\n` +
        `\n` +
        `Possible reasons:\n` +
        `  - No releases have been published yet\n` +
        `  - The release doesn't include binaries for ${platformName}\n` +
        `  - Network connectivity issues\n` +
        `\n` +
        `For development, build locally:\n` +
        `  cd pipeline-kit-rs && cargo build --release\n` +
        `  Then run: npm install (without NODE_ENV=production)`
      );
    } else {
      // Other error (network, extraction, etc.)
      throw error;
    }
  }
}

/**
 * Copies a local binary from Rust build directory to vendor directory
 * @param {object} options
 * @param {string} options.sourcePath - Source binary path
 * @param {string} options.vendorDir - Vendor directory path
 * @param {string} options.platformName - Platform name
 * @param {string} options.binaryName - Binary filename
 * @returns {Promise<void>}
 */
export async function copyLocalBinary({ sourcePath, vendorDir, platformName, binaryName }) {
  const platformDir = path.join(vendorDir, platformName, 'pipeline-kit');
  const destPath = path.join(platformDir, binaryName);

  console.log('Development mode: Installing Pipeline Kit binary from local build...');
  console.log(`  Source: ${sourcePath}`);
  console.log(`  Destination: ${destPath}`);
  console.log(`  Platform: ${platformName}`);

  // Check if source binary exists
  if (!fs.existsSync(sourcePath)) {
    throw new Error(
      `Binary not found at ${sourcePath}\n` +
      `\n` +
      `For development, build the Rust binary first:\n` +
      `  cd pipeline-kit-rs\n` +
      `  cargo build --release\n` +
      `\n` +
      `For production installation, use: NODE_ENV=production npm install`
    );
  }

  // Create platform directory
  fs.mkdirSync(platformDir, { recursive: true });

  // Copy binary
  fs.copyFileSync(sourcePath, destPath);

  // Make binary executable (Unix systems)
  if (process.platform !== 'win32') {
    fs.chmodSync(destPath, 0o755);
  }

  console.log('Binary installed successfully.');
}

/**
 * Main install function
 * @param {object} [options] - Optional configuration for testing
 * @param {string} [options.mode] - Installation mode ('production' or 'development')
 * @param {string} [options.cliRoot] - CLI root directory path
 * @param {string} [options.platform] - Platform override (for testing)
 * @param {string} [options.arch] - Architecture override (for testing)
 * @returns {Promise<void>}
 */
export async function install(options = {}) {
  // Determine installation mode
  const mode = options.mode || (process.env.NODE_ENV === 'production' ? 'production' : 'development');

  // Determine paths
  const cliRoot = options.cliRoot || path.resolve(__dirname, '..');
  const vendorDir = path.join(cliRoot, 'vendor');

  // Detect platform
  const platform = options.platform || process.platform;
  const arch = options.arch || process.arch;
  const { platformName, binaryName } = detectPlatform(platform, arch);

  if (mode === 'production') {
    // Production mode: download from GitHub Releases
    const repo = process.env.PIPELINE_KIT_REPO || 'Vooster-AI/pipeline-kit';
    const version = process.env.PIPELINE_KIT_VERSION || 'latest';

    // Allow overriding download URL for CI/local testing
    let url = process.env.PIPELINE_KIT_DOWNLOAD_URL;
    const base = process.env.PIPELINE_KIT_DOWNLOAD_BASE;
    if (!url) {
      if (base) {
        const baseTrimmed = base.endsWith('/') ? base : base + '/';
        url = `${baseTrimmed}pipeline-kit-${platformName}.tar.gz`;
      } else {
        url = buildDownloadUrl({ repo, version, platformName });
      }
    }

    await downloadAndExtract({
      url,
      vendorDir,
      platformName,
      binaryName,
      showProgress: !options.mode // Only show progress in real execution, not tests
    });
  } else {
    // Development mode: copy from local Rust build
    const rustBuildDir = path.join(cliRoot, '..', 'pipeline-kit-rs', 'target', 'release');
    const sourcePath = path.join(rustBuildDir, binaryName);

    try {
      await copyLocalBinary({
        sourcePath,
        vendorDir,
        platformName,
        binaryName
      });
    } catch (error) {
      // In development mode, show warning but don't fail
      // This allows npm install to complete even without a built binary
      console.warn('\nWarning:', error.message);
      console.warn('Installation will continue, but the binary will not be available until built.\n');
    }
  }
}

// Run install if this script is executed directly (not imported as a module)
if (import.meta.url === `file://${process.argv[1]}`) {
  install().catch((error) => {
    console.error('\nError during installation:', error.message);
    process.exit(1);
  });
}
