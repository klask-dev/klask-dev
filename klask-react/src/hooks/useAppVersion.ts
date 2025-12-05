import { useState, useEffect } from 'react';

interface VersionInfo {
  version: string;
  commit?: string;
  timestamp?: string;
}

/**
 * Formats version string for display
 * - Release versions (e.g., "2.0.0") display as "v2.0.0"
 * - Branch versions (e.g., "feature/new-ui") display with commit hash and "dev" badge
 * - Includes short commit hash when available
 */
function formatVersionDisplay(versionInfo: VersionInfo | null): string {
  if (!versionInfo) {
    return '2.0.0-dev';
  }

  const { version, commit } = versionInfo;

  // Check if this is a semantic version (release) or a branch/dev version
  const isRelease = /^\d+\.\d+\.\d+/.test(version);

  if (isRelease) {
    // Release version: v2.0.0 or v2.0.0 (a1b2c3d4)
    return commit ? `${version} (${commit.substring(0, 8)})` : version;
  } else {
    // Dev/branch version: feature/new-ui (a1b2c3d4)
    return commit ? `${version} (${commit.substring(0, 8)})` : `${version}-dev`;
  }
}

/**
 * Hook to fetch and cache the Klask application version from the backend
 * Falls back to build-time environment variables if the backend is unreachable
 *
 * Display behavior:
 * - Release versions (2.0.0, 2.1.0): Show clean version + commit hash
 * - Dev branches (upgrade-2-2-1): Show branch name + commit hash
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
        // If we can't reach the backend, use build-time environment variables
        const errorMsg = err instanceof Error ? err.message : 'Unknown error';
        console.warn(`Failed to fetch version from backend: ${errorMsg}, using build-time version`);

        // Fallback to build-time version if available
        // This allows the version to still display in development or if backend is down
        const buildVersion = import.meta.env.VITE_APP_VERSION || 'development';
        setVersion({
          version: buildVersion,
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

  const displayVersion = formatVersionDisplay(version);
  const isRelease = version ? /^\d+\.\d+\.\d+/.test(version.version) : false;

  return {
    version: version?.version || '2.0.0-dev',
    commit: version?.commit,
    timestamp: version?.timestamp,
    isLoading,
    error,
    isRelease,
    fullVersion: displayVersion,
  };
}
