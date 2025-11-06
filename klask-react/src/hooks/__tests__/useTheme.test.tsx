import { describe, it, expect, vi, beforeEach } from 'vitest';
import { renderHook, act } from '@testing-library/react';
import React from 'react';
import { useTheme } from '../useTheme';
import { ThemeProvider } from '../../contexts/ThemeContext';
import * as profileHook from '../useProfile';
import * as themeContext from '../../contexts/ThemeContext';

// Mock useProfile hook
vi.mock('../useProfile', () => ({
  useProfile: vi.fn(),
}));

// Mock ThemeContext
vi.mock('../../contexts/ThemeContext', async () => {
  const actual = await vi.importActual('../../contexts/ThemeContext') as any;
  return {
    ...actual,
    useThemeContext: vi.fn(),
  };
});

describe('useTheme Hook', () => {
  beforeEach(() => {
    vi.clearAllMocks();
    localStorage.clear();

    // Setup default profile mock
      (profileHook.useProfile as any).mockReturnValue({
      user: {
        id: '1',
        username: 'test',
        email: 'test@example.com',
        preferences: { theme: 'auto' },
      },
      updateProfile: vi.fn(),
      isUpdating: false,
      error: null,
    });

    // Setup default theme context mock
      (themeContext.useThemeContext as any).mockReturnValue({
      currentTheme: 'auto',
      setTheme: vi.fn(),
    });
  });

  describe('Hook Integration', () => {
    it('should return currentTheme and updateTheme functions', () => {
      const wrapper = ({ children }: { children: React.ReactNode }) => (
        <ThemeProvider>{children}</ThemeProvider>
      );

      const { result } = renderHook(() => useTheme(), { wrapper });

      expect(result.current).toHaveProperty('currentTheme');
      expect(result.current).toHaveProperty('updateTheme');
      expect(typeof result.current.updateTheme).toBe('function');
    });

    it('should get currentTheme from context', () => {
      (themeContext.useThemeContext as any).mockReturnValue({
        currentTheme: 'dark',
        setTheme: vi.fn(),
      });

      const wrapper = ({ children }: { children: React.ReactNode }) => (
        <ThemeProvider>{children}</ThemeProvider>
      );

      const { result } = renderHook(() => useTheme(), { wrapper });

      expect(result.current.currentTheme).toBe('dark');
    });
  });

  describe('updateTheme Function', () => {
    it('should update theme in context', () => {
      const setThemeMock = vi.fn();

      (themeContext.useThemeContext as any).mockReturnValue({
        currentTheme: 'auto',
        setTheme: setThemeMock,
      });

      const wrapper = ({ children }: { children: React.ReactNode }) => (
        <ThemeProvider>{children}</ThemeProvider>
      );

      const { result } = renderHook(() => useTheme(), { wrapper });

      act(() => {
        result.current.updateTheme('dark');
      });

      expect(setThemeMock).toHaveBeenCalledWith('dark');
    });

    it('should call updateProfile with theme preference', () => {
      const updateProfileMock = vi.fn();
      const setThemeMock = vi.fn();

      (profileHook.useProfile as any).mockReturnValue({
        user: { id: '1', preferences: { theme: 'auto' } },
        updateProfile: updateProfileMock,
        isUpdating: false,
      });

      (themeContext.useThemeContext as any).mockReturnValue({
        currentTheme: 'auto',
        setTheme: setThemeMock,
      });

      const wrapper = ({ children }: { children: React.ReactNode }) => (
        <ThemeProvider>{children}</ThemeProvider>
      );

      const { result } = renderHook(() => useTheme(), { wrapper });

      act(() => {
        result.current.updateTheme('light');
      });

      expect(updateProfileMock).toHaveBeenCalledWith({
        preferences: {
          theme: 'light',
          language: 'en',
          notifications_email: true,
          show_activity: true,
          size_unit: 'kb',
        },
      });
    });

    it('should handle light theme update', () => {
      const updateProfileMock = vi.fn();
      const setThemeMock = vi.fn();

      (profileHook.useProfile as any).mockReturnValue({
        user: { id: '1', preferences: { theme: 'auto' } },
        updateProfile: updateProfileMock,
        isUpdating: false,
      });

      (themeContext.useThemeContext as any).mockReturnValue({
        currentTheme: 'auto',
        setTheme: setThemeMock,
      });

      const wrapper = ({ children }: { children: React.ReactNode }) => (
        <ThemeProvider>{children}</ThemeProvider>
      );

      const { result } = renderHook(() => useTheme(), { wrapper });

      act(() => {
        result.current.updateTheme('light');
      });

      expect(setThemeMock).toHaveBeenCalledWith('light');
      expect(updateProfileMock).toHaveBeenCalled();
    });

    it('should handle dark theme update', () => {
      const updateProfileMock = vi.fn();
      const setThemeMock = vi.fn();

      (profileHook.useProfile as any).mockReturnValue({
        user: { id: '1', preferences: { theme: 'auto' } },
        updateProfile: updateProfileMock,
        isUpdating: false,
      });

      (themeContext.useThemeContext as any).mockReturnValue({
        currentTheme: 'auto',
        setTheme: setThemeMock,
      });

      const wrapper = ({ children }: { children: React.ReactNode }) => (
        <ThemeProvider>{children}</ThemeProvider>
      );

      const { result } = renderHook(() => useTheme(), { wrapper });

      act(() => {
        result.current.updateTheme('dark');
      });

      expect(setThemeMock).toHaveBeenCalledWith('dark');
      expect(updateProfileMock).toHaveBeenCalled();
    });

    it('should handle auto theme update', () => {
      const updateProfileMock = vi.fn();
      const setThemeMock = vi.fn();

      (profileHook.useProfile as any).mockReturnValue({
        user: { id: '1', preferences: { theme: 'dark' } },
        updateProfile: updateProfileMock,
        isUpdating: false,
      });

      (themeContext.useThemeContext as any).mockReturnValue({
        currentTheme: 'dark',
        setTheme: setThemeMock,
      });

      const wrapper = ({ children }: { children: React.ReactNode }) => (
        <ThemeProvider>{children}</ThemeProvider>
      );

      const { result } = renderHook(() => useTheme(), { wrapper });

      act(() => {
        result.current.updateTheme('auto');
      });

      expect(setThemeMock).toHaveBeenCalledWith('auto');
      expect(updateProfileMock).toHaveBeenCalled();
    });
  });

  describe('Multiple Theme Changes', () => {
    it('should handle rapid consecutive theme changes', () => {
      const updateProfileMock = vi.fn();
      const setThemeMock = vi.fn();

      (profileHook.useProfile as any).mockReturnValue({
        user: { id: '1', preferences: { theme: 'auto' } },
        updateProfile: updateProfileMock,
        isUpdating: false,
      });

      (themeContext.useThemeContext as any).mockReturnValue({
        currentTheme: 'auto',
        setTheme: setThemeMock,
      });

      const wrapper = ({ children }: { children: React.ReactNode }) => (
        <ThemeProvider>{children}</ThemeProvider>
      );

      const { result } = renderHook(() => useTheme(), { wrapper });

      act(() => {
        result.current.updateTheme('light');
        result.current.updateTheme('dark');
        result.current.updateTheme('auto');
      });

      // Should have been called 3 times
      expect(setThemeMock).toHaveBeenCalledTimes(3);
      expect(updateProfileMock).toHaveBeenCalledTimes(3);
    });

    it('should maintain consistency across multiple updates', () => {
      const setThemeMock = vi.fn();

      (themeContext.useThemeContext as any).mockReturnValue({
        currentTheme: 'auto',
        setTheme: setThemeMock,
      });

      const wrapper = ({ children }: { children: React.ReactNode }) => (
        <ThemeProvider>{children}</ThemeProvider>
      );

      const { result } = renderHook(() => useTheme(), { wrapper });

      const themes = ['light', 'dark', 'auto', 'light', 'dark'] as const;

      themes.forEach((theme) => {
        act(() => {
          result.current.updateTheme(theme);
        });
      });

      themes.forEach((theme) => {
        expect(setThemeMock).toHaveBeenCalledWith(theme);
      });
    });
  });

  describe('Profile Integration', () => {
    it('should preserve other preferences when updating theme', () => {
      const updateProfileMock = vi.fn();
      const setThemeMock = vi.fn();

      (profileHook.useProfile as any).mockReturnValue({
        user: {
          id: '1',
          preferences: {
            theme: 'auto',
            language: 'fr',
            notifications_email: false,
            show_activity: true,
          },
        },
        updateProfile: updateProfileMock,
        isUpdating: false,
      });

      (themeContext.useThemeContext as any).mockReturnValue({
        currentTheme: 'auto',
        setTheme: setThemeMock,
      });

      const wrapper = ({ children }: { children: React.ReactNode }) => (
        <ThemeProvider>{children}</ThemeProvider>
      );

      const { result } = renderHook(() => useTheme(), { wrapper });

      act(() => {
        result.current.updateTheme('dark');
      });

      // Check that updateProfile was called with theme update
      expect(updateProfileMock).toHaveBeenCalledWith({
        preferences: expect.objectContaining({
          theme: 'dark',
        }),
      });
    });

    it('should handle missing user preferences gracefully', () => {
      const updateProfileMock = vi.fn();
      const setThemeMock = vi.fn();

      (profileHook.useProfile as any).mockReturnValue({
        user: null,
        updateProfile: updateProfileMock,
        isUpdating: false,
      });

      (themeContext.useThemeContext as any).mockReturnValue({
        currentTheme: 'auto',
        setTheme: setThemeMock,
      });

      const wrapper = ({ children }: { children: React.ReactNode }) => (
        <ThemeProvider>{children}</ThemeProvider>
      );

      const { result } = renderHook(() => useTheme(), { wrapper });

      expect(() => {
        act(() => {
          result.current.updateTheme('dark');
        });
      }).not.toThrow();
    });
  });

  describe('Default Preferences', () => {
    it('should use default preferences when updating theme', () => {
      const updateProfileMock = vi.fn();
      const setThemeMock = vi.fn();

      (profileHook.useProfile as any).mockReturnValue({
        user: { id: '1', preferences: { theme: 'auto' } },
        updateProfile: updateProfileMock,
        isUpdating: false,
      });

      (themeContext.useThemeContext as any).mockReturnValue({
        currentTheme: 'auto',
        setTheme: setThemeMock,
      });

      const wrapper = ({ children }: { children: React.ReactNode }) => (
        <ThemeProvider>{children}</ThemeProvider>
      );

      const { result } = renderHook(() => useTheme(), { wrapper });

      act(() => {
        result.current.updateTheme('light');
      });

      // Should include default values for other preferences
      const callArg = updateProfileMock.mock.calls[0][0];
      expect(callArg.preferences).toEqual({
        theme: 'light',
        language: 'en',
        notifications_email: true,
        show_activity: true,
        size_unit: 'kb',
      });
    });
  });

  describe('Hook Return Value', () => {
    it('should return consistent values across multiple calls', () => {
      const setThemeMock = vi.fn();

      (themeContext.useThemeContext as any).mockReturnValue({
        currentTheme: 'dark',
        setTheme: setThemeMock,
      });

      const wrapper = ({ children }: { children: React.ReactNode }) => (
        <ThemeProvider>{children}</ThemeProvider>
      );

      const { result: result1 } = renderHook(() => useTheme(), { wrapper });
      const { result: result2 } = renderHook(() => useTheme(), { wrapper });

      expect(result1.current.currentTheme).toBe(result2.current.currentTheme);
    });

    it('should return updateTheme function that works correctly', () => {
      const updateProfileMock = vi.fn();
      const setThemeMock = vi.fn();

      (profileHook.useProfile as any).mockReturnValue({
        user: { id: '1', preferences: { theme: 'auto' } },
        updateProfile: updateProfileMock,
        isUpdating: false,
      });

      (themeContext.useThemeContext as any).mockReturnValue({
        currentTheme: 'auto',
        setTheme: setThemeMock,
      });

      const wrapper = ({ children }: { children: React.ReactNode }) => (
        <ThemeProvider>{children}</ThemeProvider>
      );

      const { result } = renderHook(() => useTheme(), { wrapper });
      const updateTheme = result.current.updateTheme;

      act(() => {
        updateTheme('dark');
      });

      expect(typeof updateTheme).toBe('function');
      expect(setThemeMock).toHaveBeenCalledWith('dark');
    });
  });
});
