import { useMutation, useQuery } from '@tanstack/react-query';
import toast from 'react-hot-toast';
import { useNavigate } from 'react-router-dom';
import type { User, UpdateProfileRequest, ChangePasswordRequest } from '../types';
import { api } from '../lib/api';
import { useAuthStore } from '../stores/auth-store';
import { queryClient } from '../lib/react-query';

/**
 * Hook for managing user profile
 */
export function useProfile() {
  const user = useAuthStore((state) => state.user);
  const setUser = useAuthStore((state) => state.setUser);

  const updateProfileMutation = useMutation({
    mutationFn: (data: UpdateProfileRequest) => api.updateProfile(data),
    onSuccess: (updatedUser: User) => {
      setUser(updatedUser);
      queryClient.setQueryData(['auth', 'profile'], updatedUser);
      toast.success('Profile updated successfully');
    },
    onError: (error: unknown) => {
      const message = error && typeof error === 'object' && 'message' in error
        ? (error as { message: string }).message
        : 'Failed to update profile';
      toast.error(message);
    },
  });

  return {
    user,
    updateProfile: updateProfileMutation.mutate,
    isUpdating: updateProfileMutation.isPending,
    error: updateProfileMutation.error,
  };
}

/**
 * Hook for changing password
 */
export function useChangePassword() {
  return useMutation({
    mutationFn: (data: ChangePasswordRequest) => api.changePassword(data),
    onSuccess: () => {
      toast.success('Password changed successfully');
    },
    onError: (error: unknown) => {
      const message = error && typeof error === 'object' && 'message' in error
        ? (error as { message: string }).message
        : 'Failed to change password';
      toast.error(message);
    },
  });
}

/**
 * Hook for uploading avatar
 */
export function useUploadAvatar() {
  const setUser = useAuthStore((state) => state.setUser);
  const user = useAuthStore((state) => state.user);

  return useMutation({
    mutationFn: (file: globalThis.File) => api.uploadAvatar(file),
    onSuccess: (response: { avatar_url: string }) => {
      if (user) {
        const updatedUser = { ...user, avatar_url: response.avatar_url };
        setUser(updatedUser);
        queryClient.setQueryData(['auth', 'profile'], updatedUser);
      }
      toast.success('Avatar uploaded successfully');
    },
    onError: (error: unknown) => {
      const message = error && typeof error === 'object' && 'message' in error
        ? (error as { message: string }).message
        : 'Failed to upload avatar';
      toast.error(message);
    },
  });
}

/**
 * Hook for fetching user activity with pagination support
 */
export function useUserActivity(page: number = 1, limit: number = 20) {
  return useQuery({
    queryKey: ['activity', page, limit],
    queryFn: async () => {
      const activity = await api.getUserActivity();
      // Basic pagination at client level
      // In production, this should be handled by the backend API
      if (Array.isArray(activity?.devices)) {
        const start = (page - 1) * limit;
        const end = start + limit;
        return {
          ...activity,
          devices: activity.devices.slice(start, end),
          total_devices: activity.devices.length,
          current_page: page,
          page_size: limit,
        };
      }
      return activity;
    },
    staleTime: 5 * 60 * 1000, // 5 minutes
    retry: 2,
  });
}

/**
 * Hook for deleting account
 */
export function useDeleteAccount() {
  const navigate = useNavigate();
  const logout = useAuthStore((state) => state.logout);

  return useMutation({
    mutationFn: (password: string) => api.deleteAccount(password),
    onSuccess: () => {
      logout();
      queryClient.clear();
      navigate('/', { replace: true });
      toast.success('Account deleted successfully');
    },
    onError: (error: unknown) => {
      const message = error && typeof error === 'object' && 'message' in error
        ? (error as { message: string }).message
        : 'Failed to delete account';
      toast.error(message);
    },
  });
}
