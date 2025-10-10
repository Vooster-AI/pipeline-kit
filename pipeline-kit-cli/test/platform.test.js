/**
 * Platform detection tests.
 * RED phase: These tests define the expected behavior.
 */

import { describe, it, expect } from 'vitest';
import { getTargetTriple, getBinaryName } from '../lib/platform.js';

describe('getTargetTriple', () => {
  describe('Linux platforms', () => {
    it('should return correct triple for linux x64', () => {
      expect(getTargetTriple('linux', 'x64')).toBe('x86_64-unknown-linux-musl');
    });

    it('should return correct triple for linux arm64', () => {
      expect(getTargetTriple('linux', 'arm64')).toBe('aarch64-unknown-linux-musl');
    });

    it('should return correct triple for android x64', () => {
      expect(getTargetTriple('android', 'x64')).toBe('x86_64-unknown-linux-musl');
    });

    it('should return correct triple for android arm64', () => {
      expect(getTargetTriple('android', 'arm64')).toBe('aarch64-unknown-linux-musl');
    });

    it('should return null for unsupported linux arch', () => {
      expect(getTargetTriple('linux', 'ia32')).toBeNull();
      expect(getTargetTriple('linux', 'arm')).toBeNull();
    });
  });

  describe('macOS (darwin) platforms', () => {
    it('should return correct triple for darwin x64', () => {
      expect(getTargetTriple('darwin', 'x64')).toBe('x86_64-apple-darwin');
    });

    it('should return correct triple for darwin arm64', () => {
      expect(getTargetTriple('darwin', 'arm64')).toBe('aarch64-apple-darwin');
    });

    it('should return null for unsupported darwin arch', () => {
      expect(getTargetTriple('darwin', 'ia32')).toBeNull();
    });
  });

  describe('Windows platforms', () => {
    it('should return correct triple for win32 x64', () => {
      expect(getTargetTriple('win32', 'x64')).toBe('x86_64-pc-windows-msvc');
    });

    it('should return correct triple for win32 arm64', () => {
      expect(getTargetTriple('win32', 'arm64')).toBe('aarch64-pc-windows-msvc');
    });

    it('should return null for unsupported win32 arch', () => {
      expect(getTargetTriple('win32', 'ia32')).toBeNull();
    });
  });

  describe('Unsupported platforms', () => {
    it('should return null for freebsd', () => {
      expect(getTargetTriple('freebsd', 'x64')).toBeNull();
    });

    it('should return null for sunos', () => {
      expect(getTargetTriple('sunos', 'x64')).toBeNull();
    });

    it('should return null for unknown platform', () => {
      expect(getTargetTriple('unknown', 'x64')).toBeNull();
    });
  });
});

describe('getBinaryName', () => {
  it('should return pipeline.exe for win32', () => {
    expect(getBinaryName('win32')).toBe('pipeline.exe');
  });

  it('should return pipeline for darwin', () => {
    expect(getBinaryName('darwin')).toBe('pipeline');
  });

  it('should return pipeline for linux', () => {
    expect(getBinaryName('linux')).toBe('pipeline');
  });

  it('should return pipeline for android', () => {
    expect(getBinaryName('android')).toBe('pipeline');
  });

  it('should return pipeline for any non-Windows platform', () => {
    expect(getBinaryName('freebsd')).toBe('pipeline');
    expect(getBinaryName('sunos')).toBe('pipeline');
  });
});
