import React from 'react';
import { ArrowPathIcon } from '@heroicons/react/24/outline';
import type { RefreshInterval } from '../../../hooks/useIndexMetrics';

interface AutoRefreshToggleProps {
  interval: RefreshInterval;
  onIntervalChange: (interval: RefreshInterval) => void;
  lastUpdate: Date | null;
  nextRefresh: Date | null;
  isLoading?: boolean;
  onManualRefresh?: () => Promise<void>;
  className?: string;
}

/**
 * Control component for auto-refresh settings
 * Allows selection of refresh interval and shows timing information
 */
export const AutoRefreshToggle: React.FC<AutoRefreshToggleProps> = ({
  interval,
  onIntervalChange,
  lastUpdate,
  nextRefresh,
  isLoading = false,
  onManualRefresh,
  className = '',
}) => {
  const [isRefreshing, setIsRefreshing] = React.useState(false);

  const intervals: RefreshInterval[] = ['off', '5s', '10s', '30s', '60s'];

  const getIntervalLabel = (val: RefreshInterval): string => {
    switch (val) {
      case '5s':
        return '5 seconds';
      case '10s':
        return '10 seconds';
      case '30s':
        return '30 seconds';
      case '60s':
        return '1 minute';
      case 'off':
      default:
        return 'Off';
    }
  };

  const formatTimeAgo = (date: Date | null): string => {
    if (!date) return 'Never';

    const seconds = Math.floor((Date.now() - date.getTime()) / 1000);
    if (seconds < 60) return `${seconds}s ago`;
    if (seconds < 3600) return `${Math.floor(seconds / 60)}m ago`;
    return `${Math.floor(seconds / 3600)}h ago`;
  };

  const formatCountdown = (date: Date | null): string => {
    if (!date) return 'Never';

    const seconds = Math.floor((date.getTime() - Date.now()) / 1000);
    if (seconds <= 0) return 'Soon';
    if (seconds < 60) return `${seconds}s`;
    if (seconds < 3600) return `${Math.floor(seconds / 60)}m`;
    return `${Math.floor(seconds / 3600)}h`;
  };

  const handleManualRefresh = async () => {
    if (!onManualRefresh) return;

    try {
      setIsRefreshing(true);
      await onManualRefresh();
    } catch (error) {
      console.error('Failed to refresh:', error);
    } finally {
      setIsRefreshing(false);
    }
  };

  const isAutoRefreshEnabled = interval !== 'off';

  return (
    <div className={`space-y-3 ${className}`}>
      {/* Main Control with Timing Information */}
      <div className="bg-gradient-to-r from-blue-50 dark:from-blue-900/20 to-indigo-50 dark:to-indigo-900/20 border border-blue-200 dark:border-blue-800 rounded-lg p-4">
        <div className="grid grid-cols-1 md:grid-cols-3 gap-4">
          {/* Left: Auto-refresh Interval */}
          <div>
            <label className="block text-sm font-semibold text-gray-900 dark:text-white mb-2">
              Auto-refresh Interval
            </label>
            <div className="flex items-center gap-2">
              <div className="relative inline-block flex-1">
                <select
                  value={interval}
                  onChange={(e) => onIntervalChange(e.target.value as RefreshInterval)}
                  disabled={isLoading || isRefreshing}
                  className="w-full appearance-none bg-white dark:bg-gray-700 border border-gray-300 dark:border-gray-600 rounded-lg px-4 py-2 pr-8 text-sm font-medium text-gray-900 dark:text-gray-100 hover:border-gray-400 dark:hover:border-gray-500 focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-transparent disabled:bg-gray-100 dark:disabled:bg-gray-800 disabled:text-gray-500 dark:disabled:text-gray-400 disabled:cursor-not-allowed transition-all"
                >
                  {intervals.map((val) => (
                    <option key={val} value={val}>
                      {getIntervalLabel(val)}
                    </option>
                  ))}
                </select>
                <div className="pointer-events-none absolute inset-y-0 right-0 flex items-center px-2 text-gray-500 dark:text-gray-400">
                  <svg
                    className="fill-current h-4 w-4"
                    xmlns="http://www.w3.org/2000/svg"
                    viewBox="0 0 20 20"
                  >
                    <path d="M9.293 12.95l.707.707L15.657 8l-1.414-1.414L10 10.828 5.757 6.586 4.343 8z" />
                  </svg>
                </div>
              </div>

              {isAutoRefreshEnabled && (
                <div className="inline-flex items-center gap-1.5 px-2.5 py-1 bg-green-100 dark:bg-green-900/30 text-green-800 dark:text-green-300 rounded-full text-xs font-semibold whitespace-nowrap">
                  <div className="w-2 h-2 bg-green-600 dark:bg-green-500 rounded-full animate-pulse" />
                  Active
                </div>
              )}
            </div>
          </div>

          {/* Center: Last Update */}
          <div className="bg-white/50 dark:bg-gray-800/30 rounded-lg p-3 border border-blue-100 dark:border-blue-900/30">
            <p className="text-xs font-medium text-gray-600 dark:text-gray-400">Last Updated</p>
            <p className="text-sm font-mono font-bold text-gray-900 dark:text-gray-100 mt-1">
              {formatTimeAgo(lastUpdate)}
            </p>
            {lastUpdate && (
              <p className="text-xs text-gray-500 dark:text-gray-400 mt-1">
                {lastUpdate.toLocaleTimeString()}
              </p>
            )}
          </div>

          {/* Right: Next Refresh + Manual Refresh Button */}
          <div className="flex flex-col justify-between gap-2">
            <div className={`${
              isAutoRefreshEnabled
                ? 'bg-green-50/50 dark:bg-green-900/20 border-green-200 dark:border-green-800'
                : 'bg-white/50 dark:bg-gray-800/30 border-blue-100 dark:border-blue-900/30'
            } border rounded-lg p-3`}>
              <p className={`text-xs font-medium ${
                isAutoRefreshEnabled
                  ? 'text-green-700 dark:text-green-400'
                  : 'text-gray-600 dark:text-gray-400'
              }`}>Next Refresh</p>
              <p className={`text-sm font-mono font-bold mt-1 ${
                isAutoRefreshEnabled
                  ? 'text-green-900 dark:text-green-300'
                  : 'text-gray-900 dark:text-gray-100'
              }`}>
                {isAutoRefreshEnabled ? formatCountdown(nextRefresh) : 'Disabled'}
              </p>
              {isAutoRefreshEnabled && nextRefresh && (
                <p className={`text-xs mt-1 ${
                  isAutoRefreshEnabled
                    ? 'text-green-700 dark:text-green-400'
                    : 'text-gray-500 dark:text-gray-400'
                }`}>
                  {nextRefresh.toLocaleTimeString()}
                </p>
              )}
            </div>

            {/* Manual Refresh Button */}
            <button
              onClick={handleManualRefresh}
              disabled={isLoading || isRefreshing}
              className="h-10 px-4 rounded-lg border border-blue-300 dark:border-blue-700 bg-white dark:bg-gray-700 hover:bg-blue-50 dark:hover:bg-gray-600 disabled:bg-gray-100 dark:disabled:bg-gray-800 disabled:border-gray-300 dark:disabled:border-gray-600 text-blue-600 dark:text-blue-400 disabled:text-gray-400 dark:disabled:text-gray-500 transition-all flex items-center justify-center gap-2 font-medium text-sm"
              title="Refresh now"
            >
              <ArrowPathIcon
                className={`h-4 w-4 ${isLoading || isRefreshing ? 'animate-spin' : ''}`}
              />
              <span>Refresh</span>
            </button>
          </div>
        </div>
      </div>

      {/* Refresh Status */}
      {(isLoading || isRefreshing) && (
        <div className="flex items-center gap-2 px-4 py-2 bg-blue-100 dark:bg-blue-900/30 border border-blue-300 dark:border-blue-700 rounded-lg">
          <div className="animate-spin rounded-full h-4 w-4 border-2 border-blue-500 dark:border-blue-400 border-t-transparent" />
          <span className="text-sm font-medium text-blue-900 dark:text-blue-300">
            {isRefreshing ? 'Refreshing metrics...' : 'Loading metrics...'}
          </span>
        </div>
      )}

      {/* Info Message */}
      <p className="text-xs text-gray-600 dark:text-gray-400 flex items-start gap-2">
        <span className="font-bold text-gray-400 dark:text-gray-500 mt-0.5">ℹ</span>
        <span>
          Enable auto-refresh to automatically update index metrics at regular intervals.
          Manual refresh always fetches the latest data immediately.
        </span>
      </p>
    </div>
  );
};
