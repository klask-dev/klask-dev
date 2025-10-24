import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen, fireEvent, waitFor } from '@testing-library/react';
import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import React from 'react';
import ProfilePage from '../ProfilePage';
import * as hooks from '../../../hooks/useProfile';

// Mock the hooks
vi.mock('../../../hooks/useProfile');

// Mock components
vi.mock('../components/ProfileHeader', () => ({
  default: () => <div data-testid="profile-header">Profile Header</div>,
}));

vi.mock('../components/ProfileInformation', () => ({
  default: () => <div data-testid="profile-information">Profile Information</div>,
}));

vi.mock('../components/PreferencesSection', () => ({
  default: () => <div data-testid="preferences-section">Preferences Section</div>,
}));

vi.mock('../components/SecuritySection', () => ({
  default: () => <div data-testid="security-section">Security Section</div>,
}));

vi.mock('../components/ActivitySection', () => ({
  default: () => <div data-testid="activity-section">Activity Section</div>,
}));

vi.mock('../components/DeleteAccountModal', () => ({
  default: ({ isOpen }: { isOpen: boolean }) => (
    isOpen ? <div data-testid="delete-modal">Delete Modal</div> : null
  ),
}));

const createWrapper = () => {
  const queryClient = new QueryClient({
    defaultOptions: {
      queries: { retry: false },
    },
  });

  return ({ children }: { children: React.ReactNode }) =>
    React.createElement(QueryClientProvider, { client: queryClient }, children);
};

describe('ProfilePage', () => {
  const mockUser = {
    id: '1',
    username: 'testuser',
    email: 'test@example.com',
    role: 'User' as const,
    active: true,
    created_at: '2024-01-01',
    updated_at: '2024-01-01',
    full_name: 'Test User',
  };

  beforeEach(() => {
    vi.clearAllMocks();

    // Mock useProfile hook
    vi.mocked(hooks.useProfile).mockReturnValue({
      user: mockUser,
      updateProfile: vi.fn(),
      isUpdating: false,
      error: null,
    });
  });

  it('should render profile page with header', () => {
    render(<ProfilePage />, { wrapper: createWrapper() });

    expect(screen.getByText('Profile Settings')).toBeInTheDocument();
    expect(screen.getByText('Manage your account information and preferences')).toBeInTheDocument();
  });

  it('should render all tabs', () => {
    render(<ProfilePage />, { wrapper: createWrapper() });

    expect(screen.getByText('Information')).toBeInTheDocument();
    expect(screen.getByText('Preferences')).toBeInTheDocument();
    expect(screen.getByText('Security')).toBeInTheDocument();
    expect(screen.getByText('Activity')).toBeInTheDocument();
  });

  it('should render profile header', () => {
    render(<ProfilePage />, { wrapper: createWrapper() });

    expect(screen.getByTestId('profile-header')).toBeInTheDocument();
  });

  it('should render information tab by default', () => {
    render(<ProfilePage />, { wrapper: createWrapper() });

    expect(screen.getByTestId('profile-information')).toBeInTheDocument();
  });

  it('should switch to preferences tab', async () => {
    render(<ProfilePage />, { wrapper: createWrapper() });

    const preferencesButton = screen.getByText('Preferences');
    fireEvent.click(preferencesButton);

    await waitFor(() => {
      expect(screen.getByTestId('preferences-section')).toBeInTheDocument();
    });
  });

  it('should switch to security tab', async () => {
    render(<ProfilePage />, { wrapper: createWrapper() });

    const securityButton = screen.getByText('Security');
    fireEvent.click(securityButton);

    await waitFor(() => {
      expect(screen.getByTestId('security-section')).toBeInTheDocument();
    });
  });

  it('should switch to activity tab', async () => {
    render(<ProfilePage />, { wrapper: createWrapper() });

    const activityButton = screen.getByText('Activity');
    fireEvent.click(activityButton);

    await waitFor(() => {
      expect(screen.getByTestId('activity-section')).toBeInTheDocument();
    });
  });

  it('should render danger zone', () => {
    render(<ProfilePage />, { wrapper: createWrapper() });

    expect(screen.getByText('Danger Zone')).toBeInTheDocument();
    expect(screen.getByText('Delete Account')).toBeInTheDocument();
  });

  it('should open delete modal when delete button clicked', async () => {
    render(<ProfilePage />, { wrapper: createWrapper() });

    const deleteButton = screen.getByText('Delete Account');
    fireEvent.click(deleteButton);

    await waitFor(() => {
      expect(screen.getByTestId('delete-modal')).toBeInTheDocument();
    });
  });

  it('should show message when not logged in', () => {
    vi.mocked(hooks.useProfile).mockReturnValue({
      user: null,
      updateProfile: vi.fn(),
      isUpdating: false,
      error: null,
    });

    render(<ProfilePage />, { wrapper: createWrapper() });

    expect(screen.getByText('Please log in to view your profile')).toBeInTheDocument();
  });
});
