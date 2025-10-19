/**
 * SearchFiltersContext - Manages search filter state with automatic facet count updates
 *
 * This context provides:
 * - Filter state management (selected filters across categories)
 * - Automatic facet fetching when filters change via useFacetsWithFilters hook
 * - Hybrid display strategy: shows dynamic counts when filters change
 * - Merging of static filter lists with dynamic counts for accurate UI display
 */
/* eslint-disable react-refresh/only-export-components */
import React, { createContext, useContext, useState, useCallback, useEffect } from 'react';
import { useSearchFilters, useFacetsWithFilters } from '../hooks/useSearch';

export interface SearchFilters {
  project?: string[];
  version?: string[];
  extension?: string[];
  language?: string[];
  repository?: string[];
  [key: string]: string[] | undefined;
}

interface FilterOption {
  value: string;
  label: string;
  count: number;
}

interface DynamicFilters {
  projects?: Array<{ value: string; count: number }>;
  versions?: Array<{ value: string; count: number }>;
  extensions?: Array<{ value: string; count: number }>;
  repositories?: Array<{ value: string; count: number }>;
}

interface SearchFiltersContextType {
  filters: SearchFilters;
  setFilters: (filters: SearchFilters) => void;
  clearFilters: () => void;
  currentQuery: string;
  setCurrentQuery: (query: string) => void;
  availableFilters: {
    projects: FilterOption[];
    versions: FilterOption[];
    extensions: FilterOption[];
    languages: FilterOption[];
    repositories: FilterOption[];
  };
  isLoading: boolean;
  updateDynamicFilters: (facets: DynamicFilters | null) => void;
}

const SearchFiltersContext = createContext<SearchFiltersContextType | undefined>(undefined);

export const useSearchFiltersContext = () => {
  const context = useContext(SearchFiltersContext);
  if (!context) {
    throw new Error('useSearchFiltersContext must be used within a SearchFiltersProvider');
  }
  return context;
};

