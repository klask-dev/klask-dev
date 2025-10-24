import React, { useState, useRef, useEffect } from 'react';
import { 
  ChevronDownIcon, 
  XMarkIcon, 
  MagnifyingGlassIcon,
  CheckIcon 
} from '@heroicons/react/24/outline';

interface FilterOption {
  value: string;
  label: string;
  count?: number;
}

interface MultiSelectFilterProps {
  label: string;
  options: FilterOption[];
  selectedValues: string[];
  onChange: (values: string[]) => void;
  placeholder?: string;
  isLoading?: boolean;
  maxDisplayed?: number;
  searchable?: boolean;
  showCounts?: boolean;
}

export const MultiSelectFilter: React.FC<MultiSelectFilterProps> = ({
  label,
  options,
  selectedValues,
  onChange,
  placeholder = `Select ${label.toLowerCase()}...`,
  isLoading = false,
  maxDisplayed = 5,
  searchable = true,
  showCounts = true,
}) => {
  const [isOpen, setIsOpen] = useState(false);
  const [searchTerm, setSearchTerm] = useState('');
  const dropdownRef = useRef<HTMLDivElement>(null);
  const searchInputRef = useRef<HTMLInputElement>(null);

  // Close dropdown when clicking outside
  useEffect(() => {
    const handleClickOutside = (event: MouseEvent) => {
      if (dropdownRef.current && !dropdownRef.current.contains(event.target as Node)) {
        setIsOpen(false);
        setSearchTerm('');
      }
    };

    document.addEventListener('mousedown', handleClickOutside);
    return () => document.removeEventListener('mousedown', handleClickOutside);
  }, []);

  // Focus search input when dropdown opens
  useEffect(() => {
    if (isOpen && searchable && searchInputRef.current) {
      searchInputRef.current.focus();
    }
  }, [isOpen, searchable]);

  const filteredOptions = options.filter(option =>
    option.label.toLowerCase().includes(searchTerm.toLowerCase()) ||
    option.value.toLowerCase().includes(searchTerm.toLowerCase())
  );

  const handleToggleOption = (value: string) => {
    if (selectedValues.includes(value)) {
      onChange(selectedValues.filter(v => v !== value));
    } else {
      onChange([...selectedValues, value]);
    }
  };

  const handleClearSelection = (e: React.MouseEvent) => {
    e.stopPropagation();
    onChange([]);
  };

  const displayText = (() => {
    if (selectedValues.length === 0) {
      return placeholder;
    }
    if (selectedValues.length === 1) {
      const option = options.find(opt => opt.value === selectedValues[0]);
      return option?.label || selectedValues[0];
    }
    if (selectedValues.length <= maxDisplayed) {
      return selectedValues
        .map(value => options.find(opt => opt.value === value)?.label || value)
        .join(', ');
    }
    return `${selectedValues.length} ${label.toLowerCase()} selected`;
  })();

  return (
    <div className="relative" ref={dropdownRef}>
      <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
        {label}
        {selectedValues.length > 0 && (
          <span className="ml-2 text-xs bg-blue-100 dark:bg-blue-900/30 text-blue-800 dark:text-blue-300 px-2 py-1 rounded-full">
            {selectedValues.length}
          </span>
        )}
      </label>

      <button
        onClick={() => setIsOpen(!isOpen)}
        disabled={isLoading}
        className={`
          w-full flex items-center justify-between px-3 py-2 text-sm border rounded-md
          focus:ring-2 focus:ring-blue-500 focus:border-blue-500
          ${isLoading ? 'bg-gray-50 dark:bg-gray-700 cursor-not-allowed' : 'bg-white dark:bg-gray-700 hover:bg-gray-50 dark:hover:bg-gray-600'}
          ${selectedValues.length > 0 ? 'border-blue-300 dark:border-blue-600' : 'border-gray-300 dark:border-gray-600'}
          ${isOpen ? 'ring-2 ring-blue-500 border-blue-500 dark:ring-blue-400 dark:border-blue-400' : ''}
          text-gray-900 dark:text-gray-100
        `}
      >
        <span className={`truncate ${selectedValues.length === 0 ? 'text-gray-500 dark:text-gray-400' : 'text-gray-900 dark:text-gray-100'}`}>
          {displayText}
        </span>

        <div className="flex items-center space-x-1">
          {selectedValues.length > 0 && (
            <button
              onClick={handleClearSelection}
              className="p-1 hover:bg-gray-200 dark:hover:bg-gray-600 rounded-full transition-colors"
              title="Clear selection"
            >
              <XMarkIcon className="h-3 w-3 text-gray-400 dark:text-gray-500" />
            </button>
          )}
          <ChevronDownIcon
            className={`h-4 w-4 text-gray-400 dark:text-gray-500 transition-transform ${
              isOpen ? 'transform rotate-180' : ''
            }`}
          />
        </div>
      </button>

      {isOpen && (
        <div className="absolute z-50 w-full mt-1 bg-white dark:bg-gray-800 border border-gray-200 dark:border-gray-700 rounded-md shadow-lg max-h-64 overflow-hidden">
          {searchable && (
            <div className="p-2 border-b border-gray-100 dark:border-gray-700">
              <div className="relative">
                <MagnifyingGlassIcon className="absolute left-3 top-1/2 transform -translate-y-1/2 h-4 w-4 text-gray-400 dark:text-gray-500" />
                <input
                  ref={searchInputRef}
                  type="text"
                  value={searchTerm}
                  onChange={(e) => setSearchTerm(e.target.value)}
                  placeholder={`Search ${label.toLowerCase()}...`}
                  className="w-full pl-9 pr-3 py-2 text-sm border border-gray-200 dark:border-gray-600 rounded-md bg-white dark:bg-gray-700 text-gray-900 dark:text-gray-100 placeholder-gray-500 dark:placeholder-gray-400 focus:ring-2 focus:ring-blue-500 focus:border-blue-500"
                />
              </div>
            </div>
          )}

          <div className="max-h-48 overflow-y-auto">
            {filteredOptions.length === 0 ? (
              <div className="px-3 py-2 text-sm text-gray-500 dark:text-gray-400 text-center">
                {searchTerm ? 'No matching options' : 'No options available'}
              </div>
            ) : (
              filteredOptions.map((option) => {
                const isSelected = selectedValues.includes(option.value);
                return (
                  <button
                    key={option.value}
                    onClick={() => handleToggleOption(option.value)}
                    className={`
                      w-full flex items-center justify-between px-3 py-2 text-sm text-left
                      hover:bg-gray-50 dark:hover:bg-gray-700 transition-colors
                      ${isSelected ? 'bg-blue-50 dark:bg-blue-900/20 text-blue-900 dark:text-blue-300' : 'text-gray-900 dark:text-gray-100'}
                    `}
                  >
                    <div className="flex items-center space-x-2 flex-1 min-w-0">
                      <div className={`
                        flex-shrink-0 w-4 h-4 border rounded flex items-center justify-center
                        ${isSelected
                          ? 'bg-blue-600 border-blue-600'
                          : 'border-gray-300 dark:border-gray-600 bg-white dark:bg-gray-700'
                        }
                      `}>
                        {isSelected && (
                          <CheckIcon className="h-3 w-3 text-white" />
                        )}
                      </div>
                      <span className="truncate">{option.label}</span>
                    </div>

                    {showCounts && option.count !== undefined && (
                      <span className="flex-shrink-0 ml-2 text-xs text-gray-500 dark:text-gray-400 bg-gray-100 dark:bg-gray-700 px-2 py-1 rounded-full">
                        {option.count.toLocaleString()}
                      </span>
                    )}
                  </button>
                );
              })
            )}
          </div>
        </div>
      )}
    </div>
  );
};