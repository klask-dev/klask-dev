import React, { useState } from 'react';
import toast from 'react-hot-toast';
import { useRevokeApiToken, useApiTokens } from '../../../api/apiTokens';

interface RevokeConfirmState {
  tokenId: string | null;
  tokenName: string | null;
}

const TokensList: React.FC = () => {
  const { data: tokens, isLoading, error } = useApiTokens();
  const revokeTokenMutation = useRevokeApiToken();
  const [revokeConfirm, setRevokeConfirm] = useState<RevokeConfirmState>({ tokenId: null, tokenName: null });

  const handleRevokeClick = (tokenId: string, tokenName: string) => {
    setRevokeConfirm({ tokenId, tokenName });
  };

  const handleConfirmRevoke = () => {
    if (!revokeConfirm.tokenId) return;

    revokeTokenMutation.mutate(revokeConfirm.tokenId, {
      onSuccess: () => {
        toast.success(`Token "${revokeConfirm.tokenName}" revoked successfully`);
        setRevokeConfirm({ tokenId: null, tokenName: null });
      },
      onError: (error: unknown) => {
        const message = error && typeof error === 'object' && 'message' in error
          ? (error as { message: string }).message
          : 'Failed to revoke token';
        toast.error(message);
      },
    });
  };

  // Helper function to format relative time
  const getRelativeTime = (dateString: string | null): string => {
    if (!dateString) return 'Never';

    const date = new Date(dateString);
    const now = new Date();
    const diffMs = now.getTime() - date.getTime();
    const diffDays = Math.floor(diffMs / (1000 * 60 * 60 * 24));
    const diffHours = Math.floor(diffMs / (1000 * 60 * 60));
    const diffMinutes = Math.floor(diffMs / (1000 * 60));

    if (diffDays > 0) return `${diffDays} day${diffDays > 1 ? 's' : ''} ago`;
    if (diffHours > 0) return `${diffHours} hour${diffHours > 1 ? 's' : ''} ago`;
    if (diffMinutes > 0) return `${diffMinutes} minute${diffMinutes > 1 ? 's' : ''} ago`;
    return 'Just now';
  };

  // Helper function to format date for tooltip
  const formatDate = (dateString: string): string => {
    return new Date(dateString).toLocaleString();
  };

  // Loading state
  if (isLoading) {
    return (
      <div className="space-y-4">
        {[1, 2, 3].map((i) => (
          <div key={i} className="bg-gray-200 dark:bg-gray-700 rounded-lg h-16 animate-pulse" />
        ))}
      </div>
    );
  }

  // Error state
  if (error) {
    return (
      <div className="bg-red-50 dark:bg-red-900/20 border border-red-200 dark:border-red-800 rounded-lg p-6 text-center">
        <p className="text-red-700 dark:text-red-400">Failed to load API tokens</p>
      </div>
    );
  }

  // Empty state
  if (!tokens || tokens.length === 0) {
    return (
      <div className="bg-gray-50 dark:bg-gray-900/50 border-2 border-dashed border-gray-300 dark:border-gray-600 rounded-lg p-8 text-center">
        <svg className="w-12 h-12 text-gray-400 dark:text-gray-500 mx-auto mb-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={1.5} d="M12 15v2m-6 4h12a2 2 0 002-2v-6a2 2 0 00-2-2H6a2 2 0 00-2 2v6a2 2 0 002 2zm10-10V7a4 4 0 00-8 0v4h8z" />
        </svg>
        <p className="text-gray-700 dark:text-gray-400 font-medium">No API tokens yet</p>
        <p className="text-gray-600 dark:text-gray-500 text-sm mt-1">Create your first token to start using the API</p>
      </div>
    );
  }

  return (
    <div className="space-y-4">
      {/* Tokens Table on larger screens */}
      <div className="hidden md:block overflow-x-auto">
        <table className="w-full">
          <thead>
            <tr className="border-b border-gray-200 dark:border-gray-700">
              <th className="px-4 py-3 text-left text-sm font-semibold text-gray-700 dark:text-gray-300">Name</th>
              <th className="px-4 py-3 text-left text-sm font-semibold text-gray-700 dark:text-gray-300">Token Prefix</th>
              <th className="px-4 py-3 text-left text-sm font-semibold text-gray-700 dark:text-gray-300">Scope</th>
              <th className="px-4 py-3 text-left text-sm font-semibold text-gray-700 dark:text-gray-300">Created</th>
              <th className="px-4 py-3 text-left text-sm font-semibold text-gray-700 dark:text-gray-300">Last Used</th>
              <th className="px-4 py-3 text-right text-sm font-semibold text-gray-700 dark:text-gray-300">Action</th>
            </tr>
          </thead>
          <tbody>
            {tokens.map((token) => (
              <tr
                key={token.id}
                className="border-b border-gray-200 dark:border-gray-700 hover:bg-gray-50 dark:hover:bg-gray-700/50 transition"
              >
                <td className="px-4 py-3 text-sm text-gray-900 dark:text-white font-medium">
                  {token.name}
                  {token.revoked_at && (
                    <span className="ml-2 inline-block px-2 py-1 bg-red-100 dark:bg-red-900/30 text-red-700 dark:text-red-400 text-xs rounded font-medium">
                      Revoked
                    </span>
                  )}
                </td>
                <td className="px-4 py-3 text-sm text-gray-700 dark:text-gray-300 font-mono">
                  {token.token_prefix}...
                </td>
                <td className="px-4 py-3 text-sm text-gray-700 dark:text-gray-300">
                  {token.scope}
                </td>
                <td className="px-4 py-3 text-sm text-gray-600 dark:text-gray-400">
                  <div title={formatDate(token.created_at)}>
                    {getRelativeTime(token.created_at)}
                  </div>
                </td>
                <td className="px-4 py-3 text-sm text-gray-600 dark:text-gray-400">
                  {token.last_used_at ? (
                    <div title={formatDate(token.last_used_at)}>
                      {getRelativeTime(token.last_used_at)}
                    </div>
                  ) : (
                    <span className="text-gray-500 dark:text-gray-500">Never</span>
                  )}
                </td>
                <td className="px-4 py-3 text-right">
                  {!token.revoked_at && (
                    <button
                      onClick={() => handleRevokeClick(token.id, token.name)}
                      className="text-red-600 dark:text-red-400 hover:text-red-700 dark:hover:text-red-300 font-medium text-sm transition"
                    >
                      Revoke
                    </button>
                  )}
                </td>
              </tr>
            ))}
          </tbody>
        </table>
      </div>

      {/* Tokens Cards on smaller screens */}
      <div className="md:hidden space-y-4">
        {tokens.map((token) => (
          <div key={token.id} className="bg-white dark:bg-gray-800 border border-gray-200 dark:border-gray-700 rounded-lg p-4 space-y-3">
            <div className="flex items-start justify-between gap-2">
              <div>
                <h3 className="font-medium text-gray-900 dark:text-white">{token.name}</h3>
                {token.revoked_at && (
                  <span className="inline-block mt-1 px-2 py-1 bg-red-100 dark:bg-red-900/30 text-red-700 dark:text-red-400 text-xs rounded font-medium">
                    Revoked
                  </span>
                )}
              </div>
              {!token.revoked_at && (
                <button
                  onClick={() => handleRevokeClick(token.id, token.name)}
                  className="text-red-600 dark:text-red-400 hover:text-red-700 dark:hover:text-red-300 font-medium text-sm transition"
                >
                  Revoke
                </button>
              )}
            </div>

            <div className="space-y-2 text-sm">
              <div>
                <p className="text-gray-600 dark:text-gray-400">Prefix:</p>
                <p className="font-mono text-gray-900 dark:text-white">{token.token_prefix}...</p>
              </div>

              <div>
                <p className="text-gray-600 dark:text-gray-400">Scope:</p>
                <p className="text-gray-900 dark:text-white">{token.scope}</p>
              </div>

              <div className="grid grid-cols-2 gap-2">
                <div>
                  <p className="text-gray-600 dark:text-gray-400">Created:</p>
                  <p className="text-gray-900 dark:text-white" title={formatDate(token.created_at)}>
                    {getRelativeTime(token.created_at)}
                  </p>
                </div>
                <div>
                  <p className="text-gray-600 dark:text-gray-400">Last Used:</p>
                  <p className="text-gray-900 dark:text-white">
                    {token.last_used_at ? (
                      <span title={formatDate(token.last_used_at)}>
                        {getRelativeTime(token.last_used_at)}
                      </span>
                    ) : (
                      <span className="text-gray-500 dark:text-gray-500">Never</span>
                    )}
                  </p>
                </div>
              </div>
            </div>
          </div>
        ))}
      </div>

      {/* Revoke Confirmation Dialog */}
      {revokeConfirm.tokenId && (
        <div className="fixed inset-0 z-50 bg-black bg-opacity-50 flex items-center justify-center p-4">
          <div className="bg-white dark:bg-gray-800 rounded-lg shadow-lg max-w-sm w-full">
            <div className="p-6 space-y-6">
              <div>
                <h3 className="text-lg font-bold text-gray-900 dark:text-white">Revoke Token?</h3>
                <p className="text-gray-600 dark:text-gray-400 mt-2">
                  Are you sure you want to revoke the token "<span className="font-medium">{revokeConfirm.tokenName}</span>"? This action cannot be undone.
                </p>
              </div>

              <div className="flex gap-3 justify-end">
                <button
                  onClick={() => setRevokeConfirm({ tokenId: null, tokenName: null })}
                  className="px-4 py-2 text-gray-700 dark:text-gray-300 hover:bg-gray-100 dark:hover:bg-gray-700 rounded-lg transition font-medium"
                  disabled={revokeTokenMutation.isPending}
                >
                  Cancel
                </button>
                <button
                  onClick={handleConfirmRevoke}
                  disabled={revokeTokenMutation.isPending}
                  className="px-4 py-2 bg-red-600 hover:bg-red-700 dark:bg-red-700 dark:hover:bg-red-600 text-white rounded-lg disabled:bg-gray-400 dark:disabled:bg-gray-600 disabled:cursor-not-allowed transition font-medium flex items-center gap-2"
                >
                  {revokeTokenMutation.isPending && (
                    <div className="w-4 h-4 border-2 border-white border-t-transparent rounded-full animate-spin" />
                  )}
                  {revokeTokenMutation.isPending ? 'Revoking...' : 'Revoke Token'}
                </button>
              </div>
            </div>
          </div>
        </div>
      )}
    </div>
  );
};

export default TokensList;
