import React, { useState } from 'react';
import { useForm } from 'react-hook-form';
import { zodResolver } from '@hookform/resolvers/zod';
import {
  XMarkIcon,
  UserIcon,
  ShieldCheckIcon,
  EyeIcon,
  EyeSlashIcon
} from '@heroicons/react/24/outline';
import type { User, CreateUserRequest, UpdateUserRequest, UserRole } from '../../types';
import { createUserSchema, updateUserFormSchema, type CreateUserForm, type UpdateUserFormData } from '../../lib/validations';
import { LoadingSpinner } from '../ui/LoadingSpinner';

interface UserFormProps {
  user?: User;
  isOpen: boolean;
  onClose: () => void;
  onSubmit: (data: CreateUserRequest | UpdateUserRequest) => void;
  isLoading?: boolean;
  title?: string;
}

export const UserForm: React.FC<UserFormProps> = ({
  user,
  isOpen,
  onClose,
  onSubmit,
  isLoading = false,
  title,
}) => {
  const isEditing = !!user;
  const formTitle = title || (isEditing ? 'Edit User' : 'Add User');
  const [showPassword, setShowPassword] = useState(false);

  const {
    register,
    handleSubmit,
    watch,
    reset,
    formState: { errors, isValid },
  } = useForm({
    resolver: zodResolver(isEditing ? updateUserFormSchema : createUserSchema),
    mode: 'onChange', // Enable real-time validation
    defaultValues: user ? {
      username: user.username,
      email: user.email,
      role: user.role,
      active: user.active,
    } : {
      username: '',
      email: '',
      password: '',
      role: 'User',
      active: true,
    },
  });

  const watchedRole = watch('role');

  React.useEffect(() => {
    if (user) {
      const formData = {
        username: user.username,
        email: user.email,
        role: user.role,
        active: user.active,
      };
      reset(formData);
    } else {
      const formData = {
        username: '',
        email: '',
        password: '',
        role: 'User' as UserRole,
        active: true,
      };
      reset(formData);
    }
  }, [user, reset]);

  const handleFormSubmit = (data: CreateUserForm | UpdateUserFormData) => {
    // Clean up data and submit
    const cleanedData = {
      ...data,
      username: data.username?.trim(),
      email: data.email?.trim(),
    };

    // Remove empty password for edit mode
    if (isEditing && 'password' in cleanedData && !cleanedData.password?.trim()) {
      delete cleanedData.password;
    }

    onSubmit(cleanedData);
  };

  const getRoleIcon = (role: UserRole) => {
    return role === 'Admin' ? <ShieldCheckIcon className="h-5 w-5" /> : <UserIcon className="h-5 w-5" />;
  };

  if (!isOpen) return null;

  return (
    <div className="fixed inset-0 z-50 overflow-y-auto">
      <div className="flex items-center justify-center min-h-screen px-4 pt-4 pb-20 text-center sm:block sm:p-0">
        {/* Backdrop */}
        <div className="fixed inset-0 transition-opacity bg-gray-500 bg-opacity-75" onClick={onClose} />

        {/* Modal */}
        <div className="inline-block w-full max-w-md p-6 my-8 overflow-hidden text-left align-middle transition-all transform bg-white shadow-xl rounded-lg">
          {/* Header */}
          <div className="flex items-center justify-between mb-6">
            <h3 className="text-lg font-semibold text-gray-900">
              {formTitle}
            </h3>
            <button
              onClick={onClose}
              className="text-gray-400 hover:text-gray-600 transition-colors"
            >
              <XMarkIcon className="h-6 w-6" />
            </button>
          </div>

          {/* Form */}
          <form onSubmit={handleSubmit(handleFormSubmit)} className="space-y-6">
            {/* Username */}
            <div>
              <label htmlFor="username" className="block text-sm font-medium text-gray-700 mb-1">
                Username
              </label>
              <input
                {...register('username')}
                id="username"
                type="text"
                className={`input-field ${errors.username ? 'border-red-300 focus:ring-red-500 focus:border-red-500' : ''}`}
                placeholder="Enter username"
              />
              {errors.username && (
                <p className="mt-1 text-sm text-red-600">{errors.username.message}</p>
              )}
            </div>

            {/* Email */}
            <div>
              <label htmlFor="email" className="block text-sm font-medium text-gray-700 mb-1">
                Email Address
              </label>
              <input
                {...register('email')}
                id="email"
                type="email"
                className={`input-field ${errors.email ? 'border-red-300 focus:ring-red-500 focus:border-red-500' : ''}`}
                placeholder="Enter email address"
              />
              {errors.email && (
                <p className="mt-1 text-sm text-red-600">{errors.email.message}</p>
              )}
            </div>

            {/* Password */}
            <div>
              <label htmlFor="password" className="block text-sm font-medium text-gray-700 mb-1">
                Password {isEditing && <span className="text-gray-500">(leave blank to keep current)</span>}
              </label>
              <div className="relative">
                <input
                  {...register('password')}
                  id="password"
                  type={showPassword ? 'text' : 'password'}
                  className={`input-field pr-10 ${errors.password ? 'border-red-300 focus:ring-red-500 focus:border-red-500' : ''}`}
                  placeholder={isEditing ? "Enter new password (optional)" : "Enter password"}
                />
                <button
                  type="button"
                  className="absolute inset-y-0 right-0 pr-3 flex items-center"
                  onClick={() => setShowPassword(!showPassword)}
                >
                  {showPassword ? (
                    <EyeSlashIcon className="h-5 w-5 text-gray-400" />
                  ) : (
                    <EyeIcon className="h-5 w-5 text-gray-400" />
                  )}
                </button>
              </div>
              {errors.password && (
                <p className="mt-1 text-sm text-red-600">{errors.password.message}</p>
              )}
              {!isEditing && (
                <p className="mt-1 text-xs text-gray-500">
                  Password must contain at least 8 characters with uppercase, lowercase, and number
                </p>
              )}
            </div>

            {/* Role Selection */}
            <div>
              <label className="block text-sm font-medium text-gray-700 mb-3">
                Role
              </label>
              <div className="grid grid-cols-2 gap-3">
                {(['User', 'Admin'] as const).map((role) => (
                  <label
                    key={role}
                    className={`relative flex items-center justify-center p-3 border rounded-lg cursor-pointer transition-colors ${
                      watchedRole === role
                        ? 'border-blue-500 bg-blue-50 text-blue-700'
                        : 'border-gray-300 hover:border-gray-400'
                    }`}
                  >
                    <input
                      {...register('role')}
                      type="radio"
                      value={role}
                      className="sr-only"
                    />
                    <div className="flex flex-col items-center space-y-1">
                      {getRoleIcon(role)}
                      <span className="text-sm font-medium">{role}</span>
                    </div>
                  </label>
                ))}
              </div>
              {errors.role && (
                <p className="mt-1 text-sm text-red-600">{errors.role.message}</p>
              )}
            </div>

            {/* Active Status */}
            <div className="flex items-center">
              <input
                {...register('active')}
                id="active"
                type="checkbox"
                className="h-4 w-4 text-blue-600 focus:ring-blue-500 border-gray-300 rounded"
              />
              <label htmlFor="active" className="ml-2 block text-sm text-gray-900">
                Active user account
              </label>
            </div>
            <p className="text-xs text-gray-500">
              Inactive users cannot log in to the system
            </p>

            {/* Actions */}
            <div className="flex items-center justify-end space-x-3 pt-4">
              <button
                type="button"
                onClick={onClose}
                className="btn-secondary"
                disabled={isLoading}
              >
                Cancel
              </button>
              <button
                type="submit"
                disabled={!isValid || isLoading}
                className="btn-primary disabled:opacity-50 disabled:cursor-not-allowed flex items-center"
              >
                {isLoading ? (
                  <>
                    <LoadingSpinner size="sm" className="mr-2" />
                    {isEditing ? 'Updating...' : 'Creating...'}
                  </>
                ) : (
                  <>
                    {isEditing ? 'Update User' : 'Create User'}
                  </>
                )}
              </button>
            </div>
          </form>
        </div>
      </div>
    </div>
  );
};
