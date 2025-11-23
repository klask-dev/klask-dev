import React, { useState } from 'react';
import { useParams, useLocation, Link } from 'react-router-dom';
import { useQuery } from '@tanstack/react-query';
import OptimizedSyntaxHighlighter from '../../components/ui/OptimizedSyntaxHighlighter';
import toast from 'react-hot-toast';
import {
  ArrowLeftIcon,
  DocumentTextIcon,
  FolderIcon,
  ClipboardDocumentIcon,
  SunIcon,
  MoonIcon,
  MagnifyingGlassIcon,
  TagIcon,
  CalendarIcon,
  UserIcon,
  ChevronRightIcon,
} from '@heroicons/react/24/outline';
import { apiClient, getErrorMessage } from '../../lib/api';
import { LoadingSpinner } from '../../components/ui/LoadingSpinner';
import type { SearchResult } from '../../types';

const FileDetailPage: React.FC = () => {
  const { id, docAddress } = useParams<{ id?: string; docAddress?: string }>();
  const location = useLocation();
  const [isDarkTheme, setIsDarkTheme] = useState(false);
  const [lineNumbersVisible, setLineNumbersVisible] = useState(true);
  const [wrapLines, setWrapLines] = useState(false);

  // Get search context from navigation state
  const searchQuery = location.state?.searchQuery as string;
  const searchResult = location.state?.searchResult as SearchResult;
  const searchState = location.state?.searchState;

  // Helper function to build search URL
  const buildSearchURL = (query?: string, state?: any) => {
    if (!query) return '/search';

    const params = new URLSearchParams();
    params.set('q', query);

    if (state?.filters?.project) params.set('project', state.filters.project);
    if (state?.filters?.version) params.set('version', state.filters.version);
    if (state?.filters?.extension) params.set('extension', state.filters.extension);
    if (state?.showAdvanced) params.set('advanced', 'true');
    if (state?.page && state.page > 1) params.set('page', state.page.toString());

    return `/search?${params.toString()}`;
  };

  // Determine which parameter to use for the query
  const fileIdentifier = docAddress || id;
  const useDocAddress = !!docAddress;

  const {
    data: file,
    isLoading,
    isError,
    error,
  } = useQuery({
    queryKey: ['file', fileIdentifier, useDocAddress],
    queryFn: () => useDocAddress ? apiClient.getFileByDocAddress(docAddress!) : apiClient.getFile(id!),
    enabled: !!fileIdentifier,
    retry: 2,
  });

  const getLanguageFromExtension = (extension: string): string => {
    const languageMap: Record<string, string> = {
      'js': 'javascript',
      'jsx': 'jsx',
      'ts': 'typescript',
      'tsx': 'tsx',
      'py': 'python',
      'java': 'java',
      'cpp': 'cpp',
      'c': 'c',
      'cs': 'csharp',
      'php': 'php',
      'rb': 'ruby',
      'go': 'go',
      'rs': 'rust',
      'kt': 'kotlin',
      'swift': 'swift',
      'dart': 'dart',
      'scala': 'scala',
      'sh': 'bash',
      'yaml': 'yaml',
      'yml': 'yaml',
      'json': 'json',
      'xml': 'xml',
      'html': 'html',
      'css': 'css',
      'scss': 'scss',
      'sass': 'sass',
      'less': 'less',
      'sql': 'sql',
      'md': 'markdown',
      'dockerfile': 'dockerfile',
    };

    return languageMap[extension.toLowerCase()] || 'text';
  };

  const copyToClipboard = async (text: string) => {
    try {
      await navigator.clipboard.writeText(text);
      toast.success('Copied to clipboard!');
    } catch (err) {
      console.error('Failed to copy to clipboard:', err);
      toast.error('Failed to copy to clipboard');
    }
  };

  const formatFileSize = (bytes: number): string => {
    const units = ['B', 'KB', 'MB', 'GB'];
    let size = bytes;
    let unitIndex = 0;

    while (size >= 1024 && unitIndex < units.length - 1) {
      size /= 1024;
      unitIndex++;
    }

    return `${size.toFixed(1)} ${units[unitIndex]}`;
  };

  const formatPath = (path: string): { directory: string; filename: string } => {
    const parts = path.split('/');
    const filename = parts.pop() || '';
    const directory = parts.join('/');
    return { directory, filename };
  };

  if (isLoading) {
    return (
      <div className="max-w-7xl mx-auto">
        <div className="flex items-center justify-center min-h-96">
          <div className="text-center">
            <LoadingSpinner size="lg" className="mb-4" />
            <p className="text-gray-500">Loading file content...</p>
          </div>
        </div>
      </div>
    );
  }

  if (isError || !file) {
    return (
      <div className="max-w-7xl mx-auto">
        <div className="text-center py-12">
          <DocumentTextIcon className="mx-auto h-16 w-16 text-gray-400 mb-4" />
          <h3 className="text-lg font-medium text-gray-900 mb-2">
            File Not Found
          </h3>
          <p className="text-gray-500 mb-6">
            {getErrorMessage(error) || 'The requested file could not be found.'}
          </p>
          <div className="space-x-3">
            <Link
              to={buildSearchURL(searchQuery, searchState)}
              className="btn-primary"
            >
              Back to Search
            </Link>
            {searchQuery && (
              <Link
                to={buildSearchURL(searchQuery, searchState)}
                className="btn-secondary"
              >
                Return to Results
              </Link>
            )}
          </div>
        </div>
      </div>
    );
  }

  const { directory, filename } = formatPath(file.path);
  const language = getLanguageFromExtension(file.extension);
  const syntaxStyle = isDarkTheme ? 'oneDark' : 'oneLight';

  return (
    <div className="max-w-7xl mx-auto space-y-6">
      {/* Navigation Bar */}
      <div className="flex items-center justify-between">
        <div className="flex items-center space-x-4">
          <Link
            to={buildSearchURL(searchQuery, searchState)}
            className="inline-flex items-center text-sm text-gray-500 hover:text-gray-700"
          >
            <ArrowLeftIcon className="h-4 w-4 mr-1" />
            Back to Search
          </Link>

          {searchQuery && (
            <>
              <ChevronRightIcon className="h-4 w-4 text-gray-400" />
              <Link
                to={buildSearchURL(searchQuery, searchState)}
                className="inline-flex items-center text-sm text-blue-600 hover:text-blue-700"
              >
                <MagnifyingGlassIcon className="h-4 w-4 mr-1" />
                "{searchQuery}" results
              </Link>
            </>
          )}
        </div>

        {/* View Controls */}
        <div className="flex items-center space-x-2">
          <button
            onClick={() => setLineNumbersVisible(!lineNumbersVisible)}
            className={`px-3 py-1 text-xs rounded ${lineNumbersVisible
              ? 'bg-blue-100 text-blue-800'
              : 'bg-gray-100 text-gray-700 hover:bg-gray-200'
              }`}
          >
            Line Numbers
          </button>

          <button
            onClick={() => setWrapLines(!wrapLines)}
            className={`px-3 py-1 text-xs rounded ${wrapLines
              ? 'bg-blue-100 text-blue-800'
              : 'bg-gray-100 text-gray-700 hover:bg-gray-200'
              }`}
          >
            Wrap Lines
          </button>

          <button
            onClick={() => setIsDarkTheme(!isDarkTheme)}
            className="p-2 text-gray-500 hover:text-gray-700 rounded"
          >
            {isDarkTheme ? (
              <SunIcon className="h-4 w-4" />
            ) : (
              <MoonIcon className="h-4 w-4" />
            )}
          </button>
        </div>
      </div>

      {/* File Header */}
      <div className="bg-white border border-gray-200 rounded-lg shadow-sm">
        <div className="px-6 py-4 border-b border-gray-200">
          <div className="flex items-start justify-between">
            <div className="min-w-0 flex-1">
              {/* File Path */}
              <div className="flex items-center space-x-2 text-sm text-gray-500 mb-2">
                <FolderIcon className="h-4 w-4 flex-shrink-0" />
                <span className="truncate">{directory}</span>
                <ChevronRightIcon className="h-4 w-4 flex-shrink-0" />
              </div>

              {/* File Name */}
              <h1 className="text-2xl font-bold text-gray-900 mb-3">
                {filename}
              </h1>

              {/* File Metadata */}
              <div className="flex flex-wrap items-center gap-4 text-sm text-gray-600">
                <div className="flex items-center space-x-1">
                  <TagIcon className="h-4 w-4" />
                  <span className="font-medium">Type:</span>
                  <span className="px-2 py-1 bg-gray-100 rounded text-xs">
                    .{file.extension}
                  </span>
                </div>

                <div className="flex items-center space-x-1">
                  <DocumentTextIcon className="h-4 w-4" />
                  <span className="font-medium">Size:</span>
                  <span>{formatFileSize(file.size)}</span>
                </div>

                <div className="flex items-center space-x-1">
                  <UserIcon className="h-4 w-4" />
                  <span className="font-medium">Project:</span>
                  <span>{file.project}</span>
                </div>

                <div className="flex items-center space-x-1">
                  <CalendarIcon className="h-4 w-4" />
                  <span className="font-medium">Version:</span>
                  <span>{file.version}</span>
                </div>
              </div>
            </div>

            {/* Actions */}
            <div className="flex items-center space-x-2">
              <button
                onClick={() => copyToClipboard(file.content || '')}
                className="inline-flex items-center px-3 py-2 text-sm bg-gray-100 text-gray-700 rounded hover:bg-gray-200 transition-colors"
              >
                <ClipboardDocumentIcon className="h-4 w-4 mr-2" />
                Copy Content
              </button>
            </div>
          </div>
        </div>

        {/* Code Content */}
        <div className="relative">
          {file.content ? (
            <div className="overflow-auto">
              <OptimizedSyntaxHighlighter
                language={language}
                style={syntaxStyle}
                customStyle={{
                  margin: 0,
                  padding: '24px',
                  backgroundColor: isDarkTheme ? '#1e1e1e' : '#fafafa',
                  fontSize: '14px',
                  lineHeight: '1.5',
                }}
                showLineNumbers={lineNumbersVisible}
                wrapLines={wrapLines}
                wrapLongLines={wrapLines}
                lineNumberStyle={{
                  color: isDarkTheme ? '#6e7681' : '#656d76',
                  paddingRight: '16px',
                  marginRight: '16px',
                  borderRight: `1px solid ${isDarkTheme ? '#30363d' : '#d0d7de'}`,
                }}
              >
                {file.content}
              </OptimizedSyntaxHighlighter>
            </div>
          ) : (
            <div className="flex items-center justify-center py-12">
              <div className="text-center">
                <DocumentTextIcon className="mx-auto h-12 w-12 text-gray-400 mb-4" />
                <p className="text-gray-500">No content available for this file</p>
              </div>
            </div>
          )}
        </div>
      </div>

      {/* Search Context */}
      {searchResult && (
        <div className="bg-blue-50 border border-primary-200 rounded-lg p-4">
          <h3 className="text-sm font-medium text-primary-900 mb-2">
            Search Context
          </h3>
          <div className="text-sm text-blue-800">
            <p>
              Found in search for "<span className="font-medium">{searchQuery}</span>"
              with a relevance score of {(searchResult.score * 100).toFixed(1)}%
            </p>
            {searchResult.line_number && (
              <p className="mt-1">
                Matched content appears around line {searchResult.line_number}
              </p>
            )}
          </div>
        </div>
      )}
    </div>
  );
};

export default FileDetailPage;