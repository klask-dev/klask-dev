import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { renderHook, act, waitFor } from '@testing-library/react';
import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import React from 'react';
import {
  useRepositories,
  useCrawlRepository,
  useBulkRepositoryOperations,
  useActiveProgress,
} from '../useRepositories';
import { apiClient } from '../../lib/api';
import type { Repository, CrawlProgressInfo } from '../../types';
import * as useProgressModule from '../useProgress';
import { renderHookWithQueryClient } from '../../test/react-query-test-utils';

// Mock the API client
vi.mock('../../lib/api', () => ({
  apiClient: {
    getRepositories: vi.fn(),
    crawlRepository: vi.fn(),
    updateRepository: vi.fn(),
    deleteRepository: vi.fn(),
    getActiveProgress: vi.fn(),
    getRepositoryProgress: vi.fn(),
  },
  getErrorMessage: vi.fn((error) => error.message || 'Unknown error'),
}));

// Mock the useProgress hook with proper structure
vi.mock('../useProgress', () => ({
  useActiveProgress: vi.fn(() => ({
    activeProgress: [],
    isLoading: false,
    error: null,
    refreshActiveProgress: vi.fn(),
  })),
  isRepositoryCrawling: vi.fn(),
  getRepositoryProgressFromActive: vi.fn(),
}));

const mockApiClient = apiClient as any;

// Test wrapper with QueryClient
const createWrapper = () => {
  const queryClient = new QueryClient({
    defaultOptions: {
      queries: {
        retry: false,
      },
      mutations: {
        retry: false,
      },
    },
  });

  return ({ children }: { children: React.ReactNode }) => (
    <QueryClientProvider client={queryClient}>
      {children}
    </QueryClientProvider>
  );
};

