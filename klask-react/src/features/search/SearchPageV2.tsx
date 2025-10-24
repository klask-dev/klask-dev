import React, { useState, useCallback, useEffect } from 'react';
import { useNavigate, useLocation } from 'react-router-dom';
import { SearchBar } from '../../components/search/SearchBar';
import { SearchFiltersPanel, type SearchFilters, type SearchFacets } from '../../components/search/SearchFiltersPanel';
import { SearchResults } from '../../components/search/SearchResults';
import { useAdvancedSearch, useSearchHistory } from '../../hooks/useSearch';
import { getErrorMessage } from '../../lib/api';
import type { SearchResult } from '../../types';
import {
  ClockIcon,
  ChartBarIcon,
  DocumentMagnifyingGlassIcon,
  AdjustmentsHorizontalIcon
} from '@heroicons/react/24/outline';

const SearchPageV2: React.FC = () => {
  const navigate = useNavigate();
  const location = useLocation();
  const [query, setQuery] = useState('');
  const [filters, setFilters] = useState<SearchFilters>({});
  const [showFilters, setShowFilters] = useState(true);

  const { history, addToHistory, clearHistory } = useSearchHistory();

  // Initialize query from location state (when returning from file view)
  useEffect(() => {
    const initialQuery = location.state?.initialQuery as string;
    const searchFilters = location.state?.filters as SearchFilters;

    if (initialQuery) {
      setQuery(initialQuery);
    }
    if (searchFilters) {
      setFilters(searchFilters);
    }

    // Clear the location state to avoid re-applying on refresh
    if (initialQuery || searchFilters) {
      window.history.replaceState({}, '', window.location.pathname);
    }
  }, [location.state]);

  const {
    results,
    totalResults,
    facets,
    isLoading,
    isFetching,
    isError,
    error,
    hasNextPage,
    isFetchingNextPage,
    fetchNextPage,
    refetch,
  } = useAdvancedSearch(query, filters as Record<string, string | undefined>, {
    enabled: !!query.trim(),
  });

  const handleSearch = useCallback((searchQuery: string) => {
    setQuery(searchQuery);
    if (searchQuery.trim()) {
      addToHistory(searchQuery.trim());
    }
  }, [addToHistory]);

  const handleFileClick = useCallback((result: SearchResult) => {
    navigate(`/files/doc/${result.doc_address}`, {
      state: {
        searchQuery: query,
        searchResult: result,
        // Preserve search state for return navigation
        searchState: {
          initialQuery: query,
          filters: filters,
        }
      }
    });
  }, [navigate, query, filters]);

  const handleHistoryClick = useCallback((historicalQuery: string) => {
    // Set query immediately and add to history
    setQuery(historicalQuery);
    if (historicalQuery.trim()) {
      addToHistory(historicalQuery.trim());
    }

    // Force a manual refetch of the search query to bypass any debounce issues
    setTimeout(() => {
      if (refetch) {
        refetch();
      }
    }, 50); // Give time for setQuery to take effect
  }, [addToHistory, refetch]);

  const handleLoadMore = useCallback(() => {
    if (hasNextPage && !isFetchingNextPage) {
      fetchNextPage();
    }
  }, [hasNextPage, isFetchingNextPage, fetchNextPage]);

  const searchError = isError ? getErrorMessage(error) : null;

  return (
    <div className="h-full flex">
      {/* Left Sidebar - Filters */}
      {showFilters && (
        <div className="w-64 flex-shrink-0 border-r border-gray-200 bg-white overflow-y-auto">
          <SearchFiltersPanel
            filters={filters}
            onFiltersChange={setFilters}
            facets={facets}
            isLoading={isFetching}
          />
        </div>
      )}

      {/* Main Content Area */}
      <div className="flex-1 overflow-y-auto">
        <div className="max-w-6xl mx-auto px-4 sm:px-6 lg:px-8 py-6 space-y-6">
          {/* Header */}
          <div className="md:flex md:items-center md:justify-between">
            <div className="min-w-0 flex-1">
              <h1 className="text-2xl font-bold leading-7 text-gray-900 sm:truncate sm:text-3xl sm:tracking-tight">
                Code Search
              </h1>
              <p className="mt-1 text-sm text-gray-500">
                Search through your indexed repositories with powerful filters and real-time results.
              </p>
            </div>

            <div className="mt-4 md:mt-0 flex items-center space-x-3">
              <button
                onClick={() => setShowFilters(!showFilters)}
                className={`inline-flex items-center px-3 py-2 border text-sm font-medium rounded-md focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-blue-500 ${
                  showFilters
                    ? 'border-blue-300 text-blue-700 bg-blue-50'
                    : 'border-gray-300 text-gray-700 bg-white hover:bg-gray-50'
                }`}
              >
                <AdjustmentsHorizontalIcon className="h-4 w-4 mr-2" />
                {showFilters ? 'Hide Filters' : 'Show Filters'}
              </button>

              {totalResults > 0 && (
                <div className="inline-flex items-center px-3 py-2 border border-gray-300 text-sm font-medium rounded-md bg-white text-gray-700">
                  <ChartBarIcon className="h-4 w-4 mr-2" />
                  {totalResults.toLocaleString()} results
                </div>
              )}
            </div>
          </div>

          {/* Search Bar */}
          <div className="bg-white rounded-lg border border-gray-200 p-6 shadow-sm">
            <SearchBar
              value={query}
              onChange={setQuery}
              onSearch={handleSearch}
              placeholder="Search functions, classes, variables, comments..."
              isLoading={isLoading || isFetching}
            />

            {/* Search History */}
            {!query && history.length > 0 && (
              <div className="mt-4 pt-4 border-t border-gray-100">
                <div className="flex items-center justify-between mb-3">
                  <div className="flex items-center space-x-2">
                    <ClockIcon className="h-4 w-4 text-gray-400" />
                    <span className="text-sm font-medium text-gray-700">Recent Searches</span>
                  </div>
                  <button
                    onClick={clearHistory}
                    className="text-xs text-gray-500 hover:text-gray-700"
                  >
                    Clear all
                  </button>
                </div>
                <div className="flex flex-wrap gap-2">
                  {history.slice(0, 5).map((item, index) => (
                    <button
                      key={index}
                      onClick={() => handleHistoryClick(item)}
                      className="inline-flex items-center px-3 py-1 text-sm bg-gray-100 text-gray-700 rounded-full hover:bg-gray-200 transition-colors"
                    >
                      <DocumentMagnifyingGlassIcon className="h-3 w-3 mr-1" />
                      {item}
                    </button>
                  ))}
                </div>
              </div>
            )}
          </div>

          {/* Search Results */}
          <SearchResults
            results={results}
            query={query}
            isLoading={isLoading}
            error={searchError}
            totalResults={totalResults}
            onFileClick={handleFileClick}
            onLoadMore={handleLoadMore}
            hasNextPage={hasNextPage}
          />

          {/* Search Tips - shown when no query */}
          {!query.trim() && !isLoading && (
            <div className="bg-gradient-to-br from-blue-50 to-indigo-50 rounded-lg p-6 border border-blue-100">
              <h3 className="text-lg font-medium text-gray-900 mb-4">
                Search Tips
              </h3>
              <div className="grid grid-cols-1 md:grid-cols-2 gap-4 text-sm">
                <div>
                  <h4 className="font-medium text-gray-900 mb-2">Basic Search</h4>
                  <ul className="space-y-1 text-gray-600">
                    <li>• Search for function names, class names, variables</li>
                    <li>• Look for specific strings in comments</li>
                    <li>• Find TODO items and FIXME comments</li>
                    <li>• Search across all indexed repositories</li>
                  </ul>
                </div>
                <div>
                  <h4 className="font-medium text-gray-900 mb-2">Advanced Features</h4>
                  <ul className="space-y-1 text-gray-600">
                    <li>• Use filters to narrow down results by project, version, or file type</li>
                    <li>• Facets show counts for each filter value</li>
                    <li>• Click any result to view the full file with syntax highlighting</li>
                    <li>• Your search history is saved locally for quick access</li>
                  </ul>
                </div>
              </div>
            </div>
          )}
        </div>
      </div>
    </div>
  );
};

export default SearchPageV2;
