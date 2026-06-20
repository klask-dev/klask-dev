import { describe, it, expect, beforeEach, vi } from 'vitest';
import { render, screen, waitFor } from '@testing-library/react';
import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import { MemoryRouter } from 'react-router-dom';
import userEvent from '@testing-library/user-event';
import SearchPageV3 from '../SearchPageV3';
import * as useSearch from '../../../hooks/useSearch';
import * as searchCapabilities from '../../../api/searchCapabilities';
import type { SearchResponse } from '../../../types';
import { SearchFiltersProvider } from '../../../contexts/SearchFiltersContext';

vi.mock('../../../hooks/useSearch');
vi.mock('../../../api/searchCapabilities');

const createWrapper = (initialEntries: string[] = ['/search']) => {
  const queryClient = new QueryClient({
    defaultOptions: {
      queries: { retry: false, refetchOnWindowFocus: false },
      mutations: { retry: false },
    },
  });
  return ({ children }: { children: React.ReactNode }) => (
    <QueryClientProvider client={queryClient}>
      <SearchFiltersProvider>
        <MemoryRouter initialEntries={initialEntries}>{children}</MemoryRouter>
      </SearchFiltersProvider>
    </QueryClientProvider>
  );
};

const mockSearchResponse: SearchResponse = {
  results: [],
  total: 0,
  page: 1,
  size: 20,
  facets: { projects: [], versions: [], extensions: [] },
} as unknown as SearchResponse;

function mockSearchHooks() {
  vi.mocked(useSearch.useSearchFilters).mockReturnValue({
    data: { projects: [], versions: [], extensions: [], repositories: [], languages: [] },
    isLoading: false,
    error: null,
    refetch: vi.fn(),
  } as unknown as ReturnType<typeof useSearch.useSearchFilters>);

  vi.mocked(useSearch.useFacetsWithFilters).mockReturnValue({
    data: undefined,
    isLoading: false,
    error: null,
    refetch: vi.fn(),
  } as unknown as ReturnType<typeof useSearch.useFacetsWithFilters>);

  vi.mocked(useSearch.useSearchHistory).mockReturnValue({
    history: [],
    addToHistory: vi.fn(),
    clearHistory: vi.fn(),
  });

  vi.mocked(useSearch.useMultiSelectSearch).mockReturnValue({
    data: mockSearchResponse,
    isLoading: false,
    isFetching: false,
    isError: false,
    error: null,
    refetch: vi.fn(),
  } as unknown as ReturnType<typeof useSearch.useMultiSelectSearch>);
}

function mockCapabilities(enabled: boolean) {
  vi.mocked(searchCapabilities.useSearchCapabilities).mockReturnValue({
    data: { semantic_enabled: enabled },
    isLoading: false,
    error: null,
  } as unknown as ReturnType<typeof searchCapabilities.useSearchCapabilities>);
}

describe('SearchPageV3 - Semantic Mode Toggle', () => {
  beforeEach(() => {
    vi.clearAllMocks();
    mockSearchHooks();
  });

  it('hides the engine selector when the server does not support semantic search', () => {
    mockCapabilities(false);
    render(<SearchPageV3 />, { wrapper: createWrapper() });
    expect(screen.queryByRole('group', { name: /search engine mode/i })).not.toBeInTheDocument();
  });

  it('shows the engine selector when semantic search is available', async () => {
    mockCapabilities(true);
    render(<SearchPageV3 />, { wrapper: createWrapper() });
    await waitFor(() => {
      expect(screen.getByRole('group', { name: /search engine mode/i })).toBeInTheDocument();
    });
    expect(screen.getByRole('button', { name: 'Keyword' })).toHaveAttribute('aria-pressed', 'true');
    expect(screen.getByRole('button', { name: 'Hybrid' })).toBeInTheDocument();
    expect(screen.getByRole('button', { name: 'Semantic' })).toBeInTheDocument();
  });

  it('passes the selected mode to the search hook when changed', async () => {
    mockCapabilities(true);
    const user = userEvent.setup();
    render(<SearchPageV3 />, { wrapper: createWrapper() });

    await user.click(screen.getByRole('button', { name: 'Hybrid' }));

    await waitFor(() => {
      expect(screen.getByRole('button', { name: 'Hybrid' })).toHaveAttribute('aria-pressed', 'true');
    });
    // useMultiSelectSearch is called with mode as the last positional arg.
    const calls = vi.mocked(useSearch.useMultiSelectSearch).mock.calls;
    const lastCall = calls[calls.length - 1];
    expect(lastCall[lastCall.length - 1]).toBe('hybrid');
  });

  it('initializes the mode from the URL', async () => {
    mockCapabilities(true);
    render(<SearchPageV3 />, { wrapper: createWrapper(['/search?q=foo&mode=semantic']) });
    await waitFor(() => {
      expect(screen.getByRole('button', { name: 'Semantic' })).toHaveAttribute('aria-pressed', 'true');
    });
  });
});
