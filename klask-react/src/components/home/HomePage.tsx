import React, { useState } from 'react';
import { useNavigate } from 'react-router-dom';
import { SearchBar } from '../search/SearchBar';
import { useSearchHistory } from '../../hooks/useSearch';
import {
  MagnifyingGlassIcon,
  RocketLaunchIcon,
  LightBulbIcon,
  ClockIcon,
  DocumentMagnifyingGlassIcon,
  SparklesIcon,
  CodeBracketIcon,
  CpuChipIcon
} from '@heroicons/react/24/outline';

const HomePage: React.FC = () => {
  const navigate = useNavigate();
  const [query, setQuery] = useState('');
  const { history, addToHistory } = useSearchHistory();

  const handleSearch = (searchQuery: string) => {
    if (searchQuery.trim()) {
      addToHistory(searchQuery.trim());
      navigate(`/search?q=${encodeURIComponent(searchQuery.trim())}`);
    }
  };

  const handleExampleSearch = (exampleQuery: string) => {
    setQuery(exampleQuery);
    handleSearch(exampleQuery);
  };

  const searchExamples = [
    {
      category: 'Basic Search',
      icon: MagnifyingGlassIcon,
      examples: [
        { query: 'function login', description: 'Find login functions' },
        { query: 'class User', description: 'Find User class definitions' },
        { query: 'TODO authentication', description: 'Find authentication TODOs' },
        { query: 'error handling', description: 'Find error handling code' }
      ]
    },
    {
      category: 'Tantivy Advanced',
      icon: RocketLaunchIcon,
      examples: [
        { query: 'password AND hash', description: 'Find password hashing (AND operator)' },
        { query: 'api OR endpoint', description: 'Find API or endpoint references (OR operator)' },
        { query: 'database -test', description: 'Find database code excluding tests (NOT operator)' },
        { query: '"exact phrase"', description: 'Find exact phrase matches' }
      ]
    },
    {
      category: 'Wildcards & Fuzzy',
      icon: SparklesIcon,
      examples: [
        { query: 'config*', description: 'Find words starting with "config"' },
        { query: 'connect~', description: 'Fuzzy search for "connect" (connection, connector, etc.)' },
        { query: 'user?', description: 'Single character wildcard (user, uses, etc.)' },
        { query: 'log*~', description: 'Combine wildcards with fuzzy search' }
      ]
    },
    {
      category: 'Field-Specific',
      icon: CodeBracketIcon,
      examples: [
        { query: 'file_name:*.js', description: 'Search only in JavaScript files' },
        { query: 'project:frontend', description: 'Search only in frontend project' },
        { query: 'extension:rs AND async', description: 'Find async code in Rust files' },
        { query: 'content:"use std"', description: 'Find Rust standard library imports' }
      ]
    }
  ];

  const features = [
    {
      icon: CpuChipIcon,
      title: 'Tantivy-Powered',
      description: 'Lightning-fast full-text search with relevance scoring and advanced query capabilities.'
    },
    {
      icon: MagnifyingGlassIcon,
      title: 'Multi-Select Filters',
      description: 'Filter by projects, versions, and file types with powerful faceted search.'
    },
    {
      icon: RocketLaunchIcon,
      title: 'Real-time Results',
      description: 'Get instant search results as you type with intelligent debouncing.'
    },
    {
      icon: LightBulbIcon,
      title: 'Smart Suggestions',
      description: 'Context-aware search suggestions and search history management.'
    }
  ];

  return (
    <div className="max-w-6xl mx-auto space-y-12">
      {/* Hero Section */}
      <div className="text-center space-y-6">
        <div className="space-y-2">
          <h1 className="text-4xl font-bold text-gray-900 dark:text-white sm:text-5xl lg:text-6xl">
            Klask Code Search
          </h1>
          <p className="text-xl text-gray-600 dark:text-gray-400 max-w-3xl mx-auto">
            Lightning-fast code search powered by Tantivy. Find functions, classes, and comments
            across all your repositories with advanced query syntax.
          </p>
        </div>

        {/* Main Search Bar */}
        <div className="max-w-2xl mx-auto">
          <SearchBar
            value={query}
            onChange={setQuery}
            onSearch={handleSearch}
            placeholder="Search functions, classes, variables, comments..."
            className="text-lg"
            autoFocus
          />
        </div>

        {/* Quick Actions */}
        <div className="flex flex-wrap justify-center gap-3">
          <button
            onClick={() => navigate('/search?advanced=true')}
            className="inline-flex items-center px-4 py-2 border border-gray-300 dark:border-gray-600 rounded-md text-sm font-medium text-gray-700 dark:text-gray-300 bg-white dark:bg-gray-800 hover:bg-gray-50 dark:hover:bg-gray-700 transition-colors"
          >
            <SparklesIcon className="h-4 w-4 mr-2" />
            Advanced Search
          </button>
          <button
            onClick={() => navigate('/admin/repositories')}
            className="inline-flex items-center px-4 py-2 border border-gray-300 dark:border-gray-600 rounded-md text-sm font-medium text-gray-700 dark:text-gray-300 bg-white dark:bg-gray-800 hover:bg-gray-50 dark:hover:bg-gray-700 transition-colors"
          >
            <DocumentMagnifyingGlassIcon className="h-4 w-4 mr-2" />
            Repositories
          </button>
        </div>

        {/* Recent Searches */}
        {history.length > 0 && (
          <div className="max-w-2xl mx-auto">
            <div className="bg-gray-50 dark:bg-gray-800 rounded-lg p-4 border border-gray-200 dark:border-gray-700">
              <div className="flex items-center justify-center mb-3">
                <ClockIcon className="h-4 w-4 text-gray-400 dark:text-gray-500 mr-2" />
                <span className="text-sm font-medium text-gray-700 dark:text-gray-300">Recent Searches</span>
              </div>
              <div className="flex flex-wrap justify-center gap-2">
                {history.slice(0, 5).map((item, index) => (
                  <button
                    key={index}
                    onClick={() => handleExampleSearch(item)}
                    className="inline-flex items-center px-3 py-1 text-sm bg-white dark:bg-gray-700 text-gray-700 dark:text-gray-300 rounded-full hover:bg-gray-100 dark:hover:bg-gray-600 transition-colors shadow-sm"
                  >
                    {item}
                  </button>
                ))}
              </div>
            </div>
          </div>
        )}
      </div>

      {/* Features Section */}
      <div className="bg-white dark:bg-gray-800 rounded-lg shadow-sm border border-gray-200 dark:border-gray-700 p-8">
        <h2 className="text-2xl font-bold text-gray-900 dark:text-white text-center mb-8">
          Powerful Search Features
        </h2>
        <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-6">
          {features.map((feature, index) => (
            <div key={index} className="text-center space-y-3">
              <div className="mx-auto w-12 h-12 bg-blue-100 dark:bg-blue-900/30 rounded-lg flex items-center justify-center">
                <feature.icon className="h-6 w-6 text-blue-600 dark:text-blue-400" />
              </div>
              <h3 className="font-semibold text-gray-900 dark:text-white">{feature.title}</h3>
              <p className="text-sm text-gray-600 dark:text-gray-400">{feature.description}</p>
            </div>
          ))}
        </div>
      </div>

      {/* Search Examples Section */}
      <div className="space-y-8">
        <div className="text-center">
          <h2 className="text-2xl font-bold text-gray-900 dark:text-white mb-4">
            Tantivy Search Examples
          </h2>
          <p className="text-gray-600 dark:text-gray-400 max-w-3xl mx-auto">
            Learn how to use advanced search syntax to find exactly what you're looking for.
            Click any example to try it out.
          </p>
        </div>

        <div className="grid grid-cols-1 lg:grid-cols-2 gap-6">
          {searchExamples.map((category, categoryIndex) => (
            <div key={categoryIndex} className="bg-white dark:bg-gray-800 rounded-lg shadow-sm border border-gray-200 dark:border-gray-700 p-6">
              <div className="flex items-center mb-4">
                <div className="w-8 h-8 bg-blue-100 dark:bg-blue-900/30 rounded-lg flex items-center justify-center mr-3">
                  <category.icon className="h-5 w-5 text-blue-600 dark:text-blue-400" />
                </div>
                <h3 className="text-lg font-semibold text-gray-900 dark:text-white">{category.category}</h3>
              </div>

              <div className="space-y-3">
                {category.examples.map((example, exampleIndex) => (
                  <div
                    key={exampleIndex}
                    className="group cursor-pointer"
                    onClick={() => handleExampleSearch(example.query)}
                  >
                    <div className="p-3 rounded-lg border border-gray-200 dark:border-gray-700 hover:border-blue-300 dark:hover:border-blue-600 hover:bg-blue-50 dark:hover:bg-blue-900/20 transition-all duration-200">
                      <div className="flex items-center justify-between">
                        <code className="text-sm font-mono text-blue-600 dark:text-blue-400 group-hover:text-blue-700 dark:group-hover:text-blue-300">
                          {example.query}
                        </code>
                        <MagnifyingGlassIcon className="h-4 w-4 text-gray-400 dark:text-gray-500 group-hover:text-blue-500 dark:group-hover:text-blue-400 opacity-0 group-hover:opacity-100 transition-opacity" />
                      </div>
                      <p className="text-xs text-gray-500 dark:text-gray-400 mt-1">
                        {example.description}
                      </p>
                    </div>
                  </div>
                ))}
              </div>
            </div>
          ))}
        </div>
      </div>

      {/* Tips Section */}
      <div className="bg-gradient-to-br from-blue-50 dark:from-slate-700/40 to-indigo-100 dark:to-indigo-900/20 rounded-lg p-8 border border-blue-100 dark:border-gray-700">
        <div className="text-center mb-6">
          <LightBulbIcon className="h-8 w-8 text-blue-600 dark:text-blue-400 mx-auto mb-3" />
          <h3 className="text-xl font-semibold text-gray-900 dark:text-white">Pro Tips</h3>
        </div>

        <div className="grid grid-cols-1 md:grid-cols-2 gap-6 text-sm">
          <div>
            <h4 className="font-medium text-gray-900 dark:text-white mb-2">Search Operators</h4>
            <ul className="space-y-1 text-gray-600 dark:text-gray-400">
              <li>• Use <code className="bg-white dark:bg-gray-700 px-1 rounded text-gray-900 dark:text-gray-100">AND</code> for required terms</li>
              <li>• Use <code className="bg-white dark:bg-gray-700 px-1 rounded text-gray-900 dark:text-gray-100">OR</code> for alternative terms</li>
              <li>• Use <code className="bg-white dark:bg-gray-700 px-1 rounded text-gray-900 dark:text-gray-100">-</code> to exclude terms</li>
              <li>• Use <code className="bg-white dark:bg-gray-700 px-1 rounded text-gray-900 dark:text-gray-100">"quotes"</code> for exact phrases</li>
            </ul>
          </div>
          <div>
            <h4 className="font-medium text-gray-900 dark:text-white mb-2">Wildcards & Fuzzy</h4>
            <ul className="space-y-1 text-gray-600 dark:text-gray-400">
              <li>• Use <code className="bg-white dark:bg-gray-700 px-1 rounded text-gray-900 dark:text-gray-100">*</code> for multiple characters</li>
              <li>• Use <code className="bg-white dark:bg-gray-700 px-1 rounded text-gray-900 dark:text-gray-100">?</code> for single character</li>
              <li>• Use <code className="bg-white dark:bg-gray-700 px-1 rounded text-gray-900 dark:text-gray-100">~</code> for fuzzy matching</li>
              <li>• Combine operators for complex queries</li>
            </ul>
          </div>
        </div>
      </div>
    </div>
  );
};

export default HomePage;
