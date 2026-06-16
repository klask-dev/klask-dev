import React from 'react';
import toast from 'react-hot-toast';
import {
  CpuChipIcon,
  ArrowPathIcon,
  XCircleIcon,
  ExclamationTriangleIcon,
} from '@heroicons/react/24/outline';
import { Button } from '../../../components/ui/Button';
import { LoadingSpinner } from '../../../components/ui/LoadingSpinner';
import {
  useSemanticStatus,
  useStartBackfill,
  useCancelBackfill,
} from '../../../api/semanticIndex';

/**
 * Admin card for the semantic (vector) search index.
 *
 * Shows the embedding model/dimension, the number of indexed chunks, and lets an
 * admin rebuild the index from the existing Tantivy documents. While a rebuild
 * runs, it polls the status endpoint and renders a progress bar.
 *
 * Renders nothing when semantic search is not enabled on the server, so it adds
 * no noise to deployments that don't use it.
 */
export const SemanticIndexCard: React.FC = () => {
  // Polls automatically while a rebuild is running (see useSemanticStatus),
  // then stops — a single hook instance, no manual timers.
  const statusQuery = useSemanticStatus();
  const status = statusQuery.data;
  const running = status?.running ?? false;

  const startBackfill = useStartBackfill();
  const cancelBackfill = useCancelBackfill();

  const handleStart = () => {
    startBackfill.mutate(undefined, {
      onSuccess: (data) => toast.success(data.message),
      onError: (err: unknown) => {
        const message = err instanceof Error ? err.message : 'Failed to start rebuild';
        // 409 = already running; message comes from the backend.
        toast.error(message);
      },
    });
  };

  const handleCancel = () => {
    cancelBackfill.mutate(undefined, {
      onSuccess: (data) => toast(data.message),
      onError: (err: unknown) => {
        const message = err instanceof Error ? err.message : 'Failed to cancel rebuild';
        toast.error(message);
      },
    });
  };

  // Hidden until we know semantic search is enabled (avoids a flash of an empty
  // card on servers that don't run it).
  if (!status || !status.enabled) {
    return null;
  }

  const total = status.total ?? 0;
  const progressPct = total > 0 ? Math.min(100, Math.round((status.processed / total) * 100)) : 0;

  return (
    <div className="bg-white dark:bg-gray-800 rounded-lg shadow">
      <div className="px-6 py-4 border-b border-gray-200 dark:border-gray-700 flex items-center gap-2">
        <CpuChipIcon className="h-5 w-5 text-purple-600 dark:text-purple-400" />
        <h2 className="text-lg font-semibold text-gray-900 dark:text-white">Semantic Index</h2>
      </div>

      <div className="p-6 space-y-4">
        <p className="text-sm text-gray-600 dark:text-gray-400">
          Vector index for natural-language code search, kept in sync with the keyword index during crawls.
          Rebuild it to (re-)embed every currently indexed document.
        </p>

        {/* Stats */}
        <div className="grid grid-cols-2 gap-4">
          <div>
            <p className="text-xs uppercase tracking-wide text-gray-500 dark:text-gray-400">Chunks indexed</p>
            <p className="text-xl font-semibold text-gray-900 dark:text-white">
              {status.chunks_indexed.toLocaleString()}
            </p>
          </div>
          <div>
            <p className="text-xs uppercase tracking-wide text-gray-500 dark:text-gray-400">Model</p>
            <p className="text-sm font-medium text-gray-900 dark:text-white break-all">
              {status.model ?? '—'}
              {status.dimension ? (
                <span className="text-gray-500 dark:text-gray-400"> ({status.dimension}d)</span>
              ) : null}
            </p>
          </div>
        </div>

        {/* Progress while running */}
        {running && (
          <div className="space-y-2">
            <div className="flex items-center justify-between text-sm">
              <span className="text-gray-700 dark:text-gray-300">
                Rebuilding… {status.processed.toLocaleString()}
                {total > 0 ? ` / ${total.toLocaleString()}` : ''} documents
              </span>
              <span className="text-gray-500 dark:text-gray-400">{progressPct}%</span>
            </div>
            <div className="w-full bg-gray-200 dark:bg-gray-700 rounded-full h-2 overflow-hidden">
              <div
                className="bg-purple-600 h-2 rounded-full transition-all duration-500"
                style={{ width: `${progressPct}%` }}
                role="progressbar"
                aria-valuenow={progressPct}
                aria-valuemin={0}
                aria-valuemax={100}
              />
            </div>
          </div>
        )}

        {/* Last-run feedback */}
        {!running && status.error && (
          <div className="flex items-start gap-2 text-sm text-red-700 dark:text-red-300 bg-red-50 dark:bg-red-900/20 border border-red-200 dark:border-red-800 rounded-md p-3">
            <ExclamationTriangleIcon className="h-5 w-5 flex-shrink-0 mt-0.5" />
            <span>Last rebuild failed: {status.error}</span>
          </div>
        )}
        {!running && !status.error && status.cancelled && (
          <p className="text-sm text-gray-500 dark:text-gray-400">Last rebuild was cancelled.</p>
        )}

        {/* Actions */}
        <div className="flex items-center gap-3 pt-2">
          {running ? (
            <Button
              variant="outline"
              size="sm"
              onClick={handleCancel}
              disabled={cancelBackfill.isPending}
              className="flex items-center"
            >
              {cancelBackfill.isPending ? (
                <LoadingSpinner size="sm" className="mr-2" />
              ) : (
                <XCircleIcon className="h-4 w-4 mr-2" />
              )}
              Cancel
            </Button>
          ) : (
            <Button
              variant="primary"
              size="sm"
              onClick={handleStart}
              disabled={startBackfill.isPending}
              className="flex items-center"
            >
              {startBackfill.isPending ? (
                <LoadingSpinner size="sm" className="mr-2" />
              ) : (
                <ArrowPathIcon className="h-4 w-4 mr-2" />
              )}
              {status.chunks_indexed > 0 ? 'Rebuild Index' : 'Build Index'}
            </Button>
          )}
        </div>
      </div>
    </div>
  );
};

export default SemanticIndexCard;
