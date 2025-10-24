import React, { useState, useCallback, useEffect } from 'react';
import { useNavigate, useLocation } from 'react-router-dom';
import { SearchBar } from '../../components/search/SearchBar';
import { SearchFiltersV2Component, type SearchFiltersV2 } from '../../components/search/SearchFiltersV2';
import { SearchResults } from '../../components/search/SearchResults';
import { useMultiSelectSearch, useSearchFilters, useSearchHistory } from '../../hooks/useSearch';
import { getErrorMessage } from '../../lib/api';
import type { SearchResult } from '../../types';
import {
  ClockIcon,
  Cog6ToothIcon,
  ChartBarIcon,
  DocumentMagnifyingGlassIcon,
  SparklesIcon
} from '@heroicons/react/24/outline';

const SearchPageV3: React.FC = () => {
  const navigate = useNavigate();
  const location = useLocation();
  const [query, setQuery] = useState('');
  const [filters, setFilters] = useState<SearchFiltersV2>({});
  const [showAdvanced, setShowAdvanced] = useState(false);
  const [currentPage, setCurrentPage] = useState(1);

  const { history, addToHistory, clearHistory } = useSearchHistory();

  // Function to update URL with current search state
  const updateURL = useCallback((searchQuery: string, searchFilters: SearchFiltersV2, advanced: boolean, page: number = 1) => {
    const params = new URLSearchParams();

    if (searchQuery.trim()) {
      params.set('q', searchQuery.trim());
    }

    // Handle multi-select filters
    if (searchFilters.projects && searchFilters.projects.length > 0) {
      searchFilters.projects.forEach(project => {
        params.append('projects', project);
      });
    }

    if (searchFilters.versions && searchFilters.versions.length > 0) {
      searchFilters.versions.forEach(version => {
        params.append('versions', version);
      });
    }

    if (searchFilters.extensions && searchFilters.extensions.length > 0) {
      searchFilters.extensions.forEach(extension => {
        params.append('extensions', extension);
      });
    }

    if (searchFilters.languages && searchFilters.languages.length > 0) {
      searchFilters.languages.forEach(language => {
        params.append('languages', language);
      });
    }

    if (advanced) {
      params.set('advanced', 'true');
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
    const urlProjects = urlParams.getAll('projects');
    const urlVersions = urlParams.getAll('versions');
    const urlExtensions = urlParams.getAll('extensions');
    const urlLanguages = urlParams.getAll('languages');
    const urlAdvanced = urlParams.get('advanced') === 'true';
    const urlPage = parseInt(urlParams.get('page') || '1', 10);

    // Set React state from URL
    setQuery(urlQuery);
    setFilters({
      projects: urlProjects.length > 0 ? urlProjects : undefined,
      versions: urlVersions.length > 0 ? urlVersions : undefined,
      extensions: urlExtensions.length > 0 ? urlExtensions : undefined,
      languages: urlLanguages.length > 0 ? urlLanguages : undefined,
    });
    setShowAdvanced(urlAdvanced);
    setCurrentPage(urlPage);
    setIsInitializing(false);
  }, [location.search]);

  // Update URL whenever search state changes (only after initialization)
  useEffect(() => {
    if (isInitializing) return;
    updateURL(query, filters, showAdvanced, currentPage);
  }, [query, filters, showAdvanced, currentPage, updateURL, isInitializing]);

  const {
    data: availableFilters,
    isLoading: filtersLoading,
    error: filtersError,
  } = useSearchFilters();

  const {
    data: searchData,
    isLoading,
    isFetching,
    isError,
    error,
    refetch,
  } = useMultiSelectSearch(query, filters, currentPage, {
    enabled: !!query.trim(),
  });

  const results = searchData?.results || [];
  const totalResults = searchData?.total || 0;
  const facets = searchData?.facets;
  const pageSize = 20;
  const totalPages = Math.ceil(totalResults / pageSize);

  const handleSearch = useCallback((searchQuery: string) => {
    if (searchQuery !== query) {
      setCurrentPage(1);
    }

    setQuery(searchQuery);
    if (searchQuery.trim()) {
      addToHistory(searchQuery.trim());
    }
  }, [addToHistory, query]);

  const handleFileClick = useCallback((result: SearchResult) => {
    navigate(`/files/doc/${result.doc_address}`, {
      state: {
        searchQuery: query,
        searchResult: result,
        searchState: {
          initialQuery: query,
          filters: filters,
          showAdvanced: showAdvanced,
          page: currentPage
        }
      }
    });
  }, [navigate, query, filters, showAdvanced, currentPage]);

  const handleHistoryClick = useCallback((historicalQuery: string) => {
    setQuery(historicalQuery);
    setCurrentPage(1);
    if (historicalQuery.trim()) {
      addToHistory(historicalQuery.trim());
    }

    setTimeout(() => {
      if (refetch) {
        refetch();
      }
    }, 50);
  }, [addToHistory, refetch]);

  const handlePageChange = useCallback((page: number) => {
    setCurrentPage(page);
    window.scrollTo({ top: 0, behavior: 'smooth' });
  }, []);

  const handleFiltersChange = useCallback((newFilters: SearchFiltersV2) => {
    setFilters(newFilters);
    setCurrentPage(1);
  }, []);

  const searchError = isError ? getErrorMessage(error) : null;

  // Count active filters
  const activeFiltersCount = Object.values(filters).reduce((count, filterArray) =>
    count + (filterArray?.length || 0), 0
  );

  return (
    <div className="max-w-7xl mx-auto space-y-6">
      {/* Header */}
      <div className="md:flex md:items-center md:justify-between">
        <div className="min-w-0 flex-1">
          <div className="flex items-center space-x-2">
            <h1 className="text-2xl font-bold leading-7 text-gray-900 dark:text-white sm:truncate sm:text-3xl sm:tracking-tight">
              Code Search
            </h1>
            <SparklesIcon className="h-6 w-6 text-blue-500" title="Enhanced with multi-select filters" />
          </div>
          <p className="mt-1 text-sm text-gray-500 dark:text-gray-400">
            Search with powerful multi-select filters, faceted results, and real-time suggestions.
          </p>
        </div>

        <div className="mt-4 md:mt-0 flex items-center space-x-3">
          <button
            onClick={() => setShowAdvanced(!showAdvanced)}
            className={`inline-flex items-center px-3 py-2 border text-sm font-medium rounded-md focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-blue-500 dark:focus:ring-offset-gray-900 ${
              showAdvanced
                ? 'border-blue-300 dark:border-blue-700 text-blue-700 dark:text-blue-300 bg-blue-50 dark:bg-blue-900/30'
                : 'border-gray-300 dark:border-gray-600 text-gray-700 dark:text-gray-300 bg-white dark:bg-gray-800 hover:bg-gray-50 dark:hover:bg-gray-700'
            }`}
          >
            <Cog6ToothIcon className="h-4 w-4 mr-2" />
            Advanced Filters
            {activeFiltersCount > 0 && (
              <span className="ml-2 bg-blue-100 dark:bg-blue-900/50 text-blue-800 dark:text-blue-300 text-xs px-2 py-1 rounded-full">
                {activeFiltersCount}
              </span>
            )}
          </button>

          {totalResults > 0 && (
            <div className="inline-flex items-center px-3 py-2 border border-gray-300 dark:border-gray-600 text-sm font-medium rounded-md bg-white dark:bg-gray-800 text-gray-700 dark:text-gray-300">
              <ChartBarIcon className="h-4 w-4 mr-2" />
              {totalResults.toLocaleString()} results
            </div>
          )}
        </div>
      </div>

      {/* Search Bar */}
      <div className="bg-white dark:bg-gray-800 rounded-lg border border-gray-200 dark:border-gray-700 p-6 shadow-sm">
        <SearchBar
          value={query}
          onChange={setQuery}
          onSearch={handleSearch}
          placeholder="Search functions, classes, variables, comments..."
          isLoading={isLoading || isFetching}
        />

        {/* Search History */}
        {!query && history.length > 0 && (
          <div className="mt-4 pt-4 border-t border-gray-100 dark:border-gray-700">
            <div className="flex items-center justify-between mb-3">
              <div className="flex items-center space-x-2">
                <ClockIcon className="h-4 w-4 text-gray-400 dark:text-gray-500" />
                <span className="text-sm font-medium text-gray-700 dark:text-gray-300">Recent Searches</span>
              </div>
              <button
                onClick={clearHistory}
                className="text-xs text-gray-500 dark:text-gray-400 hover:text-gray-700 dark:hover:text-gray-300"
              >
                Clear all
              </button>
            </div>
            <div className="flex flex-wrap gap-2">
              {history.slice(0, 5).map((item, index) => (
                <button
                  key={index}
                  onClick={() => handleHistoryClick(item)}
                  className="inline-flex items-center px-3 py-1 text-sm bg-gray-100 dark:bg-gray-700 text-gray-700 dark:text-gray-300 rounded-full hover:bg-gray-200 dark:hover:bg-gray-600 transition-colors"
                >
                  <DocumentMagnifyingGlassIcon className="h-3 w-3 mr-1" />
                  {item}
                </button>
              ))}
            </div>
          </div>
        )}
      </div>

      {/* Advanced Filters */}
      {showAdvanced && (
        <SearchFiltersV2Component
          filters={filters}
          onFiltersChange={handleFiltersChange}
          availableFilters={{
            // Use dynamic facets from search results if available, fallback to static filters
            repositories: facets?.repositories?.map(({ value, count }: { value: string; count: number }) => ({ value, label: value, count })) || availableFilters?.repositories || [],
            projects: facets?.projects?.map(({ value, count }: { value: string; count: number }) => ({ value, label: value, count })) || availableFilters?.projects || [],
            versions: facets?.versions?.map(({ value, count }: { value: string; count: number }) => ({ value, label: value, count })) || availableFilters?.versions || [],
            extensions: facets?.extensions?.map(({ value, count }: { value: string; count: number }) => ({ value, label: value, count })) || availableFilters?.extensions || [],
            languages: facets?.languages?.map(({ value, count }: { value: string; count: number }) => ({ value, label: value, count })) || availableFilters?.languages || [],
          }}
          isLoading={filtersLoading || isFetching}
          collapsible={false}
          defaultExpanded={true}
        />
      )}

      {/* Error State for Filters */}
      {filtersError && (
        <div className="bg-yellow-50 dark:bg-yellow-900/20 border border-yellow-200 dark:border-yellow-800 rounded-lg p-4">
          <div className="flex">
            <div className="flex-shrink-0">
              <DocumentMagnifyingGlassIcon className="h-5 w-5 text-yellow-400" />
            </div>
            <div className="ml-3">
              <h3 className="text-sm font-medium text-yellow-800 dark:text-yellow-300">
                Filters Unavailable
              </h3>
              <div className="mt-2 text-sm text-yellow-700 dark:text-yellow-200">
                <p>
                  Unable to load search filters. You can still search, but filtering options may be limited.
                </p>
              </div>
            </div>
          </div>
        </div>
      )}

      {/* Search Results */}
      <SearchResults
        results={results}
        query={query}
        isLoading={isLoading}
        error={searchError}
        totalResults={totalResults}
        onFileClick={handleFileClick}
        currentPage={currentPage}
        totalPages={totalPages}
        onPageChange={handlePageChange}
        pageSize={pageSize}
      />

      {/* Tantivy Search Tips */}
      {!query.trim() && !isLoading && (
        <div className="bg-gradient-to-br from-blue-50 dark:from-blue-900/20 to-indigo-50 dark:to-indigo-900/20 rounded-lg p-6 border border-blue-100 dark:border-blue-800">
          <h3 className="text-lg font-medium text-gray-900 dark:text-white mb-4 flex items-center">
            <SparklesIcon className="h-5 w-5 text-blue-500 mr-2" />
            Enhanced Search Tips
          </h3>
          <div className="grid grid-cols-1 md:grid-cols-2 gap-4 text-sm">
            <div>
              <h4 className="font-medium text-gray-900 dark:text-white mb-2">Basic Search</h4>
              <ul className="space-y-1 text-gray-600 dark:text-gray-300">
                <li>• Search for function names, class names, variables</li>
                <li>• Look for specific strings in comments</li>
                <li>• Find TODO items and FIXME comments</li>
                <li>• Full-text search across all indexed repositories</li>
              </ul>
            </div>
            <div>
              <h4 className="font-medium text-gray-900 dark:text-white mb-2">Advanced Features</h4>
              <ul className="space-y-1 text-gray-600 dark:text-gray-300">
                <li>• Multi-select filters for projects, versions, and extensions</li>
                <li>• Real-time faceted search results with counts</li>
                <li>• Powerful Tantivy search engine with relevance scoring</li>
                <li>• URL-based state management for easy sharing</li>
              </ul>
            </div>
          </div>
        </div>
      )}
    </div>
  );
};

export default SearchPageV3;
