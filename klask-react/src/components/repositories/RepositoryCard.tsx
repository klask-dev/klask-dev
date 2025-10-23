import React, { useState } from 'react';
import { formatDistanceToNow } from 'date-fns';
import {
  FolderIcon,
  PlayCircleIcon,
  PauseCircleIcon,
  ArrowPathIcon,
  PencilIcon,
  TrashIcon,
  EllipsisVerticalIcon,
  CheckCircleIcon,
  XCircleIcon,
  ClockIcon,
  ExclamationTriangleIcon,
  BoltIcon,
  StopCircleIcon,
} from '@heroicons/react/24/outline';
import type { Repository, RepositoryWithStats } from '../../types';
import { LoadingSpinner } from '../ui/LoadingSpinner';
import { CrawlProgressBar, GitLabHierarchicalProgressBar } from '../ui/ProgressBar';
import { ConfirmDialog } from '../ui/ConfirmDialog';
import { useStopCrawl } from '../../hooks/useRepositories';
import { isRepositoryCrawling, getRepositoryProgressFromActive, type CrawlProgressInfo } from '../../hooks/useProgress';

interface RepositoryCardProps {
  repository: RepositoryWithStats;
  onEdit: (repository: RepositoryWithStats) => void;
  onDelete: (repository: RepositoryWithStats) => void;
  onCrawl: (repository: RepositoryWithStats) => void;
  onStopCrawl?: (repository: RepositoryWithStats) => void;
  onToggleEnabled: (repository: RepositoryWithStats) => void;
  activeProgress: CrawlProgressInfo[];
  isLoading?: boolean;
  isCrawling?: boolean;
  className?: string;
}

