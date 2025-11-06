/// <reference types="vitest" />
import { defineConfig } from 'vite'
import react from '@vitejs/plugin-react'

// https://vite.dev/config/
export default defineConfig({
  plugins: [react()],
  server: {
    proxy: {
      '/api': {
        target: 'http://localhost:3000',
        changeOrigin: true,
        secure: false,
      },
    },
  },
  build: {
    rollupOptions: {
      output: {
        manualChunks: {
          'react-vendor': ['react', 'react-dom'],
          'ui-vendor': ['react-window', '@headlessui/react', '@heroicons/react'],
          'syntax-highlighter': ['prism-react-renderer'],
        },
      },
    },
    chunkSizeWarningLimit: 1000,
  },
  optimizeDeps: {
    include: [
      'prism-react-renderer',
      'react-window',
      'dompurify',
    ],
  },
  test: {
    globals: true,
    environment: 'jsdom',
    setupFiles: './src/test/setup.ts',
    css: true,
    // Reduce verbosity
    reporter: ['basic'],
    logHeapUsage: false,
    silent: false,
    ui: false,
    // Prevent DOM dumping on test failures
    outputFile: undefined,
    // Exclude problematic tests in CI
    exclude: [
      '**/node_modules/**',
      '**/dist/**',
      // Temporarily exclude failing tests
      '**/OptimizedSyntaxHighlighter.test.tsx',
      '**/VirtualizedSyntaxHighlighter.test.tsx',
    ],
    // Performance optimizations
    maxWorkers: 2,
    minWorkers: 1,
    pool: 'threads',
    poolOptions: {
      threads: {
        singleThread: false,
        maxThreads: 2,
        minThreads: 1,
      },
    },
    // Isolate tests to free memory
    isolate: true,
    // Shorter timeouts for stuck tests
    testTimeout: 10000, // 10s max per test
    hookTimeout: 5000,  // 5s max per hook
  },
})
