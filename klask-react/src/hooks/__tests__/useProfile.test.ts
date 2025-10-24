import { describe, it, expect, vi, beforeEach } from 'vitest';
import { renderHook, waitFor } from '@testing-library/react';
import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import React from 'react';
import { useProfile, useChangePassword, useUploadAvatar, useUserActivity } from '../useProfile';
import { api } from '../../lib/api';
import { useAuthStore } from '../../stores/auth-store';

// Mock the API
vi.mock('../../lib/api');

// Mock react-hot-toast
vi.mock('react-hot-toast', () => ({
  default: {
    success: vi.fn(),
    error: vi.fn(),
  },
}));

// Mock react-router-dom
vi.mock('react-router-dom', () => ({
  useNavigate: () => vi.fn(),
}));

const createWrapper = () => {
  const queryClient = new QueryClient({
    defaultOptions: {
      queries: { retry: false },
      mutations: { retry: false },
    },
  });

  return ({ children }: { children: React.ReactNode }) =>
    React.createElement(QueryClientProvider, { client: queryClient }, children);
};

describe('useProfile', () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it('should return user from store', () => {
    const mockUser = {
      id: '1',
      username: 'testuser',
      email: 'test@example.com',
      role: 'User' as const,
      active: true,
      created_at: '2024-01-01',
      updated_at: '2024-01-01',
    };

    useAuthStore.setState({ user: mockUser });

    const { result } = renderHook(() => useProfile(), {
      wrapper: createWrapper(),
    });

    expect(result.current.user).toEqual(mockUser);
  });

  it('should call updateProfile mutation', async () => {
    const mockUser = {
      id: '1',
      username: 'testuser',
      email: 'test@example.com',
      role: 'User' as const,
      active: true,
      created_at: '2024-01-01',
      updated_at: '2024-01-01',
    };

    useAuthStore.setState({ user: mockUser });

    vi.mocked(api.updateProfile).mockResolvedValue({
      ...mockUser,
      full_name: 'Test User',
    });

    const { result } = renderHook(() => useProfile(), {
      wrapper: createWrapper(),
    });

    result.current.updateProfile({ full_name: 'Test User' });

    await waitFor(() => {
      expect(api.updateProfile).toHaveBeenCalledWith({ full_name: 'Test User' });
    });
  });
});

describe('useChangePassword', () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it('should call changePassword mutation', async () => {
    vi.mocked(api.changePassword).mockResolvedValue({ message: 'Password changed' });

    const { result } = renderHook(() => useChangePassword(), {
      wrapper: createWrapper(),
    });

    result.current.mutate({
      current_password: 'old',
      new_password: 'new',
      new_password_confirm: 'new',
    });

    await waitFor(() => {
      expect(api.changePassword).toHaveBeenCalled();
    });
  });
});

describe('useUploadAvatar', () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it('should call uploadAvatar mutation', async () => {
    const mockUser = {
      id: '1',
      username: 'testuser',
      email: 'test@example.com',
      role: 'User' as const,
      active: true,
      created_at: '2024-01-01',
      updated_at: '2024-01-01',
    };

    useAuthStore.setState({ user: mockUser });

    vi.mocked(api.uploadAvatar).mockResolvedValue({
      avatar_url: 'https://example.com/avatar.jpg',
    });

    const { result } = renderHook(() => useUploadAvatar(), {
      wrapper: createWrapper(),
    });

    const file = new File([''], 'avatar.jpg', { type: 'image/jpeg' });
    result.current.mutate(file);

    await waitFor(() => {
      expect(api.uploadAvatar).toHaveBeenCalled();
    });
  });
});

describe('useUserActivity', () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it('should fetch user activity', async () => {
    const mockActivity = {
      login_count: 5,
      created_at: '2024-01-01',
      devices: [],
    };

    vi.mocked(api.getUserActivity).mockResolvedValue(mockActivity);

    const { result } = renderHook(() => useUserActivity(), {
      wrapper: createWrapper(),
    });

    await waitFor(() => {
      expect(result.current.data).toEqual(mockActivity);
    });
  });
});
