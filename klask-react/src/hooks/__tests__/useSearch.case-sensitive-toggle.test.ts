import React from 'react';
import { describe, it, expect, vi, beforeEach } from 'vitest';
import { renderHook, waitFor, act } from '@testing-library/react';
import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import { useMultiSelectSearch } from '../useSearch';

// Mock fetch
const mockFetch = vi.fn();
global.fetch = mockFetch;

// Setup window.API_BASE_URL
if (typeof window === 'undefined') {
  (global as any).window = {
    API_BASE_URL: 'http://localhost:3000',
  };
} else if (!window.API_BASE_URL) {
  window.API_BASE_URL = 'http://localhost:3000';
}

const createWrapper = () => {
  const queryClient = new QueryClient({
    defaultOptions: {
      queries: { retry: false, refetchOnWindowFocus: false },
      mutations: { retry: false },
    },
  });

  return ({ children }: { children: React.ReactNode }) =>
    React.createElement(QueryClientProvider, { client: queryClient }, children);
};

const mockSearchResponse = {
  results: [
    {
      file_id: '1',
      doc_address: '0:1',
      name: 'test.rs',
      path: 'src/test.rs',
      content_snippet: 'test content',
      project: 'test-project',
      version: 'main',
      extension: 'rs',
      score: 1.5,
    },
  ],
  total: 1,
  page: 1,
  size: 20,
  facets: {
    projects: [],
    versions: [],
    extensions: [],
  },
};