describe('useRepositories - Crawl Prevention', () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  afterEach(() => {
    vi.clearAllTimers();
    vi.useRealTimers();
  });

  const mockRepository: Repository = {
    id: 'repo-1',
    name: 'Test Repo',
    url: 'https://github.com/test/repo',
    repositoryType: 'Git',
    branch: 'main',
    enabled: true,
    lastCrawled: null,
    createdAt: '2024-01-01T00:00:00Z',
    updatedAt: '2024-01-01T00:00:00Z',
    autoCrawlEnabled: false,
    cronSchedule: null,
    nextCrawlAt: null,
    crawlFrequencyHours: null,
    maxCrawlDurationMinutes: 60,
  };

  const mockProgressInfo: CrawlProgressInfo = {
    repository_id: 'repo-1',
    repository_name: 'Test Repo',
    status: 'processing',
    progress_percentage: 50.0,
    files_processed: 50,
    files_total: 100,
    files_indexed: 25,
    current_file: 'src/main.rs',
    error_message: null,
    started_at: '2024-01-01T00:00:00Z',
    updated_at: '2024-01-01T00:01:00Z',
    completed_at: null,
  };

  describe('useCrawlRepository', () => {
    it('should successfully start crawl when repository is not crawling', async () => {
      mockApiClient.crawlRepository.mockResolvedValueOnce('Crawl started');

      const { result } = renderHook(() => useCrawlRepository(), {
        wrapper: createWrapper(),
      });

      // Wait for hook to initialize
      await waitFor(() => {
        expect(result.current).toBeDefined();
        expect(result.current.mutateAsync).toBeDefined();
      });

      await act(async () => {
        await result.current.mutateAsync('repo-1');
      });

      expect(mockApiClient.crawlRepository).toHaveBeenCalledWith('repo-1');
      await waitFor(() => {
        expect(result.current.isSuccess).toBe(true);
      });
      expect(result.current.error).toBeNull();
    });

    it('should handle 409 conflict error when repository is already crawling', async () => {
      const conflictError = new Error('Repository is already being crawled');
      (conflictError as any).status = 409;
      mockApiClient.crawlRepository.mockRejectedValueOnce(conflictError);

      const { result } = renderHook(() => useCrawlRepository(), {
        wrapper: createWrapper(),
      });

      // Wait for hook to initialize
      await waitFor(() => {
        expect(result.current).toBeDefined();
        expect(result.current.mutateAsync).toBeDefined();
      });

      await act(async () => {
        try {
          await result.current.mutateAsync('repo-1');
        } catch (error) {
          // Expected to throw
        }
      });

      expect(mockApiClient.crawlRepository).toHaveBeenCalledWith('repo-1');
      await waitFor(() => {
        expect(result.current.isError).toBe(true);
      });
      expect(result.current.error).toEqual(conflictError);
    });

    it('should handle other errors properly', async () => {
      const serverError = new Error('Internal server error');
      (serverError as any).status = 500;
      mockApiClient.crawlRepository.mockRejectedValueOnce(serverError);

      const { result } = renderHook(() => useCrawlRepository(), {
        wrapper: createWrapper(),
      });

      // Wait for hook to initialize
      await waitFor(() => {
        expect(result.current).toBeDefined();
        expect(result.current.mutateAsync).toBeDefined();
      });

      await act(async () => {
        try {
          await result.current.mutateAsync('repo-1');
        } catch (error) {
          // Expected to throw
        }
      });

      expect(mockApiClient.crawlRepository).toHaveBeenCalledWith('repo-1');
      await waitFor(() => {
        expect(result.current.isError).toBe(true);
      });
      expect(result.current.error).toEqual(serverError);
    });
  });

  describe('useBulkRepositoryOperations', () => {
    describe('bulkCrawl', () => {
      it('should handle successful bulk crawl when no repositories are crawling', async () => {
        mockApiClient.crawlRepository
          .mockResolvedValueOnce('Crawl started')
          .mockResolvedValueOnce('Crawl started')
          .mockResolvedValueOnce('Crawl started');

        const { result } = renderHook(() => useBulkRepositoryOperations(), {
          wrapper: createWrapper(),
        });

        let bulkResult;
        await act(async () => {
          bulkResult = await result.current.bulkCrawl.mutateAsync(['repo-1', 'repo-2', 'repo-3']);
        });

        expect(bulkResult).toEqual({
          successful: 3,
          failed: 0,
          alreadyCrawling: 0,
          total: 3,
        });
        expect(mockApiClient.crawlRepository).toHaveBeenCalledTimes(3);
      });

      it('should handle mixed success and conflict scenarios', async () => {
        const conflictError = new Error('Repository is already being crawled');
        (conflictError as any).status = 409;

        mockApiClient.crawlRepository
          .mockResolvedValueOnce('Crawl started')  // repo-1 success
          .mockRejectedValueOnce(conflictError)    // repo-2 conflict
          .mockResolvedValueOnce('Crawl started'); // repo-3 success

        const { result } = renderHook(() => useBulkRepositoryOperations(), {
          wrapper: createWrapper(),
        });

        let bulkResult;
        await act(async () => {
          bulkResult = await result.current.bulkCrawl.mutateAsync(['repo-1', 'repo-2', 'repo-3']);
        });

        expect(bulkResult).toEqual({
          successful: 2,
          failed: 1,
          alreadyCrawling: 1,
          total: 3,
        });
      });

      it('should handle all repositories already crawling', async () => {
        const conflictError = new Error('Repository is already being crawled');
        (conflictError as any).status = 409;

        mockApiClient.crawlRepository
          .mockRejectedValueOnce(conflictError)
          .mockRejectedValueOnce(conflictError);

        const { result } = renderHook(() => useBulkRepositoryOperations(), {
          wrapper: createWrapper(),
        });

        let bulkResult;
        await act(async () => {
          bulkResult = await result.current.bulkCrawl.mutateAsync(['repo-1', 'repo-2']);
        });

        expect(bulkResult).toEqual({
          successful: 0,
          failed: 2,
          alreadyCrawling: 2,
          total: 2,
        });
      });

      it('should handle server errors properly', async () => {
        const serverError = new Error('Internal server error');
        (serverError as any).status = 500;

        mockApiClient.crawlRepository
          .mockResolvedValueOnce('Crawl started')
          .mockRejectedValueOnce(serverError);

        const { result } = renderHook(() => useBulkRepositoryOperations(), {
          wrapper: createWrapper(),
        });

        let bulkResult;
        await act(async () => {
          bulkResult = await result.current.bulkCrawl.mutateAsync(['repo-1', 'repo-2']);
        });

        expect(bulkResult).toEqual({
          successful: 1,
          failed: 1,
          alreadyCrawling: 0,
          total: 2,
        });
      });

      it('should handle empty repository list', async () => {
        const { result } = renderHook(() => useBulkRepositoryOperations(), {
          wrapper: createWrapper(),
        });

        let bulkResult;
        await act(async () => {
          bulkResult = await result.current.bulkCrawl.mutateAsync([]);
        });

        expect(bulkResult).toEqual({
          successful: 0,
          failed: 0,
          alreadyCrawling: 0,
          total: 0,
        });
        expect(mockApiClient.crawlRepository).not.toHaveBeenCalled();
      });

      it('should handle race conditions with Promise.allSettled', async () => {
        // Simulate different timing for API calls
        const delay = (ms: number) => new Promise(resolve => setTimeout(resolve, ms));

        const conflictError = new Error('Repository is already being crawled');
        (conflictError as any).status = 409;

        mockApiClient.crawlRepository
          .mockImplementationOnce(async () => {
            await delay(10);
            return 'Crawl started';
          })
          .mockImplementationOnce(async () => {
            await delay(5);
            throw conflictError;
          })
          .mockImplementationOnce(async () => {
            await delay(15);
            return 'Crawl started';
          });

        const { result } = renderHook(() => useBulkRepositoryOperations(), {
          wrapper: createWrapper(),
        });

        // Wait for hook to initialize
        await waitFor(() => {
          expect(result.current).toBeDefined();
          expect(result.current.bulkCrawl).toBeDefined();
          expect(result.current.bulkCrawl.mutateAsync).toBeDefined();
        });

        let bulkResult;
        await act(async () => {
          bulkResult = await result.current.bulkCrawl.mutateAsync(['repo-1', 'repo-2', 'repo-3']);
        });

        expect(bulkResult).toEqual({
          successful: 2,
          failed: 1,
          alreadyCrawling: 1,
          total: 3,
        });
      });

      it('should handle very large repository lists', async () => {
        const repositoryIds = Array.from({ length: 50 }, (_, i) => `repo-${i}`);

        // Mock successful responses for all
        for (let i = 0; i < 50; i++) {
          mockApiClient.crawlRepository.mockResolvedValueOnce('Crawl started');
        }

        const { result } = renderHook(() => useBulkRepositoryOperations(), {
          wrapper: createWrapper(),
        });

        // Wait for hook to initialize
        await waitFor(() => {
          expect(result.current).toBeDefined();
          expect(result.current.bulkCrawl).toBeDefined();
          expect(result.current.bulkCrawl.mutateAsync).toBeDefined();
        });

        let bulkResult;
        await act(async () => {
          bulkResult = await result.current.bulkCrawl.mutateAsync(repositoryIds);
        });

        expect(bulkResult).toEqual({
          successful: 50,
          failed: 0,
          alreadyCrawling: 0,
          total: 50,
        });
        expect(mockApiClient.crawlRepository).toHaveBeenCalledTimes(50);
      });
    });
  });

  describe('useActiveProgress', () => {
    it('should fetch active progress data', async () => {
      const mockActiveProgress = [mockProgressInfo];

      // Configure the mock to return the expected data
      vi.mocked(useProgressModule.useActiveProgress).mockReturnValueOnce({
        activeProgress: mockActiveProgress,
        isLoading: false,
        error: null,
        refreshActiveProgress: vi.fn(),
      });

      const { result } = renderHook(() => useActiveProgress(), {
        wrapper: createWrapper(),
      });

      await waitFor(() => {
        expect(result.current.data).toEqual(mockActiveProgress);
      });

      // isLoading should eventually be false
      await waitFor(() => {
        expect(result.current.isLoading).toBe(false);
      });
    });

    it('should handle empty active progress', async () => {
      // Configure the mock to return empty data
      vi.mocked(useProgressModule.useActiveProgress).mockReturnValueOnce({
        activeProgress: [],
        isLoading: false,
        error: null,
        refreshActiveProgress: vi.fn(),
      });

      const { result } = renderHook(() => useActiveProgress(), {
        wrapper: createWrapper(),
      });

      await waitFor(() => {
        expect(result.current.data).toEqual([]);
      });

      await waitFor(() => {
        expect(result.current.isLoading).toBe(false);
      });
    });

    it('should handle API errors', async () => {
      const apiError = new Error('API Error');

      // Configure the mock to return error state
      vi.mocked(useProgressModule.useActiveProgress).mockReturnValueOnce({
        activeProgress: [],
        isLoading: false,
        error: apiError,
        refreshActiveProgress: vi.fn(),
      });

      const { result } = renderHook(() => useActiveProgress(), {
        wrapper: createWrapper(),
      });

      await waitFor(() => {
        expect(result.current.error).toBeTruthy();
      });

      await waitFor(() => {
        expect(result.current.isLoading).toBe(false);
      });
    });

    it('should refetch at regular intervals', async () => {
      const mockRefresh = vi.fn();

      // Configure the mock to return the expected data with refresh function
      vi.mocked(useProgressModule.useActiveProgress).mockReturnValueOnce({
        activeProgress: [mockProgressInfo],
        isLoading: false,
        error: null,
        refreshActiveProgress: mockRefresh,
      });

      const { result } = renderHook(() => useActiveProgress(), {
        wrapper: createWrapper(),
      });

      // The useActiveProgress hook from useProgress should handle polling internally
      // We just verify the data is fetched correctly
      await waitFor(() => {
        expect(result.current.data).toEqual([mockProgressInfo]);
      });

      // Verify refresh function is available
      expect(result.current.refetch).toBeDefined();
    });

    it('should handle multiple repositories in active progress', async () => {
      const mockMultipleProgress: CrawlProgressInfo[] = [
        { ...mockProgressInfo, repository_id: 'repo-1', repository_name: 'Repo 1' },
        { ...mockProgressInfo, repository_id: 'repo-2', repository_name: 'Repo 2', status: 'cloning' },
        { ...mockProgressInfo, repository_id: 'repo-3', repository_name: 'Repo 3', status: 'indexing' },
      ];

      // Configure the mock to return multiple progress data
      vi.mocked(useProgressModule.useActiveProgress).mockReturnValueOnce({
        activeProgress: mockMultipleProgress,
        isLoading: false,
        error: null,
        refreshActiveProgress: vi.fn(),
      });

      const { result } = renderHook(() => useActiveProgress(), {
        wrapper: createWrapper(),
      });

      await waitFor(() => {
        expect(result.current.data).toEqual(mockMultipleProgress);
        expect(result.current.data).toHaveLength(3);
      });
    });
  });
});

