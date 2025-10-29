import React, { useState } from 'react';
import { useMutation, useQueryClient } from '@tanstack/react-query';
import toast from 'react-hot-toast';
import { api } from '../../lib/api';
import { Button } from '../../components/ui/Button';
import { ConfirmDialog } from '../../components/ui/ConfirmDialog';
import { LoadingSpinner } from '../../components/ui/LoadingSpinner';
import {
  TrashIcon,
  ExclamationTriangleIcon,
  InformationCircleIcon,
  ChartBarIcon,
  DocumentCheckIcon
} from '@heroicons/react/24/outline';

// Import custom hooks and types
import { useIndexMetrics } from '../../hooks/useIndexMetrics';
import { useOptimizeIndex } from '../../api/indexMetrics';
import type { OptimizeIndexResponse } from '../../types/tantivy';

// Import components
import { IndexStatsCard } from './components/IndexStatsCard';
import { SegmentVisualization } from './components/SegmentVisualization';
import { CacheStatsChart } from './components/CacheStatsChart';
import { HealthIndicator } from './components/HealthIndicator';
import { TuningPanel } from './components/TuningPanel';
import { AutoRefreshToggle } from './components/AutoRefreshToggle';

interface IndexResetResponse {
  success: boolean;
  message: string;
  documents_before: number;
  documents_after: number;
}

/**
 * Advanced Index Management Dashboard
 * Displays comprehensive index metrics, health status, and tuning recommendations
 */
