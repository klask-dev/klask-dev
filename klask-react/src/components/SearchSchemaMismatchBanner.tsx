import React, { useState } from 'react';
import { Link } from 'react-router-dom';
import { ExclamationTriangleIcon, XMarkIcon } from '@heroicons/react/24/outline';
import { useSearchStatus } from '../api/indexMetrics';

/**
 * SearchSchemaMismatchBanner Component
 * Displays a warning banner when the search index schema has changed
 * and needs to be rebuilt. Provides a dismissible option but persists
 * on page reload until the issue is fixed.
 */
export const SearchSchemaMismatchBanner: React.FC = () => {
  const [isDismissed, setIsDismissed] = useState(false);

  // Dynamically refetch status every 5s when mismatch is detected,
  // every 30s when all is good (background check)
  const statusQuery = useSearchStatus(false); // Start without auto-refetch

  const shouldAutoRefetch = statusQuery.data?.schema_mismatch === true ? 5000 : 30000;

  React.useEffect(() => {
    // Set up auto-refetch interval
    const interval = setInterval(() => {
      statusQuery.refetch();
    }, shouldAutoRefetch);

    return () => clearInterval(interval);
  }, [shouldAutoRefetch, statusQuery]);

  // Don't show if dismissed, no data, or no schema mismatch
  if (isDismissed || !statusQuery.data?.schema_mismatch) {
    return null;
  }

  return (
    <div className="bg-yellow-50 dark:bg-yellow-900/20 border-b border-yellow-200 dark:border-yellow-800">
      <div className="mx-auto max-w-7xl px-4 py-4 sm:px-6 lg:px-8">
        <div className="flex items-start gap-4">
          <ExclamationTriangleIcon className="h-6 w-6 text-yellow-600 dark:text-yellow-500 flex-shrink-0 mt-0.5" />
          <div className="flex-1">
            <h3 className="font-semibold text-yellow-900 dark:text-yellow-100">
              Index schema has changed
            </h3>
            <p className="mt-1 text-sm text-yellow-800 dark:text-yellow-200">
              The search index schema needs to be rebuilt. Please go to{' '}
              <Link
                to="/admin/index"
                className="font-medium underline hover:text-yellow-700 dark:hover:text-yellow-300"
              >
                Admin Settings
              </Link>
              {' '}and click the Rebuild Index button to continue searching.
            </p>
            {statusQuery.data?.message && (
              <p className="mt-1 text-xs text-yellow-700 dark:text-yellow-300 italic">
                {statusQuery.data.message}
              </p>
            )}
          </div>
          <button
            onClick={() => setIsDismissed(true)}
            className="ml-auto flex-shrink-0 text-yellow-600 dark:text-yellow-400 hover:text-yellow-700 dark:hover:text-yellow-300 transition-colors"
            aria-label="Dismiss warning"
          >
            <XMarkIcon className="h-5 w-5" />
          </button>
        </div>
      </div>
    </div>
  );
};

export default SearchSchemaMismatchBanner;
