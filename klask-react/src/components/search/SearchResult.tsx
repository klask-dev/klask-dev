import React from 'react';
import {
  DocumentTextIcon,
  FolderIcon,
  EyeIcon,
  ArrowTopRightOnSquareIcon
} from '@heroicons/react/24/outline';
import { RepositoryBadge } from '../ui/RepositoryBadge';
import type { SearchResult as SearchResultType } from '../../types';

interface SearchResultProps {
  result: SearchResultType;
  query: string;
  onFileClick: (result: SearchResultType) => void;
  className?: string;
}

export const SearchResult: React.FC<SearchResultProps> = ({
  result,
  query,
  onFileClick,
  className = '',
}) => {

  const highlightQuery = (text: string, query: string): React.JSX.Element => {
    if (!text || !query.trim()) return <>{text || ''}</>;
    
    const parts = text.split(new RegExp(`(${query})`, 'gi'));
    return (
      <>
        {parts.map((part, index) => 
          part.toLowerCase() === query.toLowerCase() ? (
            <mark key={index} className="bg-yellow-200 font-semibold">
              {part}
            </mark>
          ) : (
            part
          )
        )}
      </>
    );
  };

  const formatPath = (path: string): { directory: string; filename: string } => {
    if (!path) {
      return { directory: '', filename: 'Unknown file' };
    }
    const parts = path.split('/');
    const filename = parts.pop() || 'Unknown file';
    const directory = parts.join('/');
    return { directory, filename };
  };

  // Try name first, then extract from path
  const extractedPath = formatPath(result.path || '');
  const filename = result.name || extractedPath.filename;
  const directory = extractedPath.directory;

  return (
    <div 
      className={`bg-white border border-gray-200 rounded-lg shadow-sm hover:shadow-md transition-shadow duration-200 ${className}`}
    >
      {/* File Header */}
      <div className="px-4 py-3 border-b border-gray-100">
        {/* Main row with file info and badges */}
        <div className="flex items-start justify-between gap-4">
          {/* Left side: Icon and file info */}
          <div className="flex items-start space-x-3 min-w-0 flex-1">
            <DocumentTextIcon className="h-5 w-5 text-gray-400 flex-shrink-0 mt-0.5" />
            <div className="min-w-0 flex-1">
              {/* File name - prominent */}
              <h3 className="font-semibold text-gray-900 text-base leading-tight">
                {highlightQuery(filename, query)}
              </h3>
            </div>
          </div>
          
          {/* Right side: Badges */}
          <div className="flex flex-col items-end space-y-2">
            <div className="flex items-center space-x-2 flex-shrink-0">
              <span className="inline-flex items-center px-2 py-1 text-xs font-medium bg-blue-50 text-blue-700 rounded">
                {result.extension || 'N/A'}
              </span>
              <span className="inline-flex items-center px-2 py-1 text-xs font-medium bg-green-50 text-green-700 rounded">
                {((result.score || 0) * 100).toFixed(0)}%
              </span>
            </div>
            {/* View File button moved here */}
            <button
              onClick={() => onFileClick(result)}
              className="inline-flex items-center space-x-1 px-2 py-1 text-xs bg-blue-100 text-blue-800 rounded hover:bg-blue-200 transition-colors"
            >
              <EyeIcon className="h-3 w-3" />
              <span>View File</span>
              <ArrowTopRightOnSquareIcon className="h-3 w-3" />
            </button>
          </div>
        </div>
        
        {/* Metadata row */}
        <div className="mt-2.5 flex flex-wrap items-center gap-x-3 gap-y-1.5 text-xs text-gray-500">
          {result.repository_name && (
            <RepositoryBadge
              name={result.repository_name}
              size="sm"
              clickable={false}
            />
          )}
          <span className="inline-flex items-center">
            <span className="font-medium text-gray-600">Project:</span>
            <span className="ml-1">{result.project || 'Unknown'}</span>
          </span>
          <span className="text-gray-300">•</span>
          <span className="inline-flex items-center">
            <span className="font-medium text-gray-600">Version:</span>
            <span className="ml-1">{result.version || 'Unknown'}</span>
          </span>
          {result.line_number && (
            <>
              <span className="text-gray-300">•</span>
              <span className="inline-flex items-center">
                <span className="font-medium text-gray-600">Line:</span>
                <span className="ml-1">{result.line_number}</span>
              </span>
            </>
          )}
        </div>
      </div>

      {/* Code Preview */}
      <div className="p-4">
        <div className="relative">
          <div className="overflow-hidden rounded border border-gray-200">
            <div
              className="p-3 bg-gray-50 text-sm font-mono overflow-auto"
              style={{
                maxHeight: '200px',
                lineHeight: '1.5',
                whiteSpace: 'pre-wrap',
                wordBreak: 'break-word',
              }}
              dangerouslySetInnerHTML={{
                __html: (() => {
                  // Tantivy sends HTML with entities encoded and <b> tags for highlighting
                  // We need to:
                  // 1. Decode HTML entities (safe because Tantivy already escaped them)
                  // 2. Convert <b> tags to safe <mark> tags for highlighting
                  const snippet = result.content_snippet || 'No content preview available';

                  // Decode HTML entities
                  const textarea = document.createElement('textarea');
                  textarea.innerHTML = snippet;
                  const decoded = textarea.value;

                  // Now re-encode for safe insertion, except for our <b> tags
                  let html = decoded
                    .replace(/&/g, '&amp;')
                    .replace(/</g, '&lt;')
                    .replace(/>/g, '&gt;');

                  // Finally, convert <b> tags to safe highlight marks
                  html = html
                    .replace(/&lt;b&gt;/g, '<mark class="bg-yellow-200 font-semibold">')
                    .replace(/&lt;\/b&gt;/g, '</mark>');

                  return html;
                })()
              }}
            />
          </div>
        </div>
        
        {/* Full Path Info */}
        <div className="mt-2 text-xs text-gray-500">
          <div className="flex items-center space-x-1">
            <FolderIcon className="h-3.5 w-3.5" />
            <span className="font-mono">{result.path || 'Unknown path'}</span>
          </div>
        </div>
      </div>
    </div>
  );
};