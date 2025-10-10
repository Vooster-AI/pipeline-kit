/**
 * Binary path resolution tests.
 * RED phase: These tests verify correct path construction.
 */

import { describe, it, expect } from 'vitest';
import { getVendorBinaryPath, getVendorBinaryPathLegacy, getDevBinaryPath, getPlatformName, platformMap } from '../lib/platform.js';
import path from 'path';

describe('getPlatformName', () => {
  it('should map darwin-x64 to macos-x64', () => {
    expect(getPlatformName('darwin', 'x64')).toBe('macos-x64');
  });

  it('should map darwin-arm64 to macos-arm64', () => {
    expect(getPlatformName('darwin', 'arm64')).toBe('macos-arm64');
  });

  it('should map linux-x64 to linux-x64', () => {
    expect(getPlatformName('linux', 'x64')).toBe('linux-x64');
  });

  it('should map linux-arm64 to linux-arm64', () => {
    expect(getPlatformName('linux', 'arm64')).toBe('linux-arm64');
  });

  it('should map win32-x64 to windows-x64', () => {
    expect(getPlatformName('win32', 'x64')).toBe('windows-x64');
  });

  it('should map win32-arm64 to windows-arm64', () => {
    expect(getPlatformName('win32', 'arm64')).toBe('windows-arm64');
  });

  it('should return null for unsupported platform', () => {
    expect(getPlatformName('freebsd', 'x64')).toBeNull();
    expect(getPlatformName('sunos', 'x64')).toBeNull();
  });
});

describe('getVendorBinaryPath', () => {
  it('should construct correct path for macos-arm64', () => {
    const vendorRoot = '/fake/vendor';
    const platformName = 'macos-arm64';
    const binaryName = 'pipeline';

    const result = getVendorBinaryPath(vendorRoot, platformName, binaryName);

    expect(result).toBe(path.join('/fake/vendor', 'macos-arm64', 'pipeline-kit', 'pipeline'));
  });

  it('should construct correct path for macos-x64', () => {
    const vendorRoot = '/fake/vendor';
    const platformName = 'macos-x64';
    const binaryName = 'pipeline';

    const result = getVendorBinaryPath(vendorRoot, platformName, binaryName);

    expect(result).toBe(path.join('/fake/vendor', 'macos-x64', 'pipeline-kit', 'pipeline'));
  });

  it('should construct correct path for linux-x64', () => {
    const vendorRoot = '/fake/vendor';
    const platformName = 'linux-x64';
    const binaryName = 'pipeline';

    const result = getVendorBinaryPath(vendorRoot, platformName, binaryName);

    expect(result).toBe(path.join('/fake/vendor', 'linux-x64', 'pipeline-kit', 'pipeline'));
  });

  it('should construct correct path for windows-x64 with .exe extension', () => {
    const vendorRoot = 'C:\\fake\\vendor';
    const platformName = 'windows-x64';
    const binaryName = 'pipeline.exe';

    const result = getVendorBinaryPath(vendorRoot, platformName, binaryName);

    expect(result).toBe(path.join('C:\\fake\\vendor', 'windows-x64', 'pipeline-kit', 'pipeline.exe'));
  });

  it('should handle relative vendor paths', () => {
    const vendorRoot = 'vendor';
    const platformName = 'macos-arm64';
    const binaryName = 'pipeline';

    const result = getVendorBinaryPath(vendorRoot, platformName, binaryName);

    expect(result).toBe(path.join('vendor', 'macos-arm64', 'pipeline-kit', 'pipeline'));
  });
});

describe('getDevBinaryPath', () => {
  it('should construct correct dev path for Unix binary', () => {
    const cliDir = '/fake/pipeline-kit-cli';
    const binaryName = 'pipeline';

    const result = getDevBinaryPath(cliDir, binaryName);

    expect(result).toBe(path.join('/fake/pipeline-kit-cli', '..', 'pipeline-kit-rs', 'target', 'release', 'pipeline'));
  });

  it('should construct correct dev path for Windows binary', () => {
    const cliDir = 'C:\\fake\\pipeline-kit-cli';
    const binaryName = 'pipeline.exe';

    const result = getDevBinaryPath(cliDir, binaryName);

    expect(result).toBe(path.join('C:\\fake\\pipeline-kit-cli', '..', 'pipeline-kit-rs', 'target', 'release', 'pipeline.exe'));
  });

  it('should handle relative cli directory paths', () => {
    const cliDir = 'pipeline-kit-cli';
    const binaryName = 'pipeline';

    const result = getDevBinaryPath(cliDir, binaryName);

    expect(result).toBe(path.join('pipeline-kit-cli', '..', 'pipeline-kit-rs', 'target', 'release', 'pipeline'));
  });

  it('should correctly navigate up one directory level', () => {
    const cliDir = '/project/pipeline-kit-cli';
    const binaryName = 'pipeline';

    const result = getDevBinaryPath(cliDir, binaryName);
    const normalized = path.normalize(result);

    // Should resolve to /project/pipeline-kit-rs/target/release/pipeline
    expect(normalized).toContain('pipeline-kit-rs');
    expect(normalized).toContain('target');
    expect(normalized).toContain('release');
  });
});

describe('Path format validation', () => {
  it('vendor paths should use correct path separators', () => {
    const vendorRoot = '/vendor';
    const result = getVendorBinaryPath(vendorRoot, 'macos-arm64', 'pipeline');

    // Should use platform-appropriate separators
    expect(result).toContain(path.sep);
  });

  it('dev paths should use correct path separators', () => {
    const cliDir = '/cli';
    const result = getDevBinaryPath(cliDir, 'pipeline');

    // Should use platform-appropriate separators
    expect(result).toContain(path.sep);
  });

  it('paths should not have double separators', () => {
    const vendorRoot = '/vendor/';
    const result = getVendorBinaryPath(vendorRoot, 'macos-arm64', 'pipeline');

    // path.join should handle this correctly
    expect(result).not.toMatch(/\/\//);
    expect(result).not.toMatch(/\\\\/);
  });
});

describe('platformMap export', () => {
  it('should export platformMap with all expected mappings', () => {
    expect(platformMap).toBeDefined();
    expect(platformMap['darwin-x64']).toBe('macos-x64');
    expect(platformMap['darwin-arm64']).toBe('macos-arm64');
    expect(platformMap['linux-x64']).toBe('linux-x64');
    expect(platformMap['linux-arm64']).toBe('linux-arm64');
    expect(platformMap['win32-x64']).toBe('windows-x64');
    expect(platformMap['win32-arm64']).toBe('windows-arm64');
    expect(platformMap['android-arm64']).toBe('linux-arm64');
  });
});

describe('getVendorBinaryPathLegacy (backward compatibility)', () => {
  it('should construct correct path using target triple', () => {
    const vendorRoot = '/fake/vendor';
    const targetTriple = 'x86_64-apple-darwin';
    const binaryName = 'pipeline';

    const result = getVendorBinaryPathLegacy(vendorRoot, targetTriple, binaryName);

    expect(result).toBe(path.join('/fake/vendor', 'x86_64-apple-darwin', 'pipeline-kit', 'pipeline'));
  });

  it('should work with different target triples', () => {
    const vendorRoot = '/vendor';

    const linux = getVendorBinaryPathLegacy(vendorRoot, 'x86_64-unknown-linux-musl', 'pipeline');
    expect(linux).toContain('x86_64-unknown-linux-musl');

    const windows = getVendorBinaryPathLegacy(vendorRoot, 'x86_64-pc-windows-msvc', 'pipeline.exe');
    expect(windows).toContain('x86_64-pc-windows-msvc');
  });
});
