import React, { useState } from 'react';
import type { CreateTokenResponse } from '../../../types';

interface TokenDisplayProps {
  token: CreateTokenResponse;
  onDone: () => void;
}

const TokenDisplay: React.FC<TokenDisplayProps> = ({ token, onDone }) => {
  const [copied, setCopied] = useState(false);

  const handleCopy = async () => {
    try {
      await navigator.clipboard.writeText(token.token);
      setCopied(true);
      setTimeout(() => setCopied(false), 2000);
    } catch {
      console.error('Failed to copy token');
    }
  };

  return (
    <div className="space-y-6">
      {/* Success Message */}
      <div className="bg-green-50 dark:bg-green-900/20 border border-green-200 dark:border-green-800 rounded-lg p-4">
        <div className="flex items-start gap-3">
          <svg className="w-5 h-5 text-green-600 dark:text-green-400 flex-shrink-0 mt-0.5" fill="currentColor" viewBox="0 0 20 20">
            <path fillRule="evenodd" d="M10 18a8 8 0 100-16 8 8 0 000 16zm3.707-9.293a1 1 0 00-1.414-1.414L9 10.586 7.707 9.293a1 1 0 00-1.414 1.414l2 2a1 1 0 001.414 0l4-4z" clipRule="evenodd" />
          </svg>
          <div>
            <h3 className="font-medium text-green-900 dark:text-green-300">Token Created Successfully</h3>
            <p className="text-sm text-green-700 dark:text-green-400 mt-1">
              Your API token has been generated. Copy it now and store it safely.
            </p>
          </div>
        </div>
      </div>

      {/* Token Display */}
      <div>
        <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
          Your API Token
        </label>
        <div className="relative">
          <div className="bg-gray-900 dark:bg-gray-950 rounded-lg p-4 font-mono text-sm text-green-400 break-all overflow-x-auto">
            {token.token}
          </div>
          <button
            type="button"
            onClick={handleCopy}
            className="absolute top-3 right-3 px-3 py-1 bg-gray-700 hover:bg-gray-600 dark:bg-gray-800 dark:hover:bg-gray-700 text-white text-xs font-medium rounded transition"
          >
            {copied ? 'Copied!' : 'Copy'}
          </button>
        </div>
      </div>

      {/* Token Prefix for Verification */}
      <div>
        <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
          Token Prefix (for verification)
        </label>
        <div className="bg-gray-50 dark:bg-gray-900 border border-gray-200 dark:border-gray-700 rounded-lg p-3">
          <code className="text-gray-900 dark:text-gray-300 font-mono text-sm">{token.token_prefix}...</code>
        </div>
      </div>

      {/* Warning Box */}
      <div className="bg-yellow-50 dark:bg-yellow-900/20 border border-yellow-200 dark:border-yellow-800 rounded-lg p-4">
        <div className="flex items-start gap-3">
          <svg className="w-5 h-5 text-yellow-600 dark:text-yellow-400 flex-shrink-0 mt-0.5" fill="currentColor" viewBox="0 0 20 20">
            <path fillRule="evenodd" d="M8.257 3.099c.765-1.36 2.722-1.36 3.486 0l5.58 9.92c.75 1.334-.213 2.98-1.742 2.98H4.42c-1.53 0-2.493-1.646-1.743-2.98l5.58-9.92zM11 13a1 1 0 11-2 0 1 1 0 012 0zm-1-8a1 1 0 00-1 1v3a1 1 0 002 0V6a1 1 0 00-1-1z" clipRule="evenodd" />
          </svg>
          <div>
            <h3 className="font-medium text-yellow-900 dark:text-yellow-300">Keep this token safe</h3>
            <p className="text-sm text-yellow-700 dark:text-yellow-400 mt-1">
              Do not commit this token to git or share it with others. Anyone with this token can access the API using your account.
            </p>
          </div>
        </div>
      </div>

      {/* Done Button */}
      <div className="flex justify-end">
        <button
          onClick={onDone}
          className="px-6 py-2 bg-blue-600 hover:bg-blue-700 dark:bg-blue-700 dark:hover:bg-blue-600 text-white rounded-lg transition font-medium"
        >
          Done
        </button>
      </div>
    </div>
  );
};

export default TokenDisplay;
