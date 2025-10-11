#!/usr/bin/env node
// Clean vendor directory before packaging/publishing to avoid bundling
// platform-specific binaries into the npm tarball.
import fs from 'fs';
import path from 'path';
import { fileURLToPath } from 'url';

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);
const cliRoot = path.resolve(__dirname, '..');
const vendorDir = path.join(cliRoot, 'vendor');

function rimraf(target) {
  if (!fs.existsSync(target)) return;
  const stat = fs.statSync(target);
  if (stat.isDirectory()) {
    for (const entry of fs.readdirSync(target)) {
      rimraf(path.join(target, entry));
    }
    fs.rmdirSync(target);
  } else {
    fs.unlinkSync(target);
  }
}

try {
  if (fs.existsSync(vendorDir)) {
    for (const entry of fs.readdirSync(vendorDir)) {
      rimraf(path.join(vendorDir, entry));
    }
  } else {
    fs.mkdirSync(vendorDir, { recursive: true });
  }
  // leave vendor/ as empty directory; npm will include the folder without contents
  console.log('vendor/ directory cleaned for packaging.');
} catch (err) {
  console.error('Failed to clean vendor directory:', err);
  process.exit(1);
}

