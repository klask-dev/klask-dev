import React from 'react';
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import { apiClient, getErrorMessage } from '../lib/api';
import type { Repository, RepositoryWithStats, CreateRepositoryRequest } from '../types';

// Get all repositories (fast load without stats)
export const useRepositories = () => {
  return useQuery({
    queryKey: ['repositories'],
    queryFn: () => apiClient.getRepositories(),
    staleTime: 30000, // 30 seconds
    retry: 2,
  });
};

// Get all repositories with full stats (slower)
export const useRepositoriesWithStats = () => {
  return useQuery({
    queryKey: ['repositories', 'with-stats'],
    queryFn: () => apiClient.getRepositoriesWithStats(),
    staleTime: 60000, // 1 minute
    retry: 2,
  });
};

// Get single repository
export const useRepository = (id: string) => {
  return useQuery({
    queryKey: ['repositories', id],
    queryFn: () => apiClient.getRepository(id),
    enabled: !!id,
    staleTime: 30000,
    retry: 2,
  });
};

// Create repository mutation
export const useCreateRepository = () => {
  const queryClient = useQueryClient();
  
  return useMutation({
    mutationFn: (data: CreateRepositoryRequest) => apiClient.createRepository(data),
    onSuccess: () => {
      // Invalidate and refetch repositories list
      queryClient.invalidateQueries({ queryKey: ['repositories'] });
    },
    onError: (error) => {
      console.error('Failed to create repository:', getErrorMessage(error));
    },
  });
};

// Update repository mutation
export const useUpdateRepository = () => {
  const queryClient = useQueryClient();
  
  return useMutation({
    mutationFn: ({ id, data }: { id: string; data: Partial<CreateRepositoryRequest> }) => 
      apiClient.updateRepository(id, data),
    onSuccess: (updatedRepo) => {
      console.log('Update success - updatedRepo:', updatedRepo);
      console.log('updatedRepo.id:', updatedRepo?.id);

      // Invalidate queries to force refetch from backend
      // This ensures we get the latest calculated fields like next_crawl_at
      queryClient.invalidateQueries({ queryKey: ['repositories'] });
      queryClient.invalidateQueries({ queryKey: ['repositories', updatedRepo.id] });
    },
    onError: (error) => {
      console.error('Failed to update repository:', getErrorMessage(error));
    },
  });
};

// Delete repository mutation
export const useDeleteRepository = () => {
  const queryClient = useQueryClient();
  
  return useMutation({
    mutationFn: (id: string) => apiClient.deleteRepository(id),
    onSuccess: (_, deletedId) => {
      // Remove from repositories list
      queryClient.setQueryData(['repositories'], (old: RepositoryWithStats[] | undefined) => {
        if (!old) return [];
        return old.filter(repoWithStats =>
          repoWithStats?.repository?.id && repoWithStats.repository.id !== deletedId
        );
      });

      // Remove individual repository cache
      queryClient.removeQueries({ queryKey: ['repositories', deletedId] });

      // Also invalidate to refetch fresh data
      queryClient.invalidateQueries({ queryKey: ['repositories'] });
    },
    onError: (error) => {
      console.error('Failed to delete repository:', getErrorMessage(error));
    },
  });
};

// Crawl repository mutation
export const useCrawlRepository = () => {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: (id: string) => {
      if (!id) {
        return Promise.reject(new Error('Repository ID is required'));
      }
      return apiClient.crawlRepository(id);
    },
    onSuccess: (_, repositoryId) => {
      // Refetch the specific repository to get updated lastCrawled timestamp
      queryClient.invalidateQueries({ queryKey: ['repositories', repositoryId] });
      queryClient.invalidateQueries({ queryKey: ['repositories'] });
      // Force immediate refetch of active progress to switch polling interval
      queryClient.refetchQueries({ queryKey: ['repositories', 'progress', 'active'] });
    },
    onError: (error: any) => {
      if (error.status === 409) {
        console.warn('Repository is already being crawled');
      } else {
        console.error('Failed to crawl repository:', getErrorMessage(error));
      }
    },
  });
};

// Stop crawl repository mutation
export const useStopCrawl = () => {
  const queryClient = useQueryClient();
  
  return useMutation({
    mutationFn: (id: string) => apiClient.stopCrawlRepository(id),
    onSuccess: (_, repositoryId) => {
      // Invalidate progress queries to reflect the stopped status
      queryClient.invalidateQueries({ queryKey: ['repositories', repositoryId, 'progress'] });
      queryClient.invalidateQueries({ queryKey: ['repositories', 'progress', 'active'] });
      queryClient.invalidateQueries({ queryKey: ['repositories', repositoryId] });
    },
    onError: (error: any) => {
      if (error.status === 404) {
        console.warn('No active crawl found for repository');
      } else {
        console.error('Failed to stop crawl:', getErrorMessage(error));
      }
    },
  });
};