describe('Integration - Crawl Prevention Edge Cases', () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  afterEach(() => {
    vi.clearAllTimers();
    vi.useRealTimers();
  });

  it('should handle rapid start/stop crawl cycles', async () => {
    let callCount = 0;
    mockApiClient.crawlRepository.mockImplementation(() => {
      callCount++;
      if (callCount % 2 === 0) {
        const error = new Error('Repository is already being crawled');
        (error as any).status = 409;
        throw error;
      }
      return Promise.resolve('Crawl started');
    });

    const { result } = renderHook(() => useCrawlRepository(), {
      wrapper: createWrapper(),
    });

    // Wait for hook to initialize
    await waitFor(() => {
      expect(result.current).toBeDefined();
      expect(result.current.mutateAsync).toBeDefined();
    });

    // First call should succeed
    await act(async () => {
      await result.current.mutateAsync('repo-1');
    });

    await waitFor(() => {
      expect(result.current.isSuccess).toBe(true);
    });

    // Second call should fail with conflict
    await act(async () => {
      try {
        await result.current.mutateAsync('repo-1');
      } catch (error) {
        // Expected
      }
    });

    await waitFor(() => {
      expect(result.current.isError).toBe(true);
    });
  });

  it('should handle network timeout during crawl operations', async () => {
    const timeoutError = new Error('Network timeout');
    (timeoutError as any).status = 408;

    mockApiClient.crawlRepository.mockRejectedValue(timeoutError);

    const { result } = renderHook(() => useBulkRepositoryOperations(), {
      wrapper: createWrapper(),
    });

    // Wait for hook to initialize
    await waitFor(() => {
      expect(result.current).toBeDefined();
      expect(result.current.bulkCrawl).toBeDefined();
      expect(result.current.bulkCrawl.mutateAsync).toBeDefined();
    });

    let bulkResult;
    await act(async () => {
      bulkResult = await result.current.bulkCrawl.mutateAsync(['repo-1', 'repo-2']);
    });

    expect(bulkResult).toEqual({
      successful: 0,
      failed: 2,
      alreadyCrawling: 0,
      total: 2,
    });
  });

  it('should handle partial bulk crawl failures gracefully', async () => {
    const conflictError = new Error('Repository is already being crawled');
    (conflictError as any).status = 409;

    const serverError = new Error('Internal server error');
    (serverError as any).status = 500;

    mockApiClient.crawlRepository
      .mockResolvedValueOnce('Success')        // repo-1: success
      .mockRejectedValueOnce(conflictError)    // repo-2: already crawling
      .mockRejectedValueOnce(serverError)      // repo-3: server error
      .mockResolvedValueOnce('Success')        // repo-4: success
      .mockRejectedValueOnce(conflictError);   // repo-5: already crawling

    const { result } = renderHook(() => useBulkRepositoryOperations(), {
      wrapper: createWrapper(),
    });

    let bulkResult;
    await act(async () => {
      bulkResult = await result.current.bulkCrawl.mutateAsync([
        'repo-1', 'repo-2', 'repo-3', 'repo-4', 'repo-5'
      ]);
    });

    expect(bulkResult).toEqual({
      successful: 2,
      failed: 3,
      alreadyCrawling: 2,
      total: 5,
    });
  });

  it('should handle concurrent bulk operations on same repositories', async () => {
    const conflictError = new Error('Repository is already being crawled');
    (conflictError as any).status = 409;

    // First bulk operation starts successfully
    mockApiClient.crawlRepository
      .mockResolvedValueOnce('Success')
      .mockResolvedValueOnce('Success');

    // Second bulk operation gets conflicts
    mockApiClient.crawlRepository
      .mockRejectedValueOnce(conflictError)
      .mockRejectedValueOnce(conflictError);

    const { result } = renderHook(() => useBulkRepositoryOperations(), {
      wrapper: createWrapper(),
    });

    // Start first bulk operation
    const promise1 = act(async () => {
      return await result.current.bulkCrawl.mutateAsync(['repo-1', 'repo-2']);
    });

    // Start second bulk operation immediately
    const promise2 = act(async () => {
      return await result.current.bulkCrawl.mutateAsync(['repo-1', 'repo-2']);
    });

    const [result1, result2] = await Promise.all([promise1, promise2]);

    expect(result1).toEqual({
      successful: 2,
      failed: 0,
      alreadyCrawling: 0,
      total: 2,
    });

    expect(result2).toEqual({
      successful: 0,
      failed: 2,
      alreadyCrawling: 2,
      total: 2,
    });
  });


});