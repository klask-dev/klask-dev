/**
 * Empty stub for refractor/lib/all
 *
 * This file is used as an alias in vite.config.ts to prevent module resolution errors
 * when react-syntax-highlighter/prism-async tries to import refractor/lib/all.
 *
 * Instead of loading all 300+ language definitions, we handle language loading
 * dynamically in OptimizedSyntaxHighlighter.tsx
 *
 * @module empty-refractor
 */

// Export a minimal refractor-like object that won't break prism-async
export default {
  refractor: {
    highlight: () => null,
  },
};

export const refractor = {
  highlight: () => null,
};
