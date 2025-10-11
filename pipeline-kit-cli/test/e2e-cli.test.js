import { describe, it, expect } from 'vitest';
import { spawn } from 'child_process';
import fs from 'fs';
import os from 'os';
import path from 'path';
import { fileURLToPath } from 'url';

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);

function mkTmpProject() {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), 'pk-e2e-'));
  const pkDir = path.join(root, '.pipeline-kit');
  const agentsDir = path.join(pkDir, 'agents');
  const pipelinesDir = path.join(pkDir, 'pipelines');
  fs.mkdirSync(agentsDir, { recursive: true });
  fs.mkdirSync(pipelinesDir, { recursive: true });

  // Minimal agent using MockAgent via model: test-model
  const agentMd = `---\nname: test-agent\ndescription: Test agent\nmodel: test-model\ncolor: blue\n---\n\nBe a helpful assistant.`;
  fs.writeFileSync(path.join(agentsDir, 'test.md'), agentMd, 'utf8');

  // Minimal pipeline that runs the test-agent
  const pipelineYaml = `name: simple-task\nmaster:\n  model: test-model\n  system-prompt: "Orchestrate simple task"\n  process:\n    - test-agent\nsub-agents:\n  - test-agent\n`;
  fs.writeFileSync(path.join(pipelinesDir, 'simple.yaml'), pipelineYaml, 'utf8');

  return root;
}

function runCli(args, options) {
  const cliPath = path.join(__dirname, '..', 'bin', 'pipeline-kit.js');
  return new Promise((resolve, reject) => {
    const child = spawn('node', [cliPath, ...args], {
      ...options,
      stdio: ['ignore', 'pipe', 'pipe'],
      env: { ...process.env, PIPELINE_KIT_PREFER_DEV: '1', ...(options?.env || {}) },
    });

    let stdout = '';
    let stderr = '';
    child.stdout.on('data', (d) => (stdout += d.toString()))
    child.stderr.on('data', (d) => (stderr += d.toString()))
    child.on('error', reject);
    child.on('exit', (code, signal) => resolve({ code, signal, stdout, stderr }));
  });
}

describe('E2E: CLI run --no-tui JSON Lines', () => {
  it('prints ProcessStarted, ProcessLogChunk, ProcessCompleted in order', async () => {
    const projectRoot = mkTmpProject();

    const res = await runCli(['run', 'simple-task', '--no-tui'], { cwd: projectRoot });

    // CLI should exit successfully
    expect(res.code).toBe(0);
    expect(res.stderr).toBe('');

    const lines = res.stdout
      .split('\n')
      .map((l) => l.trim())
      .filter((l) => l.length > 0);

    expect(lines.length).toBeGreaterThan(0);

    const events = [];
    for (const line of lines) {
      try {
        const obj = JSON.parse(line);
        if (obj && obj.type) {
          events.push(obj.type);
        }
      } catch (e) {
        // ignore non-JSON lines
      }
    }

    // Validate sequence ordering
    const startedIdx = events.indexOf('processStarted');
    const logIdx = events.indexOf('processLogChunk');
    const completedIdx = events.indexOf('processCompleted');

    expect(startedIdx).toBeGreaterThanOrEqual(0);
    expect(logIdx).toBeGreaterThan(startedIdx);
    expect(completedIdx).toBeGreaterThan(logIdx);
  }, 20000);
});
