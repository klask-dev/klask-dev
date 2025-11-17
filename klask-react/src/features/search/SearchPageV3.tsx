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
  DocumentMagnifyingGlassIcon,
  SparklesIcon,
  BoltIcon
} from '@heroicons/react/24/outline';

const SearchPageV3: React.FC = () => {
  const navigate = useNavigate();
  const location = useLocation();
  const [query, setQuery] = useState('');
  const [currentPage, setCurrentPage] = useState(1);
  const [searchMode, setSearchMode] = useState<'normal' | 'fuzzy' | 'regex'>('normal');

  // Derive boolean flags from searchMode for backward compatibility
  const fuzzySearch = searchMode === 'fuzzy';
  const regexSearch = searchMode === 'regex';

  const { history, addToHistory, clearHistory } = useSearchHistory();
  const { filters, setFilters, setCurrentQuery, updateDynamicFilters } = useSearchFiltersContext();
  const sizeFilter = filters.size;

  // Function to update URL with current search state
  const updateURL = useCallback((searchQuery: string, allFilters: typeof filters, page: number = 1) => {
    const params = new URLSearchParams();

    if (searchQuery.trim()) {
      params.set('q', searchQuery.trim());
    }

    // Handle multi-select filters
    if (allFilters.project && allFilters.project.length > 0) {
      allFilters.project.forEach(project => {
        params.append('project', project);
      });
    }

    if (allFilters.version && allFilters.version.length > 0) {
      allFilters.version.forEach(version => {
        params.append('version', version);
      });
    }

    if (allFilters.extension && allFilters.extension.length > 0) {
      allFilters.extension.forEach(extension => {
        params.append('extension', extension);
      });
    }

    if (allFilters.language && allFilters.language.length > 0) {
      allFilters.language.forEach(language => {
        params.append('language', language);
      });
    }

    if (allFilters.repository && allFilters.repository.length > 0) {
      allFilters.repository.forEach(repository => {
        params.append('repository', repository);
      });
    }

    // Handle size filter
    if (allFilters.size) {
      if (allFilters.size.min !== undefined) {
        params.set('min_size', allFilters.size.min.toString());
      }
      if (allFilters.size.max !== undefined) {
        params.set('max_size', allFilters.size.max.toString());
      }
    }

    // Handle search mode flags
    if (fuzzySearch) {
      params.set('fuzzySearch', 'true');
    }
    if (regexSearch) {
      params.set('regexSearch', 'true');
    }

    if (page > 1) {
      params.set('page', page.toString());
    }

    const newURL = params.toString() ? `${window.location.pathname}?${params.toString()}` : window.location.pathname;
    window.history.replaceState(null, '', newURL);
  }, [fuzzySearch, regexSearch]);

  // Track if we're initializing to avoid double URL updates
  const [isInitializing, setIsInitializing] = useState(true);

  // Initialize from URL parameters
  useEffect(() => {
    const urlParams = new URLSearchParams(location.search);
    const urlQuery = urlParams.get('q') || '';

    // Parse multi-select filters (can have multiple values with same key)
    const urlProject = urlParams.getAll('project');
    const urlVersion = urlParams.getAll('version');
    const urlExtension = urlParams.getAll('extension');
    const urlLanguage = urlParams.getAll('language');
    const urlRepository = urlParams.getAll('repository');

    // Parse size filter
    const urlMinSize = urlParams.get('min_size');
    const urlMaxSize = urlParams.get('max_size');

    // Parse search mode flags
    const urlFuzzySearch = urlParams.get('fuzzySearch') === 'true';
    const urlRegexSearch = urlParams.get('regexSearch') === 'true';

    const urlPage = parseInt(urlParams.get('page') || '1', 10);

    // Set React state from URL
    setQuery(urlQuery);

    // Build size filter from URL
    const sizeFilterFromUrl = (urlMinSize || urlMaxSize) ? {
      min: urlMinSize ? parseInt(urlMinSize) : undefined,
      max: urlMaxSize ? parseInt(urlMaxSize) : undefined,
    } : undefined;

    // Build complete filters object from URL
    const filtersFromUrl: typeof filters = {
      ...(urlProject.length > 0 && { project: urlProject }),
      ...(urlVersion.length > 0 && { version: urlVersion }),
      ...(urlExtension.length > 0 && { extension: urlExtension }),
      ...(urlLanguage.length > 0 && { language: urlLanguage }),
      ...(urlRepository.length > 0 && { repository: urlRepository }),
      ...(sizeFilterFromUrl && { size: sizeFilterFromUrl }),
    };

    // Update all filters from URL (only if there are any filters in URL)
    if (Object.keys(filtersFromUrl).length > 0) {
      setFilters(filtersFromUrl);
    }

    // Set search mode from URL
    if (urlRegexSearch) {
      setSearchMode('regex');
    } else if (urlFuzzySearch) {
      setSearchMode('fuzzy');
    } else {
      setSearchMode('normal');
    }

    setCurrentPage(urlPage);
    setIsInitializing(false);
  }, [location.search, setFilters]);

  // Update URL whenever search state changes (only after initialization)
  useEffect(() => {
    if (isInitializing) return;
    updateURL(query, filters, currentPage);
  }, [query, filters, currentPage, updateURL, isInitializing, fuzzySearch, regexSearch]);

  // Sync query to context for facet fetching
  useEffect(() => {
    setCurrentQuery(query);
  }, [query, setCurrentQuery]);

  const {
    data: searchData,
    isLoading,
    isFetching,
    isError,
    error,
    refetch,
  } = useMultiSelectSearch(query, {
    projects: filters?.project,     // Map 'project' (singular in context) to 'projects' (plural in API)
    versions: filters?.version,     // Map 'version' (singular in context) to 'versions' (plural in API)
    extensions: filters?.extension, // Map 'extension' (singular in context) to 'extensions' (plural in API)
    languages: filters?.language,   // Map 'language' (singular in context) to 'languages' (plural in API)
    sizeRange: filters?.size,
  }, currentPage, {
    enabled: !!query.trim(),
  }, fuzzySearch, regexSearch);

  const results = searchData?.results || [];
  const totalResults = searchData?.total || 0;
  const facets = searchData?.facets;

  // Update context with facets from search results
  // When query is empty, clear searchResultsFacets to fallback to lastValidFacets
  useEffect(() => {
    if (!query.trim()) {
      // Query is empty, clear search result facets to use filter-based facets instead
      updateDynamicFilters(null);
    } else if (facets) {
      // Query exists and we have facets - use them (even if search returned 0 results)
      updateDynamicFilters({
        projects: facets.projects,
        versions: facets.versions,
        extensions: facets.extensions,
        repositories: facets.repositories,
        // Backend returns size_ranges in snake_case, not camelCase
        size_ranges: (facets as any).size_ranges || facets.sizeRanges || [],
      });
    }
    // If query exists but facets is still loading/undefined, keep existing filters
    // Don't clear them - wait for the facets to arrive
  }, [facets, query, updateDynamicFilters]);
  const pageSize = 20;
  const totalPages = Math.ceil(totalResults / pageSize);

  const handleSearch = useCallback((searchQuery: string) => {
    if (searchQuery !== query) {
      setCurrentPage(1);
    }

    setQuery(searchQuery);
    setCurrentQuery(searchQuery); // Update context with current query for facet fetching
    if (searchQuery.trim()) {
      addToHistory(searchQuery.trim());
    }
  }, [addToHistory, query, setCurrentQuery]);

  const handleFileClick = useCallback((result: SearchResult) => {
    navigate(`/files/doc/${result.doc_address}`, {
      state: {
        searchQuery: query,
        searchResult: result,
        searchState: {
          initialQuery: query,
          sizeFilter: sizeFilter,
          page: currentPage
        }
      }
    });
  }, [navigate, query, sizeFilter, currentPage]);

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

  // Toggle handlers with mutual exclusivity logic
  const handleFuzzyToggle = useCallback(() => {
    setSearchMode(prev => (prev === 'fuzzy' ? 'normal' : 'fuzzy'));
  }, []);

  const handleRegexToggle = useCallback(() => {
    setSearchMode(prev => (prev === 'regex' ? 'normal' : 'regex'));
  }, []);

  const searchError = isError ? getErrorMessage(error) : null;

  // Count active size filter
  const activeFiltersCount = sizeFilter && (sizeFilter.min !== undefined || sizeFilter.max !== undefined) ? 1 : 0;

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
        <div className="flex gap-4">
          <div className="flex-1">
            <SearchBar
              value={query}
              onChange={setQuery}
              onSearch={handleSearch}
              placeholder="Search functions, classes, variables, comments..."
              isLoading={isLoading || isFetching}
            />
          </div>
          
          {/* Fuzzy Search Toggle */}
          <button
            onClick={handleFuzzyToggle}
            title={fuzzySearch
              ? "Disable fuzzy search (1-character edit distance)"
              : "Enable fuzzy search (1-character edit distance)"}
            className={`px-3 py-2 rounded-md text-sm font-medium transition-all whitespace-nowrap flex items-center gap-1.5 border ${
              fuzzySearch
                ? 'bg-blue-50 dark:bg-blue-900/30 border-blue-200 dark:border-blue-700/50 text-blue-700 dark:text-blue-300 shadow-sm'
                : 'border-gray-200 dark:border-gray-700 text-gray-600 dark:text-gray-400 hover:text-gray-900 dark:hover:text-gray-200 hover:bg-gray-50 dark:hover:bg-gray-800/50'
            }`}
          >
            <BoltIcon className="h-4 w-4" />
            <span className="hidden sm:inline">Fuzzy</span>
          </button>

          {/* Regex Search Toggle */}
          <button
            onClick={handleRegexToggle}
            title={regexSearch
              ? "Disable regex search - use patterns like ^test.*, [a-z]+\\.rs$"
              : "Enable regex search - use patterns like ^test.*, [a-z]+\\.rs$"}
            className={`px-3 py-2 rounded-md text-sm font-medium transition-all whitespace-nowrap flex items-center gap-1.5 border ${
              regexSearch
                ? 'bg-purple-50 dark:bg-purple-900/30 border-purple-200 dark:border-purple-700/50 text-purple-700 dark:text-purple-300 shadow-sm'
                : 'border-gray-200 dark:border-gray-700 text-gray-600 dark:text-gray-400 hover:text-gray-900 dark:hover:text-gray-200 hover:bg-gray-50 dark:hover:bg-gray-800/50'
            }`}
          >
            <span className="text-base">/</span>
            <span className="hidden sm:inline">Regex</span>
          </button>
        </div>

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

      {/* Main Content */}
      <div>
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
        regexSearch={regexSearch}
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
    </div>
  );
};

export default SearchPageV3;
