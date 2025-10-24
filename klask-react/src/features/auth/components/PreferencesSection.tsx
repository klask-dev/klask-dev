import React, { useState } from 'react';
import toast from 'react-hot-toast';
import type { UserPreferences } from '../../../types';
import { useProfile } from '../../../hooks/useProfile';
import { useTheme } from '../../../hooks/useTheme';

const PreferencesSection: React.FC = () => {
  const { user, updateProfile, isUpdating } = useProfile();
  const { updateTheme } = useTheme();
  const [preferences, setPreferences] = useState<UserPreferences>(
    user?.preferences || {
      theme: 'auto',
      language: 'en',
      notifications_email: true,
      show_activity: true,
    }
  );

  const [isSaving, setIsSaving] = useState(false);

  const handleThemeChange = (theme: 'light' | 'dark' | 'auto') => {
    setPreferences({ ...preferences, theme });
    // Apply theme immediately for visual feedback
    updateTheme(theme);
  };

  const handleLanguageChange = (language: 'en' | 'fr' | 'es' | 'de') => {
    setPreferences({ ...preferences, language });
  };

  const handleToggle = (key: 'notifications_email' | 'show_activity') => {
    setPreferences({ ...preferences, [key]: !preferences[key] });
  };

  const handleSave = async () => {
    setIsSaving(true);
    try {
      updateProfile({ preferences });
      toast.success('Preferences saved successfully');
    } catch {
      toast.error('Failed to save preferences');
    } finally {
      setIsSaving(false);
    }
  };

  return (
    <div className="space-y-6">
      {/* Theme Preference */}
      <div>
        <h3 className="text-lg font-medium text-gray-900 dark:text-white mb-4">Theme</h3>
        <div className="space-y-3">
          {(['light', 'dark', 'auto'] as const).map((theme) => (
            <label key={theme} className="flex items-center cursor-pointer">
              <input
                type="radio"
                name="theme"
                value={theme}
                checked={preferences.theme === theme}
                onChange={() => handleThemeChange(theme)}
                disabled={isUpdating || isSaving}
                className="w-4 h-4 text-blue-600 cursor-pointer"
              />
              <span className="ml-3 text-gray-700 dark:text-gray-300">
                {theme.charAt(0).toUpperCase() + theme.slice(1)}
              </span>
              {theme === 'auto' && (
                <span className="ml-2 text-xs text-gray-500 dark:text-gray-400">(follow system)</span>
              )}
            </label>
          ))}
        </div>
      </div>

      <div className="border-t border-gray-200 dark:border-gray-700 pt-6">
        {/* Language Preference */}
        <div>
          <h3 className="text-lg font-medium text-gray-900 dark:text-white mb-4">Language</h3>
          <select
            value={preferences.language || 'en'}
            onChange={(e) => handleLanguageChange(e.target.value as 'en' | 'fr' | 'es' | 'de')}
            disabled={isUpdating || isSaving}
            className="w-full max-w-md px-4 py-2 border border-gray-300 dark:border-gray-600 bg-white dark:bg-gray-800 text-gray-900 dark:text-white rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-transparent outline-none transition disabled:opacity-50 disabled:cursor-not-allowed"
          >
            <option value="en">English</option>
            <option value="fr">Francais</option>
            <option value="es">Espanol</option>
            <option value="de">Deutsch</option>
          </select>
        </div>
      </div>

      <div className="border-t border-gray-200 dark:border-gray-700 pt-6">
        {/* Notification Preferences */}
        <div className="space-y-4">
          <h3 className="text-lg font-medium text-gray-900 dark:text-white">Notifications</h3>

          <div className="flex items-center justify-between p-4 bg-gray-50 dark:bg-gray-800 rounded-lg">
            <div>
              <p className="font-medium text-gray-900 dark:text-white">Email Notifications</p>
              <p className="text-sm text-gray-600 dark:text-gray-400">
                Receive email notifications for important updates
              </p>
            </div>
            <button
              onClick={() => handleToggle('notifications_email')}
              disabled={isUpdating || isSaving}
              className={`relative inline-flex h-6 w-11 flex-shrink-0 cursor-pointer rounded-full border-2 border-transparent transition-colors focus:outline-none focus:ring-2 focus:ring-blue-500 focus:ring-offset-2 dark:focus:ring-offset-gray-900 ${
                preferences.notifications_email ? 'bg-blue-600' : 'bg-gray-300 dark:bg-gray-600'
              }`}
            >
              <span
                className={`pointer-events-none inline-block h-5 w-5 transform rounded-full bg-white shadow ring-0 transition duration-200 ease-in-out ${
                  preferences.notifications_email ? 'translate-x-4' : 'translate-x-0'
                }`}
              />
            </button>
          </div>

          <div className="flex items-center justify-between p-4 bg-gray-50 dark:bg-gray-800 rounded-lg">
            <div>
              <p className="font-medium text-gray-900 dark:text-white">Activity Visibility</p>
              <p className="text-sm text-gray-600 dark:text-gray-400">
                Allow other users to see your activity status
              </p>
            </div>
            <button
              onClick={() => handleToggle('show_activity')}
              disabled={isUpdating || isSaving}
              className={`relative inline-flex h-6 w-11 flex-shrink-0 cursor-pointer rounded-full border-2 border-transparent transition-colors focus:outline-none focus:ring-2 focus:ring-blue-500 focus:ring-offset-2 dark:focus:ring-offset-gray-900 ${
                preferences.show_activity ? 'bg-blue-600' : 'bg-gray-300 dark:bg-gray-600'
              }`}
            >
              <span
                className={`pointer-events-none inline-block h-5 w-5 transform rounded-full bg-white shadow ring-0 transition duration-200 ease-in-out ${
                  preferences.show_activity ? 'translate-x-4' : 'translate-x-0'
                }`}
              />
            </button>
          </div>
        </div>
      </div>

      {/* Save Button */}
      <div className="border-t border-gray-200 dark:border-gray-700 pt-6 flex justify-end">
        <button
          onClick={handleSave}
          disabled={isUpdating || isSaving}
          className="px-6 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700 disabled:bg-gray-400 dark:disabled:bg-gray-600 disabled:cursor-not-allowed transition flex items-center gap-2"
        >
          {isSaving && (
            <div className="w-4 h-4 border-2 border-white border-t-transparent rounded-full animate-spin" />
          )}
          {isSaving ? 'Saving...' : 'Save Preferences'}
        </button>
      </div>
    </div>
  );
};

export default PreferencesSection;
