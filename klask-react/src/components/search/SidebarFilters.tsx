import React from 'react';
import { useLocation } from 'react-router-dom';
import { XMarkIcon } from '@heroicons/react/24/outline';
import { FolderIcon, TagIcon, DocumentIcon } from '@heroicons/react/24/solid';
import { useSearchFiltersContext } from '../../contexts/SearchFiltersContext';

export interface SearchFilters {
  project?: string[];
  version?: string[];
  extension?: string[];
  language?: string[];
  [key: string]: string[] | undefined;
}

interface FilterOption {
  value: string;
  label: string;
  count: number;
}

interface FilterOptionButtonProps {
  option: FilterOption;
  isSelected: boolean;
  onFilterChange: (filterKey: string, value: string, isSelected: boolean) => void;
  filterKey: string;
}

interface FilterSectionProps {
  title: string;
  icon: React.ComponentType<{ className?: string }>;
  options: FilterOption[];
  selectedValues: string[];
  filterKey: string;
}

interface SidebarFiltersProps {
  filters?: SearchFilters;
  onFiltersChange?: (filters: SearchFilters) => void;
  availableFilters?: {
    projects: FilterOption[];
    versions: FilterOption[];
    extensions: FilterOption[];
    languages: FilterOption[];
  };
  isLoading?: boolean;
  currentQuery?: string;
}

