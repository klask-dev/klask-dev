import React, { useState, useMemo } from 'react';
import { 
  FunnelIcon, 
  XMarkIcon, 
  ChevronDownIcon,
  ChevronUpIcon,
  AdjustmentsHorizontalIcon 
} from '@heroicons/react/24/outline';
import { MultiSelectFilter } from './MultiSelectFilter';
import { SizeRangeFilter } from './SizeRangeFilter';

export interface SearchFiltersV2 {
  projects?: string[];
  versions?: string[];
  extensions?: string[];
  languages?: string[];
  repositories?: string[];
  sizeRange?: {
    min?: number; // Size in bytes
    max?: number; // Size in bytes
  };
  [key: string]: string[] | { min?: number; max?: number } | undefined;
}

interface FilterOption {
  value: string;
  label: string;
  count?: number;
}

interface AvailableFilters {
  projects: FilterOption[];
  versions: FilterOption[];
  extensions: FilterOption[];
  languages: FilterOption[];
  repositories: FilterOption[];
}

interface SearchFiltersV2Props {
  filters: SearchFiltersV2;
  onFiltersChange: (filters: SearchFiltersV2) => void;
  availableFilters: AvailableFilters;
  isLoading?: boolean;
  collapsible?: boolean;
  defaultExpanded?: boolean;
}

