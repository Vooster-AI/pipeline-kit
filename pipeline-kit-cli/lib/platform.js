/**
 * Platform detection and binary path resolution utilities.
 * This module is extracted to be testable separately from the main CLI script.
 */

import path from "path";

// Platform mapping from Node.js platform/arch to user-friendly names
// This matches the structure created by install_native_deps.sh
export const platformMap = {
  'darwin-x64': 'macos-x64',
  'darwin-arm64': 'macos-arm64',
  'linux-x64': 'linux-x64',
  'linux-arm64': 'linux-arm64',
  'android-arm64': 'linux-arm64', // Android uses Linux binaries
  'win32-x64': 'windows-x64',
  'win32-arm64': 'windows-arm64'
};

/**
 * Maps Node.js platform and architecture to user-friendly platform names.
 * @param {string} platform - process.platform value (e.g., 'darwin', 'linux', 'win32')
 * @param {string} arch - process.arch value (e.g., 'x64', 'arm64')
 * @returns {string | null} - User-friendly platform name or null if unsupported
 */
export function getPlatformName(platform, arch) {
  const platformKey = `${platform}-${arch}`;
  return platformMap[platformKey] || null;
}

/**
 * Legacy function for backward compatibility.
 * Maps Node.js platform and architecture to Rust target triples.
 * @param {string} platform - process.platform value (e.g., 'darwin', 'linux', 'win32')
 * @param {string} arch - process.arch value (e.g., 'x64', 'arm64')
 * @returns {string | null} - Rust target triple or null if unsupported
 */
export function getTargetTriple(platform, arch) {
  switch (platform) {
    case "linux":
    case "android":
      switch (arch) {
        case "x64":
          return "x86_64-unknown-linux-musl";
        case "arm64":
          return "aarch64-unknown-linux-musl";
        default:
          return null;
      }
    case "darwin":
      switch (arch) {
        case "x64":
          return "x86_64-apple-darwin";
        case "arm64":
          return "aarch64-apple-darwin";
        default:
          return null;
      }
    case "win32":
      switch (arch) {
        case "x64":
          return "x86_64-pc-windows-msvc";
        case "arm64":
          return "aarch64-pc-windows-msvc";
        default:
          return null;
      }
    default:
      return null;
  }
}

/**
 * Determines the binary filename based on platform.
 * @param {string} platform - process.platform value
 * @returns {string} - Binary filename ('pipeline.exe' on Windows, 'pipeline' elsewhere)
 */
export function getBinaryName(platform) {
  return platform === "win32" ? "pipeline.exe" : "pipeline";
}

/**
 * Constructs the vendor binary path using new platform naming.
 * @param {string} vendorRoot - Path to the vendor directory
 * @param {string} platformName - User-friendly platform name (e.g., 'macos-arm64')
 * @param {string} binaryName - Binary filename
 * @returns {string} - Full path to the vendor binary
 */
export function getVendorBinaryPath(vendorRoot, platformName, binaryName) {
  const platformRoot = path.join(vendorRoot, platformName);
  return path.join(platformRoot, "pipeline-kit", binaryName);
}

/**
 * Legacy function: Constructs the vendor binary path using target triple.
 * @param {string} vendorRoot - Path to the vendor directory
 * @param {string} targetTriple - Rust target triple
 * @param {string} binaryName - Binary filename
 * @returns {string} - Full path to the vendor binary
 */
export function getVendorBinaryPathLegacy(vendorRoot, targetTriple, binaryName) {
  const archRoot = path.join(vendorRoot, targetTriple);
  return path.join(archRoot, "pipeline-kit", binaryName);
}

/**
 * Constructs the development binary path.
 * @param {string} cliDir - Path to the pipeline-kit-cli directory
 * @param {string} binaryName - Binary filename
 * @returns {string} - Full path to the development binary
 */
export function getDevBinaryPath(cliDir, binaryName) {
  return path.join(cliDir, "..", "pipeline-kit-rs", "target", "release", binaryName);
}
