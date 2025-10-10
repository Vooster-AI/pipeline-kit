#!/usr/bin/env node
// Unified entry point for the Pipeline Kit CLI.

import { spawn } from "node:child_process";
import { existsSync } from "fs";
import path from "path";
import { fileURLToPath } from "url";
import { getPlatformName, getBinaryName, getVendorBinaryPath, getDevBinaryPath } from "../lib/platform.js";

// __dirname equivalent in ESM
const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);

const { platform, arch } = process;

// Get platform name using the platform library
const platformName = getPlatformName(platform, arch);

if (!platformName) {
  // Generate list of supported platforms for error message
  const { platformMap } = await import("../lib/platform.js");
  const supportedPlatforms = Object.keys(platformMap)
    .filter(key => !key.startsWith('android')) // Don't list android explicitly
    .map(key => `  - ${key}`)
    .join('\n');

  throw new Error(
    `Unsupported platform: ${platform} (${arch})\n\n` +
    `Supported platforms:\n${supportedPlatforms}`
  );
}

// Construct paths using the platform library
const vendorRoot = path.join(__dirname, "..", "vendor");
const binaryName = getBinaryName(platform);
const binaryPath = getVendorBinaryPath(vendorRoot, platformName, binaryName);

// Development mode fallback: if vendor binary doesn't exist, try local build
let finalBinaryPath = binaryPath;
if (!existsSync(binaryPath)) {
  const cliDir = path.join(__dirname, "..");
  const devBinaryPath = getDevBinaryPath(cliDir, binaryName);

  if (existsSync(devBinaryPath)) {
    finalBinaryPath = devBinaryPath;
  } else {
    console.error(
      `Error: Pipeline Kit binary not found.\n` +
      `  Expected at: ${binaryPath}\n` +
      `  Or (dev mode): ${devBinaryPath}\n\n` +
      `Please ensure the binary is installed correctly.\n` +
      `For development, build the Rust binary first:\n` +
      `  cd pipeline-kit-rs && cargo build --release\n`
    );
    process.exit(1);
  }
}

// Use an asynchronous spawn instead of spawnSync so that Node is able to
// respond to signals (e.g. Ctrl-C / SIGINT) while the native binary is
// executing. This allows us to forward those signals to the child process
// and guarantees that when either the child terminates or the parent
// receives a fatal signal, both processes exit in a predictable manner.

const child = spawn(finalBinaryPath, process.argv.slice(2), {
  stdio: "inherit",
  env: { ...process.env, PIPELINE_KIT_MANAGED_BY_NPM: "1" },
});

child.on("error", (err) => {
  // Typically triggered when the binary is missing or not executable.
  // Re-throwing here will terminate the parent with a non-zero exit code
  // while still printing a helpful stack trace.
  console.error(err);
  process.exit(1);
});

// Forward common termination signals to the child so that it shuts down
// gracefully. In the handler we temporarily disable the default behavior of
// exiting immediately; once the child has been signaled we simply wait for
// its exit event which will in turn terminate the parent (see below).
const forwardSignal = (signal) => {
  if (child.killed) {
    return;
  }
  try {
    child.kill(signal);
  } catch {
    /* ignore */
  }
};

["SIGINT", "SIGTERM", "SIGHUP"].forEach((sig) => {
  process.on(sig, () => forwardSignal(sig));
});

// When the child exits, mirror its termination reason in the parent so that
// shell scripts and other tooling observe the correct exit status.
// Wrap the lifetime of the child process in a Promise so that we can await
// its termination in a structured way. The Promise resolves with an object
// describing how the child exited: either via exit code or due to a signal.
const childResult = await new Promise((resolve) => {
  child.on("exit", (code, signal) => {
    if (signal) {
      resolve({ type: "signal", signal });
    } else {
      resolve({ type: "code", exitCode: code ?? 1 });
    }
  });
});

if (childResult.type === "signal") {
  // Re-emit the same signal so that the parent terminates with the expected
  // semantics (this also sets the correct exit code of 128 + n).
  process.kill(process.pid, childResult.signal);
} else {
  process.exit(childResult.exitCode);
}
