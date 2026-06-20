import { describe, it, expect, vi, beforeEach } from 'vitest';
import { renderHook, waitFor } from '@testing-library/react';
import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import React from 'react';
import { useMultiSelectSearch } from '../useSearch';
import { useAuthStore } from '../../stores/auth-store';

// Mock fetch
const mockFetch = vi.fn();
global.fetch = mockFetch;

// Setup window.API_BASE_URL
if (typeof window === 'undefined') {
  (globalThis as typeof globalThis & { window: { API_BASE_URL: string } }).window = {
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
  results: [],
  total: 0,
  page: 1,
  size: 20,
  facets: { projects: [], versions: [], extensions: [] },
};

describe('useMultiSelectSearch Hook - Search Mode (Phase 4)', () => {
  beforeEach(() => {
    vi.clearAllMocks();
    // Search hooks are gated on auth; mark the store authenticated so queries run.
    useAuthStore.setState({ isAuthenticated: true });
    mockFetch.mockResolvedValue({ ok: true, json: async () => mockSearchResponse });
  });

  const lastUrl = () => mockFetch.mock.calls[0][0] as string;

  it('does not send a mode param when mode is keyword (default)', async () => {
    const { result } = renderHook(
      () => useMultiSelectSearch('test', {}, 1, {}, false, false, undefined, false, 'keyword'),
      { wrapper: createWrapper() }
    );

    await waitFor(() => expect(result.current.isSuccess).toBe(true));
    expect(lastUrl()).not.toContain('mode=');
  });

  it('does not send a mode param when mode is omitted (backward compatible)', async () => {
    const { result } = renderHook(
      () => useMultiSelectSearch('test', {}, 1),
      { wrapper: createWrapper() }
    );

    await waitFor(() => expect(result.current.isSuccess).toBe(true));
    expect(lastUrl()).not.toContain('mode=');
  });

  it('sends mode=semantic when requested', async () => {
    const { result } = renderHook(
      () => useMultiSelectSearch('test', {}, 1, {}, false, false, undefined, false, 'semantic'),
      { wrapper: createWrapper() }
    );

    await waitFor(() => expect(result.current.isSuccess).toBe(true));
    expect(lastUrl()).toContain('mode=semantic');
  });

  it('sends mode=hybrid when requested', async () => {
    const { result } = renderHook(
      () => useMultiSelectSearch('test', {}, 1, {}, false, false, undefined, false, 'hybrid'),
      { wrapper: createWrapper() }
    );

    await waitFor(() => expect(result.current.isSuccess).toBe(true));
    expect(lastUrl()).toContain('mode=hybrid');
  });

  it('caches results separately per mode', async () => {
    const wrapper = createWrapper();
    const { result: keyword } = renderHook(
      () => useMultiSelectSearch('test', {}, 1, {}, false, false, undefined, false, 'keyword'),
      { wrapper }
    );
    await waitFor(() => expect(keyword.current.isSuccess).toBe(true));

    const { result: hybrid } = renderHook(
      () => useMultiSelectSearch('test', {}, 1, {}, false, false, undefined, false, 'hybrid'),
      { wrapper }
    );
    await waitFor(() => expect(hybrid.current.isSuccess).toBe(true));

    // Different mode => different query key => a second fetch (not a cache hit).
    expect(mockFetch).toHaveBeenCalledTimes(2);
  });
});
