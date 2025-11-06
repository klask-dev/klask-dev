import React from 'react';
import { render, RenderOptions } from '@testing-library/react';
import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import { BrowserRouter } from 'react-router-dom';
import { vi } from 'vitest';

// Create a shared QueryClient instance to reduce memory usage
// This is created once and reused across tests, only clearing the cache
let sharedQueryClient: QueryClient | null = null;

const getQueryClient = () => {
  if (!sharedQueryClient) {
    sharedQueryClient = new QueryClient({
      defaultOptions: {
        queries: {
          retry: false,
          refetchOnWindowFocus: false,
          staleTime: 0,
          gcTime: 0, // Immediately garbage collect unused queries
        },
        mutations: {
          retry: false,
        },
      },
    });
  }
  return sharedQueryClient;
};

// Helper to clear the query client cache between tests
export const clearQueryClientCache = () => {
  const client = getQueryClient();
  client.clear();
};

// Create a custom render function that includes providers
const AllTheProviders = ({ children }: { children: React.ReactNode }) => {
  const queryClient = getQueryClient();

  return (
    <BrowserRouter>
      <QueryClientProvider client={queryClient}>
        {children}
      </QueryClientProvider>
    </BrowserRouter>
  );
};

const customRender = (
  ui: React.ReactElement,
  options?: Omit<RenderOptions, 'wrapper'>
) => render(ui, { wrapper: AllTheProviders, ...options });

// Re-export everything
export * from '@testing-library/react';

// Override render method
export { customRender as render };

// Test data factories
export const createMockUser = (overrides = {}) => ({
  id: 'user-1',
  username: 'testuser',
  email: 'test@example.com',
  role: 'User' as const,
  active: true,
  createdAt: '2024-01-01T00:00:00Z',
  updatedAt: '2024-01-01T00:00:00Z',
  ...overrides,
});

export const createMockCreateUserRequest = (overrides = {}) => ({
  username: 'newuser',
  email: 'newuser@example.com',
  password: 'Password123',
  role: 'User' as const,
  active: true,
  ...overrides,
});

export const createMockUpdateUserRequest = (overrides = {}) => ({
  username: 'updateduser',
  email: 'updated@example.com',
  role: 'Admin' as const,
  active: false,
  ...overrides,
});

// Mock functions factory
export const createMockFormHandlers = () => ({
  onClose: vi.fn(),
  onSubmit: vi.fn(),
});

// Helper for waiting for async form validation
export const waitForFormValidation = () => new Promise(resolve => setTimeout(resolve, 0));