// Repository statistics (if needed)
export const useRepositoryStats = () => {
  const { data: repositories } = useRepositories();
  
  const stats = React.useMemo(() => {
    if (!repositories) return null;
    
    const validRepos = repositories.filter(repoWithStats => repoWithStats?.repository);
    const total = validRepos.length;
    const enabled = validRepos.filter(repoWithStats => repoWithStats.repository.enabled).length;
    const crawled = validRepos.filter(repoWithStats => repoWithStats.repository.lastCrawled).length;
    const gitRepos = validRepos.filter(repoWithStats => repoWithStats.repository.repositoryType === 'Git').length;
    const gitlabRepos = validRepos.filter(repoWithStats => repoWithStats.repository.repositoryType === 'GitLab').length;
    const githubRepos = validRepos.filter(repoWithStats => repoWithStats.repository.repositoryType === 'GitHub').length;
    const filesystemRepos = validRepos.filter(repoWithStats => repoWithStats.repository.repositoryType === 'FileSystem').length;

    return {
      total,
      enabled,
      disabled: total - enabled,
      crawled,
      notCrawled: total - crawled,
      byType: {
        git: gitRepos,
        gitlab: gitlabRepos,
        github: githubRepos,
        filesystem: filesystemRepos,
      },
    };
  }, [repositories]);
  
  return stats;
};

// Bulk operations
export const useBulkRepositoryOperations = () => {
  const queryClient = useQueryClient();

  const bulkEnable = useMutation({
    mutationFn: async (repositoryIds: string[]) => {
      if (!repositoryIds || repositoryIds.length === 0) {
        return [];
      }
      const promises = repositoryIds.map(id =>
        apiClient.updateRepository(id, { enabled: true })
      );
      return Promise.all(promises);
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['repositories'] });
    },
  });

  const bulkDisable = useMutation({
    mutationFn: async (repositoryIds: string[]) => {
      if (!repositoryIds || repositoryIds.length === 0) {
        return [];
      }
      const promises = repositoryIds.map(id =>
        apiClient.updateRepository(id, { enabled: false })
      );
      return Promise.all(promises);
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['repositories'] });
    },
  });

  const bulkCrawl = useMutation({
    mutationFn: async (repositoryIds: string[]) => {
      if (!repositoryIds || repositoryIds.length === 0) {
        return {
          successful: 0,
          failed: 0,
          alreadyCrawling: 0,
          total: 0
        };
      }

      const results = await Promise.allSettled(
        repositoryIds.map(async id => {
          try {
            return await apiClient.crawlRepository(id);
          } catch (error: any) {
            // If the error is a conflict (409), it means the repository is already being crawled
            if (error.status === 409) {
              throw new Error(`Repository is already being crawled`);
            }
            throw error;
          }
        })
      );

      const successful = results.filter(result => result.status === 'fulfilled').length;
      const failed = results.filter(result => result.status === 'rejected').length;
      const alreadyCrawling = results.filter(result =>
        result.status === 'rejected' &&
        result.reason?.message?.includes('already being crawled')
      ).length;

      return {
        successful,
        failed,
        alreadyCrawling,
        total: repositoryIds.length
      };
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['repositories'] });
      queryClient.invalidateQueries({ queryKey: ['repositories', 'progress', 'active'] });
    },
  });

  const bulkDelete = useMutation({
    mutationFn: async (repositoryIds: string[]) => {
      if (!repositoryIds || repositoryIds.length === 0) {
        return [];
      }
      const promises = repositoryIds.map(id => apiClient.deleteRepository(id));
      return Promise.all(promises);
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['repositories'] });
    },
  });

  return {
    bulkEnable,
    bulkDisable,
    bulkCrawl,
    bulkDelete,
  };
};

import type { CrawlProgressInfo } from '../types';
import { useActiveProgress as useActiveProgressBase } from '../hooks/useProgress';

// Wrapper to match React Query's expected return format
export const useActiveProgress = () => {
  try {
    const result = useActiveProgressBase();

    // Handle cases where the hook returns undefined/null (like in tests)
    if (!result) {
      return {
        data: [],
        isLoading: false,
        error: null,
        refetch: async () => {},
      };
    }

    const { activeProgress, isLoading, error, refreshActiveProgress } = result;

    return {
      data: activeProgress || [],
      isLoading: isLoading || false,
      error: error || null,
      refetch: refreshActiveProgress || (async () => {}),
    };
  } catch (e) {
    // In case of any error, return a safe default
    return {
      data: [],
      isLoading: false,
      error: null,
      refetch: async () => {},
    };
  }
};

// Hook to get progress for a specific repository
export const useRepositoryProgress = (repositoryId: string) => {
  return useQuery({
    queryKey: ['repositories', repositoryId, 'progress'],
    queryFn: () => apiClient.getRepositoryProgress(repositoryId),
    enabled: !!repositoryId,
    refetchInterval: 1000, // Refetch every 1 second
    staleTime: 1000, // Consider data stale after 1 second
    retry: 2,
  });
};