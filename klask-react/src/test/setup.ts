import '@testing-library/jest-dom';
import { cleanup, configure } from '@testing-library/react';
import { afterEach, afterAll, vi } from 'vitest';

// Configure testing-library to be less verbose
configure({
  getElementError: (message) => {
    return new Error(
      [
        message,
        'Tip: Try using a more specific selector or check if the element exists.',
      ].join('\n\n')
    );
  },
});

// Cleanup after each test
afterEach(async () => {
  cleanup();
  vi.clearAllMocks();
  vi.clearAllTimers();

  // Clear QueryClient cache to free memory
  // Dynamically import to avoid issues if not used
  try {
    const { clearQueryClientCache } = await import('./test-utils');
    clearQueryClientCache();
  } catch (e) {
    // Ignore if test-utils is not available
  }
});

// Restore all mocks after all tests to free memory
afterAll(() => {
  vi.restoreAllMocks();
});

// Mock localStorage
const localStorageMock = {
  getItem: vi.fn(),
  setItem: vi.fn(),
  removeItem: vi.fn(),
  clear: vi.fn(),
};
Object.defineProperty(window, 'localStorage', { value: localStorageMock });

// Mock IntersectionObserver
global.IntersectionObserver = class IntersectionObserver {
  observe = vi.fn();
  unobserve = vi.fn();
  disconnect = vi.fn();
} as any;

// Mock ResizeObserver
global.ResizeObserver = class ResizeObserver {
  observe = vi.fn();
  unobserve = vi.fn();
  disconnect = vi.fn();
} as any;

// Mock fetch
global.fetch = vi.fn();

// Mock console methods to reduce noise in tests
global.console = {
  ...console,
  warn: vi.fn(),
  error: vi.fn(),
};

// Mock react-router-dom
vi.mock('react-router-dom', async () => {
  const actual = await vi.importActual('react-router-dom');
  return {
    ...actual,
    useNavigate: () => vi.fn(),
    useLocation: () => ({ pathname: '/' }),
    useParams: () => ({}),
  };
});

// Enhanced timeout for async operations - reduced for better test performance
vi.setConfig({ testTimeout: 5000 });

// Global React Query error handler mock to prevent unhandled promise rejections
global.addEventListener = vi.fn();
global.removeEventListener = vi.fn();

// Mock for React Query's window focus refetching
Object.defineProperty(document, 'hidden', {
  writable: true,
  value: false,
});

Object.defineProperty(document, 'visibilityState', {
  writable: true,
  value: 'visible',
});

// Mock for React Query's network status
Object.defineProperty(navigator, 'onLine', {
  writable: true,
  value: true,
});

// Mock matchMedia for react-hot-toast
Object.defineProperty(window, 'matchMedia', {
  writable: true,
  value: vi.fn().mockImplementation(query => ({
    matches: false,
    media: query,
    onchange: null,
    addListener: vi.fn(), // deprecated
    removeListener: vi.fn(), // deprecated
    addEventListener: vi.fn(),
    removeEventListener: vi.fn(),
    dispatchEvent: vi.fn(),
  })),
});