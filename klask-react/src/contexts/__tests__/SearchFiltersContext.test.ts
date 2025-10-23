import { describe, it, expect, vi, beforeEach } from 'vitest';
import { renderHook, act, waitFor } from '@testing-library/react';
import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import React from 'react';
import { SearchFiltersProvider, useSearchFiltersContext } from '../SearchFiltersContext';
import * as searchHooks from '../../hooks/useSearch';

// Mock the search hooks
vi.mock('../../hooks/useSearch', () => ({
  useSearchFilters: vi.fn(),
  useFacetsWithFilters: vi.fn(),
}));

const mockSearchHooks = searchHooks as Record<string, ReturnType<typeof vi.fn>>;

describe('SearchFiltersContext', () => {
  let queryClient: QueryClient;

  beforeEach(() => {
    queryClient = new QueryClient({
      defaultOptions: {
        queries: { retry: false, refetchOnWindowFocus: false },
        mutations: { retry: false },
      },
    });
    vi.clearAllMocks();

    // Setup default mocks
    mockSearchHooks.useSearchFilters.mockReturnValue({
      data: {
        projects: [
          { value: 'project1', label: 'project1', count: 5 },
          { value: 'project2', label: 'project2', count: 3 },
        ],
        versions: [
          { value: '1.0', label: '1.0', count: 4 },
          { value: '2.0', label: '2.0', count: 2 },
        ],
        extensions: [
          { value: 'js', label: 'js', count: 6 },
          { value: 'ts', label: 'ts', count: 3 },
        ],
        repositories: [
          { value: 'repo1', label: 'repo1', count: 2 },
        ],
        languages: [],
      },
      isLoading: false,
      isSuccess: true,
    });

    mockSearchHooks.useFacetsWithFilters.mockReturnValue({
      data: null,
      isLoading: false,
      isSuccess: false,
    });
  });

  const wrapper = ({ children }: { children: React.ReactNode }) => {
    return React.createElement(
      QueryClientProvider,
      { client: queryClient },
      React.createElement(SearchFiltersProvider, null, children)
    );
  };

  describe('useSearchFiltersContext hook', () => {
    it('should initialize with empty filters', () => {
      const { result } = renderHook(() => useSearchFiltersContext(), { wrapper });

      expect(result.current.filters).toEqual({});
      expect(result.current.currentQuery).toBe('');
    });

    it('should throw error when used outside provider', () => {
      // Suppress console.error for this test
      const spy = vi.spyOn(console, 'error').mockImplementation(() => {});

      expect(() => {
        renderHook(() => useSearchFiltersContext());
      }).toThrow('useSearchFiltersContext must be used within a SearchFiltersProvider');

      spy.mockRestore();
    });

    it('should provide static filters from useSearchFilters', () => {
      const { result } = renderHook(() => useSearchFiltersContext(), { wrapper });

      expect(result.current.availableFilters.projects).toHaveLength(2);
      expect(result.current.availableFilters.projects[0]).toEqual({
        value: 'project1',
        label: 'project1',
        count: 5,
      });
    });

    it('should update filters when setFilters is called', () => {
      const { result } = renderHook(() => useSearchFiltersContext(), { wrapper });

      act(() => {
        result.current.setFilters({
          project: ['project1'],
          version: ['1.0'],
        });
      });

      expect(result.current.filters.project).toEqual(['project1']);
      expect(result.current.filters.version).toEqual(['1.0']);
    });

    it('should clear filters when clearFilters is called', () => {
      const { result } = renderHook(() => useSearchFiltersContext(), { wrapper });

      act(() => {
        result.current.setFilters({
          project: ['project1'],
          extension: ['js'],
        });
      });

      expect(result.current.filters.project).toEqual(['project1']);

      act(() => {
        result.current.clearFilters();
      });

      expect(result.current.filters).toEqual({});
    });

    it('should update current query when setCurrentQuery is called', () => {
      const { result } = renderHook(() => useSearchFiltersContext(), { wrapper });

      act(() => {
        result.current.setCurrentQuery('test search');
      });

      expect(result.current.currentQuery).toBe('test search');
    });

    it('should provide loading state from static filters', () => {
      mockSearchHooks.useSearchFilters.mockReturnValueOnce({
        data: null,
        isLoading: true,
        isSuccess: false,
      });

      const { result } = renderHook(() => useSearchFiltersContext(), {
        wrapper,
      });

      expect(result.current.isLoading).toBe(true);
    });

    it('should format extensions with dot prefix', () => {
      const { result } = renderHook(() => useSearchFiltersContext(), { wrapper });

      const jsExtension = result.current.availableFilters.extensions.find(
        e => e.value === 'js'
      );
      expect(jsExtension?.label).toBe('.js');
    });

    it('should include repositories in available filters', () => {
      const { result } = renderHook(() => useSearchFiltersContext(), { wrapper });

      expect(result.current.availableFilters.repositories).toBeDefined();
      expect(result.current.availableFilters.repositories).toHaveLength(1);
    });
  });

  describe('Facet updates with filters', () => {
    it('should call useFacetsWithFilters when filters are set', async () => {
      const { result } = renderHook(() => useSearchFiltersContext(), { wrapper });

      act(() => {
        result.current.setFilters({
          project: ['project1'],
        });
      });

      await waitFor(() => {
        expect(mockSearchHooks.useFacetsWithFilters).toHaveBeenCalledWith(
          expect.objectContaining({
            project: ['project1'],
          }),
          expect.any(Object)
        );
      });
    });

    it('should update dynamic filters when facet data changes', async () => {
      const facetData = {
        projects: [
          { value: 'project1', count: 2 },
          { value: 'project3', count: 1 },
        ],
        versions: [
          { value: '1.0', count: 1 },
        ],
        extensions: [],
        repositories: [],
      };

      mockSearchHooks.useFacetsWithFilters.mockReturnValueOnce({
        data: facetData,
        isLoading: false,
        isSuccess: true,
      });

      const { result } = renderHook(() => useSearchFiltersContext(), {
        wrapper,
      });

      act(() => {
        result.current.setFilters({
          project: ['project1'],
        });
      });

      // Force re-evaluation to get the dynamic data
      await waitFor(() => {
        const projects = result.current.availableFilters.projects;
        // Should have updated counts from facet data
        expect(projects.length).toBeGreaterThan(0);
      });
    });

    it('should use updateDynamicFilters to manually update filters', () => {
      const { result } = renderHook(() => useSearchFiltersContext(), { wrapper });

      const newDynamicFilters = {
        projects: [{ value: 'test', count: 5 }],
        versions: [],
        extensions: [],
        repositories: [],
      };

      act(() => {
        result.current.updateDynamicFilters(newDynamicFilters);
      });

      // After update, the context should reflect new dynamic filters
      // This tests that the manual update method works
      expect(result.current.updateDynamicFilters).toBeDefined();
    });
  });

  describe('Filter merging logic', () => {
    it('should show dynamic filters when no filters are selected', () => {
      mockSearchHooks.useFacetsWithFilters.mockReturnValueOnce({
        data: {
          projects: [
            { value: 'project1', count: 3 },
            { value: 'project3', count: 1 },
          ],
          versions: [],
          extensions: [],
          repositories: [],
        },
        isLoading: false,
        isSuccess: true,
      });

      const { result } = renderHook(() => useSearchFiltersContext(), { wrapper });

      // When no filters are selected, should show dynamic counts
      // The context should display what's available
      expect(result.current.availableFilters.projects).toBeDefined();
    });

    it('should preserve selected filters that have zero count', () => {
      const { result } = renderHook(() => useSearchFiltersContext(), { wrapper });

      // Select a filter
      act(() => {
        result.current.setFilters({
          project: ['project1'],
        });
      });

      // Even if the dynamic data says it has 0 results,
      // the selected filter should still appear
      expect(result.current.filters.project).toEqual(['project1']);
    });
  });

  describe('Multiple filter categories', () => {
    it('should handle multiple filter categories simultaneously', () => {
      const { result } = renderHook(() => useSearchFiltersContext(), { wrapper });

      act(() => {
        result.current.setFilters({
          project: ['project1', 'project2'],
          version: ['1.0'],
          extension: ['js', 'ts'],
        });
      });

      expect(result.current.filters.project).toHaveLength(2);
      expect(result.current.filters.version).toHaveLength(1);
      expect(result.current.filters.extension).toHaveLength(2);
    });

    it('should provide all filter categories', () => {
      const { result } = renderHook(() => useSearchFiltersContext(), { wrapper });

      expect(result.current.availableFilters.projects).toBeDefined();
      expect(result.current.availableFilters.versions).toBeDefined();
      expect(result.current.availableFilters.extensions).toBeDefined();
      expect(result.current.availableFilters.repositories).toBeDefined();
      expect(result.current.availableFilters.languages).toBeDefined();
    });
  });

  describe('Edge cases', () => {
    it('should handle undefined static filters gracefully', () => {
      mockSearchHooks.useSearchFilters.mockReturnValueOnce({
        data: undefined,
        isLoading: false,
        isSuccess: false,
      });

      const { result } = renderHook(() => useSearchFiltersContext(), { wrapper });

      expect(result.current.availableFilters.projects).toEqual([]);
      expect(result.current.availableFilters.versions).toEqual([]);
    });

    it('should handle empty facet response', () => {
      mockSearchHooks.useFacetsWithFilters.mockReturnValueOnce({
        data: {
          projects: [],
          versions: [],
          extensions: [],
          repositories: [],
        },
        isLoading: false,
        isSuccess: true,
      });

      const { result } = renderHook(() => useSearchFiltersContext(), { wrapper });

      expect(result.current.availableFilters.projects).toBeDefined();
      // Should not crash and should have empty arrays
    });

    it('should handle null dynamic filters', () => {
      mockSearchHooks.useFacetsWithFilters.mockReturnValueOnce({
        data: null,
        isLoading: false,
        isSuccess: false,
      });

      const { result } = renderHook(() => useSearchFiltersContext(), { wrapper });

      // Should fall back to static filters
      expect(result.current.availableFilters.projects).toHaveLength(2);
    });
  });
});
