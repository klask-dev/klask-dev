import { renderHook, waitFor, act } from '@testing-library/react';
import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { useIndexMetrics, type RefreshInterval } from '../useIndexMetrics';
import * as indexMetricsApi from '../../api/indexMetrics';

// Mock the API module
vi.mock('../../api/indexMetrics');

const createWrapper = () => {
  const queryClient = new QueryClient({
    defaultOptions: {
      queries: { retry: false },
      mutations: { retry: false },
    },
  });
  const Wrapper = ({ children }: { children: React.ReactNode }) => {
    return (
      <QueryClientProvider client={queryClient}>
        {children}
      </QueryClientProvider>
    );
  };
  return Wrapper;
};

const mockStats = {
  total_documents: 1000,
  total_size_mb: 250.0,
  segment_count: 10,
  segments: { total_segments: 10, total_docs: 1000, fragmentation_ratio: 0.3, segments: [] },
  cache: { hits: 500, misses: 100, hit_ratio: 0.83 },
  file_types: [],
  repositories: [],
  avg_query_time_ms: 25,
};

const mockHealth = {
  status: 'Healthy',
  overall_score: 85,
  last_check: new Date().toISOString(),
  check_duration_ms: 45,
  warnings: [],
  recommendations: [],
};

const mockTuning = {
  settings: [],
  recommendations: [],
  last_optimized: null,
};

