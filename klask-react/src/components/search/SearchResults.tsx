import React, { useState, useEffect } from 'react';
import { SearchResult } from './SearchResult';
import { LoadingSpinner } from '../ui/LoadingSpinner';
import { SearchProgress } from './SearchProgress';
import { Pagination } from '../ui/Pagination';
import {
  MagnifyingGlassIcon,
  ExclamationTriangleIcon,
  DocumentMagnifyingGlassIcon
} from '@heroicons/react/24/outline';
import type { SearchResult as SearchResultType } from '../../types';

interface SearchResultsProps {
  results: SearchResultType[];
  query: string;
  isLoading: boolean;
  error: string | null;
  totalResults: number;
  onFileClick: (result: SearchResultType) => void;
  // Pagination props
  currentPage?: number;
  totalPages?: number;
  onPageChange?: (page: number) => void;
  pageSize?: number;
  // Legacy Load More props (optional)
  onLoadMore?: () => void;
  hasNextPage?: boolean;
  className?: string;
  // Search mode
  regexSearch?: boolean;
}

export const SearchResults: React.FC<SearchResultsProps> = ({
  results,
  query,
  isLoading,
  error,
  totalResults,
  onFileClick,
  // Pagination props
  currentPage,
  totalPages,
  onPageChange,
  pageSize = 20,
  // Legacy Load More props
  onLoadMore,
  hasNextPage = false,
  className = '',
  regexSearch = false,
}) => {
  const usePagination = currentPage !== undefined && totalPages !== undefined && onPageChange !== undefined;

  // Show loading indicator only after 1 second to avoid flashing on quick responses
  const [showLoading, setShowLoading] = useState(false);

  useEffect(() => {
    if (isLoading && results.length === 0) {
      // Set timer to show loading indicator after 1 second
      const timer = setTimeout(() => setShowLoading(true), 1000);
      return () => {
        clearTimeout(timer);
        setShowLoading(false);
      };
    } else {
      setShowLoading(false);
    }
  }, [isLoading, results.length]);

  // Empty state when no query
  if (!query.trim() && !isLoading) {
    return (
      <div className={`bg-white dark:bg-gray-800 rounded-lg border border-gray-200 dark:border-gray-700 ${className}`}>
        <div className="flex flex-col items-center justify-center py-12 px-4">
          <MagnifyingGlassIcon className="h-16 w-16 text-gray-300 dark:text-gray-600 mb-4" />
          <h3 className="text-lg font-medium text-gray-900 dark:text-white mb-2">
            Search your codebase
          </h3>
          <p className="text-gray-500 dark:text-gray-400 text-center max-w-md">
            Enter a search term to find files, functions, classes, and content across
            your repositories. Use filters to narrow down your results.
          </p>
          <div className="mt-6 grid grid-cols-1 gap-2 text-sm text-gray-600 dark:text-gray-400">
            <div className="flex items-center space-x-2">
              <kbd className="px-2 py-1 bg-gray-100 dark:bg-gray-700 rounded text-xs dark:text-gray-300">function</kbd>
              <span>Search for function definitions</span>
            </div>
            <div className="flex items-center space-x-2">
              <kbd className="px-2 py-1 bg-gray-100 dark:bg-gray-700 rounded text-xs dark:text-gray-300">class MyClass</kbd>
              <span>Find class declarations</span>
            </div>
            <div className="flex items-center space-x-2">
              <kbd className="px-2 py-1 bg-gray-100 dark:bg-gray-700 rounded text-xs dark:text-gray-300">TODO</kbd>
              <span>Search in comments and strings</span>
            </div>
          </div>
        </div>
      </div>
    );
  }

  // Error state
  if (error) {
    // Detect if it's a timeout error
    const isTimeout = error.toLowerCase().includes('timeout');

    return (
      <div className={`bg-white dark:bg-gray-800 rounded-lg border ${isTimeout ? 'border-orange-200 dark:border-orange-900' : 'border-red-200 dark:border-red-900'} ${className}`}>
        <div className="flex flex-col items-center justify-center py-12 px-4">
          <ExclamationTriangleIcon className={`h-16 w-16 mb-4 ${isTimeout ? 'text-orange-400 dark:text-orange-500' : 'text-red-400 dark:text-red-500'}`} />

          <h3 className={`text-lg font-medium mb-2 ${isTimeout ? 'text-orange-900 dark:text-orange-300' : 'text-red-900 dark:text-red-300'}`}>
            {isTimeout ? 'Search Timeout' : 'Search Error'}
          </h3>

          {isTimeout ? (
            <>
              <p className="text-gray-600 dark:text-gray-400 text-center max-w-md mb-3">
                Your search took longer than 30 seconds and was automatically stopped.
              </p>

              {/* Display the query that caused the timeout */}
              <div className="bg-orange-50 dark:bg-orange-900/20 border border-orange-200 dark:border-orange-800 rounded-lg p-4 mb-4 max-w-md">
                <p className="text-sm font-medium text-orange-900 dark:text-orange-300 mb-2">
                  Query that timed out:
                </p>
                <code className="block text-sm bg-white dark:bg-gray-900 text-gray-900 dark:text-gray-100 px-3 py-2 rounded border border-orange-200 dark:border-orange-700 font-mono break-all">
                  {query}
                </code>
                {regexSearch && (
                  <p className="text-xs text-orange-700 dark:text-orange-400 mt-2">
                    Mode: <span className="font-semibold">Regex Search</span>
                  </p>
                )}
              </div>

              {/* Actionable suggestions */}
              <div className="bg-blue-50 dark:bg-blue-900/20 border border-blue-200 dark:border-blue-800 rounded-lg p-4 max-w-md">
                <p className="text-sm font-medium text-blue-900 dark:text-blue-300 mb-2">
                  💡 How to fix this:
                </p>
                <ul className="text-xs text-gray-700 dark:text-gray-300 space-y-1 list-disc list-inside">
                  {regexSearch && query.trim().startsWith('.*') ? (
                    <>
                      <li>Remove the <code className="bg-gray-200 dark:bg-gray-700 px-1 rounded">.*</code> prefix from your regex pattern</li>
                      <li>Try <code className="bg-gray-200 dark:bg-gray-700 px-1 rounded">{query.replace(/^\.\*/, '')}</code> instead of <code className="bg-gray-200 dark:bg-gray-700 px-1 rounded">{query}</code></li>
                      <li>Use more specific patterns (e.g., <code className="bg-gray-200 dark:bg-gray-700 px-1 rounded">network.*</code> instead of <code className="bg-gray-200 dark:bg-gray-700 px-1 rounded">.*network</code>)</li>
                    </>
                  ) : regexSearch ? (
                    <>
                      <li>Simplify your regex pattern</li>
                      <li>Try a normal search instead of regex mode</li>
                      <li>Add more specific anchors or constraints</li>
                    </>
                  ) : (
                    <>
                      <li>Try using more specific search terms</li>
                      <li>Add filters to narrow down results</li>
                      <li>Search for shorter, more specific phrases</li>
                    </>
                  )}
                </ul>
              </div>

              <button
                onClick={() => window.location.reload()}
                className="mt-4 px-4 py-2 bg-orange-600 hover:bg-orange-700 text-white rounded-md transition-colors"
              >
                Try a Different Query
              </button>
            </>
          ) : (
            <>
              <p className="text-red-600 dark:text-red-400 text-center max-w-md mb-4">
                {error}
              </p>
              <button
                onClick={() => window.location.reload()}
                className="btn-secondary"
              >
                Try Again
              </button>
            </>
          )}
        </div>
      </div>
    );
  }

  // Loading state with progress tracking (only show after 1 second)
  if (showLoading && results.length === 0) {
    return <SearchProgress query={query} isRegex={regexSearch} className={className} />;
  }

  // No results state
  if (!isLoading && results.length === 0 && query.trim()) {
    return (
      <div className={`bg-white dark:bg-gray-800 rounded-lg border border-gray-200 dark:border-gray-700 ${className}`}>
        <div className="flex flex-col items-center justify-center py-12 px-4">
          <DocumentMagnifyingGlassIcon className="h-16 w-16 text-gray-300 dark:text-gray-600 mb-4" />
          <h3 className="text-lg font-medium text-gray-900 dark:text-white mb-2">
            No results found
          </h3>
          <p className="text-gray-500 dark:text-gray-400 text-center max-w-md mb-4">
            No matches found for "{query}". Try adjusting your search terms or filters.
          </p>
          <div className="text-sm text-gray-600 dark:text-gray-400 space-y-1">
            <p>• Check your spelling</p>
            <p>• Try broader search terms</p>
            <p>• Remove or adjust filters</p>
            <p>• Make sure repositories are indexed</p>
          </div>
        </div>
      </div>
    );
  }

  // Results display
  return (
    <div className={`bg-white dark:bg-gray-800 rounded-lg border border-gray-200 dark:border-gray-700 ${className} relative`}>
      {/* Loading overlay banner when refetching with existing results */}
      {isLoading && results.length > 0 && (
        <div className="absolute top-0 left-0 right-0 z-10 bg-blue-50 dark:bg-blue-900/30 border-b border-blue-200 dark:border-blue-700/50 px-4 py-3 rounded-t-lg">
          <div className="flex items-center justify-between space-x-3">
            <div className="flex items-center space-x-2 text-sm text-blue-700 dark:text-blue-300">
              <LoadingSpinner size="sm" />
              <span className="font-medium">Updating search results...</span>
            </div>
            <span className="text-xs text-blue-600 dark:text-blue-400 whitespace-nowrap">Searching for "{query}"</span>
          </div>
        </div>
      )}

      {/* Results Header */}
      <div className={`px-6 py-4 border-b border-gray-200 dark:border-gray-700 ${isLoading && results.length > 0 ? 'pt-14' : ''}`}>
        <div className="flex items-center justify-between">
          <div>
            <h3 className="text-lg font-medium text-gray-900 dark:text-white">
              Search Results
            </h3>
            <p className="text-sm text-gray-500 dark:text-gray-400">
              {totalResults.toLocaleString()} {totalResults === 1 ? 'result' : 'results'} for "{query}"
            </p>
          </div>

          {results.length > 0 && !usePagination && (
            <div className="text-sm text-gray-500 dark:text-gray-400">
              Showing {results.length} of {totalResults}
            </div>
          )}
        </div>
      </div>

      {/* Results List */}
      <div className="space-y-4 p-6">
        {results.map((result, index) => (
          <SearchResult
            key={`${result.file_id}-${index}`}
            result={result}
            query={query}
            onFileClick={onFileClick}
            regexSearch={regexSearch}
          />
        ))}

        {/* Load More Indicator (Legacy) */}
        {!usePagination && hasNextPage && (
          <div className="flex items-center justify-center py-4">
            <LoadingSpinner size="md" />
            <span className="ml-2 text-gray-500 dark:text-gray-400">Loading more results...</span>
          </div>
        )}
      </div>

      {/* Pagination */}
      {usePagination && totalPages! > 1 && (
        <div className="px-6 py-4 border-t border-gray-200 dark:border-gray-700">
          <Pagination
            currentPage={currentPage!}
            totalPages={totalPages!}
            onPageChange={onPageChange!}
            totalResults={totalResults}
            pageSize={pageSize}
          />
        </div>
      )}

      {/* Load More Button (Legacy) */}
      {!usePagination && hasNextPage && !isLoading && (
        <div className="px-6 py-4 border-t border-gray-200 dark:border-gray-700 text-center">
          <button
            onClick={onLoadMore}
            className="btn-secondary"
          >
            Load More Results
          </button>
        </div>
      )}
    </div>
  );
};
