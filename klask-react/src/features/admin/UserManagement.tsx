import React, { useState, useCallback } from 'react';
import { toast } from 'react-hot-toast';
import { 
  PlusIcon,
  FunnelIcon,
  CheckIcon,
  XMarkIcon,
  UserIcon,
  ShieldCheckIcon,
  TrashIcon,
} from '@heroicons/react/24/outline';
import { LoadingSpinner } from '../../components/ui/LoadingSpinner';
import { UserForm } from '../../components/admin/UserForm';
import { 
  useUsers, 
  useCreateUser, 
  useUpdateUser, 
  useUpdateUserRole,
  useUpdateUserStatus,
  useDeleteUser, 
  useUserStats,
  useBulkUserOperations,
} from '../../hooks/useUsers';
import { getErrorMessage } from '../../lib/api';
import { formatDateTime } from '../../lib/utils';
import type { User, CreateUserRequest, UpdateUserRequest, UserRole } from '../../types';

type FilterType = 'all' | 'active' | 'inactive' | 'admins' | 'users';

const UserManagement: React.FC = () => {
  const [showForm, setShowForm] = useState(false);
  const [editingUser, setEditingUser] = useState<User | null>(null);
  const [selectedUsers, setSelectedUsers] = useState<string[]>([]);
  const [filter, setFilter] = useState<FilterType>('all');

  const { data: users = [], isLoading, error, refetch } = useUsers();
  const { data: stats } = useUserStats();
  const createMutation = useCreateUser();
  const updateMutation = useUpdateUser();
  const updateRoleMutation = useUpdateUserRole();
  const updateStatusMutation = useUpdateUserStatus();
  const deleteMutation = useDeleteUser();
  const { bulkActivate, bulkDeactivate, bulkSetRole, bulkDelete } = useBulkUserOperations();

  const filteredUsers = users.filter(user => {
    switch (filter) {
      case 'active':
        return user.active;
      case 'inactive':
        return !user.active;
      case 'admins':
        return user.role === 'Admin';
      case 'users':
        return user.role === 'User';
      default:
        return true;
    }
  });

  const handleCreate = useCallback(async (data: CreateUserRequest) => {
    try {
      await createMutation.mutateAsync(data);
      setShowForm(false);
      toast.success('User created successfully');
    } catch (error) {
      toast.error(getErrorMessage(error));
    }
  }, [createMutation]);

  const handleUpdate = useCallback(async (data: UpdateUserRequest) => {
    if (!editingUser) return;
    
    try {
      await updateMutation.mutateAsync({ 
        id: editingUser.id, 
        data 
      });
      setEditingUser(null);
      toast.success('User updated successfully');
    } catch (error) {
      toast.error(getErrorMessage(error));
    }
  }, [editingUser, updateMutation]);

  const handleDelete = useCallback(async (user: User) => {
    if (!confirm(`Are you sure you want to delete "${user.username}"?`)) return;
    
    try {
      await deleteMutation.mutateAsync(user.id);
      toast.success('User deleted successfully');
    } catch (error) {
      toast.error(getErrorMessage(error));
    }
  }, [deleteMutation]);

  const handleToggleRole = useCallback(async (user: User) => {
    const newRole: UserRole = user.role === 'Admin' ? 'User' : 'Admin';
    
    try {
      await updateRoleMutation.mutateAsync({ 
        id: user.id, 
        role: newRole 
      });
      toast.success(`User role updated to ${newRole}`);
    } catch (error) {
      toast.error(getErrorMessage(error));
    }
  }, [updateRoleMutation]);

  const handleToggleStatus = useCallback(async (user: User) => {
    try {
      await updateStatusMutation.mutateAsync({ 
        id: user.id, 
        active: !user.active 
      });
      toast.success(`User ${user.active ? 'deactivated' : 'activated'}`);
    } catch (error) {
      toast.error(getErrorMessage(error));
    }
  }, [updateStatusMutation]);

  const handleSelectUser = useCallback((userId: string, selected: boolean) => {
    setSelectedUsers(prev => 
      selected 
        ? [...prev, userId]
        : prev.filter(id => id !== userId)
    );
  }, []);

  const handleSelectAll = useCallback(() => {
    setSelectedUsers(selectedUsers.length === filteredUsers.length 
      ? [] 
      : filteredUsers.map(user => user.id)
    );
  }, [selectedUsers, filteredUsers]);

  const handleBulkAction = useCallback(async (action: 'activate' | 'deactivate' | 'admin' | 'user' | 'delete') => {
    if (selectedUsers.length === 0) return;

    const actionText = {
      activate: 'activate',
      deactivate: 'deactivate', 
      admin: 'set as admin',
      user: 'set as user',
      delete: 'delete'
    }[action];

    if (!confirm(`Are you sure you want to ${actionText} ${selectedUsers.length} users?`)) {
      return;
    }

    try {
      switch (action) {
        case 'activate':
          await bulkActivate.mutateAsync(selectedUsers);
          toast.success(`Activated ${selectedUsers.length} users`);
          break;
        case 'deactivate':
          await bulkDeactivate.mutateAsync(selectedUsers);
          toast.success(`Deactivated ${selectedUsers.length} users`);
          break;
        case 'admin':
          await bulkSetRole.mutateAsync({ userIds: selectedUsers, role: 'Admin' });
          toast.success(`Set ${selectedUsers.length} users as admins`);
          break;
        case 'user':
          await bulkSetRole.mutateAsync({ userIds: selectedUsers, role: 'User' });
          toast.success(`Set ${selectedUsers.length} users as regular users`);
          break;
        case 'delete':
          await bulkDelete.mutateAsync(selectedUsers);
          toast.success(`Deleted ${selectedUsers.length} users`);
          break;
      }
      setSelectedUsers([]);
    } catch (error) {
      toast.error(getErrorMessage(error));
    }
  }, [selectedUsers, bulkActivate, bulkDeactivate, bulkSetRole, bulkDelete]);

  if (isLoading) {
    return (
      <div className="max-w-7xl mx-auto">
        <div className="flex items-center justify-center min-h-96">
          <div className="text-center">
            <LoadingSpinner size="lg" className="mb-4" />
            <p className="text-gray-500">Loading users...</p>
          </div>
        </div>
      </div>
    );
  }

  if (error) {
    return (
      <div className="max-w-7xl mx-auto">
        <div className="text-center py-12">
          <XMarkIcon className="mx-auto h-16 w-16 text-red-400 mb-4" />
          <h3 className="text-lg font-medium text-gray-900 mb-2">
            Failed to Load Users
          </h3>
          <p className="text-gray-500 mb-6">
            {getErrorMessage(error)}
          </p>
          <button onClick={() => refetch()} className="btn-primary">
            Try Again
          </button>
        </div>
      </div>
    );
  }

  return (
    <div className="max-w-7xl mx-auto space-y-6">
      {/* Header */}
      <div className="md:flex md:items-center md:justify-between">
        <div className="min-w-0 flex-1">
          <h1 className="text-2xl font-bold leading-7 text-gray-900 sm:truncate sm:text-3xl sm:tracking-tight">
            User Management
          </h1>
          <p className="mt-1 text-sm text-gray-500">
            Manage user accounts, roles, and permissions.
          </p>
        </div>
        <div className="mt-4 md:mt-0">
          <button
            onClick={() => setShowForm(true)}
            className="btn-primary"
          >
            <PlusIcon className="h-4 w-4 mr-2" />
            Add User
          </button>
        </div>
      </div>

      {/* Stats */}
      {stats && (
        <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-4">
          <div className="bg-white border border-gray-200 rounded-lg p-4">
            <div className="text-2xl font-bold text-gray-900">{stats.total}</div>
            <div className="text-sm text-gray-500">Total Users</div>
          </div>
          <div className="bg-white border border-gray-200 rounded-lg p-4">
            <div className="text-2xl font-bold text-green-600">{stats.active}</div>
            <div className="text-sm text-gray-500">Active Users</div>
          </div>
          <div className="bg-white border border-gray-200 rounded-lg p-4">
            <div className="text-2xl font-bold text-blue-600">{stats.admins}</div>
            <div className="text-sm text-gray-500">Administrators</div>
          </div>
          <div className="bg-white border border-gray-200 rounded-lg p-4">
            <div className="text-2xl font-bold text-gray-600">{stats.users}</div>
            <div className="text-sm text-gray-500">Regular Users</div>
          </div>
        </div>
      )}

      {/* Filters and Bulk Actions */}
      <div className="flex flex-col sm:flex-row sm:items-center sm:justify-between gap-4">
        <div className="flex items-center space-x-2">
          <FunnelIcon className="h-5 w-5 text-gray-400" />
          <select
            value={filter}
            onChange={(e) => setFilter(e.target.value as FilterType)}
            className="text-sm border border-gray-300 rounded-md px-3 py-2 focus:ring-2 focus:ring-blue-500 focus:border-blue-500"
          >
            <option value="all">All Users</option>
            <option value="active">Active Only</option>
            <option value="inactive">Inactive Only</option>
            <option value="admins">Administrators</option>
            <option value="users">Regular Users</option>
          </select>
          
          <span className="text-sm text-gray-500">
            {filteredUsers.length} users
          </span>
        </div>

        {/* Bulk Actions */}
        {selectedUsers.length > 0 && (
          <div className="flex items-center space-x-2">
            <span className="text-sm text-gray-600">
              {selectedUsers.length} selected
            </span>
            <button
              onClick={() => handleBulkAction('activate')}
              className="text-sm px-3 py-1 bg-green-100 text-green-800 rounded hover:bg-green-200"
            >
              Activate
            </button>
            <button
              onClick={() => handleBulkAction('deactivate')}
              className="text-sm px-3 py-1 bg-gray-100 text-gray-800 rounded hover:bg-gray-200"
            >
              Deactivate
            </button>
            <button
              onClick={() => handleBulkAction('admin')}
              className="text-sm px-3 py-1 bg-blue-100 text-blue-800 rounded hover:bg-blue-200"
            >
              Make Admin
            </button>
            <button
              onClick={() => handleBulkAction('user')}
              className="text-sm px-3 py-1 bg-purple-100 text-purple-800 rounded hover:bg-purple-200"
            >
              Make User
            </button>
            <button
              onClick={() => handleBulkAction('delete')}
              className="text-sm px-3 py-1 bg-red-100 text-red-800 rounded hover:bg-red-200"
            >
              Delete
            </button>
          </div>
        )}
      </div>

      {/* Select All */}
      {filteredUsers.length > 0 && (
        <div className="flex items-center space-x-2">
          <input
            type="checkbox"
            checked={selectedUsers.length === filteredUsers.length}
            onChange={handleSelectAll}
            className="h-4 w-4 text-blue-600 focus:ring-blue-500 border-gray-300 rounded"
          />
          <label className="text-sm text-gray-700">
            Select all ({filteredUsers.length})
          </label>
        </div>
      )}

      {/* Users Table */}
      {filteredUsers.length === 0 ? (
        <div className="text-center py-12">
          <UserIcon className="mx-auto h-16 w-16 text-gray-300 mb-4" />
          <h3 className="text-lg font-medium text-gray-900 mb-2">
            No users found
          </h3>
          <p className="text-gray-500 mb-6">
            {filter === 'all' 
              ? "Get started by adding your first user."
              : `No users match the "${filter}" filter.`
            }
          </p>
          {filter === 'all' && (
            <button
              onClick={() => setShowForm(true)}
              className="btn-primary"
            >
              <PlusIcon className="h-4 w-4 mr-2" />
              Add User
            </button>
          )}
        </div>
      ) : (
        <div className="bg-white shadow rounded-lg overflow-hidden">
          <table className="min-w-full divide-y divide-gray-200">
            <thead className="bg-gray-50">
              <tr>
                <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                  User
                </th>
                <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                  Role
                </th>
                <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                  Status
                </th>
                <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                  Created
                </th>
                <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                  Last Login
                </th>
                <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                  Last Activity
                </th>
                <th className="px-6 py-3 text-right text-xs font-medium text-gray-500 uppercase tracking-wider">
                  Actions
                </th>
              </tr>
            </thead>
            <tbody className="bg-white divide-y divide-gray-200">
              {filteredUsers.map((user) => (
                <tr key={user.id} className="hover:bg-gray-50">
                  <td className="px-6 py-4 whitespace-nowrap">
                    <div className="flex items-center">
                      <input
                        type="checkbox"
                        checked={selectedUsers.includes(user.id)}
                        onChange={(e) => handleSelectUser(user.id, e.target.checked)}
                        className="mr-4 h-4 w-4 text-blue-600 focus:ring-blue-500 border-gray-300 rounded"
                      />
                      <div>
                        <div className="text-sm font-medium text-gray-900">
                          {user.username}
                        </div>
                        <div className="text-sm text-gray-500">
                          {user.email}
                        </div>
                      </div>
                    </div>
                  </td>
                  <td className="px-6 py-4 whitespace-nowrap">
                    <span className={`inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium ${
                      user.role === 'Admin' 
                        ? 'bg-blue-100 text-blue-800' 
                        : 'bg-gray-100 text-gray-800'
                    }`}>
                      {user.role === 'Admin' && <ShieldCheckIcon className="mr-1 h-3 w-3" />}
                      {user.role}
                    </span>
                  </td>
                  <td className="px-6 py-4 whitespace-nowrap">
                    <span className={`inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium ${
                      user.active 
                        ? 'bg-green-100 text-green-800' 
                        : 'bg-red-100 text-red-800'
                    }`}>
                      {user.active ? 'Active' : 'Inactive'}
                    </span>
                  </td>
                  <td className="px-6 py-4 whitespace-nowrap text-sm text-gray-500">
                    <time dateTime={user.created_at}>
                      {formatDateTime(user.created_at)}
                    </time>
                  </td>
                  <td className="px-6 py-4 whitespace-nowrap text-sm text-gray-500">
                    {user.last_login ? (
                      <time dateTime={user.last_login}>
                        {formatDateTime(user.last_login)}
                      </time>
                    ) : (
                      '-'
                    )}
                  </td>
                  <td className="px-6 py-4 whitespace-nowrap text-sm text-gray-500">
                    {user.last_activity ? (
                      <time dateTime={user.last_activity}>
                        {formatDateTime(user.last_activity)}
                      </time>
                    ) : (
                      '-'
                    )}
                  </td>
                  <td className="px-6 py-4 whitespace-nowrap text-right text-sm font-medium space-x-2">
                    <button
                      onClick={() => setEditingUser(user)}
                      className="text-blue-600 hover:text-blue-900"
                    >
                      Edit
                    </button>
                    <button
                      onClick={() => handleToggleRole(user)}
                      className="text-purple-600 hover:text-purple-900"
                    >
                      {user.role === 'Admin' ? 'Make User' : 'Make Admin'}
                    </button>
                    <button
                      onClick={() => handleToggleStatus(user)}
                      className={user.active ? "text-red-600 hover:text-red-900" : "text-green-600 hover:text-green-900"}
                    >
                      {user.active ? 'Deactivate' : 'Activate'}
                    </button>
                    <button
                      onClick={() => handleDelete(user)}
                      className="text-red-600 hover:text-red-900"
                    >
                      Delete
                    </button>
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      )}

      {/* User Form Modal */}
      <UserForm
        user={editingUser || undefined}
        isOpen={showForm || !!editingUser}
        onClose={() => {
          setShowForm(false);
          setEditingUser(null);
        }}
        onSubmit={(editingUser ? handleUpdate : handleCreate) as any}
        isLoading={createMutation.isPending || updateMutation.isPending}
      />
    </div>
  );
};

export default UserManagement;