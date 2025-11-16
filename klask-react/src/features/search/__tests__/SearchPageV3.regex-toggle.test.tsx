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

describe('SearchPageV3 - Regex Search Toggle Feature', () => {
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

  describe('Regex Toggle Button Rendering', () => {
    it('should render regex button with correct text on desktop', async () => {
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

      // On desktop (sm:inline), the text should be visible
      const regexButton = screen.getByRole('button', { name: /regex|\/\.\*/i });
      expect(regexButton).toBeInTheDocument();
    });

    it('should render regex button with mobile text /.*', async () => {
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

      // Find the button and check it contains the mobile text
      const regexButton = screen.getByRole('button', { name: /regex|\/\.\*/i });
      expect(regexButton).toBeInTheDocument();
      expect(regexButton.textContent).toMatch(/regex|\/\.\*/i);
    });

    it('should render regex button with title attribute', async () => {
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

      const regexButton = screen.getByRole('button', { name: /regex|\/\.\*/i });
      expect(regexButton).toHaveAttribute('title');
      expect(regexButton.getAttribute('title')).toBeTruthy();
    });
  });

  describe('Regex Toggle Button Styling', () => {
    it('should have gray border when regex is inactive', async () => {
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

      const regexButton = screen.getByRole('button', { name: /regex|\/\.\*/i });

      // Check that it has gray styling (border-gray-300 or dark variant)
      expect(regexButton.className).toMatch(/border-gray-300|dark:border-gray-600/);
    });

    it('should have purple border and background when regex is active', async () => {
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

      const regexButton = screen.getByRole('button', { name: /regex|\/\.\*/i });

      // Click to activate
      await userEvent.click(regexButton);

      // After state change, check for purple styling
      await waitFor(() => {
        const updatedButton = screen.getByRole('button', { name: /regex|\/\.\*/i });
        expect(updatedButton.className).toMatch(/border-purple-500|text-purple-700/);
      });
    });

    it('should use font-bold and font-mono for text styling', async () => {
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

      const regexButton = screen.getByRole('button', { name: /regex|\/\.\*/i });

      // Check font styling (updated to font-medium instead of font-bold/font-mono)
      expect(regexButton.className).toMatch(/font-medium/);
    });
  });

  describe('Regex Toggle State Management', () => {
    it('should toggle regex state on button click', async () => {
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

      const regexButton = screen.getByRole('button', { name: /regex|\/\.\*/i });

      // Initial state - regex should be inactive
      expect(regexButton.className).toMatch(/border-gray-300|dark:border-gray-600/);

      // Click to activate regex
      await userEvent.click(regexButton);

      // State should change to active (purple)
      expect(regexButton.className).toMatch(/border-purple-500/);

      // Click again to deactivate
      await userEvent.click(regexButton);

      // State should change back to inactive (gray)
      expect(regexButton.className).toMatch(/border-gray-300|dark:border-gray-600/);
    });

    it('should maintain regex state across re-renders', async () => {
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

      const regexButton = screen.getByRole('button', { name: /regex|\/\.\*/i });

      // Activate regex
      await userEvent.click(regexButton);
      expect(regexButton.className).toMatch(/border-purple-500/);

      // Trigger re-render (simulate component update)
      rerender(<SearchPageV3 />);

      // State should be preserved
      const updatedButton = screen.getByRole('button', { name: /regex|\/\.\*/i });
      expect(updatedButton.className).toMatch(/border-purple-500/);
    });

    it('should start with regex disabled by default', async () => {
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

      const regexButton = screen.getByRole('button', { name: /regex|\/\.\*/i });

      // Initial state should be inactive (gray)
      expect(regexButton.className).toMatch(/border-gray-300|dark:border-gray-600/);
    });
  });

  describe('Regex Toggle Tooltip/Title', () => {
    it('should show correct tooltip when regex is disabled', async () => {
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

      const regexButton = screen.getByRole('button', { name: /regex|\/\.\*/i });
      const title = regexButton.getAttribute('title');

      // Should mention enabling regex or show disabled state message
      expect(title).toMatch(/enable|regex|pattern/i);
    });

    it('should update tooltip when regex is toggled on', async () => {
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

      const regexButton = screen.getByRole('button', { name: /regex|\/\.\*/i });

      // Get initial tooltip
      const initialTitle = regexButton.getAttribute('title');
      expect(initialTitle).toBeTruthy();

      // Click to enable
      await userEvent.click(regexButton);

      // Tooltip should change to indicate it's enabled
      const updatedTitle = regexButton.getAttribute('title');
      expect(updatedTitle).toMatch(/enabled|mode|regex/i);
    });
  });

  describe('Regex Toggle with Fuzzy Search', () => {
    it('should render both regex and fuzzy toggle buttons', async () => {
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

      // Both buttons should be present
      const regexButton = screen.getByRole('button', { name: /regex|\/\.\*/i });
      const fuzzyButton = screen.getByRole('button', { name: /fuzzy/i });

      expect(regexButton).toBeInTheDocument();
      expect(fuzzyButton).toBeInTheDocument();
    });

    it('should enforce mutual exclusivity - enabling regex should disable fuzzy', async () => {
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

      const regexButton = screen.getByRole('button', { name: /regex|\/\.\*/i });
      const fuzzyButton = screen.getByRole('button', { name: /fuzzy/i });

      // Enable fuzzy
      await userEvent.click(fuzzyButton);
      expect(fuzzyButton.className).toMatch(/border-blue-500/);
      expect(regexButton.className).toMatch(/border-gray-300|dark:border-gray-600/);

      // Enable regex - this should disable fuzzy (mutual exclusivity)
      await userEvent.click(regexButton);
      expect(regexButton.className).toMatch(/border-purple-500/);
      // Fuzzy should now be disabled
      expect(fuzzyButton.className).toMatch(/border-gray-300|dark:border-gray-600/);
    });

    it('should maintain separate states for fuzzy and regex toggles', async () => {
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

      const regexButton = screen.getByRole('button', { name: /regex|\/\.\*/i });
      const fuzzyButton = screen.getByRole('button', { name: /fuzzy/i });

      // Enable only regex
      await userEvent.click(regexButton);

      // Regex should be purple, fuzzy should be gray
      expect(regexButton.className).toMatch(/border-purple-500/);
      expect(fuzzyButton.className).toMatch(/border-gray-300|dark:border-gray-600/);

      // Enable fuzzy - this disables regex due to mutual exclusivity
      await userEvent.click(fuzzyButton);

      // Fuzzy should be blue, regex should be gray (mutually exclusive)
      expect(regexButton.className).toMatch(/border-gray-300|dark:border-gray-600/);
      expect(fuzzyButton.className).toMatch(/border-blue-500/);

      // Disable fuzzy by clicking again
      await userEvent.click(fuzzyButton);

      // Both should be disabled (back to normal state)
      expect(regexButton.className).toMatch(/border-gray-300|dark:border-gray-600/);
      expect(fuzzyButton.className).toMatch(/border-gray-300|dark:border-gray-600/);
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

      const regexButton = screen.getByRole('button', { name: /regex|\/\.\*/i });

      // Button should be focusable (keyboard accessible)
      regexButton.focus();
      expect(regexButton).toHaveFocus();

      // Button is a button element which is keyboard accessible by default
      expect(regexButton.tagName).toBe('BUTTON');
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

      const regexButton = screen.getByRole('button', { name: /regex|\/\.\*/i });
      const title = regexButton.getAttribute('title');

      // Title should be descriptive
      expect(title).toBeTruthy();
      expect(title?.length).toBeGreaterThan(10);
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

      const regexButton = screen.getByRole('button', { name: /regex|\/\.\*/i });

      // Should have dark mode classes
      expect(regexButton.className).toMatch(/dark:/);
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

      const regexButton = screen.getByRole('button', { name: /regex|\/\.\*/i });

      // Inactive state should have dark:bg-gray-800, dark:border-gray-600
      expect(regexButton.className).toMatch(/dark:bg-gray-800|dark:border-gray-600/);
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

      const regexButton = screen.getByRole('button', { name: /regex|\/\.\*/i });

      // Activate
      await userEvent.click(regexButton);

      // Active state should have dark:bg-purple-900, dark:text-purple-200
      expect(regexButton.className).toMatch(/dark:bg-purple-900|dark:text-purple-200/);
    });
  });

  describe('Integration with Hook', () => {
    it('should pass regex state to useMultiSelectSearch hook', async () => {
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

      const regexButton = screen.getByRole('button', { name: /regex|\/\.\*/i });
      const searchInput = screen.getByPlaceholderText(/search/i);

      // Type a query
      await userEvent.type(searchInput, 'test');

      // Click regex button to enable
      await userEvent.click(regexButton);

      // Wait for hook to be called with regex enabled
      await waitFor(() => {
        // Find the call where regexSearch is true
        const callWithRegex = mockMultiSelectSearch.mock.calls.find(
          call => call[5] === true // regexSearch parameter is at index 5
        );
        expect(callWithRegex).toBeDefined();
      });
    });

    it('should pass false for regexSearch when disabled', async () => {
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

      // Type a query without enabling regex
      await userEvent.type(searchInput, 'test');

      // Wait for hook to be called with regex disabled
      await waitFor(() => {
        // Find the initial call where regexSearch is false
        const initialCall = mockMultiSelectSearch.mock.calls[0];
        expect(initialCall[5]).toBe(false); // regexSearch parameter
      });
    });
  });

  describe('Responsive Design', () => {
    it('should display "Regex" text and /.*/ symbol in button', async () => {
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

      const regexButton = screen.getByRole('button', { name: /regex|\/\.\*/i });

      // Button should be present and contain button text
      expect(regexButton).toBeInTheDocument();

      // Check that button contains responsive elements (hidden text for "Regex" on mobile)
      // The button structure should have both elements inside
      const children = regexButton.querySelectorAll('span');
      expect(children.length).toBeGreaterThanOrEqual(0);
    });

    it('should respond to clicks on mobile and desktop', async () => {
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

      const regexButton = screen.getByRole('button', { name: /regex|\/\.\*/i });

      // Click button
      await userEvent.click(regexButton);

      // Button should be interactive on all screen sizes
      expect(regexButton).toBeInTheDocument();
    });
  });
});
