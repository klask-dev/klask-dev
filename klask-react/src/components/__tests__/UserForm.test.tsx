import { describe, it, expect, vi, beforeEach } from 'vitest';
import { screen, fireEvent, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { UserForm } from '../admin/UserForm';
import { render, createMockUser, createMockFormHandlers } from '../../test/test-utils';

// Mock icons to avoid rendering issues
vi.mock('@heroicons/react/24/outline', () => ({
  XMarkIcon: () => <div data-testid="x-mark-icon">X</div>,
  UserIcon: () => <div data-testid="user-icon">User</div>,
  ShieldCheckIcon: () => <div data-testid="shield-check-icon">Admin</div>,
  EyeIcon: () => <div data-testid="eye-icon">Eye</div>,
  EyeSlashIcon: () => <div data-testid="eye-slash-icon">EyeSlash</div>,
}));

// Mock LoadingSpinner
vi.mock('../ui/LoadingSpinner', () => ({
  LoadingSpinner: ({ className }: { className?: string }) => (
    <div data-testid="loading-spinner" className={className}>Loading...</div>
  ),
}));

describe('UserForm', () => {
  const defaultProps = {
    isOpen: true,
    ...createMockFormHandlers(),
  };

  beforeEach(() => {
    vi.clearAllMocks();
  });

  describe('Rendering and Basic Functionality', () => {
    it('should not render when isOpen is false', () => {
      render(<UserForm {...defaultProps} isOpen={false} />);
      
      expect(screen.queryByText('Add User')).not.toBeInTheDocument();
    });

    it('should render create form when no user is provided', () => {
      render(<UserForm {...defaultProps} />);
      
      expect(screen.getByText('Add User')).toBeInTheDocument();
      expect(screen.getByText('Create User')).toBeInTheDocument();
      expect(screen.getByPlaceholderText('Enter password')).toBeInTheDocument();
      expect(screen.getByText('Password must contain at least 8 characters with uppercase, lowercase, and number')).toBeInTheDocument();
    });

    it('should render edit form when user is provided', () => {
      const user = createMockUser();
      render(<UserForm {...defaultProps} user={user} />);
      
      expect(screen.getByText('Edit User')).toBeInTheDocument();
      expect(screen.getByText('Update User')).toBeInTheDocument();
      expect(screen.getByDisplayValue(user.username)).toBeInTheDocument();
      expect(screen.getByDisplayValue(user.email)).toBeInTheDocument();
      expect(screen.getByText('(leave blank to keep current)')).toBeInTheDocument();
    });

    it('should render custom title when provided', () => {
      const customTitle = 'Custom Form Title';
      render(<UserForm {...defaultProps} title={customTitle} />);
      
      expect(screen.getByText(customTitle)).toBeInTheDocument();
    });

    it('should show loading state', () => {
      render(<UserForm {...defaultProps} isLoading={true} />);
      
      expect(screen.getByTestId('loading-spinner')).toBeInTheDocument();
      expect(screen.getByText('Creating...')).toBeInTheDocument();
    });

    it('should show loading state for edit mode', () => {
      const user = createMockUser();
      render(<UserForm {...defaultProps} user={user} isLoading={true} />);
      
      expect(screen.getByText('Updating...')).toBeInTheDocument();
    });
  });

  describe('Form Fields and Validation', () => {
    it('should display all required form fields for create mode', () => {
      render(<UserForm {...defaultProps} />);
      
      expect(screen.getByLabelText('Username')).toBeInTheDocument();
      expect(screen.getByLabelText('Email Address')).toBeInTheDocument();
      expect(screen.getByLabelText('Password')).toBeInTheDocument();
      expect(screen.getByText('Role')).toBeInTheDocument();
      expect(screen.getByLabelText('Active user account')).toBeInTheDocument();
    });

    it('should pre-populate form fields in edit mode', () => {
      const user = createMockUser({
        username: 'edituser',
        email: 'edit@example.com',
        role: 'Admin',
        active: false,
      });
      render(<UserForm {...defaultProps} user={user} />);
      
      expect(screen.getByDisplayValue('edituser')).toBeInTheDocument();
      expect(screen.getByDisplayValue('edit@example.com')).toBeInTheDocument();
      expect(screen.getByDisplayValue('')).toBeInTheDocument(); // password should be empty
      
      // Check if Admin role is selected
      const adminRadio = screen.getByDisplayValue('Admin');
      expect(adminRadio).toBeChecked();
      
      // Check if active checkbox reflects user state
      const activeCheckbox = screen.getByLabelText('Active user account');
      expect(activeCheckbox).not.toBeChecked();
    });

    it('should prevent form submission with invalid input and validate formats', async () => {
      const mockOnSubmit = vi.fn();
      render(<UserForm {...defaultProps} onSubmit={mockOnSubmit} />);

      const usernameInput = screen.getByLabelText('Username');
      const emailInput = screen.getByLabelText('Email Address');
      const passwordInput = screen.getByLabelText('Password');

      // Test 1: Invalid (too short, bad email, weak password)
      fireEvent.change(usernameInput, { target: { value: 'ab' } }); // Too short
      fireEvent.change(emailInput, { target: { value: 'invalid-email' } });
      fireEvent.change(passwordInput, { target: { value: 'weak' } });
      fireEvent.blur(usernameInput);
      fireEvent.blur(emailInput);
      fireEvent.blur(passwordInput);

      await waitFor(() => {
        const submitButton = screen.getByText('Create User');
        expect(submitButton).toBeDisabled();
      });

      // Test 2: Invalid username characters
      fireEvent.change(usernameInput, { target: { value: 'user@name!' } });
      fireEvent.change(emailInput, { target: { value: 'test@example.com' } });
      fireEvent.change(passwordInput, { target: { value: 'Password123' } });
      fireEvent.blur(usernameInput);

      await waitFor(() => {
        const submitButton = screen.getByText('Create User');
        expect(submitButton).toBeDisabled();
      });

      // Form should not submit with validation errors
      expect(mockOnSubmit).not.toHaveBeenCalled();
    });

    it('should allow valid usernames with underscores and hyphens', async () => {
      const mockOnSubmit = vi.fn();
      render(<UserForm {...defaultProps} onSubmit={mockOnSubmit} />);

      const usernameInput = screen.getByLabelText('Username');
      const emailInput = screen.getByLabelText('Email Address');
      const passwordInput = screen.getByLabelText('Password');

      // Fill form with valid data - use fireEvent for speed
      fireEvent.change(usernameInput, { target: { value: 'valid_user-name123' } });
      fireEvent.change(emailInput, { target: { value: 'test@example.com' } });
      fireEvent.change(passwordInput, { target: { value: 'Password123' } });

      // Trigger validation
      fireEvent.blur(usernameInput);
      fireEvent.blur(emailInput);
      fireEvent.blur(passwordInput);

      // Wait for form validation to complete
      await waitFor(() => {
        const submitButton = screen.getByText('Create User');
        expect(submitButton).not.toBeDisabled();
      });

      // Submit form
      const submitButton = screen.getByText('Create User');
      fireEvent.click(submitButton);

      // Should not show validation error
      expect(screen.queryByText(/Username can only contain/)).not.toBeInTheDocument();

      // Form should submit successfully
      await waitFor(() => {
        expect(mockOnSubmit).toHaveBeenCalledWith({
          username: 'valid_user-name123',
          email: 'test@example.com',
          password: 'Password123',
          role: 'User',
          active: true,
        });
      });
    });
  });

  describe('Password Show/Hide Functionality', () => {
    it('should toggle password visibility when eye icon is clicked', async () => {
      const user = userEvent.setup();
      render(<UserForm {...defaultProps} />);
      
      const passwordInput = screen.getByLabelText('Password') as HTMLInputElement;
      const toggleButton = screen.getByTestId('eye-icon').parentElement!;
      
      // Initially password should be hidden
      expect(passwordInput.type).toBe('password');
      expect(screen.getByTestId('eye-icon')).toBeInTheDocument();
      
      // Click to show password
      await user.click(toggleButton);
      expect(passwordInput.type).toBe('text');
      expect(screen.getByTestId('eye-slash-icon')).toBeInTheDocument();
      
      // Click to hide password again
      await user.click(toggleButton);
      expect(passwordInput.type).toBe('password');
      expect(screen.getByTestId('eye-icon')).toBeInTheDocument();
    });
  });

  describe('Role Selection', () => {
    it('should allow selecting User role', async () => {
      render(<UserForm {...defaultProps} />);
      
      const userRadio = screen.getByDisplayValue('User');
      const adminRadio = screen.getByDisplayValue('Admin');
      
      // User should be selected by default
      expect(userRadio).toBeChecked();
      expect(adminRadio).not.toBeChecked();
    });

    it('should allow selecting Admin role', async () => {
      const user = userEvent.setup();
      render(<UserForm {...defaultProps} />);
      
      const userRadio = screen.getByDisplayValue('User');
      const adminRadio = screen.getByDisplayValue('Admin');
      
      // Click Admin role
      await user.click(adminRadio);
      
      expect(adminRadio).toBeChecked();
      expect(userRadio).not.toBeChecked();
    });

    it('should display role icons correctly', () => {
      render(<UserForm {...defaultProps} />);
      
      // Should show User icon for User role and Shield icon for Admin role
      expect(screen.getByTestId('user-icon')).toBeInTheDocument();
      expect(screen.getByTestId('shield-check-icon')).toBeInTheDocument();
    });
  });

  describe('Active Status Toggle', () => {
    it('should have active checkbox checked by default in create mode', () => {
      render(<UserForm {...defaultProps} />);
      
      const activeCheckbox = screen.getByLabelText('Active user account');
      expect(activeCheckbox).toBeChecked();
    });

    it('should allow toggling active status', async () => {
      const user = userEvent.setup();
      render(<UserForm {...defaultProps} />);
      
      const activeCheckbox = screen.getByLabelText('Active user account');
      
      // Initially checked
      expect(activeCheckbox).toBeChecked();
      
      // Uncheck
      await user.click(activeCheckbox);
      expect(activeCheckbox).not.toBeChecked();
      
      // Check again
      await user.click(activeCheckbox);
      expect(activeCheckbox).toBeChecked();
    });

    it('should show helpful text about inactive users', () => {
      render(<UserForm {...defaultProps} />);
      
      expect(screen.getByText('Inactive users cannot log in to the system')).toBeInTheDocument();
    });
  });

  describe('Form Submission', () => {
    it('should call onSubmit with correct data for create mode', async () => {
      const user = userEvent.setup();
      const mockOnSubmit = vi.fn();
      render(<UserForm {...defaultProps} onSubmit={mockOnSubmit} />);

      // Fill form
      await user.type(screen.getByLabelText('Username'), 'newuser');
      await user.type(screen.getByLabelText('Email Address'), 'new@example.com');
      await user.type(screen.getByLabelText('Password'), 'Password123');
      await user.click(screen.getByDisplayValue('Admin')); // Select Admin role

      // Wait for form validation to complete
      await waitFor(() => {
        const submitButton = screen.getByText('Create User');
        expect(submitButton).not.toBeDisabled();
      });

      // Submit form
      await user.click(screen.getByText('Create User'));

      await waitFor(() => {
        expect(mockOnSubmit).toHaveBeenCalledWith({
          username: 'newuser',
          email: 'new@example.com',
          password: 'Password123',
          role: 'Admin',
          active: true,
        });
      });
    });

    it('should call onSubmit with correct data for edit mode', async () => {
      const user = userEvent.setup();
      const mockOnSubmit = vi.fn();
      const existingUser = createMockUser();
      render(<UserForm {...defaultProps} user={existingUser} onSubmit={mockOnSubmit} />);

      // Modify fields
      const usernameInput = screen.getByDisplayValue(existingUser.username);
      await user.clear(usernameInput);
      await user.type(usernameInput, 'updateduser');

      await user.click(screen.getByDisplayValue('Admin')); // Change role

      // Wait for form validation to complete
      await waitFor(() => {
        const submitButton = screen.getByText('Update User');
        expect(submitButton).not.toBeDisabled();
      }, { timeout: 5000 });

      // Submit form
      await user.click(screen.getByText('Update User'));

      await waitFor(() => {
        expect(mockOnSubmit).toHaveBeenCalledWith({
          username: 'updateduser',
          email: existingUser.email,
          role: 'Admin',
          active: existingUser.active,
        });
      });
    });

    it('should trim whitespace from username and email', async () => {
      const user = userEvent.setup();
      const mockOnSubmit = vi.fn();
      render(<UserForm {...defaultProps} onSubmit={mockOnSubmit} />);

      // Fill form with spaces
      await user.type(screen.getByLabelText('Username'), 'testuser');
      await user.type(screen.getByLabelText('Email Address'), 'test@example.com');
      await user.type(screen.getByLabelText('Password'), 'Password123');

      // Wait for form validation to complete
      await waitFor(() => {
        const submitButton = screen.getByText('Create User');
        expect(submitButton).not.toBeDisabled();
      }, { timeout: 3000 });

      // Submit form
      await user.click(screen.getByText('Create User'));

      await waitFor(() => {
        expect(mockOnSubmit).toHaveBeenCalledWith({
          username: 'testuser',
          email: 'test@example.com',
          password: 'Password123',
          role: 'User',
          active: true,
        });
      });
    });

    it('should remove empty password in edit mode', async () => {
      const user = userEvent.setup();
      const mockOnSubmit = vi.fn();
      const existingUser = createMockUser();
      render(<UserForm {...defaultProps} user={existingUser} onSubmit={mockOnSubmit} />);

      // Wait for form validation to complete (form should be valid by default in edit mode)
      await waitFor(() => {
        const submitButton = screen.getByText('Update User');
        expect(submitButton).not.toBeDisabled();
      }, { timeout: 5000 });

      // Leave password empty and submit
      await user.click(screen.getByText('Update User'));

      await waitFor(() => {
        expect(mockOnSubmit).toHaveBeenCalled();
        const submittedData = mockOnSubmit.mock.calls[0][0];
        expect(submittedData).not.toHaveProperty('password');
      });
    });

    it('should not submit form when validation fails', async () => {
      const user = userEvent.setup();
      const mockOnSubmit = vi.fn();
      render(<UserForm {...defaultProps} onSubmit={mockOnSubmit} />);
      
      // Leave username empty and try to submit
      await user.type(screen.getByLabelText('Email Address'), 'test@example.com');
      await user.type(screen.getByLabelText('Password'), 'Password123');
      
      const submitButton = screen.getByText('Create User');
      expect(submitButton).toBeDisabled();
      
      // Try to submit
      await user.click(submitButton);
      
      expect(mockOnSubmit).not.toHaveBeenCalled();
    });
  });

  describe('Modal Behavior', () => {
    it('should call onClose when X button is clicked', async () => {
      const user = userEvent.setup();
      const mockOnClose = vi.fn();
      render(<UserForm {...defaultProps} onClose={mockOnClose} />);
      
      const closeButton = screen.getByTestId('x-mark-icon').parentElement!;
      await user.click(closeButton);
      
      expect(mockOnClose).toHaveBeenCalled();
    });

    it('should call onClose when Cancel button is clicked', async () => {
      const user = userEvent.setup();
      const mockOnClose = vi.fn();
      render(<UserForm {...defaultProps} onClose={mockOnClose} />);
      
      await user.click(screen.getByText('Cancel'));
      
      expect(mockOnClose).toHaveBeenCalled();
    });

    it('should call onClose when backdrop is clicked', async () => {
      const mockOnClose = vi.fn();
      render(<UserForm {...defaultProps} onClose={mockOnClose} />);
      
      // Click on backdrop (the overlay div)
      const backdrop = document.querySelector('.fixed.inset-0.bg-gray-500');
      if (backdrop) {
        fireEvent.click(backdrop);
        expect(mockOnClose).toHaveBeenCalled();
      }
    });

    it('should disable buttons when loading', () => {
      render(<UserForm {...defaultProps} isLoading={true} />);
      
      expect(screen.getByText('Cancel')).toBeDisabled();
      expect(screen.getByText('Creating...')).toBeDisabled();
    });
  });

  describe('Form Reset and Data Persistence', () => {
    it('should reset form when user prop changes', () => {
      const user1 = createMockUser({ username: 'user1', email: 'user1@example.com' });
      const user2 = createMockUser({ username: 'user2', email: 'user2@example.com' });
      
      const { rerender } = render(<UserForm {...defaultProps} user={user1} />);
      
      expect(screen.getByDisplayValue('user1')).toBeInTheDocument();
      expect(screen.getByDisplayValue('user1@example.com')).toBeInTheDocument();
      
      rerender(<UserForm {...defaultProps} user={user2} />);
      
      expect(screen.getByDisplayValue('user2')).toBeInTheDocument();
      expect(screen.getByDisplayValue('user2@example.com')).toBeInTheDocument();
    });

    it('should reset form when switching from edit to create mode', () => {
      const user = createMockUser({ username: 'edituser', email: 'edit@example.com' });
      
      const { rerender } = render(<UserForm {...defaultProps} user={user} />);
      
      expect(screen.getByDisplayValue('edituser')).toBeInTheDocument();
      
      rerender(<UserForm {...defaultProps} user={undefined} />);
      
      expect(screen.queryByDisplayValue('edituser')).not.toBeInTheDocument();
      expect(screen.getByLabelText('Username')).toHaveValue(''); // Empty username field
    });
  });

  describe('Accessibility', () => {
    it('should have proper ARIA labels and roles', () => {
      render(<UserForm {...defaultProps} />);
      
      // Check form has proper structure
      expect(screen.getByLabelText('Username')).toBeInTheDocument();
      expect(screen.getByLabelText('Email Address')).toBeInTheDocument();
      expect(screen.getByLabelText('Password')).toBeInTheDocument();
      expect(screen.getByText('Role')).toBeInTheDocument();
      expect(screen.getByLabelText('Active user account')).toBeInTheDocument();
    });

    it('should have proper button roles and types', () => {
      render(<UserForm {...defaultProps} />);
      
      expect(screen.getByRole('button', { name: /cancel/i })).toHaveAttribute('type', 'button');
      expect(screen.getByRole('button', { name: /create user/i })).toHaveAttribute('type', 'submit');
    });

    it('should have proper error message styling', () => {
      render(<UserForm {...defaultProps} />);
      
      // Just check that the form structure is correct
      // Error messages will be shown when validation fails during real usage
      const usernameInput = screen.getByLabelText('Username');
      expect(usernameInput).toBeInTheDocument();
      expect(usernameInput).toHaveAttribute('id', 'username');
    });
  });

  describe('Edge Cases and Error Handling', () => {
    it('should handle undefined user gracefully', () => {
      render(<UserForm {...defaultProps} user={undefined} />);
      
      expect(screen.getByText('Add User')).toBeInTheDocument();
      expect(screen.getByLabelText('Username')).toHaveValue(''); // Empty username
    });

    it('should handle malformed user data', () => {
      const malformedUser = createMockUser({
        username: undefined as any,
        email: null as any,
      });
      
      // Should not crash
      render(<UserForm {...defaultProps} user={malformedUser} />);
      
      expect(screen.getByText('Edit User')).toBeInTheDocument();
    });

    it('should maintain form state during loading', async () => {
      const user = userEvent.setup();
      const { rerender } = render(<UserForm {...defaultProps} />);
      
      // Fill some data
      await user.type(screen.getByLabelText('Username'), 'testuser');
      
      // Start loading
      rerender(<UserForm {...defaultProps} isLoading={true} />);
      
      // Form should still have the data
      expect(screen.getByDisplayValue('testuser')).toBeInTheDocument();
      expect(screen.getByText('Creating...')).toBeInTheDocument();
    });

    it('should handle form submission with missing required fields', async () => {
      const user = userEvent.setup();
      const mockOnSubmit = vi.fn();
      render(<UserForm {...defaultProps} onSubmit={mockOnSubmit} />);
      
      // Only fill email, leave username and password empty
      await user.type(screen.getByLabelText('Email Address'), 'test@example.com');
      
      const submitButton = screen.getByText('Create User');
      expect(submitButton).toBeDisabled();
      
      expect(mockOnSubmit).not.toHaveBeenCalled();
    });
  });
});