/* eslint-disable react-refresh/only-export-components */
/**
 * ThemeContext - Manages global theme state and application to DOM
 *
 * Features:
 * - Supports light, dark, and auto (system preference) themes
 * - Applies theme via document.documentElement.classList
 * - Syncs with user preferences from auth store
 * - Persists theme preference to localStorage
 * - Auto-detects system preference changes when in 'auto' mode
 */
import React, { createContext, useContext, useEffect, useState, useCallback } from 'react';
import { useAuthStore } from '../stores/auth-store';

export type Theme = 'light' | 'dark' | 'auto';

export interface ThemeContextType {
  currentTheme: Theme;
  setTheme: (theme: Theme) => void;
}

const ThemeContext = createContext<ThemeContextType | undefined>(undefined);

export const useThemeContext = () => {
  const context = useContext(ThemeContext);
  if (!context) {
    throw new Error('useThemeContext must be used within a ThemeProvider');
  }
  return context;
};

interface ThemeProviderProps {
  children: React.ReactNode;
}

export const ThemeProvider: React.FC<ThemeProviderProps> = ({ children }) => {
  const user = useAuthStore((state) => state.user);
  const [currentTheme, setCurrentTheme] = useState<Theme>('auto');
  const [isMounted, setIsMounted] = useState(false);

  // Function to detect system preference
  const getSystemPreference = useCallback((): 'light' | 'dark' => {
    if (typeof window === 'undefined') return 'light';
    return window.matchMedia('(prefers-color-scheme: dark)').matches ? 'dark' : 'light';
  }, []);

  // Function to resolve the actual theme to apply (handles 'auto')
  const resolveTheme = useCallback((theme: Theme): 'light' | 'dark' => {
    if (theme === 'auto') {
      return getSystemPreference();
    }
    return theme;
  }, [getSystemPreference]);

  // Function to apply theme to DOM
  const applyTheme = useCallback((theme: Theme) => {
    const resolvedTheme = resolveTheme(theme);
    const root = document.documentElement;

    if (resolvedTheme === 'dark') {
      root.classList.add('dark');
    } else {
      root.classList.remove('dark');
    }
  }, [resolveTheme]);

  // Initialize theme from user preferences or localStorage
  useEffect(() => {
    let themeToUse: Theme = 'auto';

    // Priority 1: User preferences from auth store
    if (user?.preferences?.theme) {
      themeToUse = user.preferences.theme;
    } else {
      // Priority 2: LocalStorage (fallback if auth store empty)
      const savedTheme = localStorage.getItem('klask-theme') as Theme | null;
      if (savedTheme && ['light', 'dark', 'auto'].includes(savedTheme)) {
        themeToUse = savedTheme;
      }
    }

    setCurrentTheme(themeToUse);
    applyTheme(themeToUse);
    setIsMounted(true);
  }, [user?.preferences?.theme, applyTheme]);

  // Handle theme changes from user (via the useTheme hook)
  const handleSetTheme = useCallback((newTheme: Theme) => {
    setCurrentTheme(newTheme);
    applyTheme(newTheme);
    localStorage.setItem('klask-theme', newTheme);
  }, [applyTheme]);

  // Listen for system preference changes when in 'auto' mode
  useEffect(() => {
    if (!isMounted || currentTheme !== 'auto') return;

    const mediaQuery = window.matchMedia('(prefers-color-scheme: dark)');

    const handleChange = () => {
      applyTheme('auto');
    };

    // Modern browsers support addEventListener
    if (mediaQuery.addEventListener) {
      mediaQuery.addEventListener('change', handleChange);
    } else {
      // Fallback for older browsers
      mediaQuery.addListener(handleChange);
    }

    return () => {
      if (mediaQuery.removeEventListener) {
        mediaQuery.removeEventListener('change', handleChange);
      } else {
        mediaQuery.removeListener(handleChange);
      }
    };
  }, [currentTheme, isMounted, applyTheme]);

  const value: ThemeContextType = {
    currentTheme,
    setTheme: handleSetTheme,
  };

  return (
    <ThemeContext.Provider value={value}>
      {children}
    </ThemeContext.Provider>
  );
};
