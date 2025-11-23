import React, { useState, useEffect } from 'react';
import { LoadingSpinner } from '../ui/LoadingSpinner';
import { ClockIcon, ExclamationCircleIcon } from '@heroicons/react/24/outline';

interface SearchProgressProps {
  query: string;
  isRegex?: boolean;
  className?: string;
}

export const SearchProgress: React.FC<SearchProgressProps> = ({
  query,
  isRegex = false,
  className = '',
}) => {
  const [elapsedSeconds, setElapsedSeconds] = useState(0);

  // Track elapsed time
  useEffect(() => {
    const startTime = Date.now();
    const interval = setInterval(() => {
      const elapsed = Math.floor((Date.now() - startTime) / 1000);
      setElapsedSeconds(elapsed);
    }, 100); // Update every 100ms for smooth display

    return () => clearInterval(interval);
  }, [query]); // Reset timer when query changes

  const isSlowQuery = elapsedSeconds >= 1;
  const isVerySlowQuery = elapsedSeconds >= 5;

  return (
    <div className={`bg-white dark:bg-gray-800 rounded-lg border border-gray-200 dark:border-gray-700 ${className}`}>
      <div className="flex flex-col items-center justify-center py-12 px-4">
        <LoadingSpinner size="lg" className="mb-4" />

        <h3 className="text-lg font-medium text-gray-900 dark:text-white mb-2">
          Searching...
        </h3>

        <p className="text-gray-500 dark:text-gray-400 text-center mb-3">
          Looking for <span className="font-mono text-sm bg-gray-100 dark:bg-gray-700 px-2 py-1 rounded">"{query}"</span> in your codebase
        </p>

        {/* Elapsed time indicator */}
        <div className="flex items-center space-x-2 text-sm text-gray-600 dark:text-gray-400">
          <ClockIcon className="h-4 w-4" />
          <span>{elapsedSeconds}s elapsed</span>
        </div>

        {/* Slow query warning - Blue theme (normal slow) */}
        {isSlowQuery && !isVerySlowQuery && (
          <div className="mt-4 max-w-md p-4 rounded-lg border bg-blue-50 dark:bg-blue-900/20 border-blue-200 dark:border-blue-800">
            <div className="flex items-start space-x-2">
              <ExclamationCircleIcon className="h-5 w-5 flex-shrink-0 mt-0.5 text-blue-500 dark:text-blue-400" />
              <div className="flex-1">
                <p className="text-sm font-medium text-blue-900 dark:text-blue-300">
                  Processing complex search...
                </p>
                <p className="mt-1 text-xs text-gray-600 dark:text-gray-400">
                  {isRegex && query.startsWith('.*') ? (
                    <>
                      <strong>Performance tip:</strong> Regex patterns starting with <code className="bg-gray-200 dark:bg-gray-700 px-1 rounded">.*</code> require scanning the entire index and are very slow. Consider using a more specific pattern like <code className="bg-gray-200 dark:bg-gray-700 px-1 rounded">network.*</code> instead of <code className="bg-gray-200 dark:bg-gray-700 px-1 rounded">.*network</code>.
                    </>
                  ) : isRegex ? (
                    <>Complex regex patterns may take longer to process. The search will complete or timeout after 30 seconds.</>
                  ) : (
                    <>Large result sets may take a moment to process. We're working on it!</>
                  )}
                </p>
              </div>
            </div>
          </div>
        )}

        {/* Slow query warning - Orange theme (very slow) */}
        {isVerySlowQuery && (
          <div className="mt-4 max-w-md p-4 rounded-lg border bg-orange-50 dark:bg-orange-900/20 border-orange-200 dark:border-orange-800">
            <div className="flex items-start space-x-2">
              <ExclamationCircleIcon className="h-5 w-5 flex-shrink-0 mt-0.5 text-orange-500 dark:text-orange-400" />
              <div className="flex-1">
                <p className="text-sm font-medium text-orange-900 dark:text-orange-300">
                  This query is taking longer than usual
                </p>
                <p className="mt-1 text-xs text-gray-600 dark:text-gray-400">
                  {isRegex && query.startsWith('.*') ? (
                    <>
                      <strong>Performance tip:</strong> Regex patterns starting with <code className="bg-gray-200 dark:bg-gray-700 px-1 rounded">.*</code> require scanning the entire index and are very slow. Consider using a more specific pattern like <code className="bg-gray-200 dark:bg-gray-700 px-1 rounded">network.*</code> instead of <code className="bg-gray-200 dark:bg-gray-700 px-1 rounded">.*network</code>.
                    </>
                  ) : isRegex ? (
                    <>Complex regex patterns may take longer to process. The search will complete or timeout after 30 seconds.</>
                  ) : (
                    <>Large result sets may take a moment to process. We're working on it!</>
                  )}
                </p>
              </div>
            </div>
          </div>
        )}

        {/* Timeout warning */}
        {elapsedSeconds >= 25 && (
          <div className="mt-3 text-xs text-red-600 dark:text-red-400">
            Search will timeout in {30 - elapsedSeconds} seconds if not completed
          </div>
        )}
      </div>
    </div>
  );
};
