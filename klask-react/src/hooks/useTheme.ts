/**
 * useTheme - Hook for theme management
 *
 * Provides:
 * - Current theme state
 * - updateTheme function to change theme with automatic:
 *   - DOM updates
 *   - localStorage persistence
 *   - Backend profile update
 */
import { useCallback } from 'react';
import { useThemeContext, type Theme } from '../contexts/ThemeContext';
import { useProfile } from './useProfile';

export function useTheme() {
  const { currentTheme, setTheme } = useThemeContext();
  const { updateProfile } = useProfile();

  const updateTheme = useCallback(
    (newTheme: Theme) => {
      // Update theme in context (which updates DOM and localStorage)
      setTheme(newTheme);

      // Save to backend via profile update
      updateProfile({
        preferences: {
          theme: newTheme,
          language: 'en', // These will be filled from current user data by the mutation
          notifications_email: true,
          show_activity: true,
          size_unit: 'kb', // Default size unit
        },
      });
    },
    [setTheme, updateProfile]
  );

  return {
    currentTheme,
    updateTheme,
  };
}