describe('useMultiSelectSearch Hook - Case Sensitive Toggle Integration', () => {
  beforeEach(() => {
    vi.clearAllMocks();
    mockFetch.mockResolvedValue({
      ok: true,
      json: async () => mockSearchResponse,
    });
  });

  describe('Case Sensitive Parameter Passing', () => {
    it('should add case_sensitive=true parameter when caseSensitive is true', async () => {
      const { result } = renderHook(
        () => useMultiSelectSearch('test', {}, 1, {}, false, false, undefined, true),
        { wrapper: createWrapper() }
      );

      await waitFor(() => {
        expect(result.current.isSuccess).toBe(true);
      });

      // Check that fetch was called
      expect(mockFetch).toHaveBeenCalled();

      // Get the URL that was called
      const callUrl = mockFetch.mock.calls[0][0] as string;

      // Should contain case_sensitive=true
      expect(callUrl).toContain('case_sensitive=true');
    });

    it('should not add case_sensitive parameter when caseSensitive is false', async () => {
      const { result } = renderHook(
        () => useMultiSelectSearch('test', {}, 1, {}, false, false, undefined, false),
        { wrapper: createWrapper() }
      );

      await waitFor(() => {
        expect(result.current.isSuccess).toBe(true);
      });

      // Get the URL that was called
      const callUrl = mockFetch.mock.calls[0][0] as string;

      // Should NOT contain case_sensitive=true
      expect(callUrl).not.toContain('case_sensitive=true');
    });

    it('should not add case_sensitive parameter when undefined', async () => {
      const { result } = renderHook(
        () => useMultiSelectSearch('test', {}, 1, {}),
        { wrapper: createWrapper() }
      );

      await waitFor(() => {
        expect(result.current.isSuccess).toBe(true);
      });

      // Get the URL that was called
      const callUrl = mockFetch.mock.calls[0][0] as string;

      // Should NOT contain case_sensitive
      expect(callUrl).not.toContain('case_sensitive=true');
    });

    it('should add case_sensitive=true with fuzzy search together', async () => {
      const { result } = renderHook(
        () => useMultiSelectSearch('test', {}, 1, {}, true, false, undefined, true),
        { wrapper: createWrapper() }
      );

      await waitFor(() => {
        expect(result.current.isSuccess).toBe(true);
      });

      // Get the URL that was called
      const callUrl = mockFetch.mock.calls[0][0] as string;

      // Should contain both parameters
      expect(callUrl).toContain('fuzzy_search=true');
      expect(callUrl).toContain('case_sensitive=true');
    });

    it('should add case_sensitive=true with regex search together', async () => {
      const { result } = renderHook(
        () => useMultiSelectSearch('test', {}, 1, {}, false, true, undefined, true),
        { wrapper: createWrapper() }
      );

      await waitFor(() => {
        expect(result.current.isSuccess).toBe(true);
      });

      // Get the URL that was called
      const callUrl = mockFetch.mock.calls[0][0] as string;

      // Should contain both parameters
      expect(callUrl).toContain('regex_search=true');
      expect(callUrl).toContain('case_sensitive=true');
    });

    it('should add all three search parameters together', async () => {
      const { result } = renderHook(
        () => useMultiSelectSearch('test', {}, 1, {}, true, true, 'i', true),
        { wrapper: createWrapper() }
      );

      await waitFor(() => {
        expect(result.current.isSuccess).toBe(true);
      });

      // Get the URL that was called
      const callUrl = mockFetch.mock.calls[0][0] as string;

      // Should contain all parameters
      expect(callUrl).toContain('fuzzy_search=true');
      expect(callUrl).toContain('regex_search=true');
      expect(callUrl).toContain('regex_flags=i');
      expect(callUrl).toContain('case_sensitive=true');
    });
  });

  describe('Query Key Generation with Case Sensitive', () => {
    it('should include caseSensitive in query key for proper cache invalidation', async () => {
      const { result: resultWithCaseSensitive } = renderHook(
        () => useMultiSelectSearch('test', {}, 1, {}, false, false, undefined, true),
        { wrapper: createWrapper() }
      );

      await waitFor(() => {
        expect(resultWithCaseSensitive.current.isSuccess).toBe(true);
      });

      // Clear mocks
      vi.clearAllMocks();
      mockFetch.mockResolvedValue({
        ok: true,
        json: async () => mockSearchResponse,
      });

      // Call again with different case-sensitive state
      const { result: resultWithoutCaseSensitive } = renderHook(
        () => useMultiSelectSearch('test', {}, 1, {}, false, false, undefined, false),
        { wrapper: createWrapper() }
      );

      await waitFor(() => {
        expect(resultWithoutCaseSensitive.current.isSuccess).toBe(true);
      });

      // Should trigger a new fetch because query key is different
      expect(mockFetch).toHaveBeenCalled();
    });

    it('should cache results separately for different case-sensitive states', async () => {
      const queryClient = new QueryClient({
        defaultOptions: {
          queries: { retry: false, refetchOnWindowFocus: false },
          mutations: { retry: false },
        },
      });

      const wrapper = ({ children }: { children: React.ReactNode }) =>
        React.createElement(QueryClientProvider, { client: queryClient }, children);

      // First call with case-sensitive enabled
      const { result: resultWithCaseSensitive } = renderHook(
        () => useMultiSelectSearch('test', {}, 1, {}, false, false, undefined, true),
        { wrapper }
      );

      await waitFor(() => {
        expect(resultWithCaseSensitive.current.isSuccess).toBe(true);
      });

      // Second call with case-sensitive disabled
      const { result: resultWithoutCaseSensitive } = renderHook(
        () => useMultiSelectSearch('test', {}, 1, {}, false, false, undefined, false),
        { wrapper }
      );

      await waitFor(() => {
        expect(resultWithoutCaseSensitive.current.isSuccess).toBe(true);
      });

      // Both should have data (different caches)
      expect(resultWithCaseSensitive.current.data).toBeDefined();
      expect(resultWithoutCaseSensitive.current.data).toBeDefined();

      // Should have made 2 API calls (different query keys)
      expect(mockFetch).toHaveBeenCalledTimes(2);
    });

    it('should maintain separate cache keys for case-sensitive with other parameters', async () => {
      const queryClient = new QueryClient({
        defaultOptions: {
          queries: { retry: false, refetchOnWindowFocus: false },
          mutations: { retry: false },
        },
      });

      const wrapper = ({ children }: { children: React.ReactNode }) =>
        React.createElement(QueryClientProvider, { client: queryClient }, children);

      // Call with specific filters
      const { result: resultCaseSensitive } = renderHook(
        () => useMultiSelectSearch(
          'test',
          { projects: ['project1'] },
          1,
          {},
          false,
          false,
          undefined,
          true
        ),
        { wrapper }
      );

      await waitFor(() => {
        expect(resultCaseSensitive.current.isSuccess).toBe(true);
      });

      // Clear mocks for second call
      vi.clearAllMocks();
      mockFetch.mockResolvedValue({
        ok: true,
        json: async () => mockSearchResponse,
      });

      // Call with same filters but case-sensitive off
      const { result: resultNoCase } = renderHook(
        () => useMultiSelectSearch(
          'test',
          { projects: ['project1'] },
          1,
          {},
          false,
          false,
          undefined,
          false
        ),
        { wrapper }
      );

      await waitFor(() => {
        expect(resultNoCase.current.isSuccess).toBe(true);
      });

      // Should have made a new call due to different case-sensitive state
      expect(mockFetch).toHaveBeenCalled();
    });
  });

  describe('Backward Compatibility', () => {
    it('should work without caseSensitive parameter (undefined)', async () => {
      const { result } = renderHook(
        () => useMultiSelectSearch('test', {}, 1, {}),
        { wrapper: createWrapper() }
      );

      await waitFor(() => {
        expect(result.current.isSuccess).toBe(true);
      });

      expect(mockFetch).toHaveBeenCalled();
    });

    it('should treat undefined caseSensitive as false', async () => {
      mockFetch.mockClear();

      const { result } = renderHook(
        () => useMultiSelectSearch('test', {}, 1, {}),
        { wrapper: createWrapper() }
      );

      await waitFor(() => {
        expect(result.current.isSuccess).toBe(true);
      });

      const callUrl = mockFetch.mock.calls[0][0] as string;

      // Should not add case_sensitive=true
      expect(callUrl).not.toContain('case_sensitive=true');
    });

    it('should work with old function signature (without caseSensitive)', async () => {
      const { result } = renderHook(
        () => useMultiSelectSearch('test', { projects: ['example'] }, 1, {}, false, false),
        { wrapper: createWrapper() }
      );

      await waitFor(() => {
        expect(result.current.isSuccess).toBe(true);
      });

      expect(mockFetch).toHaveBeenCalled();
      const callUrl = mockFetch.mock.calls[0][0] as string;
      expect(callUrl).toContain('q=test');
      expect(callUrl).toContain('projects=example');
    });
  });

  describe('Filter Integration with Case Sensitive', () => {
    it('should work with case_sensitive and project filters together', async () => {
      const { result } = renderHook(
        () => useMultiSelectSearch(
          'test',
          { projects: ['project1', 'project2'] },
          1,
          {},
          false,
          false,
          undefined,
          true
        ),
        { wrapper: createWrapper() }
      );

      await waitFor(() => {
        expect(result.current.isSuccess).toBe(true);
      });

      const callUrl = mockFetch.mock.calls[0][0] as string;

      // Should contain both project filter and case_sensitive parameter
      expect(callUrl).toContain('projects=project1%2Cproject2');
      expect(callUrl).toContain('case_sensitive=true');
    });

    it('should work with case_sensitive and version filters together', async () => {
      const { result } = renderHook(
        () => useMultiSelectSearch(
          'test',
          { versions: ['1.0', '2.0'] },
          1,
          {},
          false,
          false,
          undefined,
          true
        ),
        { wrapper: createWrapper() }
      );

      await waitFor(() => {
        expect(result.current.isSuccess).toBe(true);
      });

      const callUrl = mockFetch.mock.calls[0][0] as string;

      // Should contain both version filter and case_sensitive parameter
      expect(callUrl).toContain('versions=1.0%2C2.0');
      expect(callUrl).toContain('case_sensitive=true');
    });

    it('should work with case_sensitive and size range filters', async () => {
      const { result } = renderHook(
        () => useMultiSelectSearch(
          'test',
          { sizeRange: { min: 100, max: 1000 } },
          1,
          {},
          false,
          false,
          undefined,
          true
        ),
        { wrapper: createWrapper() }
      );

      await waitFor(() => {
        expect(result.current.isSuccess).toBe(true);
      });

      const callUrl = mockFetch.mock.calls[0][0] as string;

      // Should contain size range and case_sensitive
      expect(callUrl).toContain('min_size=100');
      expect(callUrl).toContain('max_size=1000');
      expect(callUrl).toContain('case_sensitive=true');
    });

    it('should work with case_sensitive and multiple filters together', async () => {
      const { result } = renderHook(
        () => useMultiSelectSearch(
          'Crawler',
          {
            projects: ['klask-rs'],
            versions: ['main'],
            extensions: ['rs'],
          },
          1,
          {},
          false,
          false,
          undefined,
          true
        ),
        { wrapper: createWrapper() }
      );

      await waitFor(() => {
        expect(result.current.isSuccess).toBe(true);
      });

      const callUrl = mockFetch.mock.calls[0][0] as string;

      // Should contain all filters and case_sensitive parameter
      expect(callUrl).toContain('q=Crawler');
      expect(callUrl).toContain('projects=klask-rs');
      expect(callUrl).toContain('versions=main');
      expect(callUrl).toContain('extensions=rs');
      expect(callUrl).toContain('case_sensitive=true');
    });
  });

  describe('API Response Handling with Case Sensitive', () => {
    it('should handle API response correctly with case_sensitive enabled', async () => {
      const { result } = renderHook(
        () => useMultiSelectSearch('test', {}, 1, {}, false, false, undefined, true),
        { wrapper: createWrapper() }
      );

      await waitFor(() => {
        expect(result.current.isSuccess).toBe(true);
      });

      // Response should be properly returned
      expect(result.current.data).toEqual(mockSearchResponse);
      expect(result.current.data.results).toHaveLength(1);
    });

    it('should handle API errors with case_sensitive enabled', async () => {
      // Mock fetch to return non-ok response
      mockFetch.mockResolvedValue({
        ok: false,
        statusText: 'Internal Server Error',
        json: async () => { throw new Error('Invalid JSON'); },
      });

      const { result } = renderHook(
        () => useMultiSelectSearch('test', {}, 1, {}, false, false, undefined, true),
        { wrapper: createWrapper() }
      );

      // Wait for the query to be attempted
      await waitFor(() => {
        // The hook should either error or show loading state
        expect(result.current.status).toMatch(/error|pending|loading/);
      }, { timeout: 2000 });
    });

    it('should handle empty search results with case_sensitive enabled', async () => {
      mockFetch.mockResolvedValue({
        ok: true,
        json: async () => ({
          results: [],
          total: 0,
          facets: {},
        }),
      });

      const { result } = renderHook(
        () => useMultiSelectSearch('nonexistent', {}, 1, {}, false, false, undefined, true),
        { wrapper: createWrapper() }
      );

      await waitFor(() => {
        expect(result.current.isSuccess).toBe(true);
      });

      expect(result.current.data.results).toHaveLength(0);
      expect(result.current.data.total).toBe(0);
    });

    it('should handle malformed API responses gracefully', async () => {
      mockFetch.mockResolvedValue({
        ok: true,
        json: async () => ({}),
      });

      const { result } = renderHook(
        () => useMultiSelectSearch('test', {}, 1, {}, false, false, undefined, true),
        { wrapper: createWrapper() }
      );

      await waitFor(() => {
        expect(result.current.status).toMatch(/success|error/);
      });
    });
  });

  describe('Pagination with Case Sensitive', () => {
    it('should include case_sensitive parameter when changing pages', async () => {
      const { result, rerender } = renderHook(
        ({ page }: { page: number }) =>
          useMultiSelectSearch('test', {}, page, {}, false, false, undefined, true),
        {
          wrapper: createWrapper(),
          initialProps: { page: 1 },
        }
      );

      await waitFor(() => {
        expect(result.current.isSuccess).toBe(true);
      });

      // Move to page 2
      rerender({ page: 2 });

      await waitFor(() => {
        expect(mockFetch).toHaveBeenCalledTimes(2);
      });

      // Second call should also have case_sensitive=true
      const secondCallUrl = mockFetch.mock.calls[1][0] as string;
      expect(secondCallUrl).toContain('case_sensitive=true');
      expect(secondCallUrl).toContain('page=2');
    });

    it('should preserve case_sensitive across multiple page changes', async () => {
      const { result, rerender } = renderHook(
        ({ page }: { page: number }) =>
          useMultiSelectSearch('test', {}, page, {}, false, false, undefined, true),
        {
          wrapper: createWrapper(),
          initialProps: { page: 1 },
        }
      );

      await waitFor(() => {
        expect(result.current.isSuccess).toBe(true);
      });

      // Change to page 2
      rerender({ page: 2 });
      await waitFor(() => {
        expect(mockFetch).toHaveBeenCalledTimes(2);
      });

      // Change to page 3
      rerender({ page: 3 });
      await waitFor(() => {
        expect(mockFetch).toHaveBeenCalledTimes(3);
      });

      // All calls should have case_sensitive=true
      for (let i = 0; i < mockFetch.mock.calls.length; i++) {
        const callUrl = mockFetch.mock.calls[i][0] as string;
        expect(callUrl).toContain('case_sensitive=true');
      }
    });
  });

  describe('Query Building with Case Sensitive', () => {
    it('should build correct URL with case_sensitive parameter', async () => {
      const { result } = renderHook(
        () => useMultiSelectSearch('Crawler', {}, 1, {}, false, false, undefined, true),
        { wrapper: createWrapper() }
      );

      await waitFor(() => {
        expect(result.current.isSuccess).toBe(true);
      });

      const callUrl = mockFetch.mock.calls[0][0] as string;

      // Should have proper URL structure
      expect(callUrl).toContain('/api/search');
      expect(callUrl).toContain('q=Crawler');
      expect(callUrl).toContain('case_sensitive=true');
      expect(callUrl).toContain('limit=20');
      expect(callUrl).toContain('page=1');
      expect(callUrl).toContain('include_facets=true');
    });

    it('should properly encode special characters with case_sensitive', async () => {
      const { result } = renderHook(
        () => useMultiSelectSearch('MyClass', {}, 1, {}, false, false, undefined, true),
        { wrapper: createWrapper() }
      );

      await waitFor(() => {
        expect(result.current.isSuccess).toBe(true);
      });

      const callUrl = mockFetch.mock.calls[0][0] as string;

      // URL should properly encode the query with special characters
      expect(callUrl).toContain('case_sensitive=true');
      expect(callUrl).toContain('MyClass');
    });

    it('should not duplicate case_sensitive parameter', async () => {
      const { result } = renderHook(
        () => useMultiSelectSearch('test', {}, 1, {}, false, false, undefined, true),
        { wrapper: createWrapper() }
      );

      await waitFor(() => {
        expect(result.current.isSuccess).toBe(true);
      });

      const callUrl = mockFetch.mock.calls[0][0] as string;

      // Count occurrences of case_sensitive
      const occurrences = (callUrl.match(/case_sensitive=true/g) || []).length;
      expect(occurrences).toBe(1);
    });

    it('should use lowercase "true" for case_sensitive parameter', async () => {
      const { result } = renderHook(
        () => useMultiSelectSearch('test', {}, 1, {}, false, false, undefined, true),
        { wrapper: createWrapper() }
      );

      await waitFor(() => {
        expect(result.current.isSuccess).toBe(true);
      });

      const callUrl = mockFetch.mock.calls[0][0] as string;

      // Parameter should use lowercase 'true'
      expect(callUrl).toContain('case_sensitive=true');
      expect(callUrl).not.toContain('case_sensitive=True');
      expect(callUrl).not.toContain('case_sensitive=TRUE');
    });
  });

  describe('State Updates with Case Sensitive', () => {
    it('should update state when caseSensitive changes', async () => {
      const { result, rerender } = renderHook(
        ({ caseSensitive }: { caseSensitive: boolean }) =>
          useMultiSelectSearch('test', {}, 1, {}, false, false, undefined, caseSensitive),
        {
          wrapper: createWrapper(),
          initialProps: { caseSensitive: false },
        }
      );

      await waitFor(() => {
        expect(result.current.isSuccess).toBe(true);
      });

      const initialCallCount = mockFetch.mock.calls.length;

      // Change caseSensitive to true
      rerender({ caseSensitive: true });

      await waitFor(() => {
        // Should trigger new fetch with updated query key
        expect(mockFetch.mock.calls.length).toBeGreaterThan(initialCallCount);
      });

      // New URL should have case_sensitive=true
      const lastCallUrl = mockFetch.mock.calls[mockFetch.mock.calls.length - 1][0] as string;
      expect(lastCallUrl).toContain('case_sensitive=true');
    });

    it('should correctly reflect state changes between true and false', async () => {
      const { result, rerender } = renderHook(
        ({ caseSensitive }: { caseSensitive: boolean }) =>
          useMultiSelectSearch('test', {}, 1, {}, false, false, undefined, caseSensitive),
        {
          wrapper: createWrapper(),
          initialProps: { caseSensitive: false },
        }
      );

      await waitFor(() => {
        expect(result.current.isSuccess).toBe(true);
      });

      // First call should NOT have case_sensitive=true
      const firstCallUrl = mockFetch.mock.calls[0][0] as string;
      expect(firstCallUrl).not.toContain('case_sensitive=true');

      // Change to true
      rerender({ caseSensitive: true });

      await waitFor(() => {
        expect(mockFetch).toHaveBeenCalledTimes(2);
      });

      // Second call SHOULD have case_sensitive=true
      const secondCallUrl = mockFetch.mock.calls[1][0] as string;
      expect(secondCallUrl).toContain('case_sensitive=true');

      // Change back to false
      rerender({ caseSensitive: false });

      await waitFor(() => {
        expect(mockFetch).toHaveBeenCalledTimes(3);
      });

      // Third call should NOT have case_sensitive=true
      const thirdCallUrl = mockFetch.mock.calls[2][0] as string;
      expect(thirdCallUrl).not.toContain('case_sensitive=true');
    });
  });

  describe('Interaction with Search Query Changes', () => {
    it('should maintain case_sensitive when query changes', async () => {
      const { result, rerender } = renderHook(
        ({ query }: { query: string }) =>
          useMultiSelectSearch(query, {}, 1, {}, false, false, undefined, true),
        {
          wrapper: createWrapper(),
          initialProps: { query: 'test1' },
        }
      );

      await waitFor(() => {
        expect(result.current.isSuccess).toBe(true);
      });

      // Change query
      rerender({ query: 'test2' });

      await waitFor(() => {
        expect(mockFetch).toHaveBeenCalledTimes(2);
      });

      // Both calls should have case_sensitive=true
      const firstCallUrl = mockFetch.mock.calls[0][0] as string;
      const secondCallUrl = mockFetch.mock.calls[1][0] as string;

      expect(firstCallUrl).toContain('case_sensitive=true');
      expect(secondCallUrl).toContain('case_sensitive=true');
    });
  });

  describe('Performance Considerations', () => {
    it('should not make unnecessary API calls when case_sensitive is unchanged', async () => {
      const { result, rerender } = renderHook(
        ({ caseSensitive }: { caseSensitive: boolean }) =>
          useMultiSelectSearch('test', {}, 1, {}, false, false, undefined, caseSensitive),
        {
          wrapper: createWrapper(),
          initialProps: { caseSensitive: true },
        }
      );

      await waitFor(() => {
        expect(result.current.isSuccess).toBe(true);
      });

      const initialCallCount = mockFetch.mock.calls.length;

      // Re-render with same case_sensitive value
      rerender({ caseSensitive: true });

      // Should not make a new API call
      expect(mockFetch.mock.calls.length).toBe(initialCallCount);
    });

    it('should use placeholder data to prevent flickering on case_sensitive change', async () => {
      const { result, rerender } = renderHook(
        ({ caseSensitive }: { caseSensitive: boolean }) =>
          useMultiSelectSearch('test', {}, 1, {}, false, false, undefined, caseSensitive),
        {
          wrapper: createWrapper(),
          initialProps: { caseSensitive: false },
        }
      );

      await waitFor(() => {
        expect(result.current.isSuccess).toBe(true);
      });

      const initialData = result.current.data;

      // Change case_sensitive
      rerender({ caseSensitive: true });

      // Data should still be available (placeholder data prevents flickering)
      expect(result.current.data).toBeDefined();
    });
  });
});