export const SearchFiltersV2Component: React.FC<SearchFiltersV2Props> = ({
  filters,
  onFiltersChange,
  availableFilters,
  isLoading = false,
  collapsible = true,
  defaultExpanded = false,
}) => {
  const [isExpanded, setIsExpanded] = useState(defaultExpanded);

  // Calculate total active filter count
  const activeFiltersCount = useMemo(() => {
    let count = 0;
    Object.entries(filters).forEach(([key, value]) => {
      if (key === 'sizeRange' && value && typeof value === 'object') {
        // Count sizeRange as 1 active filter if min or max is set
        if ((value as { min?: number; max?: number }).min !== undefined ||
            (value as { min?: number; max?: number }).max !== undefined) {
          count += 1;
        }
      } else if (Array.isArray(value)) {
        count += value.length;
      }
    });
    return count;
  }, [filters]);

  const hasActiveFilters = activeFiltersCount > 0;

  const handleFilterChange = (key: keyof SearchFiltersV2, values: string[]) => {
    onFiltersChange({
      ...filters,
      [key]: values.length > 0 ? values : undefined,
    });
  };

  const handleSizeRangeChange = (sizeRange?: { min?: number; max?: number }) => {
    onFiltersChange({
      ...filters,
      sizeRange: sizeRange,
    });
  };

  const clearAllFilters = () => {
    onFiltersChange({});
  };

  const clearFilterGroup = (key: keyof SearchFiltersV2) => {
    const newFilters = { ...filters };
    delete newFilters[key];
    onFiltersChange(newFilters);
  };

  // Group filters into categories for better organization
  const filterGroups = [
    {
      title: 'Repository',
      filters: [
        {
          key: 'repositories' as keyof SearchFiltersV2,
          label: 'Repositories',
          options: availableFilters.repositories,
          searchable: true,
        },
        {
          key: 'projects' as keyof SearchFiltersV2,
          label: 'Projects',
          options: availableFilters.projects,
          searchable: true,
        },
        {
          key: 'versions' as keyof SearchFiltersV2,
          label: 'Versions',
          options: availableFilters.versions,
          searchable: false,
        },
      ],
    },
    {
      title: 'File Type',
      filters: [
        {
          key: 'extensions' as keyof SearchFiltersV2,
          label: 'Extensions',
          options: availableFilters.extensions,
          searchable: true,
        },
        {
          key: 'languages' as keyof SearchFiltersV2,
          label: 'Languages',
          options: availableFilters.languages,
          searchable: true,
        },
      ],
    },
    {
      title: 'File Properties',
      filters: [], // Size filter will be handled separately
    },
  ];

  const renderActiveFilters = () => {
    if (!hasActiveFilters) return null;

    const activeFilters: Array<{ key: string; values: string[]; label: string }> = [];

    Object.entries(filters).forEach(([key, values]) => {
      if (key === 'sizeRange' && values && typeof values === 'object') {
        const sizeRange = values as { min?: number; max?: number };
        if (sizeRange.min !== undefined || sizeRange.max !== undefined) {
          // Convert size range to display string
          const displayValue = `Size: ${sizeRange.min ? `≥${sizeRange.min} bytes` : ''}${
            sizeRange.min && sizeRange.max ? ' - ' : ''
          }${sizeRange.max ? `≤${sizeRange.max} bytes` : ''}`;
          activeFilters.push({
            key,
            values: [displayValue],
            label: 'File Size',
          });
        }
      } else if (Array.isArray(values) && values.length > 0) {
        const filterConfig = filterGroups
          .flatMap(group => group.filters)
          .find(f => f.key === key);

        activeFilters.push({
          key,
          values,
          label: filterConfig?.label || key,
        });
      }
    });

    return (
      <div className="px-4 py-3 border-t border-gray-100 dark:border-gray-700 bg-gray-50 dark:bg-gray-700/30">
        <div className="flex items-center justify-between mb-2">
          <span className="text-sm font-medium text-gray-700 dark:text-gray-300">
            Active Filters ({activeFiltersCount})
          </span>
          <button
            onClick={clearAllFilters}
            className="text-sm text-red-600 dark:text-red-400 hover:text-red-800 dark:hover:text-red-300 font-medium"
          >
            Clear All
          </button>
        </div>

        <div className="flex flex-wrap gap-2">
          {activeFilters.map(({ key, values, label }) => (
            <div key={key} className="inline-flex items-center gap-1">
              <span className="text-xs text-gray-600 dark:text-gray-400 font-medium">{label}:</span>
              {values.map((value) => {
                const option = availableFilters[key as keyof AvailableFilters]?.find(opt => opt.value === value);
                return (
                  <span
                    key={`${key}-${value}`}
                    className="inline-flex items-center gap-1 px-2 py-1 bg-blue-100 dark:bg-blue-900/30 text-blue-800 dark:text-blue-300 text-xs rounded-full"
                  >
                    <span>{option?.label || value}</span>
                    <button
                      onClick={() => {
                        const newValues = values.filter(v => v !== value);
                        handleFilterChange(key as keyof SearchFiltersV2, newValues);
                      }}
                      className="hover:text-blue-600 dark:hover:text-blue-400"
                      title="Remove filter"
                    >
                      <XMarkIcon className="h-3 w-3" />
                    </button>
                  </span>
                );
              })}
              {values.length > 1 && (
                <button
                  onClick={() => clearFilterGroup(key as keyof SearchFiltersV2)}
                  className="text-xs text-gray-500 dark:text-gray-400 hover:text-gray-700 dark:hover:text-gray-300 ml-1"
                  title={`Clear all ${label.toLowerCase()}`}
                >
                  (clear all)
                </button>
              )}
            </div>
          ))}
        </div>
      </div>
    );
  };

  const headerContent = (
    <div className="px-4 py-3 border-b border-gray-200 dark:border-gray-700">
      <div className="flex items-center justify-between">
        <div className="flex items-center space-x-2">
          {collapsible ? (
            <button
              onClick={() => setIsExpanded(!isExpanded)}
              className="flex items-center space-x-2 text-sm font-medium text-gray-700 dark:text-gray-300 hover:text-gray-900 dark:hover:text-gray-100"
            >
              <AdjustmentsHorizontalIcon className="h-4 w-4" />
              <span>Search Filters</span>
              {isExpanded ? (
                <ChevronUpIcon className="h-4 w-4" />
              ) : (
                <ChevronDownIcon className="h-4 w-4" />
              )}
            </button>
          ) : (
            <div className="flex items-center space-x-2 text-sm font-medium text-gray-700 dark:text-gray-300">
              <FunnelIcon className="h-4 w-4" />
              <span>Search Filters</span>
            </div>
          )}

          {hasActiveFilters && (
            <span className="bg-blue-100 dark:bg-blue-900/30 text-blue-800 dark:text-blue-300 text-xs px-2 py-1 rounded-full font-medium">
              {activeFiltersCount}
            </span>
          )}
        </div>

        {hasActiveFilters && (
          <button
            onClick={clearAllFilters}
            className="text-sm text-gray-500 dark:text-gray-400 hover:text-gray-700 dark:hover:text-gray-300"
          >
            Clear all
          </button>
        )}
      </div>
    </div>
  );

  const filtersContent = (
    <div className="p-4">
      <div className="space-y-6">
        {filterGroups.map((group) => (
          <div key={group.title}>
            <h4 className="text-sm font-semibold text-gray-900 dark:text-white mb-3 flex items-center">
              {group.title}
            </h4>

            {/* Regular multi-select filters */}
            {group.filters.length > 0 && (
              <div className={`grid grid-cols-1 gap-4 ${group.filters.length === 3 ? 'md:grid-cols-3' : 'md:grid-cols-2'}`}>
                {group.filters.map((filterConfig) => (
                  <MultiSelectFilter
                    key={filterConfig.key}
                    label={filterConfig.label}
                    options={filterConfig.options}
                    selectedValues={(filters[filterConfig.key] as string[]) || []}
                    onChange={(values) => handleFilterChange(filterConfig.key, values)}
                    isLoading={isLoading}
                    searchable={filterConfig.searchable}
                    showCounts={true}
                  />
                ))}
              </div>
            )}

            {/* Special handling for File Properties group */}
            {group.title === 'File Properties' && (
              <div className="max-w-md">
                <SizeRangeFilter
                  value={filters.sizeRange}
                  onChange={handleSizeRangeChange}
                />
              </div>
            )}
          </div>
        ))}
      </div>
    </div>
  );

  return (
    <div className="bg-white dark:bg-gray-800 border border-gray-200 dark:border-gray-700 rounded-lg shadow-sm">
      {headerContent}

      {(!collapsible || isExpanded) && (
        <>
          {filtersContent}
          {renderActiveFilters()}
        </>
      )}
    </div>
  );
};