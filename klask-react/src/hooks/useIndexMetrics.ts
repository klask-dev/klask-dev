import { useState, useCallback, useEffect } from 'react';
import { useAllIndexMetrics } from '../api/indexMetrics';

export type RefreshInterval = 'off' | '5s' | '10s' | '30s' | '60s';

interface UseIndexMetricsOptions {
  enableAutoRefresh?: boolean;
  defaultInterval?: RefreshInterval;
}

export interface UseIndexMetricsReturn {
  stats: ReturnType<typeof useAllIndexMetrics>['data']['stats'];
  health: ReturnType<typeof useAllIndexMetrics>['data']['health'];
  tuning: ReturnType<typeof useAllIndexMetrics>['data']['tuning'];
  isLoading: boolean;
  error: Error | null;
  autoRefreshEnabled: boolean;
  autoRefreshInterval: RefreshInterval;
  setAutoRefreshInterval: (interval: RefreshInterval) => void;
  lastUpdateTime: Date | null;
  nextRefreshTime: Date | null;
  manualRefresh: () => Promise<void>;
}

/**
 * Custom hook combining all index metrics with auto-refresh management
 * Provides a unified interface for accessing index metrics with auto-refresh capabilities
 */
export function useIndexMetrics(options: UseIndexMetricsOptions = {}): UseIndexMetricsReturn {
  const { enableAutoRefresh = false, defaultInterval = 'off' } = options;

  const [autoRefreshInterval, setAutoRefreshInterval] = useState<RefreshInterval>(defaultInterval);
  const [lastUpdateTime, setLastUpdateTime] = useState<Date | null>(null);
  const [nextRefreshTime, setNextRefreshTime] = useState<Date | null>(null);

  // Convert interval string to milliseconds
  const getRefreshMs = useCallback((interval: RefreshInterval): number | false => {
    switch (interval) {
      case '5s':
        return 5000;
      case '10s':
        return 10000;
      case '30s':
        return 30000;
      case '60s':
        return 60000;
      case 'off':
      default:
        return false;
    }
  }, []);

  // Get current auto-refresh interval in milliseconds
  const refreshMs = getRefreshMs(autoRefreshInterval);

  // Fetch all metrics with auto-refresh
  const metricsQuery = useAllIndexMetrics(refreshMs);

  // Update last update time when data changes
  useEffect(() => {
    if (metricsQuery.data.stats || metricsQuery.data.health) {
      setLastUpdateTime(new Date());
    }
  }, [metricsQuery.data]);

  // Calculate next refresh time
  useEffect(() => {
    if (refreshMs === false) {
      setNextRefreshTime(null);
      return;
    }

    // Initialize the next refresh time
    setNextRefreshTime(new Date(Date.now() + refreshMs));

    // Only update when the seconds change to avoid unnecessary re-renders
    let lastSecond = Math.floor(Date.now() / 1000);

    const interval = setInterval(() => {
      const currentSecond = Math.floor(Date.now() / 1000);
      // Only update if a second has actually passed
      if (currentSecond !== lastSecond) {
        lastSecond = currentSecond;
        setNextRefreshTime(new Date(Date.now() + refreshMs));
      }
    }, 100); // Check 10 times per second, but only update on second changes

    return () => clearInterval(interval);
  }, [refreshMs]);

  const manualRefresh = useCallback(async () => {
    await metricsQuery.refetch();
    setLastUpdateTime(new Date());
  }, [metricsQuery]);

  const autoRefreshEnabled = autoRefreshInterval !== 'off' && enableAutoRefresh;

  return {
    stats: metricsQuery.data.stats,
    health: metricsQuery.data.health,
    tuning: metricsQuery.data.tuning,
    isLoading: metricsQuery.isLoading,
    error: metricsQuery.error,
    autoRefreshEnabled,
    autoRefreshInterval,
    setAutoRefreshInterval,
    lastUpdateTime,
    nextRefreshTime,
    manualRefresh,
  };
}
