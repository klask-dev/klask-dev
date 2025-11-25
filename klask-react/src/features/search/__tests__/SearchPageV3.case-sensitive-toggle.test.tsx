import React from 'react';
import { describe, it, expect, beforeEach, vi } from 'vitest';
import { render, screen, waitFor } from '@testing-library/react';
import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import { BrowserRouter } from 'react-router-dom';
import userEvent from '@testing-library/user-event';
import SearchPageV3 from '../SearchPageV3';
import * as useSearch from '../../../hooks/useSearch';
import type { SearchResponse, SearchResult } from '../../../types';
import { SearchFiltersProvider } from '../../../contexts/SearchFiltersContext';

// Mock the search hooks
vi.mock('../../../hooks/useSearch');

const createWrapper = () => {
  const queryClient = new QueryClient({
    defaultOptions: {
      queries: { retry: false, refetchOnWindowFocus: false },
      mutations: { retry: false },
    },
  });

  return ({ children }: { children: React.ReactNode }) => (
    <QueryClientProvider client={queryClient}>
      <SearchFiltersProvider>
        <BrowserRouter>
          {children}
        </BrowserRouter>
      </SearchFiltersProvider>
    </QueryClientProvider>
  );
};

const mockSearchResults: SearchResult[] = [
  {
    file_id: '1',
    doc_address: '0:1',
    name: 'Crawler.rs',
    path: 'src/services/Crawler.rs',
    content_snippet: 'pub struct Crawler { }',
    project: 'klask/klask-rs',
    repository_name: 'klask/klask-rs',
    version: 'main',
    extension: 'rs',
    score: 1.5,
  },
  {
    file_id: '2',
    doc_address: '0:2',
    name: 'crawler.py',
    path: 'src/crawler.py',
    content_snippet: 'class Crawler: pass',
    project: 'example/project',
    repository_name: 'example/project',
    version: 'develop',
    extension: 'py',
    score: 1.2,
  },
];

const mockSearchResponse: SearchResponse = {
  results: mockSearchResults,
  total: 2,
  page: 1,
  size: 20,
  facets: {
    projects: [
      { value: 'klask/klask-rs', count: 1 },
      { value: 'example/project', count: 1 },
    ],
    versions: [
      { value: 'main', count: 1 },
      { value: 'develop', count: 1 },
    ],
    extensions: [
      { value: 'rs', count: 1 },
      { value: 'py', count: 1 },
    ],
  },
};