export const SidebarFilters: React.FC<SidebarFiltersProps> = ({
  filters: propFilters,
  onFiltersChange: propOnFiltersChange,
  availableFilters: propAvailableFilters,
  isLoading: propIsLoading = false,
  currentQuery: propCurrentQuery = '',
}) => {
  const location = useLocation();

  // Use context values if props are not provided
  const searchFiltersContext = useSearchFiltersContext();
  const filters = propFilters || searchFiltersContext.filters;
  const onFiltersChange = propOnFiltersChange || searchFiltersContext.setFilters;
  const availableFilters = propAvailableFilters || searchFiltersContext.availableFilters;
  const isLoading = propIsLoading || searchFiltersContext.isLoading;
  const currentQuery = propCurrentQuery || searchFiltersContext.currentQuery;

  // Only show on search page
  if (!location.pathname.startsWith('/search')) {
    return null;
  }

  // Don't show if no search query and no filters available
  if (!currentQuery.trim() &&
      availableFilters.projects.length === 0 &&
      availableFilters.versions.length === 0 &&
      availableFilters.extensions.length === 0) {
    return null;
  }

  const handleFilterChange = (key: keyof SearchFilters, value: string, checked: boolean) => {
    const currentValues = filters[key] || [];
    let newValues: string[];

    if (checked) {
      // Add value if not already present
      newValues = currentValues.includes(value) ? currentValues : [...currentValues, value];
    } else {
      // Remove value
      newValues = currentValues.filter(v => v !== value);
    }

    onFiltersChange({
      ...filters,
      [key]: newValues.length > 0 ? newValues : undefined,
    });
  };

  const clearFilter = (key: keyof SearchFilters) => {
    const newFilters = { ...filters };
    delete newFilters[key];
    onFiltersChange(newFilters);
  };

  const clearAllFilters = () => {
    onFiltersChange({});
  };

  const hasActiveFilters = Object.values(filters).some(filterArray => filterArray && filterArray.length > 0);

  // Memoized button component to prevent unnecessary re-renders during hover
  const FilterOptionButton = React.memo(
    ({ option, isSelected, onFilterChange: handleChange, filterKey: key }: FilterOptionButtonProps) => (
      <button
        type="button"
        onClick={() => handleChange(key, option.value, !isSelected)}
        className={`flex items-center justify-between px-2 py-1 rounded cursor-pointer transition-colors text-sm w-full text-left ${
          isSelected
            ? 'bg-blue-50 text-blue-700'
            : 'hover:bg-gray-50 text-gray-700'
        }`}
        aria-pressed={isSelected}
      >
        <span className="truncate text-xs min-w-0 flex-1" title={option.label}>
          {option.label}
        </span>
        <span className="text-xs text-gray-500 bg-gray-100 px-1.5 py-0.5 rounded ml-2">
          {option.count}
        </span>
      </button>
    )
  );

  FilterOptionButton.displayName = 'FilterOptionButton';

  // Memoized section component to prevent re-renders when sibling sections update
  const FilterSection = React.memo(
    ({ title, icon: Icon, options, selectedValues, filterKey }: FilterSectionProps) => {
    const [searchTerm, setSearchTerm] = React.useState('');

    if (options.length === 0) return null;

    // Filter options based on search term
    const filteredOptions = searchTerm
      ? options.filter(option =>
          option.label.toLowerCase().includes(searchTerm.toLowerCase()) ||
          option.value.toLowerCase().includes(searchTerm.toLowerCase())
        )
      : options;

    // Sort: selected items first, then by count (descending)
    const sortedOptions = [...filteredOptions].sort((a, b) => {
      const aSelected = selectedValues.includes(a.value);
      const bSelected = selectedValues.includes(b.value);

      // Selected items come first
      if (aSelected && !bSelected) return -1;
      if (!aSelected && bSelected) return 1;

      // Then sort by count (descending)
      return b.count - a.count;
    });

    return (
      <div className="mb-4">
        <div className="flex items-center justify-between mb-2">
          <div className="flex items-center space-x-2">
            <Icon className="h-4 w-4 text-gray-500" />
            <h4 className="text-xs font-medium text-gray-900 uppercase tracking-wide">{title}</h4>
          </div>
          {selectedValues.length > 0 && (
            <button
              onClick={() => clearFilter(filterKey)}
              className="text-xs text-gray-400 hover:text-gray-600"
              title="Clear filter"
            >
              <XMarkIcon className="h-3 w-3" />
            </button>
          )}
        </div>

        {/* Search input */}
        {options.length > 5 && (
          <div className="mb-2">
            <input
              type="text"
              value={searchTerm}
              onChange={(e) => setSearchTerm(e.target.value)}
              placeholder={`Search ${title.toLowerCase()}...`}
              className="w-full px-2 py-1 text-xs border border-gray-200 rounded focus:ring-1 focus:ring-blue-500 focus:border-blue-500"
            />
          </div>
        )}

        {/* Options list */}
        <div className="space-y-1 max-h-48 overflow-y-auto">
          {sortedOptions.length === 0 ? (
            <div className="text-xs text-gray-400 px-2 py-1">No matches</div>
          ) : (
            sortedOptions.map((option) => {
              const isSelected = selectedValues.includes(option.value);
              return (
                <FilterOptionButton
                  key={option.value}
                  option={option}
                  isSelected={isSelected}
                  onFilterChange={handleFilterChange}
                  filterKey={filterKey}
                />
              );
            })
          )}
        </div>
      </div>
    );
    }
  );

  FilterSection.displayName = 'FilterSection';

  return (
    <div>
      {/* Header */}
      <div className="flex items-center justify-between mb-3">
        <div className="text-xs font-semibold leading-6 text-gray-400 uppercase tracking-wide">
          Search Filters
        </div>
        {hasActiveFilters && (
          <button
            onClick={clearAllFilters}
            className="text-xs text-blue-600 hover:text-blue-800"
          >
            Clear all
          </button>
        )}
      </div>

      {/* Active Filters */}
      {hasActiveFilters && (
        <div className="mb-4">
          <div className="flex flex-wrap gap-1">
            {Object.entries(filters).map(([key, values]) =>
              values && values.length > 0 ? (
                <div key={key} className="space-y-1">
                  {values.map((value) => (
                    <div
                      key={`${key}-${value}`}
                      className="inline-flex items-center gap-1 px-2 py-1 bg-blue-100 text-blue-800 text-xs rounded mr-1 mb-1"
                    >
                      <span className="capitalize">{key}:</span>
                      <span className="font-medium truncate max-w-20" title={value}>{value}</span>
                      <button
                        onClick={() => {
                          const currentValues = filters[key as keyof SearchFilters] || [];
                          const newValues = currentValues.filter(v => v !== value);
                          onFiltersChange({
                            ...filters,
                            [key]: newValues.length > 0 ? newValues : undefined,
                          });
                        }}
                        className="hover:text-blue-600"
                      >
                        <XMarkIcon className="h-3 w-3" />
                      </button>
                    </div>
                  ))}
                </div>
              ) : null
            )}
          </div>
        </div>
      )}

      {/* Loading state */}
      {isLoading ? (
        <div className="text-xs text-gray-500 mb-4">Loading filters...</div>
      ) : (
        <>
          {/* Filter Sections */}
          <FilterSection
            title="Projects"
            icon={FolderIcon}
            options={availableFilters.projects}
            selectedValues={filters.project || []}
            filterKey="project"
          />

          <FilterSection
            title="Versions"
            icon={TagIcon}
            options={availableFilters.versions}
            selectedValues={filters.version || []}
            filterKey="version"
          />

          <FilterSection
            title="Extensions"
            icon={DocumentIcon}
            options={availableFilters.extensions}
            selectedValues={filters.extension || []}
            filterKey="extension"
          />

          {availableFilters.languages.length > 0 && (
            <FilterSection
              title="Languages"
              icon={DocumentIcon}
              options={availableFilters.languages}
              selectedValues={filters.language || []}
              filterKey="language"
            />
          )}
        </>
      )}
    </div>
  );
};