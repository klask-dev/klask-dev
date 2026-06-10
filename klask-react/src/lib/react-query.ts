import { QueryClient, type DefaultOptions } from '@tanstack/react-query';
import { isApiError } from './api';

// Default query options
const queryConfig: DefaultOptions = {
  queries: {
    staleTime: 5 * 60 * 1000, // 5 minutes
    gcTime: 10 * 60 * 1000, // 10 minutes (formerly cacheTime)
    retry: (failureCount, error) => {
      // Don't retry on authentication errors
      if (isApiError(error) && (error.status === 401 || error.status === 403)) {
        return false;
      }
      // Retry up to 3 times for other errors
      return failureCount < 3;
    },
    refetchOnWindowFocus: false,
    refetchOnReconnect: true,
  },
  mutations: {
    retry: false,
  },
};

// Create the query client
export const queryClient = new QueryClient({
  defaultOptions: queryConfig,
});

// Query keys factory for consistent key management
export const queryKeys = {
  // Auth
  profile: ['auth', 'profile'] as const,
  
  // Search
  search: (query: string, filters?: Record<string, any>) => 
    ['search', query, filters] as const,
  searchFilters: ['search', 'filters'] as const,
  
  // Files
  files: (params?: Record<string, any>) => ['files', params] as const,
  file: (id: string) => ['files', id] as const,
  
  // Repositories
  repositories: ['repositories'] as const,
  repository: (id: string) => ['repositories', id] as const,
  
  // Users (Admin)
  users: ['admin', 'users'] as const,
  user: (id: string) => ['admin', 'users', id] as const,
  
  // Health
  health: ['health'] as const,
};

// Mutation keys for optimistic updates
export const mutationKeys = {
  // Auth
  login: 'auth:login',
  register: 'auth:register',
  logout: 'auth:logout',
  
  // Repositories
  createRepository: 'repositories:create',
  updateRepository: 'repositories:update',
  deleteRepository: 'repositories:delete',
  crawlRepository: 'repositories:crawl',
  
  // Users
  createUser: 'users:create',
  updateUser: 'users:update',
  deleteUser: 'users:delete',
};

// Helper function to invalidate related queries
export const invalidateQueries = {
  // Invalidate all search-related queries
  search: () => {
    queryClient.invalidateQueries({ queryKey: ['search'] });
    queryClient.invalidateQueries({ queryKey: queryKeys.searchFilters });
  },
  
  // Invalidate all file-related queries
  files: () => {
    queryClient.invalidateQueries({ queryKey: ['files'] });
  },
  
  // Invalidate all repository-related queries
  repositories: () => {
    queryClient.invalidateQueries({ queryKey: queryKeys.repositories });
  },
  
  // Invalidate specific repository
  repository: (id: string) => {
    queryClient.invalidateQueries({ queryKey: queryKeys.repository(id) });
    queryClient.invalidateQueries({ queryKey: queryKeys.repositories });
  },
  
  // Invalidate all user-related queries
  users: () => {
    queryClient.invalidateQueries({ queryKey: ['admin', 'users'] });
  },
  
  // Invalidate profile
  profile: () => {
    queryClient.invalidateQueries({ queryKey: queryKeys.profile });
  },
};

// Error handling utilities
export function handleQueryError(error: unknown) {
  if (isApiError(error)) {
    // Handle authentication errors globally
    if (error.status === 401) {
      // Redirect to login or refresh token
      queryClient.clear();
      window.location.href = '/login';
    }

    // Log other API errors only in development
    if (import.meta.env.DEV) {
      console.error('API Error:', error.message, error.details);
    }
  } else if (import.meta.env.DEV) {
    console.error('Query Error:', error);
  }
}

// Optimistic update helpers
export const optimisticUpdates = {
  // Add repository optimistically
  addRepository: (newRepository: any) => {
    queryClient.setQueryData(queryKeys.repositories, (old: any[] = []) => [
      ...old,
      { ...newRepository, id: `temp-${Date.now()}` }
    ]);
  },
  
  // Update repository optimistically
  updateRepository: (id: string, updates: any) => {
    queryClient.setQueryData(queryKeys.repositories, (old: any[] = []) =>
      old.map(repo => repo.id === id ? { ...repo, ...updates } : repo)
    );
  },
  
  // Remove repository optimistically
  removeRepository: (id: string) => {
    queryClient.setQueryData(queryKeys.repositories, (old: any[] = []) =>
      old.filter(repo => repo.id !== id)
    );
  },
};

// Prefetch utilities
export const prefetch = {
  // Prefetch user profile
  profile: () => {
    queryClient.prefetchQuery({
      queryKey: queryKeys.profile,
      staleTime: 10 * 60 * 1000, // 10 minutes
    });
  },
  
  // Prefetch repositories
  repositories: () => {
    queryClient.prefetchQuery({
      queryKey: queryKeys.repositories,
      staleTime: 5 * 60 * 1000, // 5 minutes
    });
  },
  
  // Prefetch search filters
  searchFilters: () => {
    queryClient.prefetchQuery({
      queryKey: queryKeys.searchFilters,
      staleTime: 10 * 60 * 1000, // 10 minutes
    });
  },
};

// Background sync utilities for real-time updates
export const backgroundSync = {
  // Start syncing repositories (for crawl status updates)
  startRepositorySync: (interval = 30000) => { // 30 seconds
    return setInterval(() => {
      queryClient.invalidateQueries({ queryKey: queryKeys.repositories });
    }, interval);
  },
  
  // Stop syncing
  stopSync: (intervalId: number) => {
    clearInterval(intervalId);
  },
};

export default queryClient;