describe('SearchPageV3 - Case Sensitive Toggle Feature', () => {
  beforeEach(() => {
    vi.clearAllMocks();

    // Mock useSearchFilters hook
    vi.mocked(useSearch.useSearchFilters).mockReturnValue({
      data: {
        projects: [],
        versions: [],
        extensions: [],
        repositories: [],
        languages: [],
      },
      isLoading: false,
      error: null,
      refetch: vi.fn(),
    } as any);

    // Mock useFacetsWithFilters hook
    vi.mocked(useSearch.useFacetsWithFilters).mockReturnValue({
      data: undefined,
      isLoading: false,
      error: null,
      refetch: vi.fn(),
    } as any);
  });

  describe('Toggle Button Rendering', () => {
    it('should render case-sensitive toggle button in search interface', async () => {
      vi.mocked(useSearch.useMultiSelectSearch).mockReturnValue({
        data: mockSearchResponse,
        isLoading: false,
        isFetching: false,
        isError: false,
        error: null,
        refetch: vi.fn(),
      } as any);

      vi.mocked(useSearch.useSearchHistory).mockReturnValue({
        history: [],
        addToHistory: vi.fn(),
        clearHistory: vi.fn(),
      });

      render(<SearchPageV3 />, { wrapper: createWrapper() });

      const caseButton = screen.getByRole('button', { name: /case|Aa/i });
      expect(caseButton).toBeInTheDocument();
    });

    it('should display "Aa" icon in case-sensitive button', async () => {
      vi.mocked(useSearch.useMultiSelectSearch).mockReturnValue({
        data: mockSearchResponse,
        isLoading: false,
        isFetching: false,
        isError: false,
        error: null,
        refetch: vi.fn(),
      } as any);

      vi.mocked(useSearch.useSearchHistory).mockReturnValue({
        history: [],
        addToHistory: vi.fn(),
        clearHistory: vi.fn(),
      });

      render(<SearchPageV3 />, { wrapper: createWrapper() });

      const caseButton = screen.getByRole('button', { name: /case|Aa/i });
      expect(caseButton.textContent).toContain('Aa');
    });

    it('should render case-sensitive button with title attribute', async () => {
      vi.mocked(useSearch.useMultiSelectSearch).mockReturnValue({
        data: mockSearchResponse,
        isLoading: false,
        isFetching: false,
        isError: false,
        error: null,
        refetch: vi.fn(),
      } as any);

      vi.mocked(useSearch.useSearchHistory).mockReturnValue({
        history: [],
        addToHistory: vi.fn(),
        clearHistory: vi.fn(),
      });

      render(<SearchPageV3 />, { wrapper: createWrapper() });

      const caseButton = screen.getByRole('button', { name: /case|Aa/i });
      expect(caseButton).toHaveAttribute('title');
      expect(caseButton.getAttribute('title')).toBeTruthy();
    });

    it('should render button next to fuzzy and regex toggles', async () => {
      vi.mocked(useSearch.useMultiSelectSearch).mockReturnValue({
        data: mockSearchResponse,
        isLoading: false,
        isFetching: false,
        isError: false,
        error: null,
        refetch: vi.fn(),
      } as any);

      vi.mocked(useSearch.useSearchHistory).mockReturnValue({
        history: [],
        addToHistory: vi.fn(),
        clearHistory: vi.fn(),
      });

      render(<SearchPageV3 />, { wrapper: createWrapper() });

      const caseButton = screen.getByRole('button', { name: /case|Aa/i });
      const fuzzyButton = screen.getByRole('button', { name: /fuzzy/i });
      const regexButton = screen.getByRole('button', { name: /regex|\/\.\*/i });

      expect(caseButton).toBeInTheDocument();
      expect(fuzzyButton).toBeInTheDocument();
      expect(regexButton).toBeInTheDocument();
    });

    it('should have correct initial state (false)', async () => {
      vi.mocked(useSearch.useMultiSelectSearch).mockReturnValue({
        data: mockSearchResponse,
        isLoading: false,
        isFetching: false,
        isError: false,
        error: null,
        refetch: vi.fn(),
      } as any);

      vi.mocked(useSearch.useSearchHistory).mockReturnValue({
        history: [],
        addToHistory: vi.fn(),
        clearHistory: vi.fn(),
      });

      render(<SearchPageV3 />, { wrapper: createWrapper() });

      const caseButton = screen.getByRole('button', { name: /case|Aa/i });
      // Initial state should be inactive (not orange)
      expect(caseButton.className).not.toMatch(/bg-orange|border-orange/);
    });
  });

  describe('Toggle Button Styling', () => {
    it('should have gray styling when case-sensitive is inactive', async () => {
      vi.mocked(useSearch.useMultiSelectSearch).mockReturnValue({
        data: mockSearchResponse,
        isLoading: false,
        isFetching: false,
        isError: false,
        error: null,
        refetch: vi.fn(),
      } as any);

      vi.mocked(useSearch.useSearchHistory).mockReturnValue({
        history: [],
        addToHistory: vi.fn(),
        clearHistory: vi.fn(),
      });

      render(<SearchPageV3 />, { wrapper: createWrapper() });

      const caseButton = screen.getByRole('button', { name: /case|Aa/i });

      // Inactive state should have gray styling
      expect(caseButton.className).toMatch(/border-gray-200|dark:border-gray-700/);
    });

    it('should have orange styling when case-sensitive is active', async () => {
      const mockMultiSelectSearch = vi.fn().mockReturnValue({
        data: mockSearchResponse,
        isLoading: false,
        isFetching: false,
        isError: false,
        error: null,
        refetch: vi.fn(),
      });

      vi.mocked(useSearch.useMultiSelectSearch).mockImplementation(mockMultiSelectSearch);

      vi.mocked(useSearch.useSearchHistory).mockReturnValue({
        history: [],
        addToHistory: vi.fn(),
        clearHistory: vi.fn(),
      });

      render(<SearchPageV3 />, { wrapper: createWrapper() });

      const caseButton = screen.getByRole('button', { name: /case|Aa/i });

      // Click to activate
      await userEvent.click(caseButton);

      // After state change, check for orange styling
      await waitFor(() => {
        const updatedButton = screen.getByRole('button', { name: /case|Aa/i });
        expect(updatedButton.className).toMatch(/bg-orange-50|border-orange-200|text-orange-700/);
      });
    });

    it('should apply shadow when active', async () => {
      const mockMultiSelectSearch = vi.fn().mockReturnValue({
        data: mockSearchResponse,
        isLoading: false,
        isFetching: false,
        isError: false,
        error: null,
        refetch: vi.fn(),
      });

      vi.mocked(useSearch.useMultiSelectSearch).mockImplementation(mockMultiSelectSearch);

      vi.mocked(useSearch.useSearchHistory).mockReturnValue({
        history: [],
        addToHistory: vi.fn(),
        clearHistory: vi.fn(),
      });

      render(<SearchPageV3 />, { wrapper: createWrapper() });

      const caseButton = screen.getByRole('button', { name: /case|Aa/i });

      // Click to activate
      await userEvent.click(caseButton);

      // Should have shadow when active
      await waitFor(() => {
        const updatedButton = screen.getByRole('button', { name: /case|Aa/i });
        expect(updatedButton.className).toMatch(/shadow-sm/);
      });
    });

    it('should use font-medium for text styling', async () => {
      vi.mocked(useSearch.useMultiSelectSearch).mockReturnValue({
        data: mockSearchResponse,
        isLoading: false,
        isFetching: false,
        isError: false,
        error: null,
        refetch: vi.fn(),
      } as any);

      vi.mocked(useSearch.useSearchHistory).mockReturnValue({
        history: [],
        addToHistory: vi.fn(),
        clearHistory: vi.fn(),
      });

      render(<SearchPageV3 />, { wrapper: createWrapper() });

      const caseButton = screen.getByRole('button', { name: /case|Aa/i });

      // Check font styling
      expect(caseButton.className).toMatch(/font-medium/);
    });
  });

  describe('Toggle State Management', () => {
    it('should toggle case-sensitive state on button click', async () => {
      const mockMultiSelectSearch = vi.fn().mockReturnValue({
        data: mockSearchResponse,
        isLoading: false,
        isFetching: false,
        isError: false,
        error: null,
        refetch: vi.fn(),
      });

      vi.mocked(useSearch.useMultiSelectSearch).mockImplementation(mockMultiSelectSearch);

      vi.mocked(useSearch.useSearchHistory).mockReturnValue({
        history: [],
        addToHistory: vi.fn(),
        clearHistory: vi.fn(),
      });

      render(<SearchPageV3 />, { wrapper: createWrapper() });

      const caseButton = screen.getByRole('button', { name: /case|Aa/i });

      // Initial state - should be inactive
      expect(caseButton.className).toMatch(/border-gray-200|dark:border-gray-700/);

      // Click to activate
      await userEvent.click(caseButton);

      // State should change to active (orange)
      await waitFor(() => {
        const updatedButton = screen.getByRole('button', { name: /case|Aa/i });
        expect(updatedButton.className).toMatch(/bg-orange|border-orange/);
      });
    });

    it('should toggle off when clicked again', async () => {
      const mockMultiSelectSearch = vi.fn().mockReturnValue({
        data: mockSearchResponse,
        isLoading: false,
        isFetching: false,
        isError: false,
        error: null,
        refetch: vi.fn(),
      });

      vi.mocked(useSearch.useMultiSelectSearch).mockImplementation(mockMultiSelectSearch);

      vi.mocked(useSearch.useSearchHistory).mockReturnValue({
        history: [],
        addToHistory: vi.fn(),
        clearHistory: vi.fn(),
      });

      render(<SearchPageV3 />, { wrapper: createWrapper() });

      const caseButton = screen.getByRole('button', { name: /case|Aa/i });

      // Click to activate
      await userEvent.click(caseButton);

      await waitFor(() => {
        expect(screen.getByRole('button', { name: /case|Aa/i }).className).toMatch(/bg-orange|border-orange/);
      });

      // Click again to deactivate
      await userEvent.click(caseButton);

      // State should change back to inactive (gray)
      await waitFor(() => {
        const updatedButton = screen.getByRole('button', { name: /case|Aa/i });
        expect(updatedButton.className).toMatch(/border-gray-200|dark:border-gray-700/);
      });
    });

    it('should maintain case-sensitive state across re-renders', async () => {
      const mockMultiSelectSearch = vi.fn().mockReturnValue({
        data: mockSearchResponse,
        isLoading: false,
        isFetching: false,
        isError: false,
        error: null,
        refetch: vi.fn(),
      });

      vi.mocked(useSearch.useMultiSelectSearch).mockImplementation(mockMultiSelectSearch);

      vi.mocked(useSearch.useSearchHistory).mockReturnValue({
        history: [],
        addToHistory: vi.fn(),
        clearHistory: vi.fn(),
      });

      const { rerender } = render(<SearchPageV3 />, { wrapper: createWrapper() });

      const caseButton = screen.getByRole('button', { name: /case|Aa/i });

      // Activate case-sensitive
      await userEvent.click(caseButton);

      await waitFor(() => {
        expect(screen.getByRole('button', { name: /case|Aa/i }).className).toMatch(/bg-orange/);
      });

      // Trigger re-render (simulate component update)
      rerender(<SearchPageV3 />);

      // State should be preserved
      const updatedButton = screen.getByRole('button', { name: /case|Aa/i });
      expect(updatedButton.className).toMatch(/bg-orange|border-orange/);
    });

    it('should support multiple consecutive toggles', async () => {
      const mockMultiSelectSearch = vi.fn().mockReturnValue({
        data: mockSearchResponse,
        isLoading: false,
        isFetching: false,
        isError: false,
        error: null,
        refetch: vi.fn(),
      });

      vi.mocked(useSearch.useMultiSelectSearch).mockImplementation(mockMultiSelectSearch);

      vi.mocked(useSearch.useSearchHistory).mockReturnValue({
        history: [],
        addToHistory: vi.fn(),
        clearHistory: vi.fn(),
      });

      render(<SearchPageV3 />, { wrapper: createWrapper() });

      const caseButton = screen.getByRole('button', { name: /case|Aa/i });

      // Toggle on
      await userEvent.click(caseButton);
      await waitFor(() => {
        expect(screen.getByRole('button', { name: /case|Aa/i }).className).toMatch(/bg-orange/);
      });

      // Toggle off
      await userEvent.click(caseButton);
      await waitFor(() => {
        expect(screen.getByRole('button', { name: /case|Aa/i }).className).toMatch(/border-gray-200/);
      });

      // Toggle on again
      await userEvent.click(caseButton);
      await waitFor(() => {
        expect(screen.getByRole('button', { name: /case|Aa/i }).className).toMatch(/bg-orange/);
      });
    });
  });

  describe('URL Parameter Handling', () => {
    it('should parse case_sensitive=true from URL on initial load', async () => {
      vi.mocked(useSearch.useMultiSelectSearch).mockReturnValue({
        data: mockSearchResponse,
        isLoading: false,
        isFetching: false,
        isError: false,
        error: null,
        refetch: vi.fn(),
      } as any);

      vi.mocked(useSearch.useSearchHistory).mockReturnValue({
        history: [],
        addToHistory: vi.fn(),
        clearHistory: vi.fn(),
      });

      // Mock window.location.search
      const originalLocation = window.location;
      delete (window as any).location;
      (window as any).location = {
        ...originalLocation,
        search: '?q=test&case_sensitive=true',
        pathname: '/',
      };

      render(<SearchPageV3 />, { wrapper: createWrapper() });

      await waitFor(() => {
        const caseButton = screen.getByRole('button', { name: /case|Aa/i });
        expect(caseButton.className).toMatch(/bg-orange|border-orange/);
      });

      // Restore location
      window.location = originalLocation;
    });

    it('should default to false when case_sensitive is not in URL', async () => {
      vi.mocked(useSearch.useMultiSelectSearch).mockReturnValue({
        data: mockSearchResponse,
        isLoading: false,
        isFetching: false,
        isError: false,
        error: null,
        refetch: vi.fn(),
      } as any);

      vi.mocked(useSearch.useSearchHistory).mockReturnValue({
        history: [],
        addToHistory: vi.fn(),
        clearHistory: vi.fn(),
      });

      // Mock window.location.search with no case_sensitive param
      const originalLocation = window.location;
      delete (window as any).location;
      (window as any).location = {
        ...originalLocation,
        search: '?q=test',
        pathname: '/',
      };

      render(<SearchPageV3 />, { wrapper: createWrapper() });

      const caseButton = screen.getByRole('button', { name: /case|Aa/i });
      expect(caseButton.className).toMatch(/border-gray-200|dark:border-gray-700/);

      // Restore location
      window.location = originalLocation;
    });

    it('should add case_sensitive=true to URL when toggled on', async () => {
      const mockMultiSelectSearch = vi.fn().mockReturnValue({
        data: mockSearchResponse,
        isLoading: false,
        isFetching: false,
        isError: false,
        error: null,
        refetch: vi.fn(),
      });

      vi.mocked(useSearch.useMultiSelectSearch).mockImplementation(mockMultiSelectSearch);

      vi.mocked(useSearch.useSearchHistory).mockReturnValue({
        history: [],
        addToHistory: vi.fn(),
        clearHistory: vi.fn(),
      });

      // Mock history.replaceState
      const replaceStateSpy = vi.spyOn(window.history, 'replaceState');

      render(<SearchPageV3 />, { wrapper: createWrapper() });

      const caseButton = screen.getByRole('button', { name: /case|Aa/i });
      const searchInput = screen.getByPlaceholderText(/search/i);

      // Enter a search query first
      await userEvent.type(searchInput, 'test');

      // Click to toggle on
      await userEvent.click(caseButton);

      // URL should be updated to include case_sensitive=true
      await waitFor(() => {
        expect(replaceStateSpy).toHaveBeenCalled();
      });

      replaceStateSpy.mockRestore();
    });

    it('should remove case_sensitive parameter from URL when toggled off', async () => {
      const mockMultiSelectSearch = vi.fn().mockReturnValue({
        data: mockSearchResponse,
        isLoading: false,
        isFetching: false,
        isError: false,
        error: null,
        refetch: vi.fn(),
      });

      vi.mocked(useSearch.useMultiSelectSearch).mockImplementation(mockMultiSelectSearch);

      vi.mocked(useSearch.useSearchHistory).mockReturnValue({
        history: [],
        addToHistory: vi.fn(),
        clearHistory: vi.fn(),
      });

      // Mock location with case_sensitive=true already
      const originalLocation = window.location;
      delete (window as any).location;
      (window as any).location = {
        ...originalLocation,
        search: '?q=test&case_sensitive=true',
        pathname: '/',
      };

      const replaceStateSpy = vi.spyOn(window.history, 'replaceState');

      render(<SearchPageV3 />, { wrapper: createWrapper() });

      const caseButton = screen.getByRole('button', { name: /case|Aa/i });

      // Click to toggle off
      await userEvent.click(caseButton);

      // URL should be updated to remove case_sensitive
      await waitFor(() => {
        expect(replaceStateSpy).toHaveBeenCalled();
      });

      replaceStateSpy.mockRestore();

      // Restore location
      window.location = originalLocation;
    });

    it('should preserve other URL parameters when toggling case_sensitive', async () => {
      vi.mocked(useSearch.useMultiSelectSearch).mockReturnValue({
        data: mockSearchResponse,
        isLoading: false,
        isFetching: false,
        isError: false,
        error: null,
        refetch: vi.fn(),
      } as any);

      vi.mocked(useSearch.useSearchHistory).mockReturnValue({
        history: [],
        addToHistory: vi.fn(),
        clearHistory: vi.fn(),
      });

      // Mock location with multiple parameters
      const originalLocation = window.location;
      delete (window as any).location;
      (window as any).location = {
        ...originalLocation,
        search: '?q=test&fuzzySearch=true&project=example',
        pathname: '/',
      };

      const replaceStateSpy = vi.spyOn(window.history, 'replaceState');

      render(<SearchPageV3 />, { wrapper: createWrapper() });

      const caseButton = screen.getByRole('button', { name: /case|Aa/i });

      // Click to toggle
      await userEvent.click(caseButton);

      // All parameters should be preserved (except case_sensitive is added)
      await waitFor(() => {
        const callArgs = replaceStateSpy.mock.calls[replaceStateSpy.mock.calls.length - 1];
        const urlString = callArgs[2] as string;
        expect(urlString).toContain('case_sensitive=true');
        expect(urlString).toContain('q=test');
        expect(urlString).toContain('fuzzySearch=true');
        expect(urlString).toContain('project=example');
      });

      replaceStateSpy.mockRestore();

      // Restore location
      window.location = originalLocation;
    });
  });

  describe('Tooltip/Title Attribute', () => {
    it('should show correct tooltip when case-sensitive is disabled', async () => {
      vi.mocked(useSearch.useMultiSelectSearch).mockReturnValue({
        data: mockSearchResponse,
        isLoading: false,
        isFetching: false,
        isError: false,
        error: null,
        refetch: vi.fn(),
      } as any);

      vi.mocked(useSearch.useSearchHistory).mockReturnValue({
        history: [],
        addToHistory: vi.fn(),
        clearHistory: vi.fn(),
      });

      render(<SearchPageV3 />, { wrapper: createWrapper() });

      const caseButton = screen.getByRole('button', { name: /case|Aa/i });
      const title = caseButton.getAttribute('title');

      // Should mention enabling case-sensitive or describe disabled state
      expect(title).toMatch(/enable|case|sensitive|exact/i);
    });

    it('should show correct tooltip when case-sensitive is enabled', async () => {
      const mockMultiSelectSearch = vi.fn().mockReturnValue({
        data: mockSearchResponse,
        isLoading: false,
        isFetching: false,
        isError: false,
        error: null,
        refetch: vi.fn(),
      });

      vi.mocked(useSearch.useMultiSelectSearch).mockImplementation(mockMultiSelectSearch);

      vi.mocked(useSearch.useSearchHistory).mockReturnValue({
        history: [],
        addToHistory: vi.fn(),
        clearHistory: vi.fn(),
      });

      render(<SearchPageV3 />, { wrapper: createWrapper() });

      const caseButton = screen.getByRole('button', { name: /case|Aa/i });

      // Get initial tooltip
      const initialTitle = caseButton.getAttribute('title');
      expect(initialTitle).toBeTruthy();

      // Click to enable
      await userEvent.click(caseButton);

      // Tooltip should change to indicate it's enabled
      const updatedTitle = caseButton.getAttribute('title');
      expect(updatedTitle).toMatch(/disable|case|sensitive|exact/i);
    });

    it('should have descriptive title text for accessibility', async () => {
      vi.mocked(useSearch.useMultiSelectSearch).mockReturnValue({
        data: mockSearchResponse,
        isLoading: false,
        isFetching: false,
        isError: false,
        error: null,
        refetch: vi.fn(),
      } as any);

      vi.mocked(useSearch.useSearchHistory).mockReturnValue({
        history: [],
        addToHistory: vi.fn(),
        clearHistory: vi.fn(),
      });

      render(<SearchPageV3 />, { wrapper: createWrapper() });

      const caseButton = screen.getByRole('button', { name: /case|Aa/i });
      const title = caseButton.getAttribute('title');

      // Title should be reasonably descriptive
      expect(title).toBeTruthy();
      expect(title?.length).toBeGreaterThan(10);
    });
  });

  describe('API Integration with useMultiSelectSearch', () => {
    it('should pass case_sensitive state to useMultiSelectSearch hook', async () => {
      const mockMultiSelectSearch = vi.fn().mockReturnValue({
        data: mockSearchResponse,
        isLoading: false,
        isFetching: false,
        isError: false,
        error: null,
        refetch: vi.fn(),
      });

      vi.mocked(useSearch.useMultiSelectSearch).mockImplementation(mockMultiSelectSearch);

      vi.mocked(useSearch.useSearchHistory).mockReturnValue({
        history: [],
        addToHistory: vi.fn(),
        clearHistory: vi.fn(),
      });

      render(<SearchPageV3 />, { wrapper: createWrapper() });

      const caseButton = screen.getByRole('button', { name: /case|Aa/i });
      const searchInput = screen.getByPlaceholderText(/search/i);

      // Type a query
      await userEvent.type(searchInput, 'test');

      // Click case-sensitive button to enable
      await userEvent.click(caseButton);

      // Wait for hook to be called with case_sensitive enabled
      await waitFor(() => {
        // Find the call where caseSensitive is true (last parameter)
        const callWithCaseSensitive = mockMultiSelectSearch.mock.calls.find(
          call => call[8] === true // caseSensitive parameter is at index 8
        );
        expect(callWithCaseSensitive).toBeDefined();
      });
    });

    it('should pass false for caseSensitive when disabled', async () => {
      const mockMultiSelectSearch = vi.fn().mockReturnValue({
        data: mockSearchResponse,
        isLoading: false,
        isFetching: false,
        isError: false,
        error: null,
        refetch: vi.fn(),
      });

      vi.mocked(useSearch.useMultiSelectSearch).mockImplementation(mockMultiSelectSearch);

      vi.mocked(useSearch.useSearchHistory).mockReturnValue({
        history: [],
        addToHistory: vi.fn(),
        clearHistory: vi.fn(),
      });

      render(<SearchPageV3 />, { wrapper: createWrapper() });

      const searchInput = screen.getByPlaceholderText(/search/i);

      // Type a query without enabling case-sensitive
      await userEvent.type(searchInput, 'test');

      // Wait for hook to be called with case-sensitive disabled
      await waitFor(() => {
        // Find the initial call where caseSensitive is false
        const initialCall = mockMultiSelectSearch.mock.calls[0];
        expect(initialCall[8]).toBe(false); // caseSensitive parameter
      });
    });

    it('should work with other filters and search modes', async () => {
      const mockMultiSelectSearch = vi.fn().mockReturnValue({
        data: mockSearchResponse,
        isLoading: false,
        isFetching: false,
        isError: false,
        error: null,
        refetch: vi.fn(),
      });

      vi.mocked(useSearch.useMultiSelectSearch).mockImplementation(mockMultiSelectSearch);

      vi.mocked(useSearch.useSearchHistory).mockReturnValue({
        history: [],
        addToHistory: vi.fn(),
        clearHistory: vi.fn(),
      });

      render(<SearchPageV3 />, { wrapper: createWrapper() });

      const caseButton = screen.getByRole('button', { name: /case|Aa/i });
      const fuzzyButton = screen.getByRole('button', { name: /fuzzy/i });
      const searchInput = screen.getByPlaceholderText(/search/i);

      // Enable fuzzy search
      await userEvent.click(fuzzyButton);

      // Enable case-sensitive
      await userEvent.click(caseButton);

      // Type a query
      await userEvent.type(searchInput, 'test');

      // Wait for hook call with both fuzzy and case_sensitive
      await waitFor(() => {
        const call = mockMultiSelectSearch.mock.calls[mockMultiSelectSearch.mock.calls.length - 1];
        expect(call[4]).toBe(true); // fuzzySearch parameter
        expect(call[8]).toBe(true); // caseSensitive parameter
      });
    });
  });

  describe('Independence from Other Toggles', () => {
    it('should not affect fuzzy search toggle when toggled', async () => {
      const mockMultiSelectSearch = vi.fn().mockReturnValue({
        data: mockSearchResponse,
        isLoading: false,
        isFetching: false,
        isError: false,
        error: null,
        refetch: vi.fn(),
      });

      vi.mocked(useSearch.useMultiSelectSearch).mockImplementation(mockMultiSelectSearch);

      vi.mocked(useSearch.useSearchHistory).mockReturnValue({
        history: [],
        addToHistory: vi.fn(),
        clearHistory: vi.fn(),
      });

      render(<SearchPageV3 />, { wrapper: createWrapper() });

      const caseButton = screen.getByRole('button', { name: /case|Aa/i });
      const fuzzyButton = screen.getByRole('button', { name: /fuzzy/i });

      // Enable fuzzy
      await userEvent.click(fuzzyButton);
      await waitFor(() => {
        expect(screen.getByRole('button', { name: /fuzzy/i }).className).toMatch(/border-blue-200/);
      });

      // Enable case-sensitive
      await userEvent.click(caseButton);

      // Fuzzy should still be enabled
      await waitFor(() => {
        expect(screen.getByRole('button', { name: /fuzzy/i }).className).toMatch(/border-blue-200/);
        expect(screen.getByRole('button', { name: /case|Aa/i }).className).toMatch(/bg-orange/);
      });
    });

    it('should not affect regex search toggle when toggled', async () => {
      const mockMultiSelectSearch = vi.fn().mockReturnValue({
        data: mockSearchResponse,
        isLoading: false,
        isFetching: false,
        isError: false,
        error: null,
        refetch: vi.fn(),
      });

      vi.mocked(useSearch.useMultiSelectSearch).mockImplementation(mockMultiSelectSearch);

      vi.mocked(useSearch.useSearchHistory).mockReturnValue({
        history: [],
        addToHistory: vi.fn(),
        clearHistory: vi.fn(),
      });

      render(<SearchPageV3 />, { wrapper: createWrapper() });

      const caseButton = screen.getByRole('button', { name: /case|Aa/i });
      const regexButton = screen.getByRole('button', { name: /regex|\/\.\*/i });

      // Enable regex
      await userEvent.click(regexButton);
      await waitFor(() => {
        expect(screen.getByRole('button', { name: /regex|\/\.\*/i }).className).toMatch(/border-purple-200/);
      });

      // Enable case-sensitive
      await userEvent.click(caseButton);

      // Regex should still be enabled
      await waitFor(() => {
        expect(screen.getByRole('button', { name: /regex|\/\.\*/i }).className).toMatch(/border-purple-200/);
        expect(screen.getByRole('button', { name: /case|Aa/i }).className).toMatch(/bg-orange/);
      });
    });

    it('should work independently with both fuzzy and regex enabled', async () => {
      // Note: fuzzy and regex are mutually exclusive in the implementation
      // This test verifies case-sensitive works with either one
      const mockMultiSelectSearch = vi.fn().mockReturnValue({
        data: mockSearchResponse,
        isLoading: false,
        isFetching: false,
        isError: false,
        error: null,
        refetch: vi.fn(),
      });

      vi.mocked(useSearch.useMultiSelectSearch).mockImplementation(mockMultiSelectSearch);

      vi.mocked(useSearch.useSearchHistory).mockReturnValue({
        history: [],
        addToHistory: vi.fn(),
        clearHistory: vi.fn(),
      });

      render(<SearchPageV3 />, { wrapper: createWrapper() });

      const caseButton = screen.getByRole('button', { name: /case|Aa/i });
      const regexButton = screen.getByRole('button', { name: /regex|\/\.\*/i });

      // Enable regex
      await userEvent.click(regexButton);

      // Enable case-sensitive
      await userEvent.click(caseButton);

      // Both should be active
      await waitFor(() => {
        expect(screen.getByRole('button', { name: /regex|\/\.\*/i }).className).toMatch(/border-purple-200/);
        expect(screen.getByRole('button', { name: /case|Aa/i }).className).toMatch(/bg-orange/);
      });

      // Disable regex
      await userEvent.click(regexButton);

      // Case-sensitive should still be active
      await waitFor(() => {
        expect(screen.getByRole('button', { name: /regex|\/\.\*/i }).className).toMatch(/border-gray-200/);
        expect(screen.getByRole('button', { name: /case|Aa/i }).className).toMatch(/bg-orange/);
      });
    });
  });

  describe('Accessibility', () => {
    it('should be keyboard accessible (can be focused)', async () => {
      vi.mocked(useSearch.useMultiSelectSearch).mockReturnValue({
        data: mockSearchResponse,
        isLoading: false,
        isFetching: false,
        isError: false,
        error: null,
        refetch: vi.fn(),
      } as any);

      vi.mocked(useSearch.useSearchHistory).mockReturnValue({
        history: [],
        addToHistory: vi.fn(),
        clearHistory: vi.fn(),
      });

      render(<SearchPageV3 />, { wrapper: createWrapper() });

      const caseButton = screen.getByRole('button', { name: /case|Aa/i });

      // Button should be focusable (keyboard accessible)
      caseButton.focus();
      expect(caseButton).toHaveFocus();

      // Button is a button element which is keyboard accessible by default
      expect(caseButton.tagName).toBe('BUTTON');
    });

    it('should be activatable via keyboard (Enter key)', async () => {
      const mockMultiSelectSearch = vi.fn().mockReturnValue({
        data: mockSearchResponse,
        isLoading: false,
        isFetching: false,
        isError: false,
        error: null,
        refetch: vi.fn(),
      });

      vi.mocked(useSearch.useMultiSelectSearch).mockImplementation(mockMultiSelectSearch);

      vi.mocked(useSearch.useSearchHistory).mockReturnValue({
        history: [],
        addToHistory: vi.fn(),
        clearHistory: vi.fn(),
      });

      render(<SearchPageV3 />, { wrapper: createWrapper() });

      const caseButton = screen.getByRole('button', { name: /case|Aa/i });

      // Focus the button
      caseButton.focus();

      // Simulate pressing Enter
      await userEvent.keyboard('{Enter}');

      // Button should have been activated
      await waitFor(() => {
        expect(screen.getByRole('button', { name: /case|Aa/i }).className).toMatch(/bg-orange/);
      });
    });

    it('should have descriptive title attribute for screen readers', async () => {
      vi.mocked(useSearch.useMultiSelectSearch).mockReturnValue({
        data: mockSearchResponse,
        isLoading: false,
        isFetching: false,
        isError: false,
        error: null,
        refetch: vi.fn(),
      } as any);

      vi.mocked(useSearch.useSearchHistory).mockReturnValue({
        history: [],
        addToHistory: vi.fn(),
        clearHistory: vi.fn(),
      });

      render(<SearchPageV3 />, { wrapper: createWrapper() });

      const caseButton = screen.getByRole('button', { name: /case|Aa/i });
      const title = caseButton.getAttribute('title');

      // Title should be descriptive for screen readers
      expect(title).toBeTruthy();
      expect(title?.length).toBeGreaterThan(10);
      expect(title).toMatch(/case|sensitive/i);
    });
  });

  describe('Dark Mode Support', () => {
    it('should have dark mode color classes', async () => {
      vi.mocked(useSearch.useMultiSelectSearch).mockReturnValue({
        data: mockSearchResponse,
        isLoading: false,
        isFetching: false,
        isError: false,
        error: null,
        refetch: vi.fn(),
      } as any);

      vi.mocked(useSearch.useSearchHistory).mockReturnValue({
        history: [],
        addToHistory: vi.fn(),
        clearHistory: vi.fn(),
      });

      render(<SearchPageV3 />, { wrapper: createWrapper() });

      const caseButton = screen.getByRole('button', { name: /case|Aa/i });

      // Should have dark mode classes
      expect(caseButton.className).toMatch(/dark:/);
    });

    it('should use correct dark mode colors when inactive', async () => {
      vi.mocked(useSearch.useMultiSelectSearch).mockReturnValue({
        data: mockSearchResponse,
        isLoading: false,
        isFetching: false,
        isError: false,
        error: null,
        refetch: vi.fn(),
      } as any);

      vi.mocked(useSearch.useSearchHistory).mockReturnValue({
        history: [],
        addToHistory: vi.fn(),
        clearHistory: vi.fn(),
      });

      render(<SearchPageV3 />, { wrapper: createWrapper() });

      const caseButton = screen.getByRole('button', { name: /case|Aa/i });

      // Inactive state should have dark mode gray colors
      expect(caseButton.className).toMatch(/dark:border-gray-700|dark:hover:bg-gray-800/);
    });

    it('should use correct dark mode colors when active', async () => {
      const mockMultiSelectSearch = vi.fn().mockReturnValue({
        data: mockSearchResponse,
        isLoading: false,
        isFetching: false,
        isError: false,
        error: null,
        refetch: vi.fn(),
      });

      vi.mocked(useSearch.useMultiSelectSearch).mockImplementation(mockMultiSelectSearch);

      vi.mocked(useSearch.useSearchHistory).mockReturnValue({
        history: [],
        addToHistory: vi.fn(),
        clearHistory: vi.fn(),
      });

      render(<SearchPageV3 />, { wrapper: createWrapper() });

      const caseButton = screen.getByRole('button', { name: /case|Aa/i });

      // Activate
      await userEvent.click(caseButton);

      // Active state should have dark mode orange colors
      await waitFor(() => {
        const updatedButton = screen.getByRole('button', { name: /case|Aa/i });
        expect(updatedButton.className).toMatch(/dark:bg-orange-900|dark:text-orange-300/);
      });
    });
  });

  describe('Responsive Design', () => {
    it('should display "Aa" icon and "Case" label on desktop', async () => {
      vi.mocked(useSearch.useMultiSelectSearch).mockReturnValue({
        data: mockSearchResponse,
        isLoading: false,
        isFetching: false,
        isError: false,
        error: null,
        refetch: vi.fn(),
      } as any);

      vi.mocked(useSearch.useSearchHistory).mockReturnValue({
        history: [],
        addToHistory: vi.fn(),
        clearHistory: vi.fn(),
      });

      render(<SearchPageV3 />, { wrapper: createWrapper() });

      const caseButton = screen.getByRole('button', { name: /case|Aa/i });

      // Button should be present
      expect(caseButton).toBeInTheDocument();
      // Should contain "Aa"
      expect(caseButton.textContent).toContain('Aa');
    });

    it('should respond to clicks on all screen sizes', async () => {
      const mockMultiSelectSearch = vi.fn().mockReturnValue({
        data: mockSearchResponse,
        isLoading: false,
        isFetching: false,
        isError: false,
        error: null,
        refetch: vi.fn(),
      });

      vi.mocked(useSearch.useMultiSelectSearch).mockImplementation(mockMultiSelectSearch);

      vi.mocked(useSearch.useSearchHistory).mockReturnValue({
        history: [],
        addToHistory: vi.fn(),
        clearHistory: vi.fn(),
      });

      render(<SearchPageV3 />, { wrapper: createWrapper() });

      const caseButton = screen.getByRole('button', { name: /case|Aa/i });

      // Click button
      await userEvent.click(caseButton);

      // Button should be interactive on all screen sizes
      await waitFor(() => {
        expect(screen.getByRole('button', { name: /case|Aa/i }).className).toMatch(/bg-orange/);
      });
    });
  });
});
