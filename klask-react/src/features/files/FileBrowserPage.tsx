import React, { useState, useCallback } from 'react';
import { useParams, useSearchParams } from 'react-router-dom';
import { 
  FolderIcon,
  DocumentIcon,
  AdjustmentsHorizontalIcon,
  ViewColumnsIcon,
  Squares2X2Icon,
} from '@heroicons/react/24/outline';
import { FileBrowser } from '../../components/files/FileBrowser';
import { FilePreview } from '../../components/files/FilePreview';
import { useRepositories } from '../../hooks/useRepositories';
import { useFileStats, type FileTreeNode } from '../../hooks/useFiles';
import { LoadingSpinner } from '../../components/ui/LoadingSpinner';
import { formatFileSize } from '../../lib/utils';

type ViewMode = 'split' | 'preview' | 'browser';

const FileBrowserPage: React.FC = () => {
  const { id: repositoryId } = useParams<{ id: string }>();
  const [searchParams, setSearchParams] = useSearchParams();
  
  const [selectedFile, setSelectedFile] = useState<FileTreeNode | null>(null);
  const [selectedPath, setSelectedPath] = useState<string>('');
  const [viewMode, setViewMode] = useState<ViewMode>('split');

  // Get project name from repository data
  const { data: repositories } = useRepositories();
  const repositoryWithStats = repositories?.find(repo => repo.repository.id === repositoryId);
  const projectName = repositoryWithStats?.repository.name || '';

  // Get file statistics
  const { data: fileStats } = useFileStats(projectName);

  const handleFileSelect = useCallback((file: FileTreeNode) => {
    if (file.type === 'file') {
      setSelectedFile(file);
      setSelectedPath(file.path);
      setSearchParams({ file: file.path });
    }
  }, [setSearchParams]);

  const handleDirectorySelect = useCallback((directory: FileTreeNode) => {
    setSelectedPath(directory.path);
    setSearchParams({ path: directory.path });
  }, [setSearchParams]);

  const handleViewModeChange = (mode: ViewMode) => {
    setViewMode(mode);
  };

  if (!repositoryId) {
    return (
      <div className="max-w-7xl mx-auto">
        <div className="text-center py-12">
          <FolderIcon className="mx-auto h-16 w-16 text-slate-300 dark:text-slate-600 mb-4" />
          <h3 className="text-lg font-medium text-slate-900 dark:text-white mb-2">
            No Repository Selected
          </h3>
          <p className="text-slate-500 dark:text-slate-400">
            Please select a repository to browse its files.
          </p>
        </div>
      </div>
    );
  }

  if (!repositoryWithStats) {
    return (
      <div className="max-w-7xl mx-auto">
        <div className="flex items-center justify-center min-h-96">
          <div className="text-center">
            <LoadingSpinner size="lg" className="mb-4" />
            <p className="text-slate-500 dark:text-slate-400">Loading repository...</p>
          </div>
        </div>
      </div>
    );
  }

  return (
    <div className="max-w-7xl mx-auto space-y-6">
      {/* Header */}
      <div className="md:flex md:items-center md:justify-between">
        <div className="min-w-0 flex-1">
          <h1 className="text-2xl font-bold leading-7 text-slate-900 dark:text-white sm:truncate sm:text-3xl sm:tracking-tight">
            {repositoryWithStats?.repository.name}
          </h1>
          <div className="mt-1 flex flex-col sm:flex-row sm:flex-wrap sm:space-x-6">
            <div className="mt-2 flex items-center text-sm text-slate-500 dark:text-slate-400">
              <FolderIcon className="h-4 w-4 mr-1" />
              {repositoryWithStats?.repository.repositoryType}
            </div>
            {fileStats && (
              <>
                <div className="mt-2 flex items-center text-sm text-slate-500 dark:text-slate-400">
                  <DocumentIcon className="h-4 w-4 mr-1" />
                  {fileStats.totalFiles} files
                </div>
                <div className="mt-2 flex items-center text-sm text-slate-500 dark:text-slate-400">
                  <span>Total size: {formatFileSize(fileStats.totalSize)}</span>
                </div>
              </>
            )}
          </div>
        </div>

        <div className="mt-4 md:mt-0 flex items-center space-x-2">
          <div className="flex items-center bg-slate-100 dark:bg-slate-700 rounded-lg p-1">
            <button
              onClick={() => handleViewModeChange('browser')}
              className={`p-2 rounded transition-colors ${
                viewMode === 'browser'
                  ? 'bg-white dark:bg-slate-800 text-slate-900 dark:text-white shadow-sm'
                  : 'text-slate-500 dark:text-slate-400 hover:text-slate-700 dark:hover:text-slate-300'
              }`}
              title="Files only"
            >
              <FolderIcon className="h-4 w-4" />
            </button>
            <button
              onClick={() => handleViewModeChange('split')}
              className={`p-2 rounded transition-colors ${
                viewMode === 'split'
                  ? 'bg-white dark:bg-slate-800 text-slate-900 dark:text-white shadow-sm'
                  : 'text-slate-500 dark:text-slate-400 hover:text-slate-700 dark:hover:text-slate-300'
              }`}
              title="Split view"
            >
              <ViewColumnsIcon className="h-4 w-4" />
            </button>
            <button
              onClick={() => handleViewModeChange('preview')}
              className={`p-2 rounded transition-colors ${
                viewMode === 'preview'
                  ? 'bg-white dark:bg-slate-800 text-slate-900 dark:text-white shadow-sm'
                  : 'text-slate-500 dark:text-slate-400 hover:text-slate-700 dark:hover:text-slate-300'
              }`}
              title="Preview only"
            >
              <Squares2X2Icon className="h-4 w-4" />
            </button>
          </div>
        </div>
      </div>

      {/* File Statistics */}
      {fileStats && (
        <div className="grid grid-cols-1 md:grid-cols-3 lg:grid-cols-5 gap-4">
          <div className="bg-white dark:bg-gray-800 border border-slate-200 dark:border-slate-700 rounded-lg p-4">
            <div className="text-2xl font-bold text-slate-900 dark:text-white">{fileStats.totalFiles}</div>
            <div className="text-sm text-slate-500 dark:text-slate-400">Total Files</div>
          </div>

          {Object.entries(fileStats.byExtension)
            .sort((a, b) => b[1].count - a[1].count)
            .slice(0, 4)
            .map(([ext, stats]) => (
              <div key={ext} className="bg-white dark:bg-gray-800 border border-slate-200 dark:border-slate-700 rounded-lg p-4">
                <div className="text-2xl font-bold text-slate-900 dark:text-white">{stats.count}</div>
                <div className="text-sm text-slate-500 dark:text-slate-400">{ext || 'No extension'} files</div>
              </div>
            ))
          }
        </div>
      )}

      {/* Main Content */}
      <div className="bg-white dark:bg-gray-800 border border-slate-200 dark:border-slate-700 rounded-lg overflow-hidden">
        {viewMode === 'browser' && (
          <FileBrowser
            project={projectName}
            selectedPath={selectedPath}
            onFileSelect={handleFileSelect}
            onDirectorySelect={handleDirectorySelect}
            className="h-96"
          />
        )}

        {viewMode === 'preview' && (
          <div className="p-6">
            {selectedFile ? (
              <FilePreview
                file={{
                  id: selectedFile.path, // Using path as ID for now
                  name: selectedFile.name,
                  path: selectedFile.path,
                  extension: selectedFile.extension || '',
                  size: selectedFile.size || 0,
                  lastModified: selectedFile.lastModified || new Date().toISOString(),
                  project: projectName,
                  version: 'latest',
                  createdAt: new Date().toISOString(),
                  updatedAt: new Date().toISOString(),
                }}
              />
            ) : (
              <div className="text-center py-12 text-slate-500 dark:text-slate-400">
                <DocumentIcon className="mx-auto h-16 w-16 mb-4" />
                <p>Select a file from the browser to preview it here</p>
              </div>
            )}
          </div>
        )}

        {viewMode === 'split' && (
          <div className="flex h-96">
            {/* File Browser */}
            <div className="w-1/3 border-r border-slate-200 dark:border-slate-700">
              <FileBrowser
                project={projectName}
                selectedPath={selectedPath}
                onFileSelect={handleFileSelect}
                onDirectorySelect={handleDirectorySelect}
                className="h-full"
              />
            </div>

            {/* File Preview */}
            <div className="flex-1 overflow-hidden">
              {selectedFile ? (
                <FilePreview
                  file={{
                    id: selectedFile.path, // Using path as ID for now
                    name: selectedFile.name,
                    path: selectedFile.path,
                    extension: selectedFile.extension || '',
                    size: selectedFile.size || 0,
                    lastModified: selectedFile.lastModified || new Date().toISOString(),
                    project: projectName,
                    version: 'latest',
                    createdAt: new Date().toISOString(),
                    updatedAt: new Date().toISOString(),
                  }}
                  showMetadata={false}
                  maxHeight="100%"
                  className="h-full"
                />
              ) : (
                <div className="flex items-center justify-center h-full text-slate-500 dark:text-slate-400">
                  <div className="text-center">
                    <DocumentIcon className="mx-auto h-16 w-16 mb-4" />
                    <p>Select a file to preview</p>
                  </div>
                </div>
              )}
            </div>
          </div>
        )}
      </div>

      {/* File Extensions Overview */}
      {fileStats && Object.keys(fileStats.byExtension).length > 0 && (
        <div className="bg-white dark:bg-gray-800 border border-slate-200 dark:border-slate-700 rounded-lg p-6">
          <h3 className="text-lg font-semibold text-slate-900 dark:text-white mb-4">File Types</h3>
          <div className="grid grid-cols-2 md:grid-cols-3 lg:grid-cols-4 gap-4">
            {Object.entries(fileStats.byExtension)
              .sort((a, b) => b[1].count - a[1].count)
              .map(([ext, stats]) => (
                <div key={ext} className="flex items-center justify-between p-3 bg-slate-50 dark:bg-slate-700 rounded">
                  <span className="text-sm font-medium text-slate-700 dark:text-slate-300">
                    {ext || 'No extension'}
                  </span>
                  <div className="text-right">
                    <div className="text-sm font-bold text-slate-900 dark:text-white">{stats.count}</div>
                    <div className="text-xs text-slate-500 dark:text-slate-400">{formatFileSize(stats.size)}</div>
                  </div>
                </div>
              ))
            }
          </div>
        </div>
      )}
    </div>
  );
};

export default FileBrowserPage;