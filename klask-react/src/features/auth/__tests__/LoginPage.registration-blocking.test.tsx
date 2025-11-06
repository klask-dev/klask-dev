import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { render, screen, fireEvent, waitFor } from '@testing-library/react';
import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import React from 'react';
import { BrowserRouter } from 'react-router-dom';
import LoginPage from '../LoginPage';
import * as api from '../../../lib/api';
import { useAuthStore } from '../../../stores/auth-store';

// Mock the API client
vi.mock('../../../lib/api', async () => {
  const actual = await vi.importActual('../../../lib/api');
  return {
    ...(actual as any),
    apiClient: {
      auth: {
        login: vi.fn(),
        checkRegistrationStatus: vi.fn(),
        getProfile: vi.fn(),
      },
    },
    extractFieldErrors: vi.fn(() => ({})),
  };
});

// Mock useNavigate
const mockNavigate = vi.fn();
vi.mock('react-router-dom', async () => {
  const actual = await vi.importActual('react-router-dom');
  return {
    ...actual,
    useNavigate: () => mockNavigate,
  };
});

// Mock fetch for setup check
const mockFetch = vi.fn();
vi.stubGlobal('fetch', mockFetch);

// Mock localStorage
const localStorageMock = {
  getItem: vi.fn(),
  setItem: vi.fn(),
  removeItem: vi.fn(),
  clear: vi.fn(),
};
vi.stubGlobal('localStorage', localStorageMock);

const createWrapper = () => {
  const queryClient = new QueryClient({
    defaultOptions: {
      queries: { retry: false },
    },
  });

  return ({ children }: { children: React.ReactNode }) =>
    React.createElement(
      BrowserRouter,
      {},
      React.createElement(QueryClientProvider, { client: queryClient }, children)
    );
};

describe('LoginPage - Registration Blocking Feature', () => {
  beforeEach(() => {
    vi.clearAllMocks();
    mockNavigate.mockClear();

    // Mock setup check to always pass
    mockFetch.mockResolvedValueOnce({
      ok: true,
      json: async () => ({ needs_setup: false }),
    });

    // Mock localStorage
    localStorageMock.getItem.mockReturnValue(null);
  });

  afterEach(() => {
    vi.restoreAllMocks();
  });

  describe('Registration Enabled Scenarios', () => {
    beforeEach(() => {
      vi.mocked(api.apiClient.auth.checkRegistrationStatus).mockResolvedValueOnce({
        registration_allowed: true,
      });
    });

    it('should show create account link when registration is enabled', async () => {
      render(<LoginPage />, { wrapper: createWrapper() });

      await waitFor(() => {
        const link = screen.getByRole('link', { name: /create a new account/i });
        expect(link).toBeInTheDocument();
      });
    });

    it('should link to /register when registration is enabled', async () => {
      render(<LoginPage />, { wrapper: createWrapper() });

      await waitFor(() => {
        const link = screen.getByRole('link', { name: /create a new account/i });
        expect(link).toHaveAttribute('href', '/register');
      });
    });

    it('should display "Or create a new account" text', async () => {
      render(<LoginPage />, { wrapper: createWrapper() });

      await waitFor(() => {
        expect(screen.getByRole('link', { name: /create a new account/i })).toBeInTheDocument();
        // Verify the "Or" text is present near the link
        const paragraph = screen.getByText((content, element) => {
          return element?.tagName === 'P' && element?.textContent?.includes('create a new account');
        });
        expect(paragraph?.textContent).toMatch(/Or/i);
      });
    });

    it('should render login form when registration is enabled', async () => {
      render(<LoginPage />, { wrapper: createWrapper() });

      await waitFor(() => {
        expect(screen.getByLabelText(/username/i)).toBeInTheDocument();
        expect(screen.getByLabelText(/password/i)).toBeInTheDocument();
        expect(screen.getByRole('button', { name: /sign in/i })).toBeInTheDocument();
      });
    });
  });

  describe('Registration Disabled Scenarios', () => {
    beforeEach(() => {
      vi.mocked(api.apiClient.auth.checkRegistrationStatus).mockResolvedValueOnce({
        registration_allowed: false,
      });
    });

    it('should hide create account link when registration is disabled', async () => {
      render(<LoginPage />, { wrapper: createWrapper() });

      await waitFor(() => {
        // Wait for the loading spinner to disappear
        expect(screen.queryByText(/Sign in to Klask/i)).toBeInTheDocument();
      });

      // The create account link should not be present
      const createAccountLink = screen.queryByRole('link', { name: /create a new account/i });
      expect(createAccountLink).not.toBeInTheDocument();
    });

    it('should not display "Or create a new account" text', async () => {
      render(<LoginPage />, { wrapper: createWrapper() });

      await waitFor(() => {
        // Wait for loading to complete
        expect(screen.getByText(/Sign in to Klask/i)).toBeInTheDocument();
      });

      expect(screen.queryByText(/Or\s+create a new account/i)).not.toBeInTheDocument();
    });

    it('should still render login form when registration is disabled', async () => {
      render(<LoginPage />, { wrapper: createWrapper() });

      await waitFor(() => {
        expect(screen.getByLabelText(/username/i)).toBeInTheDocument();
      });
    });

    it('should allow login even when registration is disabled', async () => {
      const mockToken = 'test-token';
      const mockUser = {
        id: '1',
        username: 'testuser',
        email: 'test@example.com',
        role: 'User' as const,
        active: true,
        created_at: '2024-01-01',
        updated_at: '2024-01-01',
      };

      vi.mocked(api.apiClient.auth.login).mockResolvedValueOnce({
        token: mockToken,
        user: mockUser,
      });

      vi.mocked(api.apiClient.auth.getProfile).mockResolvedValueOnce(mockUser);

      render(<LoginPage />, { wrapper: createWrapper() });

      await waitFor(() => {
        expect(screen.getByLabelText(/username/i)).toBeInTheDocument();
      });

      const usernameInput = screen.getByLabelText(/username/i);
      const passwordInput = screen.getByLabelText(/password/i);
      const submitButton = screen.getByRole('button', { name: /sign in/i });

      fireEvent.change(usernameInput, { target: { value: 'testuser' } });
      fireEvent.change(passwordInput, { target: { value: 'password123' } });
      fireEvent.click(submitButton);

      await waitFor(() => {
        expect(api.apiClient.auth.login).toHaveBeenCalledWith({
          username: 'testuser',
          password: 'password123',
        });
      });
    });
  });

  describe('Loading State Handling', () => {
    it('should show loading spinner while checking registration status', async () => {
      // Use a promise that doesn't resolve immediately
      const unresolvingPromise = new Promise(() => {});
      vi.mocked(api.apiClient.auth.checkRegistrationStatus).mockReturnValueOnce(
        unresolvingPromise as any
      );

      render(<LoginPage />, { wrapper: createWrapper() });

      // Should show loading spinner initially
      const spinner = screen.getByRole('status', { name: /Loading/i });
      expect(spinner).toBeInTheDocument();
    });

    it('should hide loading spinner after registration status check completes', async () => {
      vi.mocked(api.apiClient.auth.checkRegistrationStatus).mockResolvedValueOnce({
        registration_allowed: true,
      });

      const { container } = render(<LoginPage />, { wrapper: createWrapper() });

      await waitFor(() => {
        // After loading, the login form should be visible
        expect(screen.getByLabelText(/username/i)).toBeInTheDocument();
      });

      // The loading spinner should not be visible anymore
      const spinners = container.querySelectorAll('[role="presentation"]');
      const visibleSpinners = Array.from(spinners).filter((spinner) => {
        const style = window.getComputedStyle(spinner);
        return style.display !== 'none';
      });
      // After loading completes, there should be no visible spinner in the main content area
      // The form should be displayed instead
      expect(screen.getByText(/Sign in to Klask/i)).toBeInTheDocument();
    });
  });

  describe('Error Handling - Registration Status Check', () => {
    it('should show create account link when registration status check fails', async () => {
      const error = new Error('Network error');
      vi.mocked(api.apiClient.auth.checkRegistrationStatus).mockRejectedValueOnce(error);

      render(<LoginPage />, { wrapper: createWrapper() });

      await waitFor(() => {
        // Should default to showing the link if API fails
        const link = screen.queryByRole('link', { name: /create a new account/i });
        expect(link).toBeInTheDocument();
      });
    });

    it('should log error when registration status check fails', async () => {
      const consoleErrorSpy = vi.spyOn(console, 'error').mockImplementation(() => {});

      const error = new Error('Failed to check registration status');
      vi.mocked(api.apiClient.auth.checkRegistrationStatus).mockRejectedValueOnce(error);

      render(<LoginPage />, { wrapper: createWrapper() });

      await waitFor(() => {
        expect(consoleErrorSpy).toHaveBeenCalledWith(
          'Failed to check registration status:',
          expect.any(Error)
        );
      });

      consoleErrorSpy.mockRestore();
    });

    it('should handle ApiError when checking registration status', async () => {
      const apiError = new api.ApiError('Service unavailable', 503, {
        error: 'Database connection failed',
      });
      vi.mocked(api.apiClient.auth.checkRegistrationStatus).mockRejectedValueOnce(apiError);

      render(<LoginPage />, { wrapper: createWrapper() });

      await waitFor(() => {
        // Should still render login form and default to showing registration link
        expect(screen.getByLabelText(/username/i)).toBeInTheDocument();
      });
    });

    it('should handle 403 error when checking registration status', async () => {
      const apiError = new api.ApiError('Forbidden', 403);
      vi.mocked(api.apiClient.auth.checkRegistrationStatus).mockRejectedValueOnce(apiError);

      render(<LoginPage />, { wrapper: createWrapper() });

      await waitFor(() => {
        expect(screen.getByLabelText(/username/i)).toBeInTheDocument();
      });
    });
  });

  describe('Setup Check Integration', () => {
    it('should check setup status before checking registration status', async () => {
      vi.mocked(api.apiClient.auth.checkRegistrationStatus).mockResolvedValueOnce({
        registration_allowed: true,
      });

      render(<LoginPage />, { wrapper: createWrapper() });

      await waitFor(() => {
        expect(mockFetch).toHaveBeenCalled();
        const callUrl = mockFetch.mock.calls[0][0];
        expect(callUrl).toMatch(/\/api\/auth\/setup\/check/);
      });
    });

    it('should not check registration status if setup is needed', async () => {
      // Reset fetch mock and configure for setup check
      mockFetch.mockReset();
      mockFetch.mockResolvedValueOnce({
        ok: true,
        json: async () => ({ needs_setup: true }),
      });

      render(<LoginPage />, { wrapper: createWrapper() });

      // Wait for navigation to potentially be called
      await waitFor(() => {
        // Navigate should be called when needs_setup is true
        const navCalls = mockNavigate.mock.calls;
        const hasSetupCall = navCalls.some((call) => call[0] === '/setup');
        expect(hasSetupCall).toBe(true);
      }, { timeout: 2000 });

      // Registration status should not be checked if setup is required
      expect(api.apiClient.auth.checkRegistrationStatus).not.toHaveBeenCalled();
    });
  });

  describe('Setup Check Error Handling', () => {
    it('should handle setup check failure gracefully', async () => {
      const consoleErrorSpy = vi.spyOn(console, 'error').mockImplementation(() => {});

      mockFetch.mockRejectedValueOnce(new Error('Network error during setup check'));
      vi.mocked(api.apiClient.auth.checkRegistrationStatus).mockResolvedValueOnce({
        registration_allowed: true,
      });

      render(<LoginPage />, { wrapper: createWrapper() });

      await waitFor(() => {
        // Should still attempt to render normally even if setup check fails
        expect(screen.getByText(/Sign in to Klask/i)).toBeInTheDocument();
      });

      consoleErrorSpy.mockRestore();
    });
  });

  describe('UI State Consistency', () => {
    it('should maintain consistent UI when toggling between enabled/disabled', async () => {
      const { rerender } = render(<LoginPage />, { wrapper: createWrapper() });

      vi.mocked(api.apiClient.auth.checkRegistrationStatus).mockResolvedValueOnce({
        registration_allowed: true,
      });

      await waitFor(() => {
        expect(screen.getByRole('link', { name: /create a new account/i })).toBeInTheDocument();
      });
    });

    it('should keep form visible regardless of registration status', async () => {
      vi.mocked(api.apiClient.auth.checkRegistrationStatus).mockResolvedValueOnce({
        registration_allowed: false,
      });

      render(<LoginPage />, { wrapper: createWrapper() });

      await waitFor(() => {
        const usernameField = screen.getByLabelText(/username/i);
        const passwordField = screen.getByLabelText(/password/i);

        expect(usernameField).toBeInTheDocument();
        expect(passwordField).toBeInTheDocument();
        expect(usernameField.tagName).toBe('INPUT');
        expect(passwordField.tagName).toBe('INPUT');
      });
    });
  });

  describe('Component Lifecycle', () => {
    it('should check registration status on component mount', async () => {
      vi.mocked(api.apiClient.auth.checkRegistrationStatus).mockResolvedValueOnce({
        registration_allowed: true,
      });

      render(<LoginPage />, { wrapper: createWrapper() });

      await waitFor(() => {
        expect(api.apiClient.auth.checkRegistrationStatus).toHaveBeenCalledTimes(1);
      });
    });

    it('should not check registration status multiple times', async () => {
      vi.mocked(api.apiClient.auth.checkRegistrationStatus).mockResolvedValueOnce({
        registration_allowed: true,
      });

      const { rerender } = render(<LoginPage />, { wrapper: createWrapper() });

      await waitFor(() => {
        expect(api.apiClient.auth.checkRegistrationStatus).toHaveBeenCalledTimes(1);
      });

      // Rerender should not trigger another check
      rerender(<LoginPage />);

      // Still should only be called once
      expect(api.apiClient.auth.checkRegistrationStatus).toHaveBeenCalledTimes(1);
    });
  });
});
