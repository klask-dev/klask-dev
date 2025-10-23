import { describe, it, expect, vi, beforeEach } from 'vitest';
import { renderHook, waitFor } from '@testing-library/react';
import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import { useRepositoryStats, useCreateRepository } from '../useRepositories';
import type { RepositoryWithStats } from '../../types';

/**
 * Tests for GitHub integration in useRepositories hooks
 * These tests verify that hooks correctly handle GitHub repository data
 */

// Mock the apiClient
vi.mock('../../lib/api', () => ({
  apiClient: {
    getRepositories: vi.fn(),
    createRepository: vi.fn(),
    updateRepository: vi.fn(),
    deleteRepository: vi.fn(),
  },
  getErrorMessage: vi.fn((error) => error?.message || 'Unknown error'),
}));

const createWrapper = () => {
  const queryClient = new QueryClient({
    defaultOptions: {
      queries: { retry: false },
      mutations: { retry: false },
    },
  });
  return ({ children }: { children: React.ReactNode }) => (
    <QueryClientProvider client={queryClient}>{children}</QueryClientProvider>
  );
};

describe('useRepositories - GitHub Integration', () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  describe('useRepositoryStats', () => {
    it('should count GitHub repositories correctly', async () => {
      const { apiClient } = await import('../../lib/api');

      const mockRepositories: RepositoryWithStats[] = [
        {
          repository: {
            id: '1',
            name: 'github-repo-1',
            url: 'https://api.github.com',
            repositoryType: 'GitHub',
            branch: 'main',
            enabled: true,
            gitlabNamespace: null,
            isGroup: false,
            lastCrawled: null,
            createdAt: new Date().toISOString(),
            updatedAt: new Date().toISOString(),
            autoCrawlEnabled: false,
            cronSchedule: null,
            nextCrawlAt: null,
            crawlFrequencyHours: null,
            maxCrawlDurationMinutes: 60,
            lastCrawlDurationSeconds: null,
            gitlabExcludedProjects: null,
            gitlabExcludedPatterns: null,
            githubNamespace: 'org1',
            githubExcludedRepositories: null,
            githubExcludedPatterns: null,
            crawlState: 'idle',
            lastProcessedProject: null,
            crawlStartedAt: null,
          },
          diskSizeMb: null,
          fileCount: null,
          lastCrawlDurationMinutes: null,
        },
        {
          repository: {
            id: '2',
            name: 'github-repo-2',
            url: 'https://api.github.com',
            repositoryType: 'GitHub',
            branch: 'main',
            enabled: true,
            gitlabNamespace: null,
            isGroup: false,
            lastCrawled: new Date().toISOString(),
            createdAt: new Date().toISOString(),
            updatedAt: new Date().toISOString(),
            autoCrawlEnabled: false,
            cronSchedule: null,
            nextCrawlAt: null,
            crawlFrequencyHours: null,
            maxCrawlDurationMinutes: 60,
            lastCrawlDurationSeconds: null,
            gitlabExcludedProjects: null,
            gitlabExcludedPatterns: null,
            githubNamespace: 'org2',
            githubExcludedRepositories: 'org2/old-repo',
            githubExcludedPatterns: '*-archive',
            crawlState: 'idle',
            lastProcessedProject: null,
            crawlStartedAt: null,
          },
          diskSizeMb: null,
          fileCount: null,
          lastCrawlDurationMinutes: null,
        },
        {
          repository: {
            id: '3',
            name: 'git-repo',
            url: 'https://github.com/user/repo.git',
            repositoryType: 'Git',
            branch: 'main',
            enabled: true,
            gitlabNamespace: null,
            isGroup: false,
            lastCrawled: null,
            createdAt: new Date().toISOString(),
            updatedAt: new Date().toISOString(),
            autoCrawlEnabled: false,
            cronSchedule: null,
            nextCrawlAt: null,
            crawlFrequencyHours: null,
            maxCrawlDurationMinutes: 60,
            lastCrawlDurationSeconds: null,
            gitlabExcludedProjects: null,
            gitlabExcludedPatterns: null,
            githubNamespace: null,
            githubExcludedRepositories: null,
            githubExcludedPatterns: null,
            crawlState: 'idle',
            lastProcessedProject: null,
            crawlStartedAt: null,
          },
          diskSizeMb: null,
          fileCount: null,
          lastCrawlDurationMinutes: null,
        },
      ];

      vi.mocked(apiClient.getRepositories).mockResolvedValue(mockRepositories);

      const { result } = renderHook(() => useRepositoryStats(), {
        wrapper: createWrapper(),
      });

      await waitFor(() => {
        expect(result.current).not.toBeNull();
      });

      expect(result.current?.total).toBe(3);
      expect(result.current?.byType.github).toBe(2);
      expect(result.current?.byType.git).toBe(1);
      expect(result.current?.enabled).toBe(3);
      expect(result.current?.crawled).toBe(1); // Only github-repo-2 has lastCrawled
    });

    it('should handle repositories with GitHub exclusions', async () => {
      const { apiClient } = await import('../../lib/api');

      const mockRepositories: RepositoryWithStats[] = [
        {
          repository: {
            id: '1',
            name: 'github-with-exclusions',
            url: 'https://api.github.com',
            repositoryType: 'GitHub',
            branch: 'main',
            enabled: true,
            gitlabNamespace: null,
            isGroup: false,
            lastCrawled: null,
            createdAt: new Date().toISOString(),
            updatedAt: new Date().toISOString(),
            autoCrawlEnabled: false,
            cronSchedule: null,
            nextCrawlAt: null,
            crawlFrequencyHours: null,
            maxCrawlDurationMinutes: 60,
            lastCrawlDurationSeconds: null,
            gitlabExcludedProjects: null,
            gitlabExcludedPatterns: null,
            githubNamespace: 'test-org',
            githubExcludedRepositories: 'test-org/repo1,test-org/repo2',
            githubExcludedPatterns: '*-archive,*-temp',
            crawlState: 'idle',
            lastProcessedProject: null,
            crawlStartedAt: null,
          },
          diskSizeMb: null,
          fileCount: null,
          lastCrawlDurationMinutes: null,
        },
      ];

      vi.mocked(apiClient.getRepositories).mockResolvedValue(mockRepositories);

      const { result } = renderHook(() => useRepositoryStats(), {
        wrapper: createWrapper(),
      });

      await waitFor(() => {
        expect(result.current).not.toBeNull();
      });

      expect(result.current?.total).toBe(1);
      expect(result.current?.byType.github).toBe(1);

      // Verify the repository has exclusions set
      const repoData = mockRepositories[0].repository;
      expect(repoData.githubExcludedRepositories).toBe('test-org/repo1,test-org/repo2');
      expect(repoData.githubExcludedPatterns).toBe('*-archive,*-temp');
    });

    it('should handle mix of GitHub, GitLab, Git, and FileSystem repositories', async () => {
      const { apiClient } = await import('../../lib/api');

      const mockRepositories: RepositoryWithStats[] = [
        {
          repository: {
            id: '1',
            name: 'github-repo',
            url: 'https://api.github.com',
            repositoryType: 'GitHub',
            branch: 'main',
            enabled: true,
            gitlabNamespace: null,
            isGroup: false,
            lastCrawled: null,
            createdAt: new Date().toISOString(),
            updatedAt: new Date().toISOString(),
            autoCrawlEnabled: false,
            cronSchedule: null,
            nextCrawlAt: null,
            crawlFrequencyHours: null,
            maxCrawlDurationMinutes: 60,
            lastCrawlDurationSeconds: null,
            gitlabExcludedProjects: null,
            gitlabExcludedPatterns: null,
            githubNamespace: 'org',
            githubExcludedRepositories: null,
            githubExcludedPatterns: null,
            crawlState: 'idle',
            lastProcessedProject: null,
            crawlStartedAt: null,
          },
          diskSizeMb: null,
          fileCount: null,
          lastCrawlDurationMinutes: null,
        },
        {
          repository: {
            id: '2',
            name: 'gitlab-repo',
            url: 'https://gitlab.com',
            repositoryType: 'GitLab',
            branch: 'main',
            enabled: true,
            gitlabNamespace: 'group',
            isGroup: true,
            lastCrawled: null,
            createdAt: new Date().toISOString(),
            updatedAt: new Date().toISOString(),
            autoCrawlEnabled: false,
            cronSchedule: null,
            nextCrawlAt: null,
            crawlFrequencyHours: null,
            maxCrawlDurationMinutes: 60,
            lastCrawlDurationSeconds: null,
            gitlabExcludedProjects: null,
            gitlabExcludedPatterns: null,
            githubNamespace: null,
            githubExcludedRepositories: null,
            githubExcludedPatterns: null,
            crawlState: 'idle',
            lastProcessedProject: null,
            crawlStartedAt: null,
          },
          diskSizeMb: null,
          fileCount: null,
          lastCrawlDurationMinutes: null,
        },
        {
          repository: {
            id: '3',
            name: 'git-repo',
            url: 'https://github.com/user/repo.git',
            repositoryType: 'Git',
            branch: 'main',
            enabled: false,
            gitlabNamespace: null,
            isGroup: false,
            lastCrawled: null,
            createdAt: new Date().toISOString(),
            updatedAt: new Date().toISOString(),
            autoCrawlEnabled: false,
            cronSchedule: null,
            nextCrawlAt: null,
            crawlFrequencyHours: null,
            maxCrawlDurationMinutes: 60,
            lastCrawlDurationSeconds: null,
            gitlabExcludedProjects: null,
            gitlabExcludedPatterns: null,
            githubNamespace: null,
            githubExcludedRepositories: null,
            githubExcludedPatterns: null,
            crawlState: 'idle',
            lastProcessedProject: null,
            crawlStartedAt: null,
          },
          diskSizeMb: null,
          fileCount: null,
          lastCrawlDurationMinutes: null,
        },
        {
          repository: {
            id: '4',
            name: 'filesystem-repo',
            url: '/path/to/code',
            repositoryType: 'FileSystem',
            branch: null,
            enabled: true,
            gitlabNamespace: null,
            isGroup: false,
            lastCrawled: null,
            createdAt: new Date().toISOString(),
            updatedAt: new Date().toISOString(),
            autoCrawlEnabled: false,
            cronSchedule: null,
            nextCrawlAt: null,
            crawlFrequencyHours: null,
            maxCrawlDurationMinutes: 60,
            lastCrawlDurationSeconds: null,
            gitlabExcludedProjects: null,
            gitlabExcludedPatterns: null,
            githubNamespace: null,
            githubExcludedRepositories: null,
            githubExcludedPatterns: null,
            crawlState: 'idle',
            lastProcessedProject: null,
            crawlStartedAt: null,
          },
          diskSizeMb: null,
          fileCount: null,
          lastCrawlDurationMinutes: null,
        },
      ];

      vi.mocked(apiClient.getRepositories).mockResolvedValue(mockRepositories);

      const { result } = renderHook(() => useRepositoryStats(), {
        wrapper: createWrapper(),
      });

      await waitFor(() => {
        expect(result.current).not.toBeNull();
      });

      expect(result.current?.total).toBe(4);
      expect(result.current?.byType.github).toBe(1);
      expect(result.current?.byType.gitlab).toBe(1);
      expect(result.current?.byType.git).toBe(1);
      expect(result.current?.byType.filesystem).toBe(1);
      expect(result.current?.enabled).toBe(3);
      expect(result.current?.disabled).toBe(1);
    });

    it('should handle empty GitHub exclusion fields', async () => {
      const { apiClient } = await import('../../lib/api');

      const mockRepositories: RepositoryWithStats[] = [
        {
          repository: {
            id: '1',
            name: 'github-no-exclusions',
            url: 'https://api.github.com',
            repositoryType: 'GitHub',
            branch: 'main',
            enabled: true,
            gitlabNamespace: null,
            isGroup: false,
            lastCrawled: null,
            createdAt: new Date().toISOString(),
            updatedAt: new Date().toISOString(),
            autoCrawlEnabled: false,
            cronSchedule: null,
            nextCrawlAt: null,
            crawlFrequencyHours: null,
            maxCrawlDurationMinutes: 60,
            lastCrawlDurationSeconds: null,
            gitlabExcludedProjects: null,
            gitlabExcludedPatterns: null,
            githubNamespace: 'user',
            githubExcludedRepositories: null, // No exclusions
            githubExcludedPatterns: null,     // No exclusions
            crawlState: 'idle',
            lastProcessedProject: null,
            crawlStartedAt: null,
          },
          diskSizeMb: null,
          fileCount: null,
          lastCrawlDurationMinutes: null,
        },
      ];

      vi.mocked(apiClient.getRepositories).mockResolvedValue(mockRepositories);

      const { result } = renderHook(() => useRepositoryStats(), {
        wrapper: createWrapper(),
      });

      await waitFor(() => {
        expect(result.current).not.toBeNull();
      });

      expect(result.current?.byType.github).toBe(1);

      // Verify exclusions are null
      const repoData = mockRepositories[0].repository;
      expect(repoData.githubExcludedRepositories).toBeNull();
      expect(repoData.githubExcludedPatterns).toBeNull();
    });
  });

  describe('useCreateRepository', () => {
    it('should create GitHub repository with all fields', async () => {
      const { apiClient } = await import('../../lib/api');

      const mockCreatedRepo = {
        id: 'new-id',
        name: 'New GitHub Repo',
        url: 'https://api.github.com',
        repositoryType: 'GitHub',
        branch: 'main',
        enabled: true,
        gitlabNamespace: null,
        isGroup: false,
        lastCrawled: null,
        createdAt: new Date().toISOString(),
        updatedAt: new Date().toISOString(),
        autoCrawlEnabled: false,
        cronSchedule: null,
        nextCrawlAt: null,
        crawlFrequencyHours: null,
        maxCrawlDurationMinutes: 60,
        lastCrawlDurationSeconds: null,
        gitlabExcludedProjects: null,
        gitlabExcludedPatterns: null,
        githubNamespace: 'test-org',
        githubExcludedRepositories: 'test-org/old-repo',
        githubExcludedPatterns: '*-archive',
        crawlState: 'idle',
        lastProcessedProject: null,
        crawlStartedAt: null,
      };

      vi.mocked(apiClient.createRepository).mockResolvedValue(mockCreatedRepo);

      const { result } = renderHook(() => useCreateRepository(), {
        wrapper: createWrapper(),
      });

      const createData = {
        name: 'New GitHub Repo',
        url: 'https://api.github.com',
        repositoryType: 'GitHub' as const,
        branch: 'main',
        enabled: true,
        accessToken: 'ghp_token',
        githubNamespace: 'test-org',
        githubExcludedRepositories: 'test-org/old-repo',
        githubExcludedPatterns: '*-archive',
      };

      result.current.mutate(createData);

      await waitFor(() => {
        expect(result.current.isSuccess).toBe(true);
      });

      expect(apiClient.createRepository).toHaveBeenCalledWith(createData);
      expect(result.current.data).toEqual(mockCreatedRepo);
    });

    it('should create GitHub repository without namespace', async () => {
      const { apiClient } = await import('../../lib/api');

      const mockCreatedRepo = {
        id: 'new-id-2',
        name: 'All Repos',
        url: 'https://api.github.com',
        repositoryType: 'GitHub',
        branch: 'main',
        enabled: true,
        gitlabNamespace: null,
        isGroup: false,
        lastCrawled: null,
        createdAt: new Date().toISOString(),
        updatedAt: new Date().toISOString(),
        autoCrawlEnabled: false,
        cronSchedule: null,
        nextCrawlAt: null,
        crawlFrequencyHours: null,
        maxCrawlDurationMinutes: 60,
        lastCrawlDurationSeconds: null,
        gitlabExcludedProjects: null,
        gitlabExcludedPatterns: null,
        githubNamespace: null, // No namespace filter
        githubExcludedRepositories: null,
        githubExcludedPatterns: null,
        crawlState: 'idle',
        lastProcessedProject: null,
        crawlStartedAt: null,
      };

      vi.mocked(apiClient.createRepository).mockResolvedValue(mockCreatedRepo);

      const { result } = renderHook(() => useCreateRepository(), {
        wrapper: createWrapper(),
      });

      const createData = {
        name: 'All Repos',
        url: 'https://api.github.com',
        repositoryType: 'GitHub' as const,
        branch: 'main',
        enabled: true,
        accessToken: 'ghp_token',
        githubNamespace: undefined, // Will discover all accessible repos
      };

      result.current.mutate(createData);

      await waitFor(() => {
        expect(result.current.isSuccess).toBe(true);
      });

      expect(apiClient.createRepository).toHaveBeenCalledWith(createData);
    });

    it('should handle error when creating GitHub repository', async () => {
      const { apiClient } = await import('../../lib/api');

      const mockError = new Error('Failed to create repository');
      vi.mocked(apiClient.createRepository).mockRejectedValue(mockError);

      const { result } = renderHook(() => useCreateRepository(), {
        wrapper: createWrapper(),
      });

      const createData = {
        name: 'Error Repo',
        url: 'https://api.github.com',
        repositoryType: 'GitHub' as const,
        enabled: true,
        accessToken: 'invalid_token',
      };

      result.current.mutate(createData);

      await waitFor(() => {
        expect(result.current.isError).toBe(true);
      });

      expect(result.current.error).toBeDefined();
    });

    it('should create GitHub Enterprise repository', async () => {
      const { apiClient } = await import('../../lib/api');

      const mockCreatedRepo = {
        id: 'enterprise-id',
        name: 'Enterprise Repo',
        url: 'https://github.enterprise.com/api/v3',
        repositoryType: 'GitHub',
        branch: 'main',
        enabled: true,
        gitlabNamespace: null,
        isGroup: false,
        lastCrawled: null,
        createdAt: new Date().toISOString(),
        updatedAt: new Date().toISOString(),
        autoCrawlEnabled: false,
        cronSchedule: null,
        nextCrawlAt: null,
        crawlFrequencyHours: null,
        maxCrawlDurationMinutes: 60,
        lastCrawlDurationSeconds: null,
        gitlabExcludedProjects: null,
        gitlabExcludedPatterns: null,
        githubNamespace: 'enterprise-org',
        githubExcludedRepositories: null,
        githubExcludedPatterns: null,
        crawlState: 'idle',
        lastProcessedProject: null,
        crawlStartedAt: null,
      };

      vi.mocked(apiClient.createRepository).mockResolvedValue(mockCreatedRepo);

      const { result } = renderHook(() => useCreateRepository(), {
        wrapper: createWrapper(),
      });

      const createData = {
        name: 'Enterprise Repo',
        url: 'https://github.enterprise.com/api/v3',
        repositoryType: 'GitHub' as const,
        branch: 'main',
        enabled: true,
        accessToken: 'ghp_enterprise_token',
        githubNamespace: 'enterprise-org',
      };

      result.current.mutate(createData);

      await waitFor(() => {
        expect(result.current.isSuccess).toBe(true);
      });

      expect(apiClient.createRepository).toHaveBeenCalledWith(createData);
      expect(result.current.data?.url).toBe('https://github.enterprise.com/api/v3');
    });
  });
});
