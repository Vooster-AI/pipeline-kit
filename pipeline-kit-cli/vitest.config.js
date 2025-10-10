import { defineConfig } from 'vitest/config';

export default defineConfig({
  test: {
    globals: true,
    environment: 'node',
    coverage: {
      provider: 'v8',
      reporter: ['text', 'json', 'html'],
      exclude: [
        'node_modules/',
        'test/',
        'scripts/',
        'vendor/',
        'vitest.config.js',
        'bin/', // Exclude bin from coverage as it's tested via integration tests
      ],
      // Set thresholds for lib/ directory only
      thresholds: {
        lines: 90,
        functions: 80,
        branches: 90,
        statements: 90,
      },
    },
  },
});
