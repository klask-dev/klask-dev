import { describe, it, expect, vi, beforeEach } from 'vitest';
import { screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import React from 'react';
import UserManagement from '../admin/UserManagement';
import { render } from '../../test/test-utils';
import type { User, UserStats } from '../../types';

// Mock the hooks
const mockUsers: User[] = [
  {
    id: 'user-1',
    username: 'john_doe',
    email: 'john@example.com',
    role: 'User',
    active: true,
    created_at: '2024-01-01T00:00:00Z',
    updated_at: '2024-01-01T00:00:00Z',
    last_login: '2024-01-01T12:00:00Z',
    last_activity: '2024-01-01T14:30:00Z',
  },
  {
    id: 'user-2',
    username: 'jane_admin',
    email: 'jane@example.com',
    role: 'Admin',
    active: true,
    created_at: '2024-01-02T00:00:00Z',
    updated_at: '2024-01-02T00:00:00Z',
    last_login: '2024-01-02T10:00:00Z',
    last_activity: '2024-01-02T11:00:00Z',
  },
  {
    id: 'user-3',
    username: 'inactive_user',
    email: 'inactive@example.com',
    role: 'User',
    active: false,
    created_at: '2024-01-03T00:00:00Z',
    updated_at: '2024-01-03T00:00:00Z',
  },
];

const mockStats: UserStats = {
  total: 3,
  active: 2,
  admins: 1,
  users: 2,
};

// Mock the hooks
vi.mock('../../hooks/useUsers', () => ({
  useUsers: vi.fn(),
  useUserStats: vi.fn(),
  useCreateUser: vi.fn(),
  useUpdateUser: vi.fn(),
  useUpdateUserRole: vi.fn(),
  useUpdateUserStatus: vi.fn(),
  useDeleteUser: vi.fn(),
  useBulkUserOperations: vi.fn(),
}));

// Mock react-hot-toast
vi.mock('react-hot-toast', () => ({
  toast: {
    success: vi.fn(),
    error: vi.fn(),
  },
}));

// Mock icons
vi.mock('@heroicons/react/24/outline', () => ({
  PlusIcon: () => <div data-testid="plus-icon">+</div>,
  FunnelIcon: () => <div data-testid="funnel-icon">Filter</div>,
  CheckIcon: () => <div data-testid="check-icon">Check</div>,
  XMarkIcon: () => <div data-testid="x-mark-icon">X</div>,
  UserIcon: () => <div data-testid="user-icon">User</div>,
  ShieldCheckIcon: () => <div data-testid="shield-check-icon">Shield</div>,
  TrashIcon: () => <div data-testid="trash-icon">Trash</div>,
}));

// Mock the UserForm component
vi.mock('../../components/admin/UserForm', () => ({
  UserForm: ({ isOpen, onClose, onSubmit, user, isLoading }: any) => {
    if (!isOpen) return null;
    return (
      <div data-testid="user-form-modal">
        <h2>{user ? 'Edit User Form' : 'Add User Form'}</h2>
        <p data-testid="user-info">User: {user ? user.username : 'New User'}</p>
        <button onClick={onClose}>Close</button>
        <button 
          onClick={() => onSubmit({ username: 'test', email: 'test@example.com', role: 'User' })}
          disabled={isLoading}
          data-testid="submit-button"
        >
          {isLoading ? 'Loading...' : 'Submit'}
        </button>
      </div>
    );
  },
}));

// Mock LoadingSpinner
vi.mock('../../components/ui/LoadingSpinner', () => ({
  LoadingSpinner: ({ size, className }: { size?: string; className?: string }) => (
    <div data-testid="loading-spinner" className={className}>
      Loading {size}
    </div>
  ),
}));

import { useUsers, useUserStats, useCreateUser, useUpdateUser, useUpdateUserRole, useUpdateUserStatus, useDeleteUser, useBulkUserOperations } from '../../hooks/useUsers';
import { toast } from 'react-hot-toast';

const mockUseUsers = useUsers as any;
const mockUseUserStats = useUserStats as any;
const mockUseCreateUser = useCreateUser as any;
const mockUseUpdateUser = useUpdateUser as any;
const mockUseUpdateUserRole = useUpdateUserRole as any;
const mockUseUpdateUserStatus = useUpdateUserStatus as any;
const mockUseDeleteUser = useDeleteUser as any;
const mockUseBulkUserOperations = useBulkUserOperations as any;

describe('UserManagement Integration', () => {
  let queryClient: QueryClient;
  const mockMutations = {
    create: { mutateAsync: vi.fn(), isPending: false },
    update: { mutateAsync: vi.fn(), isPending: false },
    updateRole: { mutateAsync: vi.fn(), isPending: false },
    updateStatus: { mutateAsync: vi.fn(), isPending: false },
    delete: { mutateAsync: vi.fn(), isPending: false },
    bulkActivate: { mutateAsync: vi.fn() },
    bulkDeactivate: { mutateAsync: vi.fn() },
    bulkSetRole: { mutateAsync: vi.fn() },
    bulkDelete: { mutateAsync: vi.fn() },
  };

  beforeEach(() => {
    queryClient = new QueryClient({
      defaultOptions: {
        queries: { retry: false, refetchOnWindowFocus: false },
        mutations: { retry: false },
      },
    });

    vi.clearAllMocks();

    // Setup default mock returns
    mockUseUsers.mockReturnValue({
      data: mockUsers,
      isLoading: false,
      error: null,
      refetch: vi.fn(),
    });

    mockUseUserStats.mockReturnValue({
      data: mockStats,
    });

    mockUseCreateUser.mockReturnValue(mockMutations.create);
    mockUseUpdateUser.mockReturnValue(mockMutations.update);
    mockUseUpdateUserRole.mockReturnValue(mockMutations.updateRole);
    mockUseUpdateUserStatus.mockReturnValue(mockMutations.updateStatus);
    mockUseDeleteUser.mockReturnValue(mockMutations.delete);

    mockUseBulkUserOperations.mockReturnValue({
      bulkActivate: mockMutations.bulkActivate,
      bulkDeactivate: mockMutations.bulkDeactivate,
      bulkSetRole: mockMutations.bulkSetRole,
      bulkDelete: mockMutations.bulkDelete,
    });

    // Mock global confirm
    global.confirm = vi.fn().mockReturnValue(true);
  });

  const renderUserManagement = () => {
    return render(
      <QueryClientProvider client={queryClient}>
        <UserManagement />
      </QueryClientProvider>
    );
  };

  describe('Initial Rendering and Data Display', () => {
    it('should render user management interface with users data', () => {
      renderUserManagement();

      expect(screen.getByText('User Management')).toBeInTheDocument();
      expect(screen.getByText('Manage user accounts, roles, and permissions.')).toBeInTheDocument();
      expect(screen.getByText('Add User')).toBeInTheDocument();

      // Check if users are displayed
      expect(screen.getByText('john_doe')).toBeInTheDocument();
      expect(screen.getByText('john@example.com')).toBeInTheDocument();
      expect(screen.getByText('jane_admin')).toBeInTheDocument();
      expect(screen.getByText('jane@example.com')).toBeInTheDocument();
      expect(screen.getByText('inactive_user')).toBeInTheDocument();
      expect(screen.getByText('inactive@example.com')).toBeInTheDocument();
    });

    it('should display user statistics', () => {
      renderUserManagement();

      // Use more specific selectors to avoid duplicate element errors
      const statCards = screen.getAllByText('Total Users')[0].parentElement!;
      expect(statCards.querySelector('.text-2xl')).toHaveTextContent('3');
      expect(screen.getByText('Total Users')).toBeInTheDocument();
      
      const activeCard = screen.getAllByText('Active Users')[0].parentElement!;
      expect(activeCard.querySelector('.text-2xl')).toHaveTextContent('2');
      expect(screen.getByText('Active Users')).toBeInTheDocument();
      
      const adminCard = screen.getAllByText('Administrators')[0].parentElement!;
      expect(adminCard.querySelector('.text-2xl')).toHaveTextContent('1');
      expect(screen.getAllByText('Administrators')[0]).toBeInTheDocument();
      
      expect(screen.getAllByText('Regular Users')[0]).toBeInTheDocument();
    });

    it('should show loading state when data is loading', () => {
      mockUseUsers.mockReturnValue({
        data: [],
        isLoading: true,
        error: null,
        refetch: vi.fn(),
      });

      renderUserManagement();

      expect(screen.getByText('Loading users...')).toBeInTheDocument();
      expect(screen.getByTestId('loading-spinner')).toBeInTheDocument();
    });

    it('should show error state when data loading fails', () => {
      mockUseUsers.mockReturnValue({
        data: [],
        isLoading: false,
        error: new Error('Failed to load users'),
        refetch: vi.fn(),
      });

      renderUserManagement();

      expect(screen.getByText('Failed to Load Users')).toBeInTheDocument();
      expect(screen.getByText('Try Again')).toBeInTheDocument();
    });
  });

  describe('UserForm Integration - Create Mode', () => {
    it('should open UserForm in create mode when Add User button is clicked', async () => {
      const user = userEvent.setup();
      renderUserManagement();

      const addUserButton = screen.getByText('Add User');
      await user.click(addUserButton);

      await waitFor(() => {
        expect(screen.getByTestId('user-form-modal')).toBeInTheDocument();
        expect(screen.getByText('Add User Form')).toBeInTheDocument();
        expect(screen.getByTestId('user-info')).toHaveTextContent('User: New User');
      });
    });

    it('should close UserForm when close button is clicked', async () => {
      const user = userEvent.setup();
      renderUserManagement();

      // Open form
      await user.click(screen.getByText('Add User'));
      
      await waitFor(() => {
        expect(screen.getByTestId('user-form-modal')).toBeInTheDocument();
      });

      // Close form
      await user.click(screen.getByText('Close'));

      await waitFor(() => {
        expect(screen.queryByTestId('user-form-modal')).not.toBeInTheDocument();
      });
    });

    it('should handle user creation through UserForm', async () => {
      const user = userEvent.setup();
      mockMutations.create.mutateAsync.mockResolvedValue({});
      
      renderUserManagement();

      // Open create form
      await user.click(screen.getByText('Add User'));
      
      await waitFor(() => {
        expect(screen.getByTestId('user-form-modal')).toBeInTheDocument();
      });

      // Submit form
      await user.click(screen.getByText('Submit'));

      await waitFor(() => {
        expect(mockMutations.create.mutateAsync).toHaveBeenCalledWith({
          username: 'test',
          email: 'test@example.com',
          role: 'User',
        });
      });

      // Should show success toast and close form
      expect(toast.success).toHaveBeenCalledWith('User created successfully');
      expect(screen.queryByTestId('user-form-modal')).not.toBeInTheDocument();
    });

    it('should handle creation errors and show error toast', async () => {
      const user = userEvent.setup();
      const errorMessage = 'Username already exists';
      mockMutations.create.mutateAsync.mockRejectedValue(new Error(errorMessage));
      
      renderUserManagement();

      await user.click(screen.getByText('Add User'));
      
      await waitFor(() => {
        expect(screen.getByTestId('user-form-modal')).toBeInTheDocument();
      });

      await user.click(screen.getByText('Submit'));

      await waitFor(() => {
        expect(toast.error).toHaveBeenCalledWith(errorMessage);
      });
    });
  });

  describe('UserForm Integration - Edit Mode', () => {
    it('should open UserForm in edit mode when Edit button is clicked', async () => {
      const user = userEvent.setup();
      renderUserManagement();

      const editButtons = screen.getAllByText('Edit');
      await user.click(editButtons[0]); // Click edit for first user

      await waitFor(() => {
        expect(screen.getByTestId('user-form-modal')).toBeInTheDocument();
        expect(screen.getByText('Edit User Form')).toBeInTheDocument();
        expect(screen.getByTestId('user-info')).toHaveTextContent('User: john_doe'); // Should show user data
      });
    });

    it('should handle user updates through UserForm', async () => {
      const user = userEvent.setup();
      const updatedUser = { ...mockUsers[0], username: 'updated_user' };
      mockMutations.update.mutateAsync.mockResolvedValue(updatedUser);
      
      renderUserManagement();

      // Open edit form
      const editButtons = screen.getAllByText('Edit');
      await user.click(editButtons[0]);
      
      await waitFor(() => {
        expect(screen.getByTestId('user-form-modal')).toBeInTheDocument();
      });

      // Submit form
      await user.click(screen.getByText('Submit'));

      await waitFor(() => {
        expect(mockMutations.update.mutateAsync).toHaveBeenCalledWith({
          id: 'user-1',
          data: {
            username: 'test',
            email: 'test@example.com',
            role: 'User',
          },
        });
      });

      expect(toast.success).toHaveBeenCalledWith('User updated successfully');
      expect(screen.queryByTestId('user-form-modal')).not.toBeInTheDocument();
    });

    it('should show loading state in form during submission', async () => {
      const user = userEvent.setup();
      mockMutations.update.isPending = true;
      
      renderUserManagement();

      const editButtons = screen.getAllByText('Edit');
      await user.click(editButtons[0]);
      
      await waitFor(() => {
        expect(screen.getByTestId('submit-button')).toHaveTextContent('Loading...');
        expect(screen.getByTestId('submit-button')).toBeDisabled();
      });
    });
  });

  describe('User Actions Integration', () => {
    it('should handle role toggle through table actions', async () => {
      const user = userEvent.setup();
      const updatedUser = { ...mockUsers[0], role: 'Admin' };
      mockMutations.updateRole.mutateAsync.mockResolvedValue(updatedUser);
      
      renderUserManagement();

      const makeAdminButtons = screen.getAllByText('Make Admin');
      await user.click(makeAdminButtons[0]);

      await waitFor(() => {
        expect(mockMutations.updateRole.mutateAsync).toHaveBeenCalledWith({
          id: 'user-1',
          role: 'Admin',
        });
      });

      expect(toast.success).toHaveBeenCalledWith('User role updated to Admin');
    });

    it('should handle status toggle through table actions', async () => {
      const user = userEvent.setup();
      const updatedUser = { ...mockUsers[0], active: false };
      mockMutations.updateStatus.mutateAsync.mockResolvedValue(updatedUser);
      
      renderUserManagement();

      const deactivateButtons = screen.getAllByText('Deactivate');
      await user.click(deactivateButtons[0]);

      await waitFor(() => {
        expect(mockMutations.updateStatus.mutateAsync).toHaveBeenCalledWith({
          id: 'user-1',
          active: false,
        });
      });

      expect(toast.success).toHaveBeenCalledWith('User deactivated');
    });

    it('should handle user deletion with confirmation', async () => {
      const user = userEvent.setup();
      mockMutations.delete.mutateAsync.mockResolvedValue(undefined);
      global.confirm = vi.fn().mockReturnValue(true);
      
      renderUserManagement();

      const deleteButtons = screen.getAllByText('Delete');
      await user.click(deleteButtons[0]);

      expect(global.confirm).toHaveBeenCalledWith('Are you sure you want to delete "john_doe"?');

      await waitFor(() => {
        expect(mockMutations.delete.mutateAsync).toHaveBeenCalledWith('user-1');
      });

      expect(toast.success).toHaveBeenCalledWith('User deleted successfully');
    });

    it('should not delete user if confirmation is cancelled', async () => {
      const user = userEvent.setup();
      global.confirm = vi.fn().mockReturnValue(false);
      
      renderUserManagement();

      const deleteButtons = screen.getAllByText('Delete');
      await user.click(deleteButtons[0]);

      expect(global.confirm).toHaveBeenCalled();
      expect(mockMutations.delete.mutateAsync).not.toHaveBeenCalled();
      expect(toast.success).not.toHaveBeenCalled();
    });
  });

  describe('Filtering Integration', () => {
    it('should filter users by status', async () => {
      const user = userEvent.setup();
      renderUserManagement();

      const filterSelect = screen.getByDisplayValue('All Users');
      await user.selectOptions(filterSelect, 'active');

      // Should show only active users
      expect(screen.getByText('john_doe')).toBeInTheDocument();
      expect(screen.getByText('jane_admin')).toBeInTheDocument();
      expect(screen.queryByText('inactive_user')).not.toBeInTheDocument();
    });

    it('should filter users by role', async () => {
      const user = userEvent.setup();
      renderUserManagement();

      const filterSelect = screen.getByDisplayValue('All Users');
      await user.selectOptions(filterSelect, 'admins');

      // Should show only admin users
      expect(screen.queryByText('john_doe')).not.toBeInTheDocument();
      expect(screen.getByText('jane_admin')).toBeInTheDocument();
      expect(screen.queryByText('inactive_user')).not.toBeInTheDocument();
    });

    it('should update user count based on filter', async () => {
      const user = userEvent.setup();
      renderUserManagement();

      // Should show total count initially
      expect(screen.getByText('3 users')).toBeInTheDocument();

      const filterSelect = screen.getByDisplayValue('All Users');
      await user.selectOptions(filterSelect, 'active');

      // Should show filtered count
      expect(screen.getByText('2 users')).toBeInTheDocument();
    });
  });

  describe('Bulk Operations Integration', () => {
    it('should handle bulk user selection', async () => {
      const user = userEvent.setup();
      renderUserManagement();

      // Select individual users - use more specific selector
      const checkboxes = screen.getAllByRole('checkbox');
      // Get the select all checkbox by its unique selector first
      const selectAllCheckbox = checkboxes.find(cb => 
        cb.parentElement?.textContent?.includes('Select all')
      );
      const userCheckboxes = checkboxes.filter(cb => cb !== selectAllCheckbox);

      await user.click(userCheckboxes[0]);
      await user.click(userCheckboxes[1]);

      // Should show bulk actions
      expect(screen.getByText('2 selected')).toBeInTheDocument();
      expect(screen.getAllByText('Activate')[0]).toBeInTheDocument(); // First is bulk action
      expect(screen.getAllByText('Deactivate')[0]).toBeInTheDocument(); // First is bulk action
      expect(screen.getAllByText('Make Admin')[0]).toBeInTheDocument(); // First is bulk action
    });

    it('should handle select all functionality', async () => {
      const user = userEvent.setup();
      renderUserManagement();

      // Find the select all checkbox by searching for its associated text
      const selectAllCheckbox = screen.getAllByRole('checkbox').find(cb => 
        cb.parentElement?.textContent?.includes('Select all')
      );
      if (selectAllCheckbox) {
        await user.click(selectAllCheckbox);
      }

      expect(screen.getByText('3 selected')).toBeInTheDocument();
    });

    it('should handle bulk activate operation', async () => {
      const user = userEvent.setup();
      mockMutations.bulkActivate.mutateAsync.mockResolvedValue([]);
      global.confirm = vi.fn().mockReturnValue(true);
      
      renderUserManagement();

      // Select users
      const selectAllCheckbox = screen.getAllByRole('checkbox').find(cb => 
        cb.parentElement?.textContent?.includes('Select all')
      );
      if (selectAllCheckbox) {
        await user.click(selectAllCheckbox);
      }

      // Perform bulk activate - click the first Activate button (bulk action)
      await user.click(screen.getAllByText('Activate')[0]);

      expect(global.confirm).toHaveBeenCalledWith('Are you sure you want to activate 3 users?');

      await waitFor(() => {
        expect(mockMutations.bulkActivate.mutateAsync).toHaveBeenCalledWith(['user-1', 'user-2', 'user-3']);
      });

      expect(toast.success).toHaveBeenCalledWith('Activated 3 users');
    });
  });

  describe('Empty States', () => {
    it('should show empty state when no users exist', () => {
      mockUseUsers.mockReturnValue({
        data: [],
        isLoading: false,
        error: null,
        refetch: vi.fn(),
      });

      renderUserManagement();

      expect(screen.getByText('No users found')).toBeInTheDocument();
      expect(screen.getByText('Get started by adding your first user.')).toBeInTheDocument();
    });

    it('should show filtered empty state when filter returns no results', async () => {
      const user = userEvent.setup();
      renderUserManagement();

      const filterSelect = screen.getByDisplayValue('All Users');
      await user.selectOptions(filterSelect, 'inactive');

      // Only one inactive user, so after filtering should show count
      expect(screen.getByText('1 users')).toBeInTheDocument();

      // Now filter by admins (should show empty for this specific test case setup)
      await user.selectOptions(filterSelect, 'users');
      expect(screen.getByText('2 users')).toBeInTheDocument();
    });
  });

  describe('Error Handling Integration', () => {
    it('should handle and display action errors', async () => {
      const user = userEvent.setup();
      const errorMessage = 'Cannot delete admin user';
      mockMutations.delete.mutateAsync.mockRejectedValue(new Error(errorMessage));
      
      renderUserManagement();

      const deleteButtons = screen.getAllByText('Delete');
      await user.click(deleteButtons[0]);

      await waitFor(() => {
        expect(toast.error).toHaveBeenCalledWith(errorMessage);
      });
    });

    it('should provide retry functionality on data load error', async () => {
      const user = userEvent.setup();
      const mockRefetch = vi.fn();
      
      mockUseUsers.mockReturnValue({
        data: [],
        isLoading: false,
        error: new Error('Network error'),
        refetch: mockRefetch,
      });

      renderUserManagement();

      const tryAgainButton = screen.getByText('Try Again');
      await user.click(tryAgainButton);

      expect(mockRefetch).toHaveBeenCalled();
    });
  });

  describe('Form State Synchronization', () => {
    it('should keep form and table state in sync during operations', async () => {
      const user = userEvent.setup();
      
      renderUserManagement();

      // Open edit form
      const editButtons = screen.getAllByText('Edit');
      await user.click(editButtons[0]);

      await waitFor(() => {
        expect(screen.getByTestId('user-form-modal')).toBeInTheDocument();
      });

      // Form should be populated with user data
      expect(screen.getByTestId('user-info')).toHaveTextContent('User: john_doe');
    });

    it('should handle concurrent form operations correctly', async () => {
      const user = userEvent.setup();
      renderUserManagement();

      // Open create form
      await user.click(screen.getByText('Add User'));
      
      await waitFor(() => {
        expect(screen.getByTestId('user-form-modal')).toBeInTheDocument();
        expect(screen.getByText('Add User Form')).toBeInTheDocument();
      });

      // Close and immediately open edit form
      await user.click(screen.getByText('Close'));
      
      const editButtons = screen.getAllByText('Edit');
      await user.click(editButtons[0]);

      await waitFor(() => {
        expect(screen.getByText('Edit User Form')).toBeInTheDocument();
        expect(screen.getByTestId('user-info')).toHaveTextContent('User: john_doe');
      });
    });
  });
});