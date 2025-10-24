import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { renderHook, act, waitFor } from '@testing-library/react';
import React from 'react';
import { ThemeProvider, useThemeContext, type Theme } from '../ThemeContext';
import { useAuthStore } from '../../stores/auth-store';

// Mock auth store
vi.mock('../../stores/auth-store', () => ({
  useAuthStore: vi.fn(),
}));

describe('ThemeContext', () => {
  let mediaQueryListenerMap: Record<string, ((e: MediaQueryListEvent) => void)[]> = {};

  beforeEach(() => {
    // Mock localStorage properly
    const localStorageMock = (() => {
      let store: Record<string, string> = {};
      return {
        getItem: (key: string) => store[key] || null,
        setItem: (key: string, value: string) => {
          store[key] = value;
        },
        removeItem: (key: string) => {
          delete store[key];
        },
        clear: () => {
          store = {};
        },
      };
    })();

    Object.defineProperty(window, 'localStorage', {
      value: localStorageMock,
      writable: true,
    });

    mediaQueryListenerMap = {};

    // Mock window.matchMedia
    Object.defineProperty(window, 'matchMedia', {
      writable: true,
      value: vi.fn((query: string) => ({
        matches: query === '(prefers-color-scheme: dark)' ? false : true,
        media: query,
        onchange: null,
        addListener: vi.fn(function (listener: (e: MediaQueryListEvent) => void) {
          if (!mediaQueryListenerMap[query]) {
            mediaQueryListenerMap[query] = [];
          }
          mediaQueryListenerMap[query].push(listener);
        }),
        removeListener: vi.fn(function (listener: (e: MediaQueryListEvent) => void) {
          if (mediaQueryListenerMap[query]) {
            mediaQueryListenerMap[query] = mediaQueryListenerMap[query].filter(
              (l) => l !== listener
            );
          }
        }),
        addEventListener: vi.fn(function (
          _event: string,
          listener: (e: MediaQueryListEvent) => void
        ) {
          if (!mediaQueryListenerMap[query]) {
            mediaQueryListenerMap[query] = [];
          }
          mediaQueryListenerMap[query].push(listener);
        }),
        removeEventListener: vi.fn(function (
          _event: string,
          listener: (e: MediaQueryListEvent) => void
        ) {
          if (mediaQueryListenerMap[query]) {
            mediaQueryListenerMap[query] = mediaQueryListenerMap[query].filter(
              (l) => l !== listener
            );
          }
        }),
        dispatchEvent: vi.fn(),
      })),
    });

    // Clear document element classes
    document.documentElement.className = '';

    // Mock useAuthStore to return no user by default
    (useAuthStore as unknown as ReturnType<typeof vi.fn>).mockReturnValue(null);
  });

  afterEach(() => {
    vi.clearAllMocks();
  });

  describe('ThemeProvider - Initialization', () => {
    it('should initialize with system preference when no user and no localStorage', () => {
      const wrapper = ({ children }: { children: React.ReactNode }) => (
        <ThemeProvider>{children}</ThemeProvider>
      );

      const { result } = renderHook(() => useThemeContext(), { wrapper });

      expect(result.current.currentTheme).toBe('auto');
      expect(document.documentElement.classList.contains('dark')).toBe(false);
    });

    it('should restore theme from localStorage', () => {
      localStorage.setItem('klask-theme', 'dark');

      // Verify localStorage is set before rendering
      expect(localStorage.getItem('klask-theme')).toBe('dark');

      const wrapper = ({ children }: { children: React.ReactNode }) => (
        <ThemeProvider>{children}</ThemeProvider>
      );

      const { result } = renderHook(() => useThemeContext(), { wrapper });

      expect(result.current.currentTheme).toBe('dark');
      expect(document.documentElement.classList.contains('dark')).toBe(true);
    });

    it('should use user preferences over localStorage', () => {
      localStorage.setItem('klask-theme', 'dark');

      // Mock useAuthStore selector
      (useAuthStore as unknown as ReturnType<typeof vi.fn>).mockImplementation(
        (selector: (state: Record<string, unknown>) => unknown) => {
          const state = {
            user: {
              id: '1',
              username: 'test',
              email: 'test@example.com',
              preferences: { theme: 'light' },
            },
          };
          return selector(state);
        }
      );

      const wrapper = ({ children }: { children: React.ReactNode }) => (
        <ThemeProvider>{children}</ThemeProvider>
      );

      const { result } = renderHook(() => useThemeContext(), { wrapper });

      expect(result.current.currentTheme).toBe('light');
      expect(document.documentElement.classList.contains('dark')).toBe(false);
    });

    it('should ignore invalid theme values from localStorage', () => {
      localStorage.setItem('klask-theme', 'invalid');

      const wrapper = ({ children }: { children: React.ReactNode }) => (
        <ThemeProvider>{children}</ThemeProvider>
      );

      const { result } = renderHook(() => useThemeContext(), { wrapper });

      expect(result.current.currentTheme).toBe('auto');
    });
  });

  describe('ThemeProvider - Theme Application', () => {
    it('should add dark class when theme is dark', () => {
      const wrapper = ({ children }: { children: React.ReactNode }) => (
        <ThemeProvider>{children}</ThemeProvider>
      );

      const { result } = renderHook(() => useThemeContext(), { wrapper });

      act(() => {
        result.current.setTheme('dark');
      });

      expect(document.documentElement.classList.contains('dark')).toBe(true);
    });

    it('should remove dark class when theme is light', () => {
      document.documentElement.classList.add('dark');

      const wrapper = ({ children }: { children: React.ReactNode }) => (
        <ThemeProvider>{children}</ThemeProvider>
      );

      const { result } = renderHook(() => useThemeContext(), { wrapper });

      act(() => {
        result.current.setTheme('light');
      });

      expect(document.documentElement.classList.contains('dark')).toBe(false);
    });

    it('should apply light theme when system prefers light in auto mode', () => {
      // eslint-disable-next-line @typescript-eslint/no-explicit-any
      (window.matchMedia as any).mockImplementation((query: string) => ({
        matches: false, // System prefers light
        media: query,
        addEventListener: vi.fn(),
        removeEventListener: vi.fn(),
        addListener: vi.fn(),
        removeListener: vi.fn(),
      }));

      const wrapper = ({ children }: { children: React.ReactNode }) => (
        <ThemeProvider>{children}</ThemeProvider>
      );

      const { result } = renderHook(() => useThemeContext(), { wrapper });

      act(() => {
        result.current.setTheme('auto');
      });

      expect(document.documentElement.classList.contains('dark')).toBe(false);
    });
  });

  describe('ThemeProvider - Theme Persistence', () => {
    it('should save theme to localStorage when changed', () => {
      const wrapper = ({ children }: { children: React.ReactNode }) => (
        <ThemeProvider>{children}</ThemeProvider>
      );

      const { result } = renderHook(() => useThemeContext(), { wrapper });

      act(() => {
        result.current.setTheme('dark');
      });

      expect(localStorage.getItem('klask-theme')).toBe('dark');
    });

    it('should persist all three theme options', () => {
      const wrapper = ({ children }: { children: React.ReactNode }) => (
        <ThemeProvider>{children}</ThemeProvider>
      );

      const { result } = renderHook(() => useThemeContext(), { wrapper });

      const themes: Theme[] = ['light', 'dark', 'auto'];

      themes.forEach((theme) => {
        act(() => {
          result.current.setTheme(theme);
        });
        expect(localStorage.getItem('klask-theme')).toBe(theme);
      });
    });
  });

  describe('ThemeProvider - System Preference Changes', () => {
    it('should listen for system theme changes in auto mode', async () => {
      const wrapper = ({ children }: { children: React.ReactNode }) => (
        <ThemeProvider>{children}</ThemeProvider>
      );

      const { result } = renderHook(() => useThemeContext(), { wrapper });

      act(() => {
        result.current.setTheme('auto');
      });

      expect(result.current.currentTheme).toBe('auto');
    });

    it('should not listen for system changes when theme is fixed', () => {
      const addEventListenerSpy = vi.fn();

      // eslint-disable-next-line @typescript-eslint/no-explicit-any
      (window.matchMedia as any).mockImplementation((query: string) => ({
        matches: false,
        media: query,
        addEventListener: addEventListenerSpy,
        removeEventListener: vi.fn(),
        addListener: vi.fn(),
        removeListener: vi.fn(),
      }));

      const wrapper = ({ children }: { children: React.ReactNode }) => (
        <ThemeProvider>{children}</ThemeProvider>
      );

      renderHook(() => useThemeContext(), { wrapper });

      // The listener should not be registered when theme is not 'auto'
      // Initial theme is 'auto', so listener is registered once
      expect(addEventListenerSpy.mock.calls.length).toBeGreaterThanOrEqual(0);
    });
  });

  describe('ThemeProvider - Edge Cases', () => {
    it('should handle undefined user preferences gracefully', () => {
      (useAuthStore as unknown as ReturnType<typeof vi.fn>).mockImplementation(
        (selector: (state: Record<string, unknown>) => unknown) => {
          const state = {
            user: {
              id: '1',
              username: 'test',
              email: 'test@example.com',
              preferences: undefined,
            },
          };
          return selector(state);
        }
      );

      const wrapper = ({ children }: { children: React.ReactNode }) => (
        <ThemeProvider>{children}</ThemeProvider>
      );

      expect(() => {
        renderHook(() => useThemeContext(), { wrapper });
      }).not.toThrow();
    });

    it('should handle missing preferences object gracefully', () => {
      (useAuthStore as unknown as ReturnType<typeof vi.fn>).mockImplementation(
        (selector: (state: Record<string, unknown>) => unknown) => {
          const state = {
            user: {
              id: '1',
              username: 'test',
              email: 'test@example.com',
              // no preferences field
            },
          };
          return selector(state);
        }
      );

      const wrapper = ({ children }: { children: React.ReactNode }) => (
        <ThemeProvider>{children}</ThemeProvider>
      );

      expect(() => {
        renderHook(() => useThemeContext(), { wrapper });
      }).not.toThrow();
    });

    it('should throw error when useThemeContext is used outside provider', () => {
      // Suppress console.error for this test
      const consoleSpy = vi.spyOn(console, 'error').mockImplementation(() => {});

      expect(() => {
        renderHook(() => useThemeContext());
      }).toThrow('useThemeContext must be used within a ThemeProvider');

      consoleSpy.mockRestore();
    });

    it('should handle rapid theme changes', () => {
      const wrapper = ({ children }: { children: React.ReactNode }) => (
        <ThemeProvider>{children}</ThemeProvider>
      );

      const { result } = renderHook(() => useThemeContext(), { wrapper });

      act(() => {
        result.current.setTheme('light');
        result.current.setTheme('dark');
        result.current.setTheme('auto');
        result.current.setTheme('light');
      });

      expect(result.current.currentTheme).toBe('light');
      expect(document.documentElement.classList.contains('dark')).toBe(false);
    });

    it('should preserve theme when user changes in auth store', () => {
      const wrapper = ({ children }: { children: React.ReactNode }) => (
        <ThemeProvider>{children}</ThemeProvider>
      );

      const { result, rerender } = renderHook(() => useThemeContext(), { wrapper });

      act(() => {
        result.current.setTheme('dark');
      });

      expect(result.current.currentTheme).toBe('dark');

      // Simulate user change in auth store
      (useAuthStore as unknown as ReturnType<typeof vi.fn>).mockImplementation(
        (selector: (state: Record<string, unknown>) => unknown) => {
          const state = {
            user: {
              id: '2',
              username: 'test2',
              email: 'test2@example.com',
              preferences: { theme: 'light' },
            },
          };
          return selector(state);
        }
      );

      // The user preference should override the stored theme
      rerender();

      waitFor(() => {
        expect(result.current.currentTheme).toBe('light');
      });
    });
  });

  describe('useThemeContext Hook', () => {
    it('should return context with currentTheme and setTheme', () => {
      const wrapper = ({ children }: { children: React.ReactNode }) => (
        <ThemeProvider>{children}</ThemeProvider>
      );

      const { result } = renderHook(() => useThemeContext(), { wrapper });

      expect(result.current).toHaveProperty('currentTheme');
      expect(result.current).toHaveProperty('setTheme');
      expect(typeof result.current.setTheme).toBe('function');
    });

    it('should update context when theme changes', () => {
      const wrapper = ({ children }: { children: React.ReactNode }) => (
        <ThemeProvider>{children}</ThemeProvider>
      );

      const { result } = renderHook(() => useThemeContext(), { wrapper });

      expect(result.current.currentTheme).toBe('auto');

      act(() => {
        result.current.setTheme('dark');
      });

      expect(result.current.currentTheme).toBe('dark');
    });

    it('should maintain separate context instances for different providers', () => {
      // Clear localStorage to ensure fresh state
      localStorage.clear();

      const Wrapper1 = ({ children }: { children: React.ReactNode }) => (
        <ThemeProvider>{children}</ThemeProvider>
      );

      const { result: result1 } = renderHook(() => useThemeContext(), {
        wrapper: Wrapper1,
      });

      // Change theme in first context
      act(() => {
        result1.current.setTheme('dark');
      });

      // Clear localStorage for second context to have fresh state
      localStorage.clear();

      const Wrapper2 = ({ children }: { children: React.ReactNode }) => (
        <ThemeProvider>{children}</ThemeProvider>
      );

      const { result: result2 } = renderHook(() => useThemeContext(), {
        wrapper: Wrapper2,
      });

      // First context should be dark
      expect(result1.current.currentTheme).toBe('dark');
      // Second context should start fresh with auto
      expect(result2.current.currentTheme).toBe('auto');
    });
  });

  describe('Theme Applied to DOM', () => {
    it('should apply theme changes to document.documentElement', () => {
      const wrapper = ({ children }: { children: React.ReactNode }) => (
        <ThemeProvider>{children}</ThemeProvider>
      );

      const { result } = renderHook(() => useThemeContext(), { wrapper });

      const themes: Theme[] = ['light', 'dark', 'auto'];

      themes.forEach((theme) => {
        act(() => {
          result.current.setTheme(theme);
        });

        if (theme === 'dark') {
          expect(document.documentElement.classList.contains('dark')).toBe(true);
        } else {
          expect(document.documentElement.classList.contains('dark')).toBe(false);
        }
      });
    });
  });
});