describe('useIndexMetrics Hook', () => {
  beforeEach(() => {
    vi.clearAllMocks();
    vi.useFakeTimers();
  });

  afterEach(() => {
    vi.useRealTimers();
  });

  describe('Initialization', () => {
    it('should initialize with default options', async () => {
      vi.spyOn(indexMetricsApi, 'useAllIndexMetrics').mockReturnValue({
        data: { stats: mockStats, health: mockHealth, tuning: mockTuning },
        isLoading: false,
        error: null,
        refetch: vi.fn(),
      } as any);

      const { result } = renderHook(() => useIndexMetrics(), {
        wrapper: createWrapper(),
      });

      expect(result.current.autoRefreshInterval).toBe('off');
      expect(result.current.autoRefreshEnabled).toBe(false);
      expect(result.current.stats).toEqual(mockStats);
    });

    it('should initialize with custom default interval', async () => {
      vi.spyOn(indexMetricsApi, 'useAllIndexMetrics').mockReturnValue({
        data: { stats: mockStats, health: mockHealth, tuning: mockTuning },
        isLoading: false,
        error: null,
        refetch: vi.fn(),
      } as any);

      const { result } = renderHook(() => useIndexMetrics({ defaultInterval: '10s' }), {
        wrapper: createWrapper(),
      });

      expect(result.current.autoRefreshInterval).toBe('10s');
    });
  });

  describe('Auto-Refresh Intervals', () => {
    it('should convert 5s interval to milliseconds', async () => {
      const mockRefetch = vi.fn();
      vi.spyOn(indexMetricsApi, 'useAllIndexMetrics').mockReturnValue({
        data: { stats: mockStats, health: mockHealth, tuning: mockTuning },
        isLoading: false,
        error: null,
        refetch: mockRefetch,
      } as any);

      const { result } = renderHook(() => useIndexMetrics({ enableAutoRefresh: true, defaultInterval: '5s' }), {
        wrapper: createWrapper(),
      });

      expect(result.current.autoRefreshInterval).toBe('5s');
      // API should be called with 5000ms
      expect(indexMetricsApi.useAllIndexMetrics).toHaveBeenCalledWith(5000);
    });

    it('should convert 10s interval to milliseconds', async () => {
      vi.spyOn(indexMetricsApi, 'useAllIndexMetrics').mockReturnValue({
        data: { stats: mockStats, health: mockHealth, tuning: mockTuning },
        isLoading: false,
        error: null,
        refetch: vi.fn(),
      } as any);

      const { result } = renderHook(() => useIndexMetrics({ enableAutoRefresh: true, defaultInterval: '10s' }), {
        wrapper: createWrapper(),
      });

      expect(indexMetricsApi.useAllIndexMetrics).toHaveBeenCalledWith(10000);
    });

    it('should convert 30s interval to milliseconds', async () => {
      vi.spyOn(indexMetricsApi, 'useAllIndexMetrics').mockReturnValue({
        data: { stats: mockStats, health: mockHealth, tuning: mockTuning },
        isLoading: false,
        error: null,
        refetch: vi.fn(),
      } as any);

      const { result } = renderHook(() => useIndexMetrics({ enableAutoRefresh: true, defaultInterval: '30s' }), {
        wrapper: createWrapper(),
      });

      expect(indexMetricsApi.useAllIndexMetrics).toHaveBeenCalledWith(30000);
    });

    it('should convert 60s interval to milliseconds', async () => {
      vi.spyOn(indexMetricsApi, 'useAllIndexMetrics').mockReturnValue({
        data: { stats: mockStats, health: mockHealth, tuning: mockTuning },
        isLoading: false,
        error: null,
        refetch: vi.fn(),
      } as any);

      const { result } = renderHook(() => useIndexMetrics({ enableAutoRefresh: true, defaultInterval: '60s' }), {
        wrapper: createWrapper(),
      });

      expect(indexMetricsApi.useAllIndexMetrics).toHaveBeenCalledWith(60000);
    });

    it('should disable refresh when interval is off', async () => {
      vi.spyOn(indexMetricsApi, 'useAllIndexMetrics').mockReturnValue({
        data: { stats: mockStats, health: mockHealth, tuning: mockTuning },
        isLoading: false,
        error: null,
        refetch: vi.fn(),
      } as any);

      const { result } = renderHook(() => useIndexMetrics({ defaultInterval: 'off' }), {
        wrapper: createWrapper(),
      });

      expect(indexMetricsApi.useAllIndexMetrics).toHaveBeenCalledWith(false);
    });
  });

  describe('Data Updates', () => {
    it('should update lastUpdateTime when data changes', async () => {
      const mockRefetch = vi.fn().mockResolvedValue(undefined);
      vi.spyOn(indexMetricsApi, 'useAllIndexMetrics').mockReturnValue({
        data: { stats: mockStats, health: mockHealth, tuning: mockTuning },
        isLoading: false,
        error: null,
        refetch: mockRefetch,
      } as any);

      const { result } = renderHook(() => useIndexMetrics(), {
        wrapper: createWrapper(),
      });

      await waitFor(() => {
        expect(result.current.lastUpdateTime).not.toBeNull();
      });
    });

    it('should track loading state', async () => {
      vi.spyOn(indexMetricsApi, 'useAllIndexMetrics').mockReturnValue({
        data: { stats: null, health: null, tuning: null },
        isLoading: true,
        error: null,
        refetch: vi.fn(),
      } as any);

      const { result } = renderHook(() => useIndexMetrics(), {
        wrapper: createWrapper(),
      });

      expect(result.current.isLoading).toBe(true);
    });

    it('should track error state', async () => {
      const error = new Error('Test error');
      vi.spyOn(indexMetricsApi, 'useAllIndexMetrics').mockReturnValue({
        data: { stats: null, health: null, tuning: null },
        isLoading: false,
        error,
        refetch: vi.fn(),
      } as any);

      const { result } = renderHook(() => useIndexMetrics(), {
        wrapper: createWrapper(),
      });

      expect(result.current.error).toEqual(error);
    });
  });

  describe('Next Refresh Time Calculation', () => {
    it('should calculate next refresh time with 5s interval', async () => {
      vi.spyOn(indexMetricsApi, 'useAllIndexMetrics').mockReturnValue({
        data: { stats: mockStats, health: mockHealth, tuning: mockTuning },
        isLoading: false,
        error: null,
        refetch: vi.fn(),
      } as any);

      const { result } = renderHook(() => useIndexMetrics({ enableAutoRefresh: true, defaultInterval: '5s' }), {
        wrapper: createWrapper(),
      });

      await waitFor(() => {
        expect(result.current.nextRefreshTime).not.toBeNull();
      });

      // Should be approximately 5 seconds from now
      const now = Date.now();
      const nextRefresh = result.current.nextRefreshTime?.getTime() ?? 0;
      const diff = nextRefresh - now;
      expect(diff).toBeGreaterThan(4900);
      expect(diff).toBeLessThan(5100);
    });

    it('should clear next refresh time when refresh is off', async () => {
      vi.spyOn(indexMetricsApi, 'useAllIndexMetrics').mockReturnValue({
        data: { stats: mockStats, health: mockHealth, tuning: mockTuning },
        isLoading: false,
        error: null,
        refetch: vi.fn(),
      } as any);

      const { result } = renderHook(() => useIndexMetrics({ defaultInterval: 'off' }), {
        wrapper: createWrapper(),
      });

      await waitFor(() => {
        expect(result.current.nextRefreshTime).toBeNull();
      });
    });

    it('should update next refresh time every second', async () => {
      vi.spyOn(indexMetricsApi, 'useAllIndexMetrics').mockReturnValue({
        data: { stats: mockStats, health: mockHealth, tuning: mockTuning },
        isLoading: false,
        error: null,
        refetch: vi.fn(),
      } as any);

      const { result } = renderHook(() => useIndexMetrics({ enableAutoRefresh: true, defaultInterval: '10s' }), {
        wrapper: createWrapper(),
      });

      await waitFor(() => {
        expect(result.current.nextRefreshTime).not.toBeNull();
      });

      const firstTime = result.current.nextRefreshTime;

      // Advance time by 1 second
      act(() => {
        vi.advanceTimersByTime(1000);
      });

      // Next refresh time should have updated
      const secondTime = result.current.nextRefreshTime;
      expect(secondTime).not.toEqual(firstTime);
    });
  });

  describe('Manual Refresh', () => {
    it('should provide manual refresh function', async () => {
      const mockRefetch = vi.fn().mockResolvedValue(undefined);
      vi.spyOn(indexMetricsApi, 'useAllIndexMetrics').mockReturnValue({
        data: { stats: mockStats, health: mockHealth, tuning: mockTuning },
        isLoading: false,
        error: null,
        refetch: mockRefetch,
      } as any);

      const { result } = renderHook(() => useIndexMetrics(), {
        wrapper: createWrapper(),
      });

      expect(typeof result.current.manualRefresh).toBe('function');
    });

    it('should refetch data on manual refresh', async () => {
      const mockRefetch = vi.fn().mockResolvedValue(undefined);
      vi.spyOn(indexMetricsApi, 'useAllIndexMetrics').mockReturnValue({
        data: { stats: mockStats, health: mockHealth, tuning: mockTuning },
        isLoading: false,
        error: null,
        refetch: mockRefetch,
      } as any);

      const { result } = renderHook(() => useIndexMetrics(), {
        wrapper: createWrapper(),
      });

      await act(async () => {
        await result.current.manualRefresh();
      });

      expect(mockRefetch).toHaveBeenCalled();
    });

    it('should update lastUpdateTime after manual refresh', async () => {
      const mockRefetch = vi.fn().mockResolvedValue(undefined);
      vi.spyOn(indexMetricsApi, 'useAllIndexMetrics').mockReturnValue({
        data: { stats: mockStats, health: mockHealth, tuning: mockTuning },
        isLoading: false,
        error: null,
        refetch: mockRefetch,
      } as any);

      const { result } = renderHook(() => useIndexMetrics(), {
        wrapper: createWrapper(),
      });

      const beforeTime = result.current.lastUpdateTime;

      // Wait a bit
      act(() => {
        vi.advanceTimersByTime(100);
      });

      await act(async () => {
        await result.current.manualRefresh();
      });

      const afterTime = result.current.lastUpdateTime;
      expect(afterTime).not.toEqual(beforeTime);
    });
  });

  describe('Auto-Refresh Toggle', () => {
    it('should determine auto-refresh enabled state correctly', async () => {
      vi.spyOn(indexMetricsApi, 'useAllIndexMetrics').mockReturnValue({
        data: { stats: mockStats, health: mockHealth, tuning: mockTuning },
        isLoading: false,
        error: null,
        refetch: vi.fn(),
      } as any);

      const { result } = renderHook(() => useIndexMetrics({ enableAutoRefresh: true, defaultInterval: '10s' }), {
        wrapper: createWrapper(),
      });

      expect(result.current.autoRefreshEnabled).toBe(true);
    });

    it('should disable auto-refresh when enableAutoRefresh is false', async () => {
      vi.spyOn(indexMetricsApi, 'useAllIndexMetrics').mockReturnValue({
        data: { stats: mockStats, health: mockHealth, tuning: mockTuning },
        isLoading: false,
        error: null,
        refetch: vi.fn(),
      } as any);

      const { result } = renderHook(() => useIndexMetrics({ enableAutoRefresh: false, defaultInterval: '10s' }), {
        wrapper: createWrapper(),
      });

      expect(result.current.autoRefreshEnabled).toBe(false);
    });

    it('should allow changing auto-refresh interval', async () => {
      vi.spyOn(indexMetricsApi, 'useAllIndexMetrics').mockReturnValue({
        data: { stats: mockStats, health: mockHealth, tuning: mockTuning },
        isLoading: false,
        error: null,
        refetch: vi.fn(),
      } as any);

      const { result } = renderHook(() => useIndexMetrics({ enableAutoRefresh: true, defaultInterval: 'off' }), {
        wrapper: createWrapper(),
      });

      expect(result.current.autoRefreshInterval).toBe('off');

      act(() => {
        result.current.setAutoRefreshInterval('10s');
      });

      expect(result.current.autoRefreshInterval).toBe('10s');
    });
  });

  describe('Data Accessibility', () => {
    it('should provide access to stats data', async () => {
      vi.spyOn(indexMetricsApi, 'useAllIndexMetrics').mockReturnValue({
        data: { stats: mockStats, health: mockHealth, tuning: mockTuning },
        isLoading: false,
        error: null,
        refetch: vi.fn(),
      } as any);

      const { result } = renderHook(() => useIndexMetrics(), {
        wrapper: createWrapper(),
      });

      expect(result.current.stats).toEqual(mockStats);
    });

    it('should provide access to health data', async () => {
      vi.spyOn(indexMetricsApi, 'useAllIndexMetrics').mockReturnValue({
        data: { stats: mockStats, health: mockHealth, tuning: mockTuning },
        isLoading: false,
        error: null,
        refetch: vi.fn(),
      } as any);

      const { result } = renderHook(() => useIndexMetrics(), {
        wrapper: createWrapper(),
      });

      expect(result.current.health).toEqual(mockHealth);
    });

    it('should provide access to tuning data', async () => {
      vi.spyOn(indexMetricsApi, 'useAllIndexMetrics').mockReturnValue({
        data: { stats: mockStats, health: mockHealth, tuning: mockTuning },
        isLoading: false,
        error: null,
        refetch: vi.fn(),
      } as any);

      const { result } = renderHook(() => useIndexMetrics(), {
        wrapper: createWrapper(),
      });

      expect(result.current.tuning).toEqual(mockTuning);
    });
  });

  describe('Cleanup', () => {
    it('should cleanup interval on unmount', async () => {
      const clearIntervalSpy = vi.spyOn(global, 'clearInterval');

      vi.spyOn(indexMetricsApi, 'useAllIndexMetrics').mockReturnValue({
        data: { stats: mockStats, health: mockHealth, tuning: mockTuning },
        isLoading: false,
        error: null,
        refetch: vi.fn(),
      } as any);

      const { unmount } = renderHook(() => useIndexMetrics({ enableAutoRefresh: true, defaultInterval: '10s' }), {
        wrapper: createWrapper(),
      });

      unmount();

      // clearInterval should have been called
      expect(clearIntervalSpy).toHaveBeenCalled();
    });
  });
});