export const IndexManagement: React.FC = () => {
  const [showResetDialog, setShowResetDialog] = useState(false);
  const queryClient = useQueryClient();

  // Use custom hook for metrics with auto-refresh
  const {
    stats,
    health,
    tuning,
    isLoading,
    error,
    autoRefreshInterval,
    setAutoRefreshInterval,
    lastUpdateTime,
    nextRefreshTime,
    manualRefresh,
  } = useIndexMetrics({ enableAutoRefresh: true, defaultInterval: 'off' });

  // Optimize index mutation
  const optimizeIndexMutation = useOptimizeIndex();

  // Reset index mutation
  const resetIndexMutation = useMutation({
    mutationFn: async (): Promise<IndexResetResponse> => {
      const response = await api.post<IndexResetResponse>('/api/admin/search/reset-index');
      return response;
    },
    onSuccess: (data) => {
      if (data.success) {
        toast.success(`Index reset successfully. ${data.documents_before} documents removed.`);
        queryClient.invalidateQueries({ queryKey: ['index-metrics'] });
        queryClient.invalidateQueries({ queryKey: ['admin', 'dashboard'] });
      } else {
        toast.error(`Reset failed: ${data.message}`);
      }
      setShowResetDialog(false);
    },
    onError: (error: Error | unknown) => {
      const message = error instanceof Error ? error.message : 'Unknown error';
      toast.error(`Failed to reset index: ${message}`);
      setShowResetDialog(false);
    },
  });

  const handleResetIndex = () => {
    resetIndexMutation.mutate();
  };

  const handleOptimize = async () => {
    try {
      optimizeIndexMutation.mutate(
        { remove_deleted_docs: true, merge_segments: true, rebuild_cache: false },
        {
          onSuccess: (data: OptimizeIndexResponse) => {
            toast.success(
              `Index optimized successfully. Reduced from ${data.segments_before} to ${data.segments_after} segments.`
            );
          },
          onError: (error: any) => {
            toast.error(`Failed to optimize index: ${error.message}`);
          },
        }
      );
    } catch (err) {
      console.error('Optimize error:', err);
    }
  };

  // Error state
  if (error) {
    return (
      <div className="space-y-6">
        <div>
          <h1 className="text-2xl font-bold text-gray-900 dark:text-white">Index Management</h1>
        </div>
        <div className="bg-red-50 dark:bg-red-900/20 border border-red-200 dark:border-red-800 rounded-lg p-6">
          <div className="flex items-start gap-3">
            <ExclamationTriangleIcon className="h-6 w-6 text-red-600 dark:text-red-400 flex-shrink-0 mt-0.5" />
            <div>
              <h3 className="font-semibold text-red-900 dark:text-red-200">Failed to Load Metrics</h3>
              <p className="text-red-700 dark:text-red-300 text-sm mt-1">{error.message}</p>
              <Button
                variant="outline"
                size="sm"
                onClick={manualRefresh}
                className="mt-4"
              >
                Try Again
              </Button>
            </div>
          </div>
        </div>
      </div>
    );
  }

  // Loading state for initial load
  if (isLoading && !stats) {
    return (
      <div className="space-y-6">
        <div>
          <h1 className="text-2xl font-bold text-gray-900 dark:text-white">Index Management</h1>
        </div>
        <div className="flex items-center justify-center py-12">
          <LoadingSpinner />
        </div>
      </div>
    );
  }

  return (
    <div className="space-y-6">
      {/* Header */}
      <div className="flex items-start justify-between">
        <div>
          <h1 className="text-2xl font-bold text-gray-900 dark:text-white">Index Management</h1>
          <p className="mt-2 text-sm text-gray-600 dark:text-gray-400">
            Manage and monitor the Tantivy search index used for file content search.
          </p>
        </div>
      </div>

      {/* Auto-Refresh Control */}
      <AutoRefreshToggle
        interval={autoRefreshInterval}
        onIntervalChange={setAutoRefreshInterval}
        lastUpdate={lastUpdateTime}
        nextRefresh={nextRefreshTime}
        isLoading={isLoading}
        onManualRefresh={manualRefresh}
      />

      {/* Quick Stats Summary */}
      {stats && (
        <div>
          <h2 className="text-lg font-semibold text-gray-900 dark:text-white mb-4">Quick Stats</h2>
          <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-4">
            <IndexStatsCard
              title="Total Documents"
              value={stats.total_documents}
              icon={DocumentCheckIcon}
              healthStatus={stats.total_documents > 0 ? 'healthy' : 'warning'}
            />
            <IndexStatsCard
              title="Index Size"
              value={stats.total_size_mb.toFixed(2)}
              unit="MB"
              icon={ChartBarIcon}
            />
            <IndexStatsCard
              title="Segments"
              value={stats.segment_count}
              healthStatus={
                stats.segment_count > 20
                  ? 'warning'
                  : 'healthy'
              }
            />
            <IndexStatsCard
              title="Cache Hits"
              value={stats.cache_stats.hits}
              healthStatus={
                stats.cache_stats.hit_ratio > 0.5 ? 'healthy' : 'warning'
              }
            />
          </div>
        </div>
      )}

      {/* Main Content Grid */}
      <div className="grid grid-cols-1 lg:grid-cols-3 gap-6">
        {/* Left Column - Health and Tuning */}
        <div className="lg:col-span-1 space-y-6">
          {/* Health Status */}
          {health && (
            <div className="bg-white dark:bg-gray-800 rounded-lg shadow">
              <div className="px-6 py-4 border-b border-gray-200 dark:border-gray-700">
                <h2 className="text-lg font-semibold text-gray-900 dark:text-white">Index Health</h2>
              </div>
              <div className="p-6">
                <HealthIndicator health={health} />
              </div>
            </div>
          )}

          {/* Tuning Recommendations */}
          {tuning && (
            <div className="bg-white dark:bg-gray-800 rounded-lg shadow">
              <div className="px-6 py-4 border-b border-gray-200 dark:border-gray-700">
                <h2 className="text-lg font-semibold text-gray-900 dark:text-white">Tuning</h2>
              </div>
              <div className="p-6">
                <TuningPanel
                  tuning={tuning}
                  onOptimize={handleOptimize}
                  isOptimizing={optimizeIndexMutation.isPending}
                />
              </div>
            </div>
          )}
        </div>

        {/* Right Column - Detailed Metrics */}
        <div className="lg:col-span-2 space-y-6">
          {/* Segments */}
          {stats && (
            <div className="bg-white dark:bg-gray-800 rounded-lg shadow">
              <div className="px-6 py-4 border-b border-gray-200 dark:border-gray-700">
                <h2 className="text-lg font-semibold text-gray-900 dark:text-white">Segments</h2>
              </div>
              <div className="p-6">
                <SegmentVisualization segments={stats.segments} />
              </div>
            </div>
          )}

          {/* Cache Statistics */}
          {stats && (
            <div className="bg-white dark:bg-gray-800 rounded-lg shadow">
              <div className="px-6 py-4 border-b border-gray-200 dark:border-gray-700">
                <h2 className="text-lg font-semibold text-gray-900 dark:text-white">Cache Statistics</h2>
              </div>
              <div className="p-6">
                <CacheStatsChart cache={stats.cache_stats} />
              </div>
            </div>
          )}
        </div>
      </div>

      {/* Danger Zone - Reset Index */}
      <div className="bg-white dark:bg-gray-800 shadow rounded-lg">
        <div className="px-6 py-4 border-b border-gray-200 dark:border-gray-700">
          <h2 className="text-lg font-medium text-gray-900 dark:text-white">Danger Zone</h2>
          <p className="mt-1 text-sm text-gray-600 dark:text-gray-400">
            Irreversible index operations that require caution
          </p>
        </div>

        <div className="p-6">
          <div className="border border-red-200 dark:border-red-800 rounded-lg p-4 bg-red-50 dark:bg-red-900/40">
            <div className="flex items-start gap-3">
              <ExclamationTriangleIcon className="h-6 w-6 text-red-600 dark:text-red-400 flex-shrink-0 mt-0.5" />
              <div className="flex-1">
                <h3 className="text-lg font-semibold text-red-900 dark:text-red-200 mb-2">Reset Search Index</h3>
                <p className="text-sm text-red-700 dark:text-red-300 mb-4">
                  This will completely delete all documents from the search index.
                  All search functionality will be unavailable until repositories are crawled again.
                  <strong className="block mt-2">This action cannot be undone.</strong>
                </p>
                <Button
                  variant="danger"
                  size="sm"
                  onClick={() => setShowResetDialog(true)}
                  disabled={resetIndexMutation.isPending}
                  className="flex items-center"
                >
                  {resetIndexMutation.isPending ? (
                    <LoadingSpinner size="sm" className="mr-2" />
                  ) : (
                    <TrashIcon className="h-4 w-4 mr-2" />
                  )}
                  Reset Index
                </Button>
              </div>
            </div>
          </div>

          {/* Additional Information */}
          <div className="bg-blue-50 dark:bg-blue-900/20 border border-blue-200 dark:border-blue-800 rounded-lg p-4 mt-6">
            <div className="flex items-start gap-3">
              <InformationCircleIcon className="h-5 w-5 text-blue-600 dark:text-blue-400 flex-shrink-0 mt-0.5" />
              <div className="text-sm text-blue-700 dark:text-blue-300">
                <p className="font-semibold mb-2">Index Management Guide</p>
                <ul className="list-disc list-inside space-y-1 text-xs">
                  <li><strong>Optimize Index:</strong> Merge segments and remove deleted documents</li>
                  <li><strong>Reset Index:</strong> Delete all indexed documents (requires recrawl)</li>
                  <li><strong>Tuning:</strong> Follow recommendations for better performance</li>
                  <li>Files are indexed automatically during repository crawling</li>
                  <li>This operation requires administrator privileges</li>
                </ul>
              </div>
            </div>
          </div>
        </div>
      </div>

      {/* Reset Confirmation Dialog */}
      <ConfirmDialog
        isOpen={showResetDialog}
        onClose={() => setShowResetDialog(false)}
        onConfirm={handleResetIndex}
        title="Reset Search Index"
        message="Are you sure you want to reset the search index? This will delete all indexed documents and cannot be undone. Search functionality will be unavailable until files are reindexed."
        confirmText="Reset Index"
        cancelText="Cancel"
        variant="danger"
      />
    </div>
  );
};

export default IndexManagement;
