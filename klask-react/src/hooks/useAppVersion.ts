import { useState, useEffect } from 'react';

interface VersionInfo {
  version: string;
  commit?: string;
  timestamp?: string;
}

/**
 * Hook to fetch and cache the Klask application version from the backend
 * Falls back to a default version if the backend is unreachable
 */
export function useAppVersion() {
  const [version, setVersion] = useState<VersionInfo | null>(null);
  const [isLoading, setIsLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    const fetchVersion = async () => {
      try {
        // Fetch version from backend with a short timeout
        const controller = new AbortController();
        const timeout = setTimeout(() => controller.abort(), 5000); // 5 second timeout

        const response = await fetch(`${import.meta.env.VITE_API_URL || 'http://localhost:3000'}/version`, {
          signal: controller.signal,
        });

        clearTimeout(timeout);

        if (response.ok) {
          const data = await response.json();
          setVersion(data);
          setError(null);
        } else {
          throw new Error(`HTTP ${response.status}`);
        }
      } catch (err) {
        // If we can't reach the backend, use fallback
        const errorMsg = err instanceof Error ? err.message : 'Unknown error';
        console.warn(`Failed to fetch version from backend: ${errorMsg}, using fallback`);

        // Fallback to build-time version if available
        setVersion({
          version: import.meta.env.VITE_APP_VERSION || '2.0.0',
          commit: import.meta.env.VITE_APP_COMMIT,
          timestamp: import.meta.env.VITE_APP_BUILD_TIME,
        });
        setError(null);
      } finally {
        setIsLoading(false);
      }
    };

    // Only fetch once on mount
    fetchVersion();
  }, []);

  return {
    version: version?.version || '2.0.0',
    commit: version?.commit,
    timestamp: version?.timestamp,
    isLoading,
    error,
    fullVersion: version
      ? `v${version.version}${version.commit ? ` (${version.commit.substring(0, 8)})` : ''}`
      : 'v2.0.0',
  };
}
