import React, { useState, useCallback, useEffect } from 'react';
import { useNavigate, useLocation } from 'react-router-dom';
import { SearchBar } from '../../components/search/SearchBar';
import { SearchResults } from '../../components/search/SearchResults';
import { useMultiSelectSearch, useSearchHistory } from '../../hooks/useSearch';
import { getErrorMessage } from '../../lib/api';
import type { SearchResult } from '../../types';
import { useSearchFiltersContext } from '../../contexts/SearchFiltersContext';
import {
  ClockIcon,
  ChartBarIcon,
  DocumentMagnifyingGlassIcon
} from '@heroicons/react/24/outline';

const SearchPage: React.FC = () => {
  const navigate = useNavigate();
  const location = useLocation();
  const [currentPage, setCurrentPage] = useState(1);

  const { history, addToHistory, clearHistory } = useSearchHistory();
  const { filters, setFilters, currentQuery, setCurrentQuery, updateDynamicFilters } = useSearchFiltersContext();

  // Function to update URL with current search state
  const updateURL = useCallback((searchQuery: string, searchFilters: any, page: number = 1) => {
    const params = new URLSearchParams();

    if (searchQuery.trim()) {
      params.set('q', searchQuery.trim());
    }
    // Handle array-based filters
    if (searchFilters.project && searchFilters.project.length > 0) {
      params.set('project', searchFilters.project.join(','));
    }
    if (searchFilters.version && searchFilters.version.length > 0) {
      params.set('version', searchFilters.version.join(','));
    }
    if (searchFilters.extension && searchFilters.extension.length > 0) {
      params.set('extension', searchFilters.extension.join(','));
    }
    if (page > 1) {
      params.set('page', page.toString());
    }

    const newURL = params.toString() ? `${window.location.pathname}?${params.toString()}` : window.location.pathname;
    window.history.replaceState(null, '', newURL);
  }, []);

  // Track if we're initializing to avoid double URL updates
  const [isInitializing, setIsInitializing] = useState(true);

  // Initialize from URL parameters
  useEffect(() => {
    const urlParams = new URLSearchParams(location.search);
    const urlQuery = urlParams.get('q') || '';
    const urlProject = urlParams.get('project');
    const urlVersion = urlParams.get('version');
    const urlExtension = urlParams.get('extension');
    const urlPage = parseInt(urlParams.get('page') || '1', 10);

    // Parse comma-separated values into arrays
    const parseFilterValues = (value: string | null): string[] | undefined => {
      return value ? value.split(',').filter(v => v.trim()) : undefined;
    };

    // Set state from URL
    setCurrentQuery(urlQuery);
    setFilters({
      project: parseFilterValues(urlProject),
      version: parseFilterValues(urlVersion),
      extension: parseFilterValues(urlExtension),
    });
    setCurrentPage(urlPage);
    setIsInitializing(false);
  }, [location.search, setCurrentQuery, setFilters]);

  // Update URL whenever search state changes (only after initialization)
  useEffect(() => {
    if (isInitializing) return;
    updateURL(currentQuery, filters, currentPage);
  }, [currentQuery, filters, currentPage, updateURL, isInitializing]);

  const {
    data: searchData,
    isLoading,
    isFetching,
    isError,
    error,
    refetch,
  } = useMultiSelectSearch(currentQuery, filters as Record<string, string[] | undefined>, currentPage, {
    enabled: !!currentQuery.trim(),
  });

  // Update dynamic filters when search data changes
  useEffect(() => {
    if (searchData?.facets) {
      updateDynamicFilters(searchData.facets);
    } else if (!currentQuery.trim()) {
      // Clear dynamic filters when no query (use static filters)
      updateDynamicFilters(null);
    }
  }, [searchData, currentQuery, updateDynamicFilters]);


  const results = searchData?.results || [];
  const totalResults = searchData?.total || 0;
  const pageSize = 20;
  const totalPages = Math.ceil(totalResults / pageSize);


  const handleSearch = useCallback((searchQuery: string) => {
    // Only reset to page 1 if the query actually changed
    if (searchQuery !== currentQuery) {
      setCurrentPage(1); // Reset to first page on new search
    }

    setCurrentQuery(searchQuery);
    if (searchQuery.trim()) {
      addToHistory(searchQuery.trim());
    }
    // URL will be updated by the useEffect automatically
  }, [addToHistory, currentQuery, setCurrentQuery]);

  const handleFileClick = useCallback((result: SearchResult) => {
    navigate(`/files/doc/${result.doc_address}`, {
      state: {
        searchQuery: currentQuery,
        searchResult: result,
        // Preserve search state for return navigation
        searchState: {
          initialQuery: currentQuery,
          filters: filters,
          page: currentPage
        }
      }
    });
  }, [navigate, currentQuery, filters, currentPage]);

  const handleHistoryClick = useCallback((historicalQuery: string) => {
    // Set query immediately and add to history
    setCurrentQuery(historicalQuery);
    setCurrentPage(1); // Reset to first page
    if (historicalQuery.trim()) {
      addToHistory(historicalQuery.trim());
    }

    // Force a manual refetch of the search query to bypass any debounce issues
    setTimeout(() => {
      if (refetch) {
        refetch();
      }
    }, 50); // Give time for setCurrentQuery to take effect
  }, [addToHistory, refetch, setCurrentQuery]);

  const handlePageChange = useCallback((page: number) => {
    setCurrentPage(page);
    // Scroll to top when changing pages
    window.scrollTo({ top: 0, behavior: 'smooth' });
  }, []);


  const searchError = isError ? getErrorMessage(error) : null;

  return (
    <div className="max-w-7xl mx-auto space-y-6">
      {/* Header */}
      <div className="md:flex md:items-center md:justify-between">
        <div className="min-w-0 flex-1">
          <h1 className="text-2xl font-bold leading-7 text-gray-900 dark:text-white sm:truncate sm:text-3xl sm:tracking-tight">
            Code Search
          </h1>
          <p className="mt-1 text-sm text-gray-500 dark:text-gray-400">
            Search through your indexed repositories with powerful filters and real-time results.
          </p>
        </div>

        <div className="mt-4 md:mt-0 flex items-center space-x-3">

          {totalResults > 0 && (
            <div className="inline-flex items-center px-3 py-2 border border-gray-300 text-sm font-medium rounded-md bg-white text-gray-700 dark:bg-gray-700 dark:text-gray-200">
              <ChartBarIcon className="h-4 w-4 mr-2" />
              {totalResults.toLocaleString()} results
            </div>
          )}
        </div>
      </div>

      {/* Search Bar */}
      <div className="bg-white rounded-lg border border-gray-200 p-6 shadow-sm  dark:bg-gray-800 dark:border-gray-700">
        <SearchBar
          value={currentQuery}
          onChange={setCurrentQuery}
          onSearch={handleSearch}
          placeholder="Search functions, classes, variables, comments..."
          isLoading={isLoading || isFetching}
        />

        {/* Search History */}
        {!currentQuery && history.length > 0 && (
          <div className="mt-4 pt-4 border-t border-gray-100 dark:border-gray-400">
            <div className="flex items-center justify-between mb-3">
              <div className="flex items-center space-x-2">
                <ClockIcon className="h-4 w-4 text-gray-400" />
                <span className="text-sm font-medium text-gray-700 dark:text-gray-400">Recent Searches</span>
              </div>
              <button
                onClick={clearHistory}
                className="text-xs text-gray-500 hover:text-gray-700 dark:text-gray-400"
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
        query={currentQuery}
        isLoading={isLoading}
        error={searchError}
        totalResults={totalResults}
        onFileClick={handleFileClick}
        currentPage={currentPage}
        totalPages={totalPages}
        onPageChange={handlePageChange}
        pageSize={pageSize}
      />

      {/* Search Tips - shown when no query */}
      {!currentQuery.trim() && !isLoading && (
        <div className="bg-gradient-to-br from-primary-50 to-secondary-50 rounded-lg p-6 border border-primary-100 dark:border-gray-700 dark:bg-slate-900">
          <h3 className="text-lg font-medium text-gray-900 mb-4 dark:text-white">
            Search Tips
          </h3>
          <div className="grid grid-cols-1 md:grid-cols-2 gap-4 text-sm">
            <div>
              <h4 className="font-medium text-gray-900 mb-2 dark:text-blue-200">Basic Search</h4>
              <ul className="space-y-1 text-gray-600 dark:text-blue-400">
                <li>• Search for function names, class names, variables</li>
                <li>• Look for specific strings in comments</li>
                <li>• Find TODO items and FIXME comments</li>
                <li>• Search across all indexed repositories</li>
              </ul>
            </div>
            <div>
              <h4 className="font-medium text-gray-900 mb-2 dark:text-blue-200">Advanced Features</h4>
              <ul className="space-y-1 text-gray-600 dark:text-blue-400">
                <li>• Filter by project, version, or file type</li>
                <li>• Results include syntax highlighting</li>
                <li>• Click any result to view the full file</li>
                <li>• Search history is saved locally</li>
              </ul>
            </div>
          </div>
        </div>
      )}
    </div>
  );
};

export default SearchPage;
