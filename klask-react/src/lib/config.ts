// Configuration utilities with runtime override support

/**
 * Gets the API base URL with runtime configuration support
 * Priority: window.RUNTIME_CONFIG > build-time env > default
 *
 * Returns empty string for relative paths when running behind nginx proxy
 */
export const getApiBaseUrl = (): string => {
  // Check for runtime configuration first (set by Docker entrypoint)
  if (typeof window !== 'undefined' && window.RUNTIME_CONFIG?.VITE_API_BASE_URL !== undefined) {
      return window.RUNTIME_CONFIG.VITE_API_BASE_URL;
  }
  // Fallback to build-time environment variable
  return import.meta.env.VITE_API_BASE_URL || 'http://localhost:3000';
};

/**
 * API base URL for the application
 */
export const API_BASE_URL = getApiBaseUrl();
