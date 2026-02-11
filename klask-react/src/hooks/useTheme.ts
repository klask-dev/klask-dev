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
import type { UserPreferences } from '../types';

export function useTheme() {
  const { currentTheme, setTheme } = useThemeContext();
  const { updateProfile } = useProfile();

  const updateTheme = useCallback(
    (newTheme: Theme, preferences?: UserPreferences) => {
      // Update theme in context (which updates DOM and localStorage)
      setTheme(newTheme);

      // Use provided preferences or fallback to user's preferences
      const currentPreferences = preferences || {
        language: 'en',
        notifications_email: true,
        show_activity: true,
        size_unit: 'kb',
      };

      // Save to backend via profile update, preserving other preferences
      updateProfile({
        preferences: {
          ...currentPreferences,
          theme: newTheme,
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