export const SearchFiltersProvider: React.FC<{ children: React.ReactNode }> = ({ children }) => {
  const [filters, setFilters] = useState<SearchFilters>({});
  const [currentQuery, setCurrentQuery] = useState('');

  const {
    data: staticFilters,
    isLoading,
  } = useSearchFilters({ enabled: true }); // Only load filters in search context

  // Fix 5: Memoize filterParams to prevent unnecessary hook re-triggers
  const filterParams = React.useMemo(() => ({
    project: filters.project,
    version: filters.version,
    extension: filters.extension,
    repository: filters.repository,
  }), [filters.project, filters.version, filters.extension, filters.repository]);

  // Track the last successfully fetched facets to avoid showing zero counts during debounce
  // Initialize with staticFilters to have data during first filter selection debounce
  const [lastValidFacets, setLastValidFacets] = React.useState<DynamicFilters | null>(() => {
    if (!staticFilters) return null;
    return {
      projects: staticFilters.projects,
      versions: staticFilters.versions,
      extensions: staticFilters.extensions,
      repositories: staticFilters.repositories,
    };
  });

  // Track facets from search results when a query is performed
  // These facets are for the current search query and take precedence over filter-based facets
  const [searchResultsFacets, setSearchResultsFacets] = React.useState<DynamicFilters | null>(null);

  // Use a ref to track whether we need to reset on filter clear
  const shouldResetRef = React.useRef(false);

  // Check if any filters are active
  const hasActiveFilters = Object.values(filterParams).some(arr => arr && arr.length > 0);

  // Fetch facets when filters change (automatically triggered when filters have values)
  // Pass currentQuery to combine with filters for accurate facet counts
  // Debounce is set to 300ms to prevent excessive API calls during rapid filter selections
  const { data: filterFacets, isLoading: isFacetsLoading } = useFacetsWithFilters(
    filterParams,
    currentQuery,
    { enabled: true, staleTime: 60000, debounceMs: 300 }
  );

  // Initialize lastValidFacets with staticFilters when they become available
  // This ensures we have initial data even before any filter is applied
  React.useEffect(() => {
    if (staticFilters && !lastValidFacets) {
      setLastValidFacets({
        projects: staticFilters.projects || [],
        versions: staticFilters.versions || [],
        extensions: staticFilters.extensions || [],
        repositories: staticFilters.repositories || [],
      });
    }
  }, [staticFilters, lastValidFacets]);

  // Update lastValidFacets when new data arrives from API
  React.useEffect(() => {
    if (filterFacets) {
      // We have new facets from the API, use them
      setLastValidFacets(filterFacets);
      shouldResetRef.current = true; // Next time filters clear, we should reset
    }
  }, [filterFacets]);

  // When all filters are cleared, reset to staticFilters to show all options again
  React.useEffect(() => {
    if (!hasActiveFilters && shouldResetRef.current && staticFilters) {
      setLastValidFacets({
        projects: staticFilters.projects || [],
        versions: staticFilters.versions || [],
        extensions: staticFilters.extensions || [],
        repositories: staticFilters.repositories || [],
      });
      shouldResetRef.current = false; // Only reset once per clear
    }
  }, [hasActiveFilters]);

  const clearFilters = useCallback(() => {
    setFilters({});
  }, []);

  // Update dynamic filters from search results
  // When a search query is performed, SearchPage calls this with the facets from the response
  // These facets are specific to the current search query
  const updateDynamicFilters = useCallback((facets: DynamicFilters | null) => {
    if (facets) {
      // We have facets from search results, use them
      setSearchResultsFacets(facets);
      setLastValidFacets(facets);
    } else {
      // No facets (e.g., cleared search), clear search result facets
      setSearchResultsFacets(null);
    }
  }, []);

  /**
   * Merges static filter lists with dynamic facet counts from the API.
   *
   * Strategy:
   * - Static list: Initial comprehensive list of all available filter options
   * - Dynamic list: Real-time facet counts based on currently applied filters
   * - Selected values: Currently active filter selections
   *
   * Behavior:
   * 1. When no dynamic data: Returns static list as fallback
   * 2. When no static data: Returns dynamic list (API response only)
   * 3. When both present: Combines both lists, prioritizing dynamic counts
   * 4. Always includes selected items, even if they have 0 count
   * 5. Filters out items with 0 count that aren't currently selected
   *
   * Use cases:
   * - Shows accurate result counts when filters are applied (via dynamic counts)
   * - Maintains comprehensive option visibility when no filters active (via static list)
   * - Preserves user selections even if temporarily out of current result set
   *
   * @param staticList - Initial comprehensive list from backend (all options)
   * @param dynamicList - Real-time facet counts based on active filters
   * @param selectedValues - Currently selected filter values by the user
   * @returns Merged list with accurate counts and relevant options for current filter state
   */
  // Fix 4: Memoize mergeFiltersWithDynamicCounts to avoid unnecessary array recreations
  const mergeFiltersWithDynamicCounts = useCallback(
    (
      staticList: Array<{ value: string; count: number }>,
      dynamicList: Array<{ value: string; count: number }>,
      selectedValues: string[] = []
    ): Array<{ value: string; count: number }> => {
      if (!staticList) return dynamicList || [];
      if (!dynamicList) return staticList;

      // Create a map of dynamic counts for O(1) lookup
      const dynamicMap = new Map(dynamicList.map(item => [item.value, item.count]));

      // Merge static and dynamic lists, keeping selected items even if not in static
      const staticValues = new Set(staticList.map(item => item.value));
      const allItems = [...staticList];

      // Add selected items that are not in static list (handles edge cases)
      selectedValues.forEach(selected => {
        if (!staticValues.has(selected)) {
          const dynamicItem = dynamicList.find(d => d.value === selected);
          if (dynamicItem) {
            allItems.push(dynamicItem);
          } else {
            allItems.push({ value: selected, count: 0 });
          }
        }
      });

      // Update all items with dynamic counts and filter meaningfully
      const result = allItems
        .map(item => ({
          value: item.value,
          count: dynamicMap.get(item.value) || 0
        }))
        // Keep items that either have results OR are currently selected
        .filter(item => item.count > 0 || selectedValues.includes(item.value));

      return result;
    },
    []
  );

  // Fix 3: Memoize hybridFilters to prevent recreation every render
  // Smart hybrid strategy:
  // - If no filter selected in a category → show only items with results (dynamic)
  // - If filters selected in a category → show all items (static) with current counts (dynamic)
  // Use lastValidFacets instead of dynamicFilters to avoid showing zero counts during debounce
  const hybridFilters: Record<string, Array<{ value: string; count: number }>> = React.useMemo(() => ({
      projects: (filters.project && filters.project.length > 0)
        ? mergeFiltersWithDynamicCounts(
            (staticFilters?.projects as Array<{ value: string; count: number }>) || [],
            (lastValidFacets?.projects as Array<{ value: string; count: number }>) || [],
            filters.project
          )
        : (lastValidFacets?.projects as Array<{ value: string; count: number }>) ||
          (staticFilters?.projects as Array<{ value: string; count: number }>) || [],
      versions: (filters.version && filters.version.length > 0)
        ? mergeFiltersWithDynamicCounts(
            (staticFilters?.versions as Array<{ value: string; count: number }>) || [],
            (lastValidFacets?.versions as Array<{ value: string; count: number }>) || [],
            filters.version
          )
        : (lastValidFacets?.versions as Array<{ value: string; count: number }>) ||
          (staticFilters?.versions as Array<{ value: string; count: number }>) || [],
      extensions: (filters.extension && filters.extension.length > 0)
        ? mergeFiltersWithDynamicCounts(
            (staticFilters?.extensions as Array<{ value: string; count: number }>) || [],
            (lastValidFacets?.extensions as Array<{ value: string; count: number }>) || [],
            filters.extension
          )
        : (lastValidFacets?.extensions as Array<{ value: string; count: number }>) ||
          (staticFilters?.extensions as Array<{ value: string; count: number }>) || [],
      repositories: (filters.repository && filters.repository.length > 0)
        ? mergeFiltersWithDynamicCounts(
            (staticFilters?.repositories as Array<{ value: string; count: number }>) || [],
            (lastValidFacets?.repositories as Array<{ value: string; count: number }>) || [],
            filters.repository
          )
        : (lastValidFacets?.repositories as Array<{ value: string; count: number }>) ||
          (staticFilters?.repositories as Array<{ value: string; count: number }>) || [],
  }), [filters, staticFilters, lastValidFacets, mergeFiltersWithDynamicCounts]);

  // Fix 2: Memoize availableFiltersList to prevent .map() recreations every render
  const availableFiltersList: {
    projects: FilterOption[];
    versions: FilterOption[];
    extensions: FilterOption[];
    repositories: FilterOption[];
    languages: FilterOption[];
  } = React.useMemo(() => ({
    projects: (hybridFilters.projects || []).map((p: { value: string; count: number }) => ({
      value: p.value,
      label: p.value,
      count: p.count || 0,
    })),
    versions: (hybridFilters.versions || []).map((v: { value: string; count: number }) => ({
      value: v.value,
      label: v.value,
      count: v.count || 0,
    })),
    extensions: (hybridFilters.extensions || []).map((e: { value: string; count: number }) => ({
      value: e.value,
      label: `.${e.value}`,
      count: e.count || 0,
    })),
    repositories: (hybridFilters.repositories || []).map((r: { value: string; count: number }) => ({
      value: r.value,
      label: r.value,
      count: r.count || 0,
    })),
    languages: [], // Will be derived from extensions in the future
  }), [hybridFilters]);

  // Fix 1: Memoize the context value to prevent all consumers from re-rendering
  const value: SearchFiltersContextType = React.useMemo(() => ({
    filters,
    setFilters,
    clearFilters,
    currentQuery,
    setCurrentQuery,
    availableFilters: availableFiltersList,
    isLoading,
    updateDynamicFilters,
  }), [filters, setFilters, clearFilters, currentQuery, setCurrentQuery, availableFiltersList, isLoading, updateDynamicFilters]);

  return (
    <SearchFiltersContext.Provider value={value}>
      {children}
    </SearchFiltersContext.Provider>
  );
};