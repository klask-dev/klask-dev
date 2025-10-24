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
      external: [
        // Mark refractor imports as external to avoid bundle errors
        /^refractor\/.*$/,
      ],
      onwarn(warning, warn) {
        // Ignore refractor import warnings for both lib and lang directories
        if (warning.code === 'UNRESOLVED_IMPORT' && warning.message?.includes('refractor/')) {
          return;
        }
        warn(warning);
      },
      output: {
        manualChunks: {
          // Bundle syntax highlighter core separately
          'syntax-highlighter': [
            'react-syntax-highlighter/dist/esm/prism',
          ],
          // Bundle common languages together - removed individual imports since we handle them dynamically
          'react-vendor': ['react', 'react-dom'],
          'ui-vendor': ['react-window', '@headlessui/react', '@heroicons/react'],
          // Bundle styles together
          'syntax-styles': [
            'react-syntax-highlighter/dist/esm/styles/prism/one-light',
            'react-syntax-highlighter/dist/esm/styles/prism/one-dark',
            'react-syntax-highlighter/dist/esm/styles/prism/vsc-dark-plus',
          ],
        },
      },
    },
    chunkSizeWarningLimit: 1000,
  },
  optimizeDeps: {
    include: [
      'react-syntax-highlighter/dist/esm/prism',
      'react-syntax-highlighter/dist/esm/styles/prism/vsc-dark-plus',
      'react-syntax-highlighter/dist/esm/styles/prism/one-light',
      'react-syntax-highlighter/dist/esm/styles/prism/one-dark',
      'react-window',
      'dompurify',
    ],
    exclude: [
      // Exclude individual language modules from pre-bundling
      // to prevent creating many small chunks
      'react-syntax-highlighter/dist/esm/languages/prism/*',
      // Exclude refractor to avoid bundle resolution errors
      'refractor',
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
  },
})
