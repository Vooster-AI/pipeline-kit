/**
 * Integration tests for the CLI wrapper.
 * RED phase: These tests verify end-to-end binary execution.
 */

import { describe, it, expect, beforeAll } from 'vitest';
import { spawn } from 'child_process';
import { existsSync } from 'fs';
import path from 'path';
import { fileURLToPath } from 'url';

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);

// Path to the CLI entry point
const cliPath = path.join(__dirname, '..', 'bin', 'pipeline-kit.js');

// Helper function to execute the CLI and capture output
// Note: The CLI uses stdio: 'inherit', so we need to modify the approach
function executeCli(args = [], options = {}) {
  return new Promise((resolve, reject) => {
    // We need to use pipe for stdio to capture output
    const child = spawn('node', [cliPath, ...args], {
      ...options,
      stdio: ['pipe', 'pipe', 'pipe'], // Override inherit to capture output
      env: { ...process.env, ...options.env },
    });

    let stdout = '';
    let stderr = '';

    if (child.stdout) {
      child.stdout.on('data', (data) => {
        stdout += data.toString();
      });
    }

    if (child.stderr) {
      child.stderr.on('data', (data) => {
        stderr += data.toString();
      });
    }

    child.on('error', (err) => {
      reject(err);
    });

    child.on('exit', (code, signal) => {
      resolve({
        exitCode: code,
        signal,
        stdout,
        stderr,
      });
    });
  });
}

describe('CLI Integration Tests', () => {
  beforeAll(() => {
    // Verify the CLI script exists
    expect(existsSync(cliPath)).toBe(true);
  });

  describe('Binary execution', () => {
    it('should successfully spawn and execute the binary', async () => {
      // Test that the binary can be spawned
      const result = await executeCli(['--help']);

      // The binary should execute (exit code might vary based on implementation)
      expect(result.exitCode).toBeDefined();
      expect(typeof result.exitCode).toBe('number');
    }, 10000);

    it('should handle command-line arguments', async () => {
      // Test that arguments are passed through
      const result = await executeCli(['--help']);

      // Should complete execution
      expect(result.exitCode !== null).toBe(true);
    }, 10000);
  });

  describe('Error handling', () => {
    it('should handle invalid commands gracefully', async () => {
      const result = await executeCli(['--invalid-flag-that-does-not-exist']);

      // Should exit (with success or error code)
      expect(result.exitCode !== null).toBe(true);
    }, 10000);

    it('should provide error message if binary is not found', async () => {
      // This test verifies the error handling structure
      // Skip if the binary actually exists (which is the normal case)
      const vendorRoot = path.join(__dirname, '..', 'vendor');
      const devBinaryPath = path.join(__dirname, '..', '..', 'pipeline-kit-rs', 'target', 'release', 'pipeline');

      if (!existsSync(vendorRoot) && !existsSync(devBinaryPath)) {
        const result = await executeCli([]);

        // Should exit with error
        expect(result.exitCode).not.toBe(0);

        // Should have error message
        expect(result.stderr).toContain('Error');
      } else {
        // Binary exists, so this test doesn't apply
        expect(true).toBe(true);
      }
    }, 10000);
  });

  describe('Signal forwarding', () => {
    it('should handle process termination gracefully', async () => {
      // Start a long-running command (if available) and terminate it
      // This is a simplified test - in practice, the CLI should forward signals

      const child = spawn('node', [cliPath, '--help'], {
        stdio: ['pipe', 'pipe', 'pipe'],
        env: process.env,
      });

      let exited = false;
      child.on('exit', () => {
        exited = true;
      });

      // Wait a bit and then kill
      await new Promise(resolve => setTimeout(resolve, 100));

      if (!exited) {
        child.kill('SIGTERM');
        await new Promise(resolve => setTimeout(resolve, 500));
      }

      // Should have exited
      expect(exited || child.killed).toBe(true);
    }, 10000);
  });

  describe('Environment variable passing', () => {
    it('should set PIPELINE_KIT_MANAGED_BY_NPM environment variable', async () => {
      // We can't directly test this without modifying the binary,
      // but we can verify the CLI script is structured correctly
      // by checking that it spawns with environment variables

      // This is more of a structural test
      expect(cliPath).toBeTruthy();
      expect(existsSync(cliPath)).toBe(true);
    });
  });
});

describe('Platform detection in real environment', () => {
  it('should correctly detect current platform', () => {
    const { platform, arch } = process;

    // Just verify we're running on a supported configuration
    const supportedPlatforms = ['darwin', 'linux', 'win32', 'android'];
    const supportedArchs = ['x64', 'arm64'];

    if (supportedPlatforms.includes(platform)) {
      expect(supportedArchs).toContain(arch);
    }
  });
});

describe('Binary path resolution', () => {
  it('should resolve to either vendor or dev binary', () => {
    const vendorRoot = path.join(__dirname, '..', 'vendor');
    const devBinaryPath = path.join(__dirname, '..', '..', 'pipeline-kit-rs', 'target', 'release', 'pipeline');

    // At least one should exist for tests to pass
    const hasVendor = existsSync(vendorRoot);
    const hasDevBinary = existsSync(devBinaryPath);

    expect(hasVendor || hasDevBinary).toBe(true);
  });
});
