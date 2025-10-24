import React, { useState } from 'react';
import { useDeleteAccount } from '../../../hooks/useProfile';

interface DeleteAccountModalProps {
  isOpen: boolean;
  onClose: () => void;
}

const DeleteAccountModal: React.FC<DeleteAccountModalProps> = ({ isOpen, onClose }) => {
  const deleteAccountMutation = useDeleteAccount();
  const [password, setPassword] = useState('');
  const [confirmed, setConfirmed] = useState(false);
  const [error, setError] = useState('');

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    setError('');

    if (!password) {
      setError('Please enter your password to confirm');
      return;
    }

    if (!confirmed) {
      setError('Please confirm you understand the consequences');
      return;
    }

    deleteAccountMutation.mutate(password, {
      onError: (err: unknown) => {
        const errorMessage = err && typeof err === 'object' && 'message' in err
          ? (err as { message: string }).message
          : 'Failed to delete account';
        setError(errorMessage);
      },
    });
  };

  const handleClose = () => {
    setPassword('');
    setConfirmed(false);
    setError('');
    onClose();
  };

  if (!isOpen) return null;

  return (
    <div className="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50 p-4">
      <div className="bg-white rounded-lg shadow-xl max-w-md w-full">
        {/* Header */}
        <div className="bg-red-50 border-b border-red-200 px-6 py-4">
          <h2 className="text-lg font-bold text-red-900">Delete Account</h2>
          <p className="text-sm text-red-700 mt-1">This action cannot be undone</p>
        </div>

        {/* Content */}
        <div className="px-6 py-4">
          <div className="mb-4 p-4 bg-red-50 border border-red-200 rounded-lg">
            <p className="text-sm text-red-900 font-medium mb-2">Warning:</p>
            <ul className="text-xs text-red-800 space-y-1">
              <li>- Your account will be permanently deleted</li>
              <li>- All your data will be removed</li>
              <li>- This cannot be reversed</li>
            </ul>
          </div>

          <form onSubmit={handleSubmit} className="space-y-4">
            {/* Password Confirmation */}
            <div>
              <label htmlFor="delete_password" className="block text-sm font-medium text-gray-700 mb-1">
                Enter your password
              </label>
              <input
                id="delete_password"
                type="password"
                value={password}
                onChange={(e) => {
                  setPassword(e.target.value);
                  setError('');
                }}
                placeholder="••••••••"
                className="w-full px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-red-500 focus:border-transparent outline-none transition"
                disabled={deleteAccountMutation.isPending}
              />
            </div>

            {/* Confirmation Checkbox */}
            <label className="flex items-start gap-3 cursor-pointer">
              <input
                type="checkbox"
                checked={confirmed}
                onChange={(e) => {
                  setConfirmed(e.target.checked);
                  setError('');
                }}
                disabled={deleteAccountMutation.isPending}
                className="w-4 h-4 mt-0.5 rounded border-gray-300 text-red-600 focus:ring-2 focus:ring-red-500 cursor-pointer"
              />
              <span className="text-sm text-gray-700">
                I understand this action is permanent and will delete my account and all associated data
              </span>
            </label>

            {/* Error Message */}
            {error && (
              <div className="p-3 bg-red-50 border border-red-200 rounded-lg">
                <p className="text-sm text-red-700">{error}</p>
              </div>
            )}

            {/* Buttons */}
            <div className="flex gap-3 pt-4">
              <button
                type="button"
                onClick={handleClose}
                disabled={deleteAccountMutation.isPending}
                className="flex-1 px-4 py-2 border border-gray-300 text-gray-700 rounded-lg hover:bg-gray-50 disabled:bg-gray-100 disabled:cursor-not-allowed transition"
              >
                Cancel
              </button>
              <button
                type="submit"
                disabled={
                  deleteAccountMutation.isPending ||
                  !password ||
                  !confirmed
                }
                className="flex-1 px-4 py-2 bg-red-600 text-white rounded-lg hover:bg-red-700 disabled:bg-gray-400 disabled:cursor-not-allowed transition flex items-center justify-center gap-2"
              >
                {deleteAccountMutation.isPending && (
                  <div className="w-4 h-4 border-2 border-white border-t-transparent rounded-full animate-spin" />
                )}
                {deleteAccountMutation.isPending ? 'Deleting...' : 'Delete Account'}
              </button>
            </div>
          </form>
        </div>
      </div>
    </div>
  );
};

export default DeleteAccountModal;
