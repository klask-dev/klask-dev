import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import React from 'react';
import { BrowserRouter } from 'react-router-dom';
import RegisterPage from '../RegisterPage';
import * as api from '../../../lib/api';

// Mock the API client
vi.mock('../../../lib/api', () => {
    class ApiError extends Error {
        public status: number;
        public details?: Record<string, any>;

        constructor(message: string, status: number, details?: Record<string, any>) {
            super(message);
            this.name = 'ApiError';
            this.status = status;
            this.details = details;
        }
    }

    return {
        apiClient: {
            auth: {
                register: vi.fn(),
                checkRegistrationStatus: vi.fn(),
            },
        },
        ApiError,
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

// Mock auth store
vi.mock('../../../stores/auth-store', () => ({
    useAuthStore: vi.fn((selector) => {
        const mockStore = {
            login: vi.fn(),
        };
        return selector ? selector(mockStore) : mockStore;
    }),
}));

const createWrapper = () => {
    const queryClient = new QueryClient({
        defaultOptions: {
            queries: { retry: false },
            mutations: { retry: false },
        },
    });

    return ({ children }: { children: React.ReactNode }) =>
        React.createElement(
            BrowserRouter,
            {},
            React.createElement(QueryClientProvider, { client: queryClient }, children)
        );
};

describe('RegisterPage', () => {
    beforeEach(() => {
        vi.clearAllMocks();
        mockNavigate.mockClear();
    });

    describe('Registration Status Check', () => {
        it('shows loading spinner while checking registration status', () => {
            // Mock a never-resolving promise to keep loading state
            vi.mocked(api.apiClient.auth.checkRegistrationStatus).mockImplementation(
                () => new Promise(() => { })
            );

            render(<RegisterPage />, { wrapper: createWrapper() });

            expect(screen.getByRole('status')).toBeTruthy();
        });

        it('shows registration form when registration is enabled', async () => {
            vi.mocked(api.apiClient.auth.checkRegistrationStatus).mockResolvedValue({
                registration_allowed: true,
            });

            render(<RegisterPage />, { wrapper: createWrapper() });

            await waitFor(() => {
                expect(screen.getByText(/Create your account/i)).toBeTruthy();
            });

            expect(screen.getByLabelText(/First name/i)).toBeTruthy();
            expect(screen.getByLabelText(/Last name/i)).toBeTruthy();
            expect(screen.getByLabelText(/Username/i)).toBeTruthy();
            expect(screen.getByLabelText(/Email address/i)).toBeTruthy();
            expect(screen.getByLabelText(/^Password$/i)).toBeTruthy();
            expect(screen.getByLabelText(/Confirm password/i)).toBeTruthy();
        });

        it('shows disabled message when registration is disabled', async () => {
            vi.mocked(api.apiClient.auth.checkRegistrationStatus).mockResolvedValue({
                registration_allowed: false,
            });

            render(<RegisterPage />, { wrapper: createWrapper() });

            await waitFor(() => {
                expect(screen.getByText(/Registration Disabled/i)).toBeTruthy();
            });

            expect(
                screen.getByText(/Registration is currently disabled/i)
            ).toBeTruthy();
            expect(screen.getByText(/Redirecting to login page/i)).toBeTruthy();
        });

        it('shows form when status check fails (fallback behavior)', async () => {
            vi.mocked(api.apiClient.auth.checkRegistrationStatus).mockRejectedValue(
                new Error('Network error')
            );

            render(<RegisterPage />, { wrapper: createWrapper() });

            await waitFor(() => {
                expect(screen.getByText(/Create your account/i)).toBeTruthy();
            });
        });
    });

    describe('Form Submission', () => {
        beforeEach(() => {
            vi.mocked(api.apiClient.auth.checkRegistrationStatus).mockResolvedValue({
                registration_allowed: true,
            });
        });

        it('successfully registers a new user', async () => {
            const user = userEvent.setup();
            const mockResponse = {
                token: 'test-token',
                user: {
                    id: '1',
                    username: 'testuser',
                    email: 'test@example.com',
                    role: 'User' as const,
                    active: true,
                    created_at: '2024-01-01',
                    updated_at: '2024-01-01',
                },
            };

            vi.mocked(api.apiClient.auth.register).mockResolvedValue(mockResponse);

            render(<RegisterPage />, { wrapper: createWrapper() });

            await waitFor(() => {
                expect(screen.getByLabelText(/First name/i)).toBeTruthy();
            });

            // Fill out the form
            await user.type(screen.getByLabelText(/First name/i), 'John');
            await user.type(screen.getByLabelText(/Last name/i), 'Doe');
            await user.type(screen.getByLabelText(/Username/i), 'johndoe');
            await user.type(screen.getByLabelText(/Email address/i), 'john@example.com');
            await user.type(screen.getByLabelText(/^Password$/i), 'Password123!');
            await user.type(screen.getByLabelText(/Confirm password/i), 'Password123!');

            // Submit the form
            await user.click(screen.getByRole('button', { name: /Create account/i }));

            await waitFor(() => {
                expect(api.apiClient.auth.register).toHaveBeenCalledWith({
                    firstName: 'John',
                    lastName: 'Doe',
                    username: 'johndoe',
                    email: 'john@example.com',
                    password: 'Password123!',
                    confirmPassword: 'Password123!',
                });
            });

            expect(mockNavigate).toHaveBeenCalledWith('/home');
        });

        it('shows error message when registration fails with 403', async () => {
            const user = userEvent.setup();
            const apiError = new api.ApiError('Registration disabled', 403);

            vi.mocked(api.apiClient.auth.register).mockRejectedValue(apiError);

            render(<RegisterPage />, { wrapper: createWrapper() });

            await waitFor(() => {
                expect(screen.getByLabelText(/First name/i)).toBeTruthy();
            });

            // Fill and submit form
            await user.type(screen.getByLabelText(/First name/i), 'John');
            await user.type(screen.getByLabelText(/Last name/i), 'Doe');
            await user.type(screen.getByLabelText(/Username/i), 'johndoe');
            await user.type(screen.getByLabelText(/Email address/i), 'john@example.com');
            await user.type(screen.getByLabelText(/^Password$/i), 'Password123!');
            await user.type(screen.getByLabelText(/Confirm password/i), 'Password123!');

            await user.click(screen.getByRole('button', { name: /Create account/i }));

            await waitFor(() => {
                expect(
                    screen.getByText(/Registration is currently disabled/i)
                ).toBeTruthy();
            });
        });

        it('shows server error message on validation failure', async () => {
            const user = userEvent.setup();
            const apiError = new api.ApiError('Validation failed', 400, {
                error: 'Email already exists',
            });

            vi.mocked(api.apiClient.auth.register).mockRejectedValue(apiError);

            render(<RegisterPage />, { wrapper: createWrapper() });

            await waitFor(() => {
                expect(screen.getByLabelText(/First name/i)).toBeTruthy();
            });

            // Fill and submit form
            await user.type(screen.getByLabelText(/First name/i), 'John');
            await user.type(screen.getByLabelText(/Last name/i), 'Doe');
            await user.type(screen.getByLabelText(/Username/i), 'johndoe');
            await user.type(screen.getByLabelText(/Email address/i), 'existing@example.com');
            await user.type(screen.getByLabelText(/^Password$/i), 'Password123!');
            await user.type(screen.getByLabelText(/Confirm password/i), 'Password123!');

            await user.click(screen.getByRole('button', { name: /Create account/i }));

            await waitFor(() => {
                expect(screen.getByText(/Email already exists/i)).toBeTruthy();
            });
        });
    });

    describe('UI Elements', () => {
        beforeEach(() => {
            vi.mocked(api.apiClient.auth.checkRegistrationStatus).mockResolvedValue({
                registration_allowed: true,
            });
        });

        it('shows link to login page', async () => {
            render(<RegisterPage />, { wrapper: createWrapper() });

            await waitFor(() => {
                const loginLink = screen.getByRole('link', {
                    name: /sign in to your existing account/i,
                });
                expect(loginLink).toBeTruthy();
                expect(loginLink.getAttribute('href')).toBe('/login');
            });
        });

        it('toggles password visibility', async () => {
            const user = userEvent.setup();

            render(<RegisterPage />, { wrapper: createWrapper() });

            await waitFor(() => {
                expect(screen.getByLabelText(/^Password$/i)).toBeTruthy();
            });

            const passwordInput = screen.getByLabelText(/^Password$/i);
            expect(passwordInput.getAttribute('type')).toBe('password');

            // Find and click the toggle button (there are multiple buttons, get the right one)
            const buttons = screen.getAllByRole('button');
            const passwordToggle = buttons.find(
                (btn) => btn.querySelector('svg') && btn.type === 'button'
            );

            if (passwordToggle) {
                await user.click(passwordToggle);
                expect(passwordInput.getAttribute('type')).toBe('text');
            }
        });

        it('shows loading state during submission', async () => {
            const user = userEvent.setup();

            vi.mocked(api.apiClient.auth.register).mockImplementation(
                () => new Promise(() => { }) // Never resolves to keep loading state
            );

            render(<RegisterPage />, { wrapper: createWrapper() });

            await waitFor(() => {
                expect(screen.getByLabelText(/First name/i)).toBeTruthy();
            });

            // Fill form minimally
            await user.type(screen.getByLabelText(/First name/i), 'John');
            await user.type(screen.getByLabelText(/Last name/i), 'Doe');
            await user.type(screen.getByLabelText(/Username/i), 'johndoe');
            await user.type(screen.getByLabelText(/Email address/i), 'john@example.com');
            await user.type(screen.getByLabelText(/^Password$/i), 'Password123!');
            await user.type(screen.getByLabelText(/Confirm password/i), 'Password123!');

            await user.click(screen.getByRole('button', { name: /Create account/i }));

            await waitFor(() => {
                expect(screen.getByText(/Creating account/i)).toBeTruthy();
            });
        });
    });
});
