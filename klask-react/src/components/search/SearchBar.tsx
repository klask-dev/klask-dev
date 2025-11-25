import React, { useState, useCallback } from 'react';
import { MagnifyingGlassIcon, XMarkIcon, ExclamationTriangleIcon } from '@heroicons/react/24/outline';
import { useDebounce } from 'use-debounce';
import { useSearchStatus } from '../../api/indexMetrics';

interface SearchBarProps {
  value: string;
  onChange: (value: string) => void;
  onSearch: (query: string) => void;
  placeholder?: string;
  isLoading?: boolean;
  className?: string;
  autoFocus?: boolean;
}

export const SearchBar: React.FC<SearchBarProps> = ({
  value,
  onChange,
  onSearch,
  placeholder = "Search in your codebase...",
  isLoading = false,
  className = "",
  autoFocus = false,
}) => {
  const [localValue, setLocalValue] = useState(value);
  const [debouncedValue] = useDebounce(localValue, 300);
  const prevValue = React.useRef(value);
  const isExternalChange = React.useRef(false);

  // Check if search is disabled due to schema mismatch
  const statusQuery = useSearchStatus(false);
  const isSearchDisabled = statusQuery.data?.schema_mismatch === true;
  const statusMessage = isSearchDisabled ? 'Search unavailable - index being rebuilt' : placeholder;

  // Sync localValue with prop value when it changes externally (for recent searches)
  React.useEffect(() => {
    if (value !== prevValue.current) {
      isExternalChange.current = true;
      setLocalValue(value);
      prevValue.current = value;
      
      // For external changes, don't use debounce - call directly
      if (value && value.trim()) {
        onChange(value);
        onSearch(value);
      }
    }
  }, [value, onChange, onSearch]);

  // Handle debounced internal changes (user typing only)
  React.useEffect(() => {
    // Only apply debounce if it's not an external change and if the debounced value matches what user typed
    if (!isExternalChange.current && debouncedValue !== value && debouncedValue === localValue) {
      onChange(debouncedValue);
      onSearch(debouncedValue);
    }
    // Reset the flag after debounce processing
    isExternalChange.current = false;
  }, [debouncedValue, onChange, onSearch, value, localValue]);

  const handleClear = useCallback(() => {
    setLocalValue('');
    onChange('');
    onSearch('');
  }, [onChange, onSearch]);

  const handleSubmit = useCallback((e: React.FormEvent) => {
    e.preventDefault();
    onSearch(localValue);
  }, [localValue, onSearch]);

  return (
    <div className={className}>
      <form onSubmit={handleSubmit} className="w-full">
        <div className="relative">
          <div className="absolute inset-y-0 left-0 pl-3 flex items-center pointer-events-none">
            {isSearchDisabled ? (
              <ExclamationTriangleIcon className="h-5 w-5 text-yellow-500" />
            ) : (
              <MagnifyingGlassIcon
                className={`h-5 w-5 ${isLoading ? 'text-primary-500 animate-pulse' : 'text-gray-400'}`}
              />
            )}
          </div>

          <input
            type="text"
            value={localValue}
            onChange={(e) => setLocalValue(e.target.value)}
            disabled={isSearchDisabled}
            title={isSearchDisabled ? 'The search index schema has changed. Go to admin settings to rebuild it.' : ''}
            className={`block w-full pl-10 pr-12 py-3 text-lg border rounded-lg placeholder-gray-400 dark:placeholder-gray-500 transition-colors ${
              isSearchDisabled
                ? 'border-yellow-300 dark:border-yellow-700 bg-yellow-50 dark:bg-yellow-900/20 text-gray-600 dark:text-gray-400 cursor-not-allowed'
                : 'border-gray-300 dark:border-gray-600 bg-white dark:bg-gray-800 text-gray-900 dark:text-white focus:ring-2 focus:ring-blue-500 focus:border-blue-500'
            }`}
            placeholder={statusMessage}
            autoComplete="off"
            spellCheck={false}
            autoFocus={autoFocus}
          />

          {localValue && !isSearchDisabled && (
            <button
              type="button"
              onClick={handleClear}
              className="absolute inset-y-0 right-0 pr-3 flex items-center hover:text-gray-600 dark:hover:text-gray-300"
            >
              <XMarkIcon className="h-5 w-5 text-gray-400 dark:text-gray-500" />
            </button>
          )}
        </div>
      </form>
    </div>
  );
};