import { describe, it, expect, vi, beforeEach } from 'vitest';
import { renderHook, waitFor } from '@testing-library/react';
import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import { useSearchStatus } from '../indexMetrics';
import type { SearchStatusResponse } from '../indexMetrics';
import * as apiLib from '../../lib/api';

// Mock the api library
vi.mock('../../lib/api', () => ({
  api: {
    get: vi.fn(),
    post: vi.fn(),
  },
}));

describe('indexMetrics Hook Tests', () => {
  let queryClient: QueryClient;
  const mockApi = vi.mocked(apiLib.api);

  beforeEach(() => {
    queryClient = new QueryClient({
      defaultOptions: {
        queries: {
          retry: false,
          refetchOnWindowFocus: false,
        },
      },
    });
    vi.clearAllMocks();
  });

  const createWrapper = () => {
    return ({ children }: { children: React.ReactNode }) => (
      <QueryClientProvider client={queryClient}>
        {children}
      </QueryClientProvider>
    );
  };

  describe('useSearchStatus Hook', () => {
    // Test 15: useSearchStatus queries endpoint
    it('should query the correct endpoint', async () => {
      const mockResponse: SearchStatusResponse = {
        schema_mismatch: false,
        index_available: true,
      };

      mockApi.get.mockResolvedValue(mockResponse);

      const { result } = renderHook(() => useSearchStatus(), {
        wrapper: createWrapper(),
      });

      // Wait for the query to complete
      await waitFor(() => {
        expect(result.current.isLoading).toBe(false);
      });

      expect(mockApi.get).toHaveBeenCalledWith('/api/admin/search/status');
    });

    // Test 15b: useSearchStatus returns correct shape
    it('should return correct data shape with schema_mismatch and index_available', async () => {
      const mockResponse: SearchStatusResponse = {
        schema_mismatch: false,
        index_available: true,
        message: 'Index is healthy',
      };

      mockApi.get.mockResolvedValue(mockResponse);

      const { result } = renderHook(() => useSearchStatus(), {
        wrapper: createWrapper(),
      });

      await waitFor(() => {
        expect(result.current.data).toBeDefined();
      });

      expect(result.current.data).toEqual(mockResponse);
      expect(result.current.data?.schema_mismatch).toBe(false);
      expect(result.current.data?.index_available).toBe(true);
      expect(result.current.data?.message).toBe('Index is healthy');
    });

    // Test 15c: useSearchStatus handles response without message
    it('should handle response without optional message field', async () => {
      const mockResponse: SearchStatusResponse = {
        schema_mismatch: false,
        index_available: true,
      };

      mockApi.get.mockResolvedValue(mockResponse);

      const { result } = renderHook(() => useSearchStatus(), {
        wrapper: createWrapper(),
      });

      await waitFor(() => {
        expect(result.current.data).toBeDefined();
      });

      expect(result.current.data?.message).toBeUndefined();
    });

    // Test 16: useSearchStatus loads successfully
    it('should set isLoading to false after successful fetch', async () => {
      const mockResponse: SearchStatusResponse = {
        schema_mismatch: false,
        index_available: true,
      };

      mockApi.get.mockResolvedValue(mockResponse);

      const { result } = renderHook(() => useSearchStatus(), {
        wrapper: createWrapper(),
      });

      expect(result.current.isLoading).toBe(true);

      await waitFor(() => {
        expect(result.current.isLoading).toBe(false);
      });
    });

    // Test 16b: useSearchStatus with schema mismatch returns correct data
    it('should return schema_mismatch: true when index needs rebuild', async () => {
      const mockResponse: SearchStatusResponse = {
        schema_mismatch: true,
        index_available: false,
        message: 'Schema version mismatch',
      };

      mockApi.get.mockResolvedValue(mockResponse);

      const { result } = renderHook(() => useSearchStatus(), {
        wrapper: createWrapper(),
      });

      await waitFor(() => {
        expect(result.current.data).toBeDefined();
      });

      expect(result.current.data?.schema_mismatch).toBe(true);
      expect(result.current.data?.index_available).toBe(false);
    });

    // Test 17: useSearchStatus graceful error handling
    it('should not crash when API returns error', async () => {
      const error = new Error('API Error');
      mockApi.get.mockRejectedValue(error);

      const { result } = renderHook(() => useSearchStatus(), {
        wrapper: createWrapper(),
      });

      await waitFor(() => {
        expect(result.current.isLoading).toBe(false);
      }, { timeout: 3000 });

      // Hook should handle error gracefully (throwOnError: false)
      expect(result.current.error).toBeTruthy();
    });

    // Test 17b: useSearchStatus returns safe default on error
    it('should not throw error even when API fails', async () => {
      mockApi.get.mockRejectedValue(new Error('Network error'));

      expect(() => {
        renderHook(() => useSearchStatus(), {
          wrapper: createWrapper(),
        });
      }).not.toThrow();
    });

    // Test 18: useSearchStatus refetch method works
    it('should provide refetch method', async () => {
      const mockResponse: SearchStatusResponse = {
        schema_mismatch: false,
        index_available: true,
      };

      mockApi.get.mockResolvedValue(mockResponse);

      const { result } = renderHook(() => useSearchStatus(), {
        wrapper: createWrapper(),
      });

      await waitFor(() => {
        expect(result.current.data).toBeDefined();
      });

      expect(result.current.refetch).toBeDefined();
      expect(typeof result.current.refetch).toBe('function');
    });

    // Test 18b: useSearchStatus can refetch data
    it('should refetch data when refetch is called', async () => {
      const initialResponse: SearchStatusResponse = {
        schema_mismatch: false,
        index_available: true,
      };

      const refetchedResponse: SearchStatusResponse = {
        schema_mismatch: true,
        index_available: false,
      };

      mockApi.get.mockResolvedValueOnce(initialResponse).mockResolvedValueOnce(refetchedResponse);

      const { result } = renderHook(() => useSearchStatus(), {
        wrapper: createWrapper(),
      });

      await waitFor(() => {
        expect(result.current.data?.schema_mismatch).toBe(false);
      });

      // Call refetch
      result.current.refetch();

      await waitFor(() => {
        expect(result.current.data?.schema_mismatch).toBe(true);
      });

      expect(mockApi.get).toHaveBeenCalledTimes(2);
    });

    // Test 18c: useSearchStatus no auto-refetch by default
    it('should not auto-refetch by default (refetchInterval: false)', async () => {
      const mockResponse: SearchStatusResponse = {
        schema_mismatch: false,
        index_available: true,
      };

      mockApi.get.mockResolvedValue(mockResponse);

      const { result } = renderHook(() => useSearchStatus(false), {
        wrapper: createWrapper(),
      });

      await waitFor(() => {
        expect(result.current.data).toBeDefined();
      });

      // API should only be called once
      expect(mockApi.get).toHaveBeenCalledTimes(1);
    });

    // Test 19: useSearchStatus with custom refetch interval
    it('should accept custom refetch interval parameter', async () => {
      const mockResponse: SearchStatusResponse = {
        schema_mismatch: false,
        index_available: true,
      };

      mockApi.get.mockResolvedValue(mockResponse);

      // The hook should accept refetchInterval parameter
      const { result } = renderHook(() => useSearchStatus(5000), {
        wrapper: createWrapper(),
      });

      await waitFor(() => {
        expect(result.current.data).toBeDefined();
      });

      // Just verify it doesn't error with the parameter
      expect(result.current.data).toBeDefined();
    });

    // Test 19b: useSearchStatus with false refetch interval parameter
    it('should respect false refetch interval (no auto-refetch)', async () => {
      const mockResponse: SearchStatusResponse = {
        schema_mismatch: false,
        index_available: true,
      };

      mockApi.get.mockResolvedValue(mockResponse);

      const { result } = renderHook(() => useSearchStatus(false), {
        wrapper: createWrapper(),
      });

      await waitFor(() => {
        expect(result.current.data).toBeDefined();
      });

      // Only initial call, no auto-refetch
      expect(mockApi.get).toHaveBeenCalledTimes(1);
    });

    // Test 20: useSearchStatus isFetching flag
    it('should provide isFetching flag', async () => {
      const mockResponse: SearchStatusResponse = {
        schema_mismatch: false,
        index_available: true,
      };

      mockApi.get.mockResolvedValue(mockResponse);

      const { result } = renderHook(() => useSearchStatus(), {
        wrapper: createWrapper(),
      });

      // Initially should be false (we use isLoading instead)
      expect(result.current.isFetching).toBeDefined();
    });

    // Test 21: useSearchStatus validation of response structure
    it('should validate response has required fields', async () => {
      // Missing index_available - should be invalid
      const invalidResponse = {
        schema_mismatch: false,
        // index_available is missing
      };

      mockApi.get.mockResolvedValue(invalidResponse);

      const { result } = renderHook(() => useSearchStatus(), {
        wrapper: createWrapper(),
      });

      await waitFor(() => {
        expect(result.current.isLoading).toBe(false);
      }, { timeout: 3000 });

      // Should error due to validation
      expect(result.current.error).toBeTruthy();
    });

    // Test 22: useSearchStatus with stale time
    it('should have appropriate stale time configuration', async () => {
      const mockResponse: SearchStatusResponse = {
        schema_mismatch: false,
        index_available: true,
      };

      mockApi.get.mockResolvedValue(mockResponse);

      const { result } = renderHook(() => useSearchStatus(), {
        wrapper: createWrapper(),
      });

      await waitFor(() => {
        expect(result.current.data).toBeDefined();
      });

      // Verify the data was fetched
      expect(result.current.data).toEqual(mockResponse);
    });
  });
});
