import { useState, useEffect, useCallback } from 'react';
import { api } from '../lib/api';
import type { CrawlProgressInfo } from '../types';

// Re-export CrawlProgressInfo for components
export type { CrawlProgressInfo } from '../types';

export interface UseProgressOptions {
  repositoryId?: string;
  pollingInterval?: number; // ms, default 1000 (1 second)
  enabled?: boolean;
}

export interface UseProgressReturn {
  progress: CrawlProgressInfo | null;
  activeProgress: CrawlProgressInfo[];
  isLoading: boolean;
  error: string | null;
  refreshProgress: () => Promise<void>;
  refreshActiveProgress: () => Promise<void>;
}

export function useProgress({
  repositoryId,
  pollingInterval = 1000,
  enabled = true
}: UseProgressOptions = {}): UseProgressReturn {
  const [progress, setProgress] = useState<CrawlProgressInfo | null>(null);
  const [activeProgress, setActiveProgress] = useState<CrawlProgressInfo[]>([]);
  const [isLoading, setIsLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [isRepositoryLoading, setIsRepositoryLoading] = useState(false);
  const [isActiveLoading, setIsActiveLoading] = useState(false);

  // Fetch progress for a specific repository
  const refreshProgress = useCallback(async () => {
    if (!repositoryId || !enabled) return;

    try {
      setIsRepositoryLoading(true);
      setError(null);
      const progressData = await api.getRepositoryProgress(repositoryId);
      setProgress(progressData);
    } catch (err) {
      const errorMessage = err instanceof Error ? err.message : 'Failed to fetch progress';
      setError(errorMessage);
      console.error('Error fetching repository progress:', err);
    } finally {
      setIsRepositoryLoading(false);
    }
  }, [repositoryId, enabled]);

  // Fetch all active progress
  const refreshActiveProgress = useCallback(async () => {
    if (!enabled) return;

    try {
      setIsActiveLoading(true);
      setError(null);
      const activeProgressData = await api.getActiveProgress();
      setActiveProgress(activeProgressData);
      
      // No restart logic needed - just continue polling
    } catch (err) {
      const errorMessage = err instanceof Error ? err.message : 'Failed to fetch active progress';
      setError(errorMessage);
      console.error('Error fetching active progress:', err);
    } finally {
      setIsActiveLoading(false);
    }
  }, [enabled]);

  // Poll repository progress with intelligent interval
  useEffect(() => {
    if (!repositoryId || !enabled) return;

    let intervalId: number;
    let isMounted = true;

    const pollProgress = async () => {
      if (!isMounted) return;
      await refreshProgress();
    };

    // Initial fetch
    pollProgress();

    // Start polling only when pollingInterval > 0
    if (pollingInterval > 0) {
      intervalId = setInterval(async () => {
        if (!isMounted || document.hidden) return;
        await pollProgress();
      }, pollingInterval) as unknown as number;
    }

    return () => {
      isMounted = false;
      if (intervalId) {
        clearInterval(intervalId);
      }
    };
  }, [repositoryId, pollingInterval, enabled, refreshProgress]);

  // Poll active progress with intelligent interval (only if no specific repositoryId)
  useEffect(() => {
    if (!enabled || repositoryId) return; // Skip if we're tracking a specific repository

    let intervalId: number;
    let isMounted = true;

    const pollActiveProgress = async () => {
      if (!isMounted) return;

      try {
        setIsActiveLoading(true);
        setError(null);
        const activeProgressData = await api.getActiveProgress();

        if (!isMounted) return;

        // Keep both active AND failed crawls visible for a few seconds to show errors
        // This allows users to see error messages before they disappear
        setActiveProgress(activeProgressData);

        // Continue polling - no restart needed
      } catch (err) {
        if (!isMounted) return;

        const errorMessage = err instanceof Error ? err.message : 'Failed to fetch active progress';
        setError(errorMessage);
        console.error('Error fetching active progress:', err);
      } finally {
        if (isMounted) {
          setIsActiveLoading(false);
        }
      }
    };

    // Initial fetch
    pollActiveProgress();

    // Start polling only when pollingInterval > 0
    if (pollingInterval > 0) {
      // Use shorter intervals in test environment
      const isTestEnv = typeof process !== 'undefined' && process.env.NODE_ENV === 'test';
      const idleInterval = isTestEnv ? 100 : 15000; // 100ms in tests, 15s in production when idle

      intervalId = setInterval(async () => {
        if (!isMounted || document.hidden) return;
        await pollActiveProgress();
      }, pollingInterval) as unknown as number;
    }

    return () => {
      isMounted = false;
      if (intervalId) {
        clearInterval(intervalId);
      }
    };
  }, [repositoryId, pollingInterval, enabled]);

  // Combine loading states
  const combinedIsLoading = isRepositoryLoading || isActiveLoading;

  return {
    progress,
    activeProgress,
    isLoading: combinedIsLoading,
    error,
    refreshProgress,
    refreshActiveProgress,
  };
}

// Specific hook for repository progress
export function useRepositoryProgress(
  repositoryId: string,
  options: Omit<UseProgressOptions, 'repositoryId'> = {}
): Omit<UseProgressReturn, 'activeProgress' | 'refreshActiveProgress'> {
  const { progress, isLoading, error, refreshProgress } = useProgress({
    repositoryId,
    ...options,
  });

  return {
    progress,
    isLoading,
    error,
    refreshProgress,
  };
}

// Specific hook for active progress across all repositories
export function useActiveProgress(
  options: Omit<UseProgressOptions, 'repositoryId'> = {}
): Omit<UseProgressReturn, 'progress' | 'refreshProgress'> {
  const { activeProgress, isLoading, error, refreshActiveProgress } = useProgress({
    repositoryId: undefined,
    ...options,
  });

  return {
    activeProgress,
    isLoading,
    error,
    refreshActiveProgress,
  };
}

// Utility function to check if a repository is currently crawling
export function isRepositoryCrawling(repositoryId: string, activeProgress: CrawlProgressInfo[]): boolean {
  return activeProgress.some(
    progress => 
      progress.repository_id === repositoryId && 
      !['completed', 'failed', 'cancelled'].includes(progress.status.toLowerCase())
  );
}

// Utility function to get progress for a specific repository from active progress list
export function getRepositoryProgressFromActive(
  repositoryId: string, 
  activeProgress: CrawlProgressInfo[]
): CrawlProgressInfo | null {
  return activeProgress.find(progress => progress.repository_id === repositoryId) || null;
}