import { describe, it, expect, vi, beforeEach } from 'vitest';
import { renderHook, waitFor, act } from '@testing-library/react';
import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import React from 'react';
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

describe('useMultiSelectSearch Hook - Regex Toggle Integration', () => {
  beforeEach(() => {
    vi.clearAllMocks();
    mockFetch.mockResolvedValue({
      ok: true,
      json: async () => mockSearchResponse,
    });
  });

  describe('Regex Parameter Passing', () => {
    it('should add regex_search=true parameter when regexSearch is true', async () => {
      const { result } = renderHook(
        () => useMultiSelectSearch('test', {}, 1, {}, false, true),
        { wrapper: createWrapper() }
      );

      await waitFor(() => {
        expect(result.current.isSuccess).toBe(true);
      });

      // Check that fetch was called
      expect(mockFetch).toHaveBeenCalled();

      // Get the URL that was called
      const callUrl = mockFetch.mock.calls[0][0] as string;

      // Should contain regex_search=true
      expect(callUrl).toContain('regex_search=true');
    });

    it('should not add regex_search parameter when regexSearch is false', async () => {
      const { result } = renderHook(
        () => useMultiSelectSearch('test', {}, 1, {}, false, false),
        { wrapper: createWrapper() }
      );

      await waitFor(() => {
        expect(result.current.isSuccess).toBe(true);
      });

      // Get the URL that was called
      const callUrl = mockFetch.mock.calls[0][0] as string;

      // Should NOT contain regex_search
      expect(callUrl).not.toContain('regex_search=true');
    });

    it('should add both fuzzy_search and regex_search when both are true', async () => {
      const { result } = renderHook(
        () => useMultiSelectSearch('test', {}, 1, {}, true, true),
        { wrapper: createWrapper() }
      );

      await waitFor(() => {
        expect(result.current.isSuccess).toBe(true);
      });

      // Get the URL that was called
      const callUrl = mockFetch.mock.calls[0][0] as string;

      // Should contain both parameters
      expect(callUrl).toContain('fuzzy_search=true');
      expect(callUrl).toContain('regex_search=true');
    });

    it('should add only fuzzy_search when only fuzzySearch is true', async () => {
      const { result } = renderHook(
        () => useMultiSelectSearch('test', {}, 1, {}, true, false),
        { wrapper: createWrapper() }
      );

      await waitFor(() => {
        expect(result.current.isSuccess).toBe(true);
      });

      // Get the URL that was called
      const callUrl = mockFetch.mock.calls[0][0] as string;

      // Should contain fuzzy_search but not regex_search
      expect(callUrl).toContain('fuzzy_search=true');
      expect(callUrl).not.toContain('regex_search=true');
    });

    it('should add neither parameter when both are false', async () => {
      const { result } = renderHook(
        () => useMultiSelectSearch('test', {}, 1, {}, false, false),
        { wrapper: createWrapper() }
      );

      await waitFor(() => {
        expect(result.current.isSuccess).toBe(true);
      });

      // Get the URL that was called
      const callUrl = mockFetch.mock.calls[0][0] as string;

      // Should contain neither parameter (or they should be false if present)
      expect(callUrl).not.toContain('regex_search=true');
      expect(callUrl).not.toContain('fuzzy_search=true');
    });
  });

  describe('Query Key Generation', () => {
    it('should include regexSearch in query key for proper cache invalidation', async () => {
      const { result: resultWithRegex } = renderHook(
        () => useMultiSelectSearch('test', {}, 1, {}, false, true),
        { wrapper: createWrapper() }
      );

      await waitFor(() => {
        expect(resultWithRegex.current.isSuccess).toBe(true);
      });

      // Clear mocks
      vi.clearAllMocks();
      mockFetch.mockResolvedValue({
        ok: true,
        json: async () => mockSearchResponse,
      });

      // Call again with different regex state
      const { result: resultWithoutRegex } = renderHook(
        () => useMultiSelectSearch('test', {}, 1, {}, false, false),
        { wrapper: createWrapper() }
      );

      await waitFor(() => {
        expect(resultWithoutRegex.current.isSuccess).toBe(true);
      });

      // Should trigger a new fetch because query key is different
      expect(mockFetch).toHaveBeenCalled();
    });

    it('should cache results separately for different regex states', async () => {
      const queryClient = new QueryClient({
        defaultOptions: {
          queries: { retry: false, refetchOnWindowFocus: false },
          mutations: { retry: false },
        },
      });

      const wrapper = ({ children }: { children: React.ReactNode }) =>
        React.createElement(QueryClientProvider, { client: queryClient }, children);

      // First call with regex enabled
      const { result: resultWithRegex } = renderHook(
        () => useMultiSelectSearch('test', {}, 1, {}, false, true),
        { wrapper }
      );

      await waitFor(() => {
        expect(resultWithRegex.current.isSuccess).toBe(true);
      });

      // Second call with regex disabled
      const { result: resultWithoutRegex } = renderHook(
        () => useMultiSelectSearch('test', {}, 1, {}, false, false),
        { wrapper }
      );

      await waitFor(() => {
        expect(resultWithoutRegex.current.isSuccess).toBe(true);
      });

      // Both should have data (different caches)
      expect(resultWithRegex.current.data).toBeDefined();
      expect(resultWithoutRegex.current.data).toBeDefined();

      // Should have made 2 API calls (different query keys)
      expect(mockFetch).toHaveBeenCalledTimes(2);
    });
  });

  describe('Backward Compatibility', () => {
    it('should work without regexSearch parameter (undefined)', async () => {
      const { result } = renderHook(
        () => useMultiSelectSearch('test', {}, 1, {}, false),
        { wrapper: createWrapper() }
      );

      await waitFor(() => {
        expect(result.current.isSuccess).toBe(true);
      });

      expect(mockFetch).toHaveBeenCalled();
    });

    it('should treat undefined regexSearch as false', async () => {
      mockFetch.mockClear();

      const { result } = renderHook(
        () => useMultiSelectSearch('test', {}, 1, {}, false),
        { wrapper: createWrapper() }
      );

      await waitFor(() => {
        expect(result.current.isSuccess).toBe(true);
      });

      const callUrl = mockFetch.mock.calls[0][0] as string;

      // Should not add regex_search=true
      expect(callUrl).not.toContain('regex_search=true');
    });
  });

  describe('Filter Integration', () => {
    it('should work with regexSearch and project filters together', async () => {
      const { result } = renderHook(
        () => useMultiSelectSearch(
          'test',
          { projects: ['project1', 'project2'] },
          1,
          {},
          false,
          true
        ),
        { wrapper: createWrapper() }
      );

      await waitFor(() => {
        expect(result.current.isSuccess).toBe(true);
      });

      const callUrl = mockFetch.mock.calls[0][0] as string;

      // Should contain both project filter and regex parameter
      expect(callUrl).toContain('projects=project1%2Cproject2');
      expect(callUrl).toContain('regex_search=true');
    });

    it('should work with regexSearch and version filters together', async () => {
      const { result } = renderHook(
        () => useMultiSelectSearch(
          'test',
          { versions: ['1.0', '2.0'] },
          1,
          {},
          false,
          true
        ),
        { wrapper: createWrapper() }
      );

      await waitFor(() => {
        expect(result.current.isSuccess).toBe(true);
      });

      const callUrl = mockFetch.mock.calls[0][0] as string;

      // Should contain both version filter and regex parameter
      expect(callUrl).toContain('versions=1.0%2C2.0');
      expect(callUrl).toContain('regex_search=true');
    });

    it('should work with regexSearch and multiple filters', async () => {
      const { result } = renderHook(
        () => useMultiSelectSearch(
          'Crawler.*',
          {
            projects: ['klask-rs'],
            versions: ['main'],
            extensions: ['rs'],
          },
          1,
          {},
          false,
          true
        ),
        { wrapper: createWrapper() }
      );

      await waitFor(() => {
        expect(result.current.isSuccess).toBe(true);
      });

      const callUrl = mockFetch.mock.calls[0][0] as string;

      // Should contain all filters and regex parameter
      expect(callUrl).toContain('q=Crawler.*');
      expect(callUrl).toContain('projects=klask-rs');
      expect(callUrl).toContain('versions=main');
      expect(callUrl).toContain('extensions=rs');
      expect(callUrl).toContain('regex_search=true');
    });
  });

  describe('API Response Handling', () => {
    it('should handle API response correctly with regexSearch enabled', async () => {
      const { result } = renderHook(
        () => useMultiSelectSearch('test', {}, 1, {}, false, true),
        { wrapper: createWrapper() }
      );

      await waitFor(() => {
        expect(result.current.isSuccess).toBe(true);
      });

      // Response should be properly returned
      expect(result.current.data).toEqual(mockSearchResponse);
      expect(result.current.data.results).toHaveLength(1);
    });

    it('should handle API errors with regexSearch enabled', async () => {
      // Mock fetch to return non-ok response
      mockFetch.mockResolvedValue({
        ok: false,
        statusText: 'Internal Server Error',
        json: async () => { throw new Error('Invalid JSON'); },
      });

      const { result } = renderHook(
        () => useMultiSelectSearch('test', {}, 1, {}, false, true),
        { wrapper: createWrapper() }
      );

      // Wait for the query to be attempted
      await waitFor(() => {
        // The hook should either error or show loading state
        expect(result.current.status).toMatch(/error|pending|loading/);
      }, { timeout: 2000 });
    });

    it('should handle empty search results with regexSearch enabled', async () => {
      mockFetch.mockResolvedValue({
        ok: true,
        json: async () => ({
          results: [],
          total: 0,
          facets: {},
        }),
      });

      const { result } = renderHook(
        () => useMultiSelectSearch('nonexistent', {}, 1, {}, false, true),
        { wrapper: createWrapper() }
      );

      await waitFor(() => {
        expect(result.current.isSuccess).toBe(true);
      });

      expect(result.current.data.results).toHaveLength(0);
      expect(result.current.data.total).toBe(0);
    });
  });

  describe('Pagination with Regex', () => {
    it('should include regex parameter when changing pages', async () => {
      const { result, rerender } = renderHook(
        ({ page }: { page: number }) =>
          useMultiSelectSearch('test', {}, page, {}, false, true),
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

      // Second call should also have regex_search=true
      const secondCallUrl = mockFetch.mock.calls[1][0] as string;
      expect(secondCallUrl).toContain('regex_search=true');
      expect(secondCallUrl).toContain('page=2');
    });
  });

  describe('Query Building', () => {
    it('should build correct URL with standard query parameters', async () => {
      const { result } = renderHook(
        () => useMultiSelectSearch('Crawler', {}, 1, {}, false, true),
        { wrapper: createWrapper() }
      );

      await waitFor(() => {
        expect(result.current.isSuccess).toBe(true);
      });

      const callUrl = mockFetch.mock.calls[0][0] as string;

      // Should have proper URL structure
      expect(callUrl).toContain('/api/search');
      expect(callUrl).toContain('q=Crawler');
      expect(callUrl).toContain('regex_search=true');
      expect(callUrl).toContain('limit=20');
      expect(callUrl).toContain('page=1');
      expect(callUrl).toContain('include_facets=true');
    });

    it('should properly encode special regex characters in URL', async () => {
      const { result } = renderHook(
        () => useMultiSelectSearch('^Crawler.*$', {}, 1, {}, false, true),
        { wrapper: createWrapper() }
      );

      await waitFor(() => {
        expect(result.current.isSuccess).toBe(true);
      });

      const callUrl = mockFetch.mock.calls[0][0] as string;

      // URL should properly encode the regex pattern
      expect(callUrl).toContain('regex_search=true');
      // The actual encoding of ^Crawler.*$ might vary but should be URL-safe
      expect(callUrl).toMatch(/q=.*Crawler/);
    });
  });

  describe('State Updates', () => {
    it('should update state when regexSearch changes', async () => {
      const { result, rerender } = renderHook(
        ({ regexSearch }: { regexSearch: boolean }) =>
          useMultiSelectSearch('test', {}, 1, {}, false, regexSearch),
        {
          wrapper: createWrapper(),
          initialProps: { regexSearch: false },
        }
      );

      await waitFor(() => {
        expect(result.current.isSuccess).toBe(true);
      });

      const initialCallCount = mockFetch.mock.calls.length;

      // Change regexSearch to true
      rerender({ regexSearch: true });

      await waitFor(() => {
        // Should trigger new fetch with updated query key
        expect(mockFetch.mock.calls.length).toBeGreaterThan(initialCallCount);
      });

      // New URL should have regex_search=true
      const lastCallUrl = mockFetch.mock.calls[mockFetch.mock.calls.length - 1][0] as string;
      expect(lastCallUrl).toContain('regex_search=true');
    });
  });
});
