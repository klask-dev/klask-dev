import React, { useState, useCallback, useRef, useEffect } from 'react';
import { toast } from 'react-hot-toast';
import { useQueryClient } from '@tanstack/react-query';
import { 
  PlusIcon,
  FunnelIcon,
  XMarkIcon,
} from '@heroicons/react/24/outline';
import { SelectableRepositoryCard } from '../../components/repositories/SelectableRepositoryCard';
import { Checkbox } from '../../components/ui/Checkbox';
import { RepositoryForm } from '../../components/repositories/RepositoryForm';
import { LoadingSpinner } from '../../components/ui/LoadingSpinner';
import { 
  useRepositories, 
  useCreateRepository, 
  useUpdateRepository, 
  useDeleteRepository, 
  useCrawlRepository,
  useRepositoryStats,
  useBulkRepositoryOperations,
  useActiveProgress,
} from '../../hooks/useRepositories';
import { getErrorMessage, apiClient } from '../../lib/api';
import type { Repository, RepositoryWithStats, CreateRepositoryRequest } from '../../types';

type FilterType = 'all' | 'enabled' | 'disabled' | 'crawled' | 'not-crawled';

const RepositoriesPage: React.FC = () => {
  const queryClient = useQueryClient();
  const [showForm, setShowForm] = useState(false);
  const [editingRepository, setEditingRepository] = useState<RepositoryWithStats | null>(null);
  const [selectedRepos, setSelectedRepos] = useState<string[]>([]);
  const [filter, setFilter] = useState<FilterType>('all');
  const [crawlingRepos, setCrawlingRepos] = useState<Set<string>>(new Set());

  // Ref for select-all checkbox to properly handle indeterminate state
  const selectAllCheckboxRef = useRef<HTMLInputElement>(null);

  const { data: repositories = [], isLoading, error, refetch } = useRepositories();
  const { data: activeProgress = [], refetch: refetchActiveProgress } = useActiveProgress();
  const stats = useRepositoryStats();
  
  // Optimize stats calculation to avoid recalculating on every render
  const memoizedStats = React.useMemo(() => stats, [stats]);
  const createMutation = useCreateRepository();
  const updateMutation = useUpdateRepository();
  const deleteMutation = useDeleteRepository();
  const crawlMutation = useCrawlRepository();
  const { bulkEnable, bulkDisable, bulkCrawl, bulkDelete } = useBulkRepositoryOperations();

  // Helper to check if repository is currently crawling
  const isRepositoryCrawling = useCallback((repositoryId: string) => {
    return activeProgress.some(progress => progress.repository_id === repositoryId) ||
           crawlingRepos.has(repositoryId);
  }, [activeProgress, crawlingRepos]);

  // Helper to get selected repositories that are not crawling
  const selectedReposNotCrawling = useCallback(() => {
    return selectedRepos.filter(repoId => !isRepositoryCrawling(repoId));
  }, [selectedRepos, isRepositoryCrawling]);

  // Helper to get selected repositories that are crawling
  const selectedReposCrawling = useCallback(() => {
    return selectedRepos.filter(repoId => isRepositoryCrawling(repoId));
  }, [selectedRepos, isRepositoryCrawling]);

  const filteredRepositories = repositories.filter(repoWithStats => {
    // Debug: log the structure to understand what we're getting
    if (!repoWithStats) {
      console.log('Found null/undefined repoWithStats');
      return false;
    }
    
    // Check if it's already a Repository (not wrapped in RepositoryWithStats)
    const repo = repoWithStats.repository || repoWithStats;
    if (!repo) {
      console.log('No repository found in:', repoWithStats);
      return false;
    }
    
    switch (filter) {
      case 'enabled':
        return repo.enabled;
      case 'disabled':
        return !repo.enabled;
      case 'crawled':
        return repo.lastCrawled;
      case 'not-crawled':
        return !repo.lastCrawled;
      default:
        return true;
    }
  });

  // Set indeterminate state on select-all checkbox
  useEffect(() => {
    if (selectAllCheckboxRef.current) {
      const isIndeterminate = selectedRepos.length > 0 && selectedRepos.length < filteredRepositories.length;
      selectAllCheckboxRef.current.indeterminate = isIndeterminate;
    }
  }, [selectedRepos.length, filteredRepositories.length]);

  const handleCreate = useCallback(async (data: any) => {
    try {
      // Use the regular create endpoint for all repository types
      await createMutation.mutateAsync(data);
      toast.success('Repository created successfully');
      setShowForm(false);
    } catch (error) {
      toast.error(getErrorMessage(error));
    }
  }, [createMutation]);

  const handleUpdate = useCallback(async (data: any) => {
    if (!editingRepository) return;
    
    try {
      await updateMutation.mutateAsync({ 
        id: editingRepository.repository.id, 
        data 
      });
      setEditingRepository(null);
      toast.success('Repository updated successfully');
    } catch (error) {
      toast.error(getErrorMessage(error));
    }
  }, [editingRepository, updateMutation]);

  const handleDelete = useCallback(async (repositoryWithStats: RepositoryWithStats) => {
    const repository = repositoryWithStats.repository || repositoryWithStats;
    if (!confirm(`Are you sure you want to delete "${repository.name}"?`)) return;
    
    try {
      await deleteMutation.mutateAsync(repository.id);
      toast.success('Repository deleted successfully');
    } catch (error) {
      toast.error(getErrorMessage(error));
    }
  }, [deleteMutation]);

  const handleCrawl = useCallback(async (repositoryWithStats: RepositoryWithStats) => {
    const repository = repositoryWithStats.repository || repositoryWithStats;
    // Check if already crawling
    if (isRepositoryCrawling(repository.id)) {
      toast.error(`Repository "${repository.name}" is already being crawled`);
      return;
    }

    setCrawlingRepos(prev => new Set(prev).add(repository.id));
    
    try {
      await crawlMutation.mutateAsync(repository.id);
      toast.success(`Started crawling "${repository.name}"`);
      // Immediately refresh active progress to show the new crawl
      refetchActiveProgress();
    } catch (error: any) {
      if (error.status === 409) {
        toast.error(`Repository "${repository.name}" is already being crawled`);
      } else {
        toast.error(getErrorMessage(error));
      }
    } finally {
      setCrawlingRepos(prev => {
        const newSet = new Set(prev);
        newSet.delete(repository.id);
        return newSet;
      });
    }
  }, [crawlMutation, isRepositoryCrawling]);

  const handleToggleEnabled = useCallback(async (repositoryWithStats: RepositoryWithStats) => {
    const repository = repositoryWithStats.repository || repositoryWithStats;
    try {
      await updateMutation.mutateAsync({
        id: repository.id,
        data: { enabled: !repository.enabled }
      });
      toast.success(`Repository ${repository.enabled ? 'disabled' : 'enabled'}`);
    } catch (error) {
      toast.error(getErrorMessage(error));
    }
  }, [updateMutation]);

  const handleSelectRepo = useCallback((repoId: string, selected: boolean) => {
    setSelectedRepos(prev => 
      selected 
        ? [...prev, repoId]
        : prev.filter(id => id !== repoId)
    );
  }, []);

  const handleSelectAll = useCallback(() => {
    setSelectedRepos(selectedRepos.length === filteredRepositories.length 
      ? [] 
      : filteredRepositories.map(repoWithStats => (repoWithStats.repository || repoWithStats).id)
    );
  }, [selectedRepos, filteredRepositories]);

  const handleBulkAction = useCallback(async (action: 'enable' | 'disable' | 'crawl' | 'delete') => {
    if (selectedRepos.length === 0) return;

    const actionText = {
      enable: 'enable',
      disable: 'disable', 
      crawl: 'crawl',
      delete: 'delete'
    }[action];

    // For crawl action, warn if some repositories are already being crawled
    if (action === 'crawl') {
      const crawlingCount = selectedReposCrawling().length;
      const availableCount = selectedReposNotCrawling().length;
      
      if (crawlingCount > 0) {
        if (availableCount === 0) {
          toast.error(`All selected repositories are already being crawled`);
          return;
        }
        
        if (!confirm(`${crawlingCount} repositories are already being crawled. Continue with crawling the remaining ${availableCount} repositories?`)) {
          return;
        }
      } else {
        if (!confirm(`Are you sure you want to ${actionText} ${selectedRepos.length} repositories?`)) {
          return;
        }
      }
    } else {
      if (!confirm(`Are you sure you want to ${actionText} ${selectedRepos.length} repositories?`)) {
        return;
      }
    }

    try {
      switch (action) {
        case 'enable':
          await bulkEnable.mutateAsync(selectedRepos);
          toast.success(`Enabled ${selectedRepos.length} repositories`);
          break;
        case 'disable':
          await bulkDisable.mutateAsync(selectedRepos);
          toast.success(`Disabled ${selectedRepos.length} repositories`);
          break;
        case 'crawl':
          const reposToProcess = action === 'crawl' ? selectedReposNotCrawling() : selectedRepos;
          const result = await bulkCrawl.mutateAsync(reposToProcess);
          
          if (result.successful > 0) {
            toast.success(`Started crawling ${result.successful} repositories`);
            // Immediately refresh active progress to show the new crawls
            refetchActiveProgress();
          }
          if (result.alreadyCrawling > 0) {
            toast(`${result.alreadyCrawling} repositories were already being crawled`);
          }
          if (result.failed > 0) {
            toast.error(`Failed to crawl ${result.failed} repositories`);
          }
          break;
        case 'delete':
          await bulkDelete.mutateAsync(selectedRepos);
          toast.success(`Deleted ${selectedRepos.length} repositories`);
          break;
      }
      setSelectedRepos([]);
    } catch (error) {
      toast.error(getErrorMessage(error));
    }
  }, [selectedRepos, selectedReposCrawling, selectedReposNotCrawling, bulkEnable, bulkDisable, bulkCrawl, bulkDelete]);

  if (isLoading) {
    return (
      <div className="max-w-7xl mx-auto">
        <div className="flex items-center justify-center min-h-96">
          <div className="text-center">
            <LoadingSpinner size="lg" className="mb-4" />
            <p className="text-gray-500">Loading repositories...</p>
          </div>
        </div>
      </div>
    );
  }

  if (error) {
    return (
      <div className="max-w-7xl mx-auto">
        <div className="text-center py-12">
          <XMarkIcon className="mx-auto h-16 w-16 text-red-400 mb-4" />
          <h3 className="text-lg font-medium text-gray-900 mb-2">
            Failed to Load Repositories
          </h3>
          <p className="text-gray-500 mb-6">
            {getErrorMessage(error)}
          </p>
          <button onClick={() => refetch()} className="btn-primary">
            Try Again
          </button>
        </div>
      </div>
    );
  }

  return (
    <div className="max-w-7xl mx-auto space-y-6">
      {/* Header */}
      <div className="md:flex md:items-center md:justify-between">
        <div className="min-w-0 flex-1">
          <h1 className="text-2xl font-bold leading-7 text-gray-900 sm:truncate sm:text-3xl sm:tracking-tight">
            Repositories
          </h1>
          <p className="mt-1 text-sm text-gray-500">
            Manage your code repositories and configure crawling settings.
          </p>
        </div>
        <div className="mt-4 md:mt-0">
          <button
            onClick={() => setShowForm(true)}
            className="btn-primary"
          >
            <PlusIcon className="h-4 w-4 mr-2" />
            Add Repository
          </button>
        </div>
      </div>

      {/* Stats */}
      {memoizedStats && (
        <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-4">
          <div className="bg-white border border-gray-200 rounded-lg p-4">
            <div className="text-2xl font-bold text-gray-900">{memoizedStats.total}</div>
            <div className="text-sm text-gray-500">Total Repositories</div>
          </div>
          <div className="bg-white border border-gray-200 rounded-lg p-4">
            <div className="text-2xl font-bold text-green-600">{memoizedStats.enabled}</div>
            <div className="text-sm text-gray-500">Enabled</div>
          </div>
          <div className="bg-white border border-gray-200 rounded-lg p-4">
            <div className="text-2xl font-bold text-blue-600">{memoizedStats.crawled}</div>
            <div className="text-sm text-gray-500">Crawled</div>
          </div>
          <div className="bg-white border border-gray-200 rounded-lg p-4">
            <div className="text-2xl font-bold text-gray-600">
              {memoizedStats.byType.git + memoizedStats.byType.gitlab + memoizedStats.byType.filesystem}
            </div>
            <div className="text-sm text-gray-500">
              Git: {memoizedStats.byType.git} | GitLab: {memoizedStats.byType.gitlab} | FS: {memoizedStats.byType.filesystem}
            </div>
          </div>
        </div>
      )}

      {/* Filters and Bulk Actions */}
      <div className="flex flex-col sm:flex-row sm:items-center sm:justify-between gap-4">
        <div className="flex items-center space-x-2">
          <FunnelIcon className="h-5 w-5 text-gray-400" />
          <select
            value={filter}
            onChange={(e) => setFilter(e.target.value as FilterType)}
            className="text-sm border border-gray-300 rounded-md px-3 py-2 focus:ring-2 focus:ring-blue-500 focus:border-blue-500"
          >
            <option value="all">All Repositories</option>
            <option value="enabled">Enabled Only</option>
            <option value="disabled">Disabled Only</option>
            <option value="crawled">Crawled</option>
            <option value="not-crawled">Not Crawled</option>
          </select>
          
          <span className="text-sm text-gray-500">
            {filteredRepositories.length} repositories
          </span>
        </div>

        {/* Bulk Actions */}
        {selectedRepos.length > 0 && (
          <div className="flex items-center space-x-2">
            <div className="flex flex-col">
              <span className="text-sm text-gray-600">
                {selectedRepos.length} selected
              </span>
              {selectedReposCrawling().length > 0 && (
                <span className="text-xs text-amber-600">
                  ({selectedReposCrawling().length} currently crawling)
                </span>
              )}
            </div>
            <button
              onClick={() => handleBulkAction('enable')}
              className="text-sm px-3 py-1 bg-green-100 text-green-800 rounded hover:bg-green-200"
            >
              Enable
            </button>
            <button
              onClick={() => handleBulkAction('disable')}
              className="text-sm px-3 py-1 bg-gray-100 text-gray-800 rounded hover:bg-gray-200"
            >
              Disable
            </button>
            <button
              onClick={() => handleBulkAction('crawl')}
              disabled={selectedReposNotCrawling().length === 0}
              className={`text-sm px-3 py-1 rounded ${
                selectedReposNotCrawling().length === 0
                  ? 'bg-gray-50 text-gray-400 cursor-not-allowed'
                  : 'bg-blue-100 text-blue-800 hover:bg-blue-200'
              }`}
              title={selectedReposNotCrawling().length === 0
                ? 'All selected repositories are already being crawled'
                : `Crawl ${selectedReposNotCrawling().length} repositories`
              }
              data-testid="bulk-crawl-button"
            >
              Crawl {selectedReposNotCrawling().length > 0 && selectedReposCrawling().length > 0 &&
                `(${selectedReposNotCrawling().length})`}
            </button>
            <button
              onClick={() => handleBulkAction('delete')}
              className="text-sm px-3 py-1 bg-red-100 text-red-800 rounded hover:bg-red-200"
            >
              Delete
            </button>
          </div>
        )}
      </div>

      {/* Select All */}
      {filteredRepositories.length > 0 && (
        <div className="bg-white border border-gray-200 rounded-lg p-4 shadow-sm">
          <div className="flex items-center justify-between">
            <div className="flex items-center space-x-3">
              <input
                ref={selectAllCheckboxRef}
                type="checkbox"
                checked={selectedRepos.length === filteredRepositories.length}
                onChange={(e) => handleSelectAll()}
                className="h-4 w-4 text-blue-600 focus:ring-blue-500 border-gray-300 rounded"
                id="select-all"
              />
              <label htmlFor="select-all" className="text-sm text-gray-700 cursor-pointer">
                Select all repositories ({filteredRepositories.length})
              </label>
            </div>
            
            {selectedRepos.length > 0 && (
              <div className="flex items-center space-x-2 text-sm text-gray-600">
                <span className="font-medium">{selectedRepos.length} selected</span>
                <button
                  onClick={() => setSelectedRepos([])}
                  className="text-blue-600 hover:text-blue-800 font-medium"
                >
                  Clear selection
                </button>
              </div>
            )}
          </div>
        </div>
      )}

      {/* Repositories List - Grouped for GitLab */}
      {filteredRepositories.length === 0 ? (
        <div className="text-center py-12">
          <FunnelIcon className="mx-auto h-16 w-16 text-gray-300 mb-4" />
          <h3 className="text-lg font-medium text-gray-900 mb-2">
            No repositories found
          </h3>
          <p className="text-gray-500 mb-6">
            {filter === 'all' 
              ? "Get started by adding your first repository."
              : `No repositories match the "${filter}" filter.`
            }
          </p>
          {filter === 'all' && (
            <button
              onClick={() => setShowForm(true)}
              className="btn-primary"
            >
              <PlusIcon className="h-4 w-4 mr-2" />
              Add Repository
            </button>
          )}
        </div>
      ) : (
        <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6" style={{ isolation: 'isolate' }}>
          {filteredRepositories.map((repositoryWithStats) => {
            // Handle both RepositoryWithStats and plain Repository
            const repo = repositoryWithStats.repository || repositoryWithStats;
            const repoWithStatsNormalized: RepositoryWithStats = repositoryWithStats.repository 
              ? repositoryWithStats 
              : { 
                  repository: repositoryWithStats as any, 
                  diskSizeMb: undefined, 
                  fileCount: undefined, 
                  lastCrawlDurationMinutes: undefined 
                };
            
            return (
              <div key={repo.id} className="relative" style={{ isolation: 'isolate' }}>
                <SelectableRepositoryCard
                  repository={repoWithStatsNormalized}
                  selected={selectedRepos.includes(repo.id)}
                  onSelect={(selected) => handleSelectRepo(repo.id, selected)}
                  onEdit={setEditingRepository}
                  onDelete={handleDelete}
                  onCrawl={handleCrawl}
                  onToggleEnabled={handleToggleEnabled}
                  activeProgress={activeProgress}
                  isCrawling={crawlingRepos.has(repo.id)}
                />
              </div>
            );
          })}
        </div>
      )}

      {/* Repository Form Modal */}
      <RepositoryForm
        key={editingRepository?.repository.id || 'new'}
        repository={editingRepository?.repository || undefined}
        isOpen={showForm || !!editingRepository}
        onClose={() => {
          setShowForm(false);
          setEditingRepository(null);
        }}
        onSubmit={editingRepository ? handleUpdate : handleCreate}
        isLoading={createMutation.isPending || updateMutation.isPending}
      />

    </div>
  );
};

export default RepositoriesPage;