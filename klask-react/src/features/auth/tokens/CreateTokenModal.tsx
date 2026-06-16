import React, { useState, useRef, useEffect } from 'react';
import toast from 'react-hot-toast';
import { useCreateApiToken } from '../../../api/apiTokens';
import type { CreateTokenResponse } from '../../../types';
import TokenDisplay from './TokenDisplay';

interface CreateTokenModalProps {
  isOpen: boolean;
  onClose: () => void;
}

const CreateTokenModal: React.FC<CreateTokenModalProps> = ({ isOpen, onClose }) => {
  const [tokenName, setTokenName] = useState('');
  const [showToken, setShowToken] = useState(false);
  const [createdToken, setCreatedToken] = useState<CreateTokenResponse | null>(null);
  const [error, setError] = useState<string>('');
  const createTokenMutation = useCreateApiToken();
  const inputRef = useRef<HTMLInputElement>(null);

  // Focus on mount for accessibility
  useEffect(() => {
    if (isOpen && !showToken) {
      inputRef.current?.focus();
    }
  }, [isOpen, showToken]);

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    setError('');

    // Validate input
    if (!tokenName.trim()) {
      setError('Token name is required');
      return;
    }

    if (tokenName.length > 50) {
      setError('Token name must be 50 characters or less');
      return;
    }

    // Create token
    createTokenMutation.mutate(tokenName, {
      onSuccess: (token) => {
        setCreatedToken(token);
        setShowToken(true);
        toast.success('API token created successfully');
      },
      onError: (error: unknown) => {
        const message = error && typeof error === 'object' && 'message' in error
          ? (error as { message: string }).message
          : 'Failed to create token';
        setError(message);
        toast.error(message);
      },
    });
  };

  const handleDone = () => {
    setTokenName('');
    setShowToken(false);
    setCreatedToken(null);
    setError('');
    onClose();
  };

  const handleClose = () => {
    if (!showToken) {
      handleDone();
    }
  };

  if (!isOpen) {
    return null;
  }

  return (
    <div className="fixed inset-0 z-50 bg-black bg-opacity-50 flex items-center justify-center p-4">
      <div className="bg-white dark:bg-gray-800 rounded-lg shadow-lg max-w-md w-full max-h-[90vh] overflow-y-auto">
        {/* Header */}
        <div className="flex items-center justify-between p-6 border-b border-gray-200 dark:border-gray-700">
          <h2 className="text-lg font-bold text-gray-900 dark:text-white">
            {showToken ? 'Token Created' : 'Create API Token'}
          </h2>
          <button
            onClick={handleClose}
            className="text-gray-400 dark:text-gray-500 hover:text-gray-600 dark:hover:text-gray-400 transition"
            aria-label="Close modal"
          >
            <svg className="w-6 h-6" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M6 18L18 6M6 6l12 12" />
            </svg>
          </button>
        </div>

        {/* Content */}
        <div className="p-6">
          {showToken && createdToken ? (
            <TokenDisplay token={createdToken} onDone={handleDone} />
          ) : (
            <form onSubmit={handleSubmit} className="space-y-6">
              {/* Description */}
              <p className="text-sm text-gray-600 dark:text-gray-400">
                Create a new personal API token to authenticate programmatically with the Klask API.
              </p>

              {/* Token Name Input */}
              <div>
                <label htmlFor="token-name" className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
                  Token Name <span className="text-red-500">*</span>
                </label>
                <input
                  ref={inputRef}
                  id="token-name"
                  type="text"
                  value={tokenName}
                  onChange={(e) => {
                    setTokenName(e.target.value);
                    setError('');
                  }}
                  placeholder="e.g., Development, Production, CI/CD"
                  maxLength={50}
                  className={`w-full px-4 py-2 border rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-transparent outline-none transition bg-white dark:bg-gray-900 text-gray-900 dark:text-white ${
                    error ? 'border-red-500 dark:border-red-500' : 'border-gray-300 dark:border-gray-600'
                  }`}
                  disabled={createTokenMutation.isPending}
                />
                <p className="text-xs text-gray-500 dark:text-gray-400 mt-1">
                  {tokenName.length}/50 characters
                </p>
              </div>

              {/* Error Message */}
              {error && (
                <div className="bg-red-50 dark:bg-red-900/20 border border-red-200 dark:border-red-800 rounded-lg p-3">
                  <p className="text-sm text-red-700 dark:text-red-400">{error}</p>
                </div>
              )}

              {/* Generate Button */}
              <div className="flex justify-end gap-3">
                <button
                  type="button"
                  onClick={() => handleClose()}
                  className="px-6 py-2 text-gray-700 dark:text-gray-300 hover:bg-gray-100 dark:hover:bg-gray-700 rounded-lg transition font-medium"
                  disabled={createTokenMutation.isPending}
                >
                  Cancel
                </button>
                <button
                  type="submit"
                  disabled={!tokenName.trim() || createTokenMutation.isPending}
                  className="px-6 py-2 bg-blue-600 hover:bg-blue-700 dark:bg-blue-700 dark:hover:bg-blue-600 text-white rounded-lg disabled:bg-gray-400 dark:disabled:bg-gray-600 disabled:cursor-not-allowed transition font-medium flex items-center gap-2"
                >
                  {createTokenMutation.isPending && (
                    <div className="w-4 h-4 border-2 border-white border-t-transparent rounded-full animate-spin" />
                  )}
                  {createTokenMutation.isPending ? 'Generating...' : 'Generate Token'}
                </button>
              </div>
            </form>
          )}
        </div>
      </div>
    </div>
  );
};

export default CreateTokenModal;
