import React, { useState } from 'react';
import ErrorBoundary from '../../../components/ErrorBoundary';
import CreateTokenModal from './CreateTokenModal';
import TokensList from './TokensList';

const ApiTokensPage: React.FC = () => {
  const [createModalOpen, setCreateModalOpen] = useState(false);

  return (
    <ErrorBoundary onError={(error) => console.error('ApiTokensPage error:', error)}>
      <div className="max-w-4xl mx-auto py-8 px-4 space-y-8">
        {/* Page Header */}
        <div>
          <h1 className="text-3xl font-bold text-gray-900 dark:text-white">API Tokens</h1>
          <p className="text-gray-600 dark:text-gray-400 mt-1">
            Manage personal API tokens to authenticate with the Klask API
          </p>
        </div>

        {/* Info Box */}
        <div className="bg-blue-50 dark:bg-blue-900/20 border border-blue-200 dark:border-blue-800 rounded-lg p-6">
          <div className="flex gap-4">
            <svg className="w-5 h-5 text-blue-600 dark:text-blue-400 flex-shrink-0 mt-0.5" fill="currentColor" viewBox="0 0 20 20">
              <path fillRule="evenodd" d="M18 5v8a2 2 0 01-2 2h-5l-5 4v-4H4a2 2 0 01-2-2V5a2 2 0 012-2h12a2 2 0 012 2zm-11-1a1 1 0 11-2 0 1 1 0 012 0zM8 7a1 1 0 100-2 1 1 0 000 2zm5-1a1 1 0 11-2 0 1 1 0 012 0zM14 7a1 1 0 100-2 1 1 0 000 2z" clipRule="evenodd" />
            </svg>
            <div>
              <h3 className="font-medium text-blue-900 dark:text-blue-300">API Token Documentation</h3>
              <p className="text-sm text-blue-700 dark:text-blue-400 mt-1">
                Use personal API tokens to authenticate with the Klask API. Each token has limited scope and can be revoked individually without affecting other tokens.
              </p>
            </div>
          </div>
        </div>

        {/* Active Tokens Section */}
        <div className="space-y-4">
          <div className="flex flex-col sm:flex-row items-start sm:items-center justify-between gap-4">
            <div>
              <h2 className="text-2xl font-bold text-gray-900 dark:text-white">Active Tokens</h2>
              <p className="text-gray-600 dark:text-gray-400 text-sm mt-1">View and manage your API tokens</p>
            </div>
            <button
              onClick={() => setCreateModalOpen(true)}
              className="px-6 py-2 bg-blue-600 hover:bg-blue-700 dark:bg-blue-700 dark:hover:bg-blue-600 text-white rounded-lg transition font-medium flex items-center gap-2 flex-shrink-0"
            >
              <svg className="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M12 4v16m8-8H4" />
              </svg>
              Create New Token
            </button>
          </div>

          {/* Tokens List */}
          <div className="bg-white dark:bg-gray-800 rounded-lg shadow-sm border border-gray-200 dark:border-gray-700 p-6">
            <TokensList />
          </div>
        </div>

        {/* Security Notes */}
        <div className="bg-yellow-50 dark:bg-yellow-900/20 border border-yellow-200 dark:border-yellow-800 rounded-lg p-6">
          <h3 className="font-bold text-yellow-900 dark:text-yellow-300 mb-4">Security Best Practices</h3>
          <ul className="space-y-2 text-sm text-yellow-700 dark:text-yellow-400">
            <li className="flex gap-3">
              <span className="flex-shrink-0">•</span>
              <span>Never share your API tokens with others or commit them to version control</span>
            </li>
            <li className="flex gap-3">
              <span className="flex-shrink-0">•</span>
              <span>Rotate tokens regularly and revoke tokens you no longer use</span>
            </li>
            <li className="flex gap-3">
              <span className="flex-shrink-0">•</span>
              <span>Use short-lived tokens when possible for scripts and CI/CD pipelines</span>
            </li>
            <li className="flex gap-3">
              <span className="flex-shrink-0">•</span>
              <span>Monitor token usage in the "Last Used" column to detect unauthorized access</span>
            </li>
            <li className="flex gap-3">
              <span className="flex-shrink-0">•</span>
              <span>If a token is compromised, revoke it immediately</span>
            </li>
          </ul>
        </div>
      </div>

      {/* Create Token Modal */}
      <CreateTokenModal isOpen={createModalOpen} onClose={() => setCreateModalOpen(false)} />
    </ErrorBoundary>
  );
};

export default ApiTokensPage;
export { ApiTokensPage };
