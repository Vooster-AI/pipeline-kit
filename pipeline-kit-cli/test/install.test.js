/**
 * Tests for the install.js script
 * These tests verify that the installation script correctly:
 * 1. Detects the platform and architecture
 * 2. Downloads binaries from GitHub releases in production mode
 * 3. Copies local binaries in development mode
 * 4. Extracts archives correctly
 * 5. Sets proper file permissions
 */

import { describe, it, expect, beforeEach, afterEach, vi } from 'vitest';
import fs from 'fs';
import path from 'path';
import os from 'os';
import { fileURLToPath } from 'url';

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);

// Import functions that will be tested
// These will be exported from install.js
let detectPlatform, buildDownloadUrl, downloadAndExtract, copyLocalBinary;

// Mock setup
let nock;

beforeEach(async () => {
  // Dynamically import nock for HTTP mocking
  const nockModule = await import('nock');
  nock = nockModule.default;

  // Clean all HTTP mocks
  nock.cleanAll();
});

afterEach(() => {
  // Clean all HTTP mocks
  if (nock) {
    nock.cleanAll();
  }

  // Clear module cache to get fresh imports
  vi.resetModules();
});

describe('install.js - Platform Detection', () => {
  it('should detect macOS ARM64 platform correctly', async () => {
    // This test will fail until install.js is created
    const installModule = await import('../scripts/install.js');
    detectPlatform = installModule.detectPlatform;

    const platform = detectPlatform('darwin', 'arm64');
    expect(platform).toEqual({
      platformName: 'macos-arm64',
      binaryName: 'pipeline'
    });
  });

  it('should detect macOS x64 platform correctly', async () => {
    const installModule = await import('../scripts/install.js');
    detectPlatform = installModule.detectPlatform;

    const platform = detectPlatform('darwin', 'x64');
    expect(platform).toEqual({
      platformName: 'macos-x64',
      binaryName: 'pipeline'
    });
  });

  it('should detect Linux x64 platform correctly', async () => {
    const installModule = await import('../scripts/install.js');
    detectPlatform = installModule.detectPlatform;

    const platform = detectPlatform('linux', 'x64');
    expect(platform).toEqual({
      platformName: 'linux-x64',
      binaryName: 'pipeline'
    });
  });

  it('should detect Windows x64 platform correctly', async () => {
    const installModule = await import('../scripts/install.js');
    detectPlatform = installModule.detectPlatform;

    const platform = detectPlatform('win32', 'x64');
    expect(platform).toEqual({
      platformName: 'windows-x64',
      binaryName: 'pipeline.exe'
    });
  });

  it('should throw error for unsupported platform', async () => {
    const installModule = await import('../scripts/install.js');
    detectPlatform = installModule.detectPlatform;

    expect(() => detectPlatform('freebsd', 'x64')).toThrow('Unsupported platform');
  });
});

describe('install.js - Download URL Construction', () => {
  it('should build correct GitHub release URL for latest version', async () => {
    const installModule = await import('../scripts/install.js');
    buildDownloadUrl = installModule.buildDownloadUrl;

    const url = buildDownloadUrl({
      repo: 'Vooster-AI/pipeline-kit',
      version: 'latest',
      platformName: 'macos-arm64'
    });

    expect(url).toBe(
      'https://github.com/Vooster-AI/pipeline-kit/releases/latest/download/pipeline-kit-macos-arm64.tar.gz'
    );
  });

  it('should build correct GitHub release URL for specific version', async () => {
    const installModule = await import('../scripts/install.js');
    buildDownloadUrl = installModule.buildDownloadUrl;

    const url = buildDownloadUrl({
      repo: 'Vooster-AI/pipeline-kit',
      version: 'v0.1.0',
      platformName: 'linux-x64'
    });

    expect(url).toBe(
      'https://github.com/Vooster-AI/pipeline-kit/releases/download/v0.1.0/pipeline-kit-linux-x64.tar.gz'
    );
  });
});

describe('install.js - Binary Download and Extraction (Production Mode)', () => {
  it('should handle download errors gracefully', async () => {
    // Mock HTTP request that fails
    nock('https://github.com')
      .get('/Vooster-AI/pipeline-kit/releases/latest/download/pipeline-kit-macos-arm64.tar.gz')
      .reply(404, 'Not Found');

    const installModule = await import('../scripts/install.js');
    downloadAndExtract = installModule.downloadAndExtract;

    // Use a real temp directory for this test (mock-fs doesn't work well with tar extraction)
    const tempDir = fs.mkdtempSync(path.join(os.tmpdir(), 'test-install-'));

    try {
      await expect(async () => {
        await downloadAndExtract({
          url: 'https://github.com/Vooster-AI/pipeline-kit/releases/latest/download/pipeline-kit-macos-arm64.tar.gz',
          vendorDir: tempDir,
          platformName: 'macos-arm64',
          binaryName: 'pipeline',
          showProgress: false
        });
      }).rejects.toThrow();
    } finally {
      // Clean up
      fs.rmSync(tempDir, { recursive: true, force: true });
    }
  });
});

describe('install.js - Local Binary Copy (Development Mode)', () => {
  it('should copy local binary from Rust build directory', async () => {
    // Use real temp directories (mock-fs doesn't work well with tar and axios)
    const tempRoot = fs.mkdtempSync(path.join(os.tmpdir(), 'test-install-'));
    const sourceDir = path.join(tempRoot, 'pipeline-kit-rs', 'target', 'release');
    const vendorDir = path.join(tempRoot, 'vendor');

    try {
      // Create source directory and fake binary
      fs.mkdirSync(sourceDir, { recursive: true });
      fs.writeFileSync(path.join(sourceDir, 'pipeline'), 'fake-binary-content');

      const installModule = await import('../scripts/install.js');
      copyLocalBinary = installModule.copyLocalBinary;

      await copyLocalBinary({
        sourcePath: path.join(sourceDir, 'pipeline'),
        vendorDir,
        platformName: 'macos-arm64',
        binaryName: 'pipeline'
      });

      // Verify binary was copied
      const destPath = path.join(vendorDir, 'macos-arm64', 'pipeline-kit', 'pipeline');
      expect(fs.existsSync(destPath)).toBe(true);

      // Verify file permissions (should be executable on Unix)
      if (process.platform !== 'win32') {
        const stats = fs.statSync(destPath);
        // Check if user execute bit is set (0o100)
        expect((stats.mode & 0o100) !== 0).toBe(true);
      }
    } finally {
      // Clean up
      fs.rmSync(tempRoot, { recursive: true, force: true });
    }
  });

  it('should create vendor directory if it does not exist', async () => {
    const tempRoot = fs.mkdtempSync(path.join(os.tmpdir(), 'test-install-'));
    const sourceDir = path.join(tempRoot, 'source');
    const vendorDir = path.join(tempRoot, 'vendor');

    try {
      // Create source directory with binary, but NOT vendor directory
      fs.mkdirSync(sourceDir, { recursive: true });
      fs.writeFileSync(path.join(sourceDir, 'pipeline'), 'fake-binary-content');

      const installModule = await import('../scripts/install.js');
      copyLocalBinary = installModule.copyLocalBinary;

      await copyLocalBinary({
        sourcePath: path.join(sourceDir, 'pipeline'),
        vendorDir,
        platformName: 'macos-arm64',
        binaryName: 'pipeline'
      });

      // Verify directory structure was created
      expect(fs.existsSync(path.join(vendorDir, 'macos-arm64', 'pipeline-kit'))).toBe(true);
    } finally {
      // Clean up
      fs.rmSync(tempRoot, { recursive: true, force: true });
    }
  });

  it('should handle missing source binary gracefully', async () => {
    const tempRoot = fs.mkdtempSync(path.join(os.tmpdir(), 'test-install-'));

    try {
      const installModule = await import('../scripts/install.js');
      copyLocalBinary = installModule.copyLocalBinary;

      await expect(async () => {
        await copyLocalBinary({
          sourcePath: path.join(tempRoot, 'nonexistent', 'pipeline'),
          vendorDir: path.join(tempRoot, 'vendor'),
          platformName: 'macos-arm64',
          binaryName: 'pipeline'
        });
      }).rejects.toThrow();
    } finally {
      // Clean up
      fs.rmSync(tempRoot, { recursive: true, force: true });
    }
  });
});

describe('install.js - Integration Test', () => {
  it('should successfully install binary in development mode (acceptance test)', async () => {
    // This is the main acceptance test that validates the entire flow
    // Use real temp directories
    const tempRoot = fs.mkdtempSync(path.join(os.tmpdir(), 'test-install-'));
    const cliRoot = path.join(tempRoot, 'pipeline-kit-cli');
    const rustBuildDir = path.join(tempRoot, 'pipeline-kit-rs', 'target', 'release');

    try {
      // Setup directory structure with source binary
      fs.mkdirSync(rustBuildDir, { recursive: true });
      fs.writeFileSync(path.join(rustBuildDir, 'pipeline'), 'mock-binary-content');
      fs.mkdirSync(cliRoot, { recursive: true });

      // Import the main install function
      const installModule = await import('../scripts/install.js');
      const install = installModule.install;

      // Run installation in development mode
      await install({
        mode: 'development',
        cliRoot,
        platform: 'darwin',
        arch: 'arm64'
      });

      // Verify binary exists in correct location
      const binaryPath = path.join(
        cliRoot,
        'vendor',
        'macos-arm64',
        'pipeline-kit',
        'pipeline'
      );

      expect(fs.existsSync(binaryPath)).toBe(true);

      // Verify binary is executable on Unix
      if (process.platform !== 'win32') {
        const stats = fs.statSync(binaryPath);
        expect((stats.mode & 0o100) !== 0).toBe(true);
      }
    } finally {
      // Clean up
      fs.rmSync(tempRoot, { recursive: true, force: true });
    }
  });
});
