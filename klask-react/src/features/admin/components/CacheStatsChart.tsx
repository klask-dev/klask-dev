import React from 'react';
import type { CacheStats } from '../../../types/tantivy';

interface CacheStatsChartProps {
  cache: CacheStats;
  className?: string;
}

/**
 * Displays cache statistics with visual representation
 * Shows hits, misses, and hit ratio
 */
export const CacheStatsChart: React.FC<CacheStatsChartProps> = ({
  cache,
  className = '',
}) => {
  const total = cache.hits + cache.misses;
  const hitsPercentage = total > 0 ? (cache.hits / total) * 100 : 0;
  const missesPercentage = total > 0 ? (cache.misses / total) * 100 : 0;

  const hitRatioColor = cache.hit_ratio > 0.8
    ? 'text-green-600'
    : cache.hit_ratio > 0.6
    ? 'text-yellow-600'
    : 'text-red-600';

  const hitRatioBgColor = cache.hit_ratio > 0.8
    ? 'bg-green-100'
    : cache.hit_ratio > 0.6
    ? 'bg-yellow-100'
    : 'bg-red-100';

  return (
    <div className={`space-y-6 ${className}`}>
      {/* Hit Ratio Display */}
      <div className={`${hitRatioBgColor} border-2 ${hitRatioColor} rounded-lg p-6 text-center`}>
        <p className="text-sm font-medium text-gray-600 mb-2">Cache Hit Ratio</p>
        <p className={`text-4xl font-bold ${hitRatioColor}`}>
          {(cache.hit_ratio * 100).toFixed(1)}%
        </p>
        <p className="text-xs text-gray-500 mt-2">
          {cache.hit_ratio > 0.8 && 'Excellent cache performance'}
          {cache.hit_ratio > 0.6 && cache.hit_ratio <= 0.8 && 'Good cache performance'}
          {cache.hit_ratio <= 0.6 && 'Cache performance could be improved'}
        </p>
      </div>

      {/* Pie Chart Representation */}
      <div>
        <div className="flex items-center justify-between mb-4">
          <h3 className="text-sm font-semibold text-gray-900">Hit/Miss Distribution</h3>
          <span className="text-xs font-mono text-gray-500">
            {total.toLocaleString()} total
          </span>
        </div>

        <div className="flex items-end gap-2 mb-6">
          {/* Hits Bar */}
          <div className="flex-1">
            <div className="flex items-end justify-between mb-2">
              <span className="text-xs font-medium text-gray-600">Cache Hits</span>
              <span className="text-sm font-bold text-green-600">
                {cache.hits.toLocaleString()}
              </span>
            </div>
            <div className="h-32 bg-green-100 border-2 border-green-300 rounded-lg relative overflow-hidden">
              <div
                className="w-full h-full bg-green-500 transition-all flex items-center justify-center"
                style={{ height: `${Math.max(hitsPercentage * 0.32, 20)}px` }}
              >
                {hitsPercentage > 15 && (
                  <span className="text-white font-bold text-sm">
                    {hitsPercentage.toFixed(0)}%
                  </span>
                )}
              </div>
            </div>
          </div>

          {/* Misses Bar */}
          <div className="flex-1">
            <div className="flex items-end justify-between mb-2">
              <span className="text-xs font-medium text-gray-600">Cache Misses</span>
              <span className="text-sm font-bold text-red-600">
                {cache.misses.toLocaleString()}
              </span>
            </div>
            <div className="h-32 bg-red-100 border-2 border-red-300 rounded-lg relative overflow-hidden">
              <div
                className="w-full h-full bg-red-500 transition-all flex items-center justify-center"
                style={{ height: `${Math.max(missesPercentage * 0.32, 20)}px` }}
              >
                {missesPercentage > 15 && (
                  <span className="text-white font-bold text-sm">
                    {missesPercentage.toFixed(0)}%
                  </span>
                )}
              </div>
            </div>
          </div>
        </div>
      </div>

      {/* Cache Entries */}
      <div className="bg-gray-50 border border-gray-200 rounded-lg p-4">
        <div className="flex items-center justify-between">
          <div>
            <p className="text-sm font-medium text-gray-600">Cached Entries</p>
            <p className="text-xs text-gray-500 mt-1">
              Number of documents currently in cache
            </p>
          </div>
          <p className="text-3xl font-bold text-gray-900">
            {cache.num_entries.toLocaleString()}
          </p>
        </div>
      </div>

      {/* Legend */}
      <div className="grid grid-cols-3 gap-3 text-xs">
        <div className="flex items-center gap-2">
          <div className="w-3 h-3 rounded-full bg-green-500" />
          <span className="text-gray-600">Hits</span>
        </div>
        <div className="flex items-center gap-2">
          <div className="w-3 h-3 rounded-full bg-red-500" />
          <span className="text-gray-600">Misses</span>
        </div>
        <div className="flex items-center gap-2">
          <div className="w-3 h-3 rounded-full bg-yellow-500" />
          <span className="text-gray-600">Evictions</span>
        </div>
      </div>
    </div>
  );
};