export const RepositoryCard: React.FC<RepositoryCardProps> = ({
  repository,
  onEdit,
  onDelete,
  onCrawl,
  onStopCrawl,
  onToggleEnabled,
  activeProgress,
  isLoading = false,
  isCrawling = false,
  className = '',
}) => {
  const [showMenu, setShowMenu] = useState(false);
  const [showStopConfirm, setShowStopConfirm] = useState(false);
  // activeProgress is now passed as prop to avoid multiple polling instances
  const stopCrawlMutation = useStopCrawl();

  // Extract repository data and stats
  const isWithStats = 'repository' in repository;
  const repoData = isWithStats ? (repository as RepositoryWithStats).repository : (repository as Repository);
  const stats = isWithStats ? (repository as RepositoryWithStats) : null;

  // Check if this repository is currently crawling
  const isCurrentlyCrawling = isRepositoryCrawling(repoData.id, activeProgress);
  const crawlProgress = getRepositoryProgressFromActive(repoData.id, activeProgress);

  // Override the isCrawling prop with real-time data
  const actuallyIsCrawling = isCurrentlyCrawling || isCrawling;

  const getTypeIcon = (type: string) => {
    switch (type) {
      case 'Git':
        return (
          <svg className="h-5 w-5" viewBox="0 0 24 24" fill="currentColor">
            <title>Git Repository</title>
            <path fill="#F03C2E" d="M23.546 10.93L13.067.452c-.604-.603-1.582-.603-2.188 0L8.708 2.627l2.76 2.76c.645-.215 1.379-.07 1.889.441.516.515.658 1.258.438 1.9l2.658 2.66c.645-.223 1.387-.078 1.9.435.721.721.721 1.884 0 2.604-.719.719-1.881.719-2.6 0-.539-.541-.674-1.337-.404-1.996L12.86 8.955v6.525c.176.086.342.203.488.348.713.721.713 1.883 0 2.6-.719.721-1.889.721-2.609 0-.719-.719-.719-1.879 0-2.598.182-.18.387-.316.605-.406V8.835c-.217-.091-.424-.222-.6-.401-.545-.545-.676-1.342-.396-2.009L7.636 3.7.45 10.881c-.6.605-.6 1.584 0 2.189l10.48 10.477c.604.604 1.582.604 2.186 0l10.43-10.43c.605-.603.605-1.582 0-2.187"/>
          </svg>
        );
      case 'GitHub':
        return (
          <svg className="h-5 w-5" viewBox="0 0 24 24" fill="currentColor">
            <title>GitHub Repository</title>
            <path fill="#24292e" d="M12 0c-6.626 0-12 5.373-12 12 0 5.302 3.438 9.8 8.207 11.387.599.111.793-.261.793-.577v-2.234c-3.338.726-4.033-1.416-4.033-1.416-.546-1.387-1.333-1.756-1.333-1.756-1.089-.745.083-.729.083-.729 1.205.084 1.839 1.237 1.839 1.237 1.07 1.834 2.807 1.304 3.492.997.107-.775.418-1.305.762-1.604-2.665-.305-5.467-1.334-5.467-5.931 0-1.311.469-2.381 1.236-3.221-.124-.303-.535-1.524.117-3.176 0 0 1.008-.322 3.301 1.23.957-.266 1.983-.399 3.003-.404 1.02.005 2.047.138 3.006.404 2.291-1.552 3.297-1.23 3.297-1.23.653 1.653.242 2.874.118 3.176.77.84 1.235 1.911 1.235 3.221 0 4.609-2.807 5.624-5.479 5.921.43.372.823 1.102.823 2.222v3.293c0 .319.192.694.801.576 4.765-1.589 8.199-6.086 8.199-11.386 0-6.627-5.373-12-12-12z"/>
          </svg>
        );
      case 'GitLab':
        return (
          <svg className="h-5 w-5" viewBox="0 0 24 24" fill="currentColor">
            <title>GitLab Repository</title>
            <path fill="#FC6D26" d="M23.955 13.587l-1.342-4.135-2.664-8.189c-.135-.423-.73-.423-.867 0L16.418 9.45H7.582L4.919 1.263c-.135-.423-.73-.423-.867 0L1.386 9.452L.044 13.587a.905.905 0 0 0 .331 1.023L12 23.054l11.625-8.443a.905.905 0 0 0 .33-1.024"/>
          </svg>
        );
      case 'FileSystem':
        return <FolderIcon className="h-5 w-5 text-blue-600" title="Local FileSystem" />;
      default:
        return <FolderIcon className="h-5 w-5 text-gray-400" title="Unknown Repository Type" />;
    }
  };

  const getStatusColor = () => {
    if (!repoData.enabled) return 'text-gray-400';
    if (repoData.lastCrawled) return 'text-green-500';
    return 'text-yellow-500';
  };

  const getStatusIcon = () => {
    if (!repoData.enabled) {
      return <PauseCircleIcon className="h-5 w-5 text-gray-400" />;
    }
    if (repoData.lastCrawled) {
      return <CheckCircleIcon className="h-5 w-5 text-green-500" />;
    }
    return <ExclamationTriangleIcon className="h-5 w-5 text-yellow-500" />;
  };

  const getStatusText = () => {
    if (!repoData.enabled) return 'Disabled';
    if (repoData.lastCrawled) return 'Ready';
    return 'Not Crawled';
  };

  const handleMenuClick = (e: React.MouseEvent) => {
    e.stopPropagation();
    setShowMenu(!showMenu);
  };

  const handleStopCrawlClick = (e: React.MouseEvent) => {
    e.stopPropagation();
    setShowMenu(false);
    setShowStopConfirm(true);
  };

  const handleConfirmStopCrawl = async () => {
    try {
      await stopCrawlMutation?.mutateAsync(repoData.id);
      if (onStopCrawl) {
        onStopCrawl(repository);
      }
      setShowStopConfirm(false);
    } catch (error) {
      console.error('Failed to stop crawl:', error);
    }
  };

  const formatLastCrawled = (date: string | null | undefined) => {
    if (!date) return 'Never';
    try {
      return formatDistanceToNow(new Date(date), { addSuffix: true });
    } catch {
      return 'Unknown';
    }
  };

  const formatDuration = (seconds: number | null | undefined) => {
    if (!seconds) return null;

    if (seconds < 60) {
      return `${seconds}s`;
    } else if (seconds < 3600) {
      const minutes = Math.floor(seconds / 60);
      const remainingSeconds = seconds % 60;
      return remainingSeconds > 0 ? `${minutes}min ${remainingSeconds}sec` : `${minutes}min`;
    } else {
      const hours = Math.floor(seconds / 3600);
      const minutes = Math.floor((seconds % 3600) / 60);
      return minutes > 0 ? `${hours}h ${minutes}min` : `${hours}h`;
    }
  };

  const formatCreatedAt = (date: string | null | undefined) => {
    if (!date) return 'Unknown time ago';
    try {
      const dateObj = new Date(date);
      if (isNaN(dateObj.getTime())) return 'Unknown time ago';
      return formatDistanceToNow(dateObj, { addSuffix: true });
    } catch {
      return 'Unknown time ago';
    }
  };

  const formatNextCrawl = (date: string | null | undefined) => {
    if (!date) return null;
    try {
      const dateObj = new Date(date);
      if (isNaN(dateObj.getTime())) return null;
      return formatDistanceToNow(dateObj, { addSuffix: true });
    } catch {
      return null;
    }
  };

  const formatCrawlDuration = (progressInfo: CrawlProgressInfo | null) => {
    if (!progressInfo) return null;

    const startTime = new Date(progressInfo.started_at);
    const endTime = progressInfo.completed_at
      ? new Date(progressInfo.completed_at)
      : new Date(progressInfo.updated_at);

    const durationMs = endTime.getTime() - startTime.getTime();
    const durationSeconds = Math.floor(durationMs / 1000);

    if (durationSeconds < 60) {
      return `${durationSeconds}s`;
    } else if (durationSeconds < 3600) {
      const minutes = Math.floor(durationSeconds / 60);
      const seconds = durationSeconds % 60;
      return seconds > 0 ? `${minutes}m ${seconds}s` : `${minutes}m`;
    } else {
      const hours = Math.floor(durationSeconds / 3600);
      const minutes = Math.floor((durationSeconds % 3600) / 60);
      return minutes > 0 ? `${hours}h ${minutes}m` : `${hours}h`;
    }
  };

  return (
    <div className={`bg-white border border-gray-200 rounded-lg shadow-sm hover:shadow-md transition-shadow duration-200 relative z-10 ${className}`}>
      <div className="p-6 relative">
        {/* Actions Menu - positioned absolutely in top-right */}
        <div className="absolute top-2 right-2">
          <div className="relative">
            <button
              onClick={handleMenuClick}
              className="p-2 text-gray-400 hover:text-gray-600 rounded-full hover:bg-gray-100"
              disabled={isLoading}
            >
              {isLoading ? (
                <LoadingSpinner size="sm" />
              ) : (
                <EllipsisVerticalIcon className="h-5 w-5" />
              )}
            </button>

            {showMenu && (
              <div className="absolute right-0 mt-2 w-48 bg-white border border-gray-200 rounded-lg shadow-lg z-10">
                <div className="py-1">
                  <button
                    onClick={() => {
                      onEdit(repository);
                      setShowMenu(false);
                    }}
                    className="flex items-center w-full px-4 py-2 text-sm text-gray-700 hover:bg-gray-100"
                  >
                    <PencilIcon className="h-4 w-4 mr-3" />
                    Edit
                  </button>

                  <button
                    onClick={() => {
                      onToggleEnabled(repository);
                      setShowMenu(false);
                    }}
                    className="flex items-center w-full px-4 py-2 text-sm text-gray-700 hover:bg-gray-100"
                  >
                    {repoData.enabled ? (
                      <>
                        <PauseCircleIcon className="h-4 w-4 mr-3" />
                        Disable
                      </>
                    ) : (
                      <>
                        <PlayCircleIcon className="h-4 w-4 mr-3" />
                        Enable
                      </>
                    )}
                  </button>

                  {actuallyIsCrawling ? (
                    <button
                      onClick={handleStopCrawlClick}
                      disabled={stopCrawlMutation?.isPending}
                      className="flex items-center w-full px-4 py-2 text-sm text-red-700 hover:bg-red-50 disabled:opacity-50 disabled:cursor-not-allowed"
                    >
                      {stopCrawlMutation?.isPending ? (
                        <LoadingSpinner size="sm" className="mr-3" />
                      ) : (
                        <StopCircleIcon className="h-4 w-4 mr-3" />
                      )}
                      {stopCrawlMutation?.isPending ? 'Stopping...' : 'Stop Crawl'}
                    </button>
                  ) : (
                    <button
                      onClick={() => {
                        onCrawl(repository);
                        setShowMenu(false);
                      }}
                      disabled={!repoData.enabled}
                      className="flex items-center w-full px-4 py-2 text-sm text-gray-700 hover:bg-gray-100 disabled:opacity-50 disabled:cursor-not-allowed"
                      data-testid={`individual-crawl-menu-button-${repoData.id}`}
                    >
                      <ArrowPathIcon className="h-4 w-4 mr-3" />
                      Crawl Now
                    </button>
                  )}

                  <hr className="my-1" />

                  <button
                    onClick={() => {
                      onDelete(repository);
                      setShowMenu(false);
                    }}
                    className="flex items-center w-full px-4 py-2 text-sm text-red-700 hover:bg-red-50"
                  >
                    <TrashIcon className="h-4 w-4 mr-3" />
                    Delete
                  </button>
                </div>
              </div>
            )}
          </div>
        </div>

        {/* Header */}
        <div className="mb-4 pr-12">
          <div className="flex items-center space-x-3 min-w-0">
            <div className="flex-shrink-0">
              {getTypeIcon(repoData.repositoryType)}
            </div>
            <div className="min-w-0 flex-1">
              <h3 className="text-lg font-semibold text-gray-900 truncate">
                {repoData.name}
              </h3>
              <p className="text-sm text-gray-500 truncate">
                {repoData.url}
              </p>
              {/* Display stats if available */}
              {stats && (stats.diskSizeMb !== undefined || stats.fileCount !== undefined) && (
                <div className="flex items-center space-x-3 mt-1">
                  {stats.diskSizeMb !== undefined && stats.diskSizeMb > 0 && (
                    <span className="inline-flex items-center px-2 py-0.5 rounded text-xs font-medium bg-purple-100 text-purple-800">
                      📁 {stats.diskSizeMb < 1
                        ? `${(stats.diskSizeMb * 1024).toFixed(0)} KB`
                        : stats.diskSizeMb < 1024
                        ? `${stats.diskSizeMb.toFixed(1)} MB`
                        : `${(stats.diskSizeMb / 1024).toFixed(1)} GB`
                      }
                    </span>
                  )}
                  {stats.fileCount !== undefined && stats.fileCount > 0 && (
                    <span className="inline-flex items-center px-2 py-0.5 rounded text-xs font-medium bg-blue-100 text-blue-800">
                      📄 {stats.fileCount.toLocaleString()} files
                    </span>
                  )}
                </div>
              )}
            </div>
          </div>
        </div>

        {/* Status */}
        <div className="flex items-center space-x-4 mb-4">
          <div className="flex items-center space-x-2">
            {getStatusIcon()}
            <span className={`text-sm font-medium ${getStatusColor()}`}>
              {getStatusText()}
              {crawlProgress && actuallyIsCrawling && formatCrawlDuration(crawlProgress) && (
                <span className="text-xs text-gray-500 ml-2">
                  ({formatCrawlDuration(crawlProgress)})
                </span>
              )}
            </span>
          </div>
        </div>

        {/* Badges - positioned above metadata, horizontal and right-aligned */}
        <div className="flex justify-end items-center gap-2 mb-3">
          <span className="text-xs bg-gray-100 px-2 py-0.5 rounded">
            {repoData.repositoryType}
          </span>
          {repoData.branch && (
            <span className="text-xs bg-blue-100 text-blue-800 px-2 py-0.5 rounded">
              {repoData.branch}
            </span>
          )}
          {repoData.autoCrawlEnabled && (
            <span className="text-xs bg-green-100 text-green-800 px-2 py-0.5 rounded flex items-center space-x-1">
              <BoltIcon className="h-3 w-3" />
              <span>Auto-crawl</span>
            </span>
          )}
        </div>

        {/* Metadata */}
        <div className="space-y-2 text-sm text-gray-600">
          <div className="flex items-center space-x-2">
            <ClockIcon className="h-4 w-4 flex-shrink-0" />
            <span>Last crawled: {formatLastCrawled(repoData.lastCrawled)}</span>
            {repoData.lastCrawlDurationSeconds && (
              <span className="text-gray-400">
                ({formatDuration(repoData.lastCrawlDurationSeconds)})
              </span>
            )}
          </div>

          {repoData.autoCrawlEnabled && repoData.nextCrawlAt && (
            <div className="flex items-center space-x-2 text-green-600">
              <BoltIcon className="h-4 w-4 flex-shrink-0" />
              <span>Next crawl: {formatNextCrawl(repoData.nextCrawlAt)}</span>
            </div>
          )}

          <div className="flex items-center space-x-2">
            <span className="text-xs text-gray-500">
              Created {formatCreatedAt(repoData.createdAt)}
            </span>
          </div>
        </div>

        {/* Progress Bar - Show when crawling */}
        {crawlProgress && actuallyIsCrawling && (
          <div className="mt-4">
            {repoData.repositoryType === 'GitLab' ? (
              <GitLabHierarchicalProgressBar
                progressInfo={crawlProgress}
              />
            ) : (
              <CrawlProgressBar
                repositoryName={crawlProgress.repository_name}
                status={crawlProgress.status}
                progress={crawlProgress.progress_percentage}
                filesProcessed={crawlProgress.files_processed}
                filesTotal={crawlProgress.files_total}
                filesIndexed={crawlProgress.files_indexed}
                currentFile={crawlProgress.current_file}
              />
            )}
          </div>
        )}

        {/* Quick Actions */}
        <div className="flex items-center justify-between mt-4 pt-4 border-t border-gray-100">
          <div className="flex space-x-2">
            <button
              onClick={() => onToggleEnabled(repository)}
              disabled={isLoading}
              className={`inline-flex items-center px-3 py-1 text-xs font-medium rounded-full transition-colors ${
                repoData.enabled
                  ? 'bg-green-100 text-green-800 hover:bg-green-200'
                  : 'bg-gray-100 text-gray-800 hover:bg-gray-200'
              }`}
            >
              {repoData.enabled ? (
                <>
                  <CheckCircleIcon className="h-3 w-3 mr-1" />
                  Enabled
                </>
              ) : (
                <>
                  <XCircleIcon className="h-3 w-3 mr-1" />
                  Disabled
                </>
              )}
            </button>
          </div>

          {actuallyIsCrawling ? (
            <button
              onClick={handleStopCrawlClick}
              disabled={stopCrawlMutation?.isPending || isLoading}
              className="inline-flex items-center px-3 py-1 text-xs font-medium text-red-700 bg-red-100 rounded-full hover:bg-red-200 disabled:opacity-50 disabled:cursor-not-allowed transition-colors"
            >
              {stopCrawlMutation?.isPending ? (
                <LoadingSpinner size="sm" className="h-3 w-3 mr-1" />
              ) : (
                <StopCircleIcon className="h-3 w-3 mr-1" />
              )}
              {stopCrawlMutation?.isPending ? 'Stopping' : 'Stop'}
            </button>
          ) : (
            <button
              onClick={() => onCrawl(repository)}
              disabled={!repoData.enabled || isLoading}
              className="inline-flex items-center px-3 py-1 text-xs font-medium text-blue-700 bg-blue-100 rounded-full hover:bg-blue-200 disabled:opacity-50 disabled:cursor-not-allowed transition-colors"
              data-testid={`individual-crawl-button-${repoData.id}`}
            >
              <ArrowPathIcon className="h-3 w-3 mr-1" />
              Crawl
            </button>
          )}
        </div>
      </div>

      {/* Click outside to close menu */}
      {showMenu && (
        <div
          className="fixed inset-0 z-0"
          onClick={() => setShowMenu(false)}
        />
      )}

      {/* Stop crawl confirmation dialog */}
      <ConfirmDialog
        isOpen={showStopConfirm}
        onClose={() => setShowStopConfirm(false)}
        onConfirm={handleConfirmStopCrawl}
        title="Stop Crawl"
        message={`Are you sure you want to stop the crawl for "${repoData.name}"? This will cancel the current operation and any progress will be lost.`}
        confirmText="Stop Crawl"
        cancelText="Cancel"
        variant="warning"
        isLoading={stopCrawlMutation?.isPending}
      />
    </div>
  );
};
