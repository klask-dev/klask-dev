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
    const url = window.RUNTIME_CONFIG.VITE_API_BASE_URL;
    // Empty string means use relative paths (for nginx proxy)
    return url === '' ? '' : url;
  }
  // Fallback to build-time environment variable
  const envUrl = import.meta.env.VITE_API_BASE_URL;
  // Empty string means use relative paths (for nginx proxy)
  if (envUrl === '') return '';
  return envUrl || 'http://localhost:3000';
};

/**
 * API base URL for the application
 * Empty string = relative paths (/api), used when behind nginx proxy
 */
export const API_BASE_URL = getApiBaseUrl();