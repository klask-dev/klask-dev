# Klask Theme System Documentation

## Overview

The Klask theme system provides a complete, production-ready solution for managing light, dark, and automatic theme preferences. The system integrates seamlessly with user profiles, localStorage for persistence, and system preference detection.

## Architecture

### Components

#### 1. ThemeContext (`/src/contexts/ThemeContext.tsx`)

The core context provider that manages global theme state and application of themes to the DOM.

**Key Features:**
- Supports three theme modes: `'light'`, `'dark'`, and `'auto'`
- Applies themes to `document.documentElement` by adding/removing the `dark` CSS class
- Persists theme preference to localStorage
- Detects system dark mode preference using `window.matchMedia('(prefers-color-scheme: dark)')`
- Listens for system preference changes when in `'auto'` mode
- Syncs with user preferences from the auth store
- Handles gracefully when user authentication state changes

**Context Type:**
```typescript
interface ThemeContextType {
  currentTheme: Theme;  // 'light' | 'dark' | 'auto'
  setTheme: (theme: Theme) => void;
}
```

**Initialization Priority:**
1. User preferences from auth store (highest priority)
2. localStorage value
3. System preference (default)

**API:**
```typescript
export const useThemeContext = () => {
  const context = useContext(ThemeContext);
  if (!context) {
    throw new Error('useThemeContext must be used within a ThemeProvider');
  }
  return context;
};
```

#### 2. useTheme Hook (`/src/hooks/useTheme.ts`)

A custom hook for consuming and updating theme preferences with automatic backend synchronization.

**Returns:**
```typescript
{
  currentTheme: Theme;  // Current effective theme
  updateTheme: (theme: Theme) => void;  // Function to update theme
}
```

**Features:**
- Updates theme in context (DOM and localStorage)
- Automatically syncs theme changes to backend via `updateProfile()`
- Preserves other user preferences when updating theme
- Handles errors gracefully with toast notifications

**Usage:**
```typescript
const { currentTheme, updateTheme } = useTheme();

// Get current theme
console.log(currentTheme);  // 'light' | 'dark' | 'auto'

// Update theme
updateTheme('dark');  // Also persists to backend
```

#### 3. PreferencesSection Component (`/src/features/auth/components/PreferencesSection.tsx`)

UI component for managing user preferences including theme selection.

**Features:**
- Radio buttons for theme selection (light, dark, auto)
- Label for 'auto' mode explaining it follows system preference
- Immediate visual feedback when changing theme
- Persistence to backend via profile update
- Disabled state during update operations

## How It Works

### Initialization Flow

1. App loads and mounts `ThemeProvider` at the top level
2. ThemeProvider initializes:
   - Gets user from auth store (may be null on first load)
   - Checks user.preferences.theme
   - Falls back to localStorage value
   - Falls back to system preference
3. Theme is applied to `document.documentElement`
4. MediaQuery listener is set up if theme is 'auto'

### Theme Application

The theme is applied by manipulating the document element's class list:

```typescript
// Apply dark theme
document.documentElement.classList.add('dark');

// Apply light theme
document.documentElement.classList.remove('dark');
```

TailwindCSS is configured to use `darkMode: 'class'`, which means:
- Dark mode styles are applied when `.dark` class is on the root element
- All dark variants (e.g., `dark:bg-gray-900`) become active

### System Preference Detection

When theme is set to 'auto':
1. `window.matchMedia('(prefers-color-scheme: dark)')` is used to detect system preference
2. A listener is registered for `'change'` events
3. When system preference changes, theme is automatically re-applied
4. If user changes theme away from 'auto', listener is removed

### Persistence

Theme preference is persisted in two places:

1. **localStorage**: For immediate visual feedback before API response
   ```typescript
   localStorage.setItem('klask-theme', 'dark');
   ```

2. **Backend**: Via `updateProfile()` mutation
   ```typescript
   updateProfile({
     preferences: {
       theme: 'dark',
       language: 'en',
       notifications_email: true,
       show_activity: true,
     },
   });
   ```

### User Profile Integration

When user preferences are fetched or updated:
- Theme preference is included in the `UserPreferences` interface
- Theme is a required field with default value 'auto'
- Other preferences are preserved when updating theme

## Type Definitions

### Theme Type
```typescript
type Theme = 'light' | 'dark' | 'auto';
```

### UserPreferences Interface
```typescript
interface UserPreferences {
  theme: 'light' | 'dark' | 'auto';
  language: 'en' | 'fr' | 'es' | 'de';
  notifications_email: boolean;
  show_activity: boolean;
}
```

## TailwindCSS Configuration

The project's `tailwind.config.js` is configured with:
```javascript
{
  darkMode: 'class',
  // ... rest of config
}
```

This enables dark mode based on CSS class presence rather than system preference (the class is managed by ThemeContext).

## Testing

Comprehensive test suites are included:

### ThemeContext Tests (`/src/contexts/__tests__/ThemeContext.test.tsx`)
- 20 tests covering all functionality
- Tests for initialization, theme application, persistence
- System preference change detection
- Edge cases and error handling
- localStorage and auth store synchronization

### useTheme Hook Tests (`/src/hooks/__tests__/useTheme.test.tsx`)
- 14 tests covering hook behavior
- Profile update integration
- Multiple theme changes
- Default preference handling
- Error scenarios

**Run tests:**
```bash
npm test -- Theme --run
```

## Edge Cases Handled

1. **Undefined User Preferences**: Gracefully defaults to 'auto' if user preferences are missing
2. **System Preference Changes**: Automatically applies new theme when system preference changes in 'auto' mode
3. **localStorage Corruption**: Invalid themes are ignored and reset to 'auto'
4. **Auth Store Changes**: Theme preference is updated when user logs in/out
5. **Rapid Theme Changes**: All changes are applied correctly even with rapid consecutive updates
6. **Outside Provider Context**: Clear error message when useThemeContext is used outside ThemeProvider

## Integration Example

Complete theme system integration in an app:

```typescript
// App.tsx
import { ThemeProvider } from './contexts/ThemeContext';
import { QueryClientProvider } from '@tanstack/react-query';

export default function App() {
  return (
    <QueryClientProvider client={queryClient}>
      <BrowserRouter>
        <ThemeProvider>
          <div className="min-h-screen bg-gray-50 dark:bg-gray-950">
            {/* Your app routes here */}
          </div>
        </ThemeProvider>
      </BrowserRouter>
    </QueryClientProvider>
  );
}
```

Usage in components:

```typescript
// In any component
import { useTheme } from '@/hooks/useTheme';

export function ThemeToggle() {
  const { currentTheme, updateTheme } = useTheme();

  return (
    <div>
      <p>Current theme: {currentTheme}</p>
      <button onClick={() => updateTheme('light')}>Light</button>
      <button onClick={() => updateTheme('dark')}>Dark</button>
      <button onClick={() => updateTheme('auto')}>Auto</button>
    </div>
  );
}
```

Styling with dark mode:

```typescript
// Dark mode styles are automatically applied
export function Component() {
  return (
    <div className="bg-white dark:bg-gray-900 text-gray-900 dark:text-white">
      Content
    </div>
  );
}
```

## API Integration

The theme system integrates with the user profile API:

**Update Profile Endpoint:**
- `PATCH /api/user/profile`
- Accepts `UpdateProfileRequest` with optional `preferences` field
- Returns updated `User` object with preferences

**Get Profile Endpoint:**
- `GET /api/user/profile`
- Returns `User` object including `preferences.theme`

## Browser Compatibility

- Requires `window.matchMedia` support (all modern browsers)
- Uses modern event listener methods with fallback to legacy `addListener`
- localStorage support required
- CSS custom properties not required (uses Tailwind classes)

## Performance Considerations

1. **Theme Switching**: O(1) operation - just manipulates DOM class
2. **localStorage Access**: Synchronous but fast
3. **API Calls**: Debounced in practice via React Query's mutation caching
4. **System Preference Listener**: Minimal overhead - only registered in 'auto' mode
5. **Re-renders**: Only components using `useThemeContext` or `useTheme` re-render on change

## Future Enhancements

Potential improvements:
1. CSS transition animation when switching themes
2. Per-component theme override capability
3. Custom theme colors beyond light/dark
4. Theme change history/undo
5. Schedule-based theme switching (e.g., dark at night)

## Troubleshooting

### Theme Not Persisting
- Check localStorage is enabled: `console.log(localStorage.getItem('klask-theme'))`
- Verify user preferences are being updated in backend
- Check auth store is properly initialized

### Theme Not Applying
- Ensure ThemeProvider wraps your entire app
- Verify Tailwind's `darkMode: 'class'` is configured
- Check for CSS conflicts overriding dark mode styles

### System Preference Not Detected
- Verify browser supports `window.matchMedia`
- Check OS dark mode is properly enabled
- Theme must be set to 'auto' for system preference to apply

## References

- [TailwindCSS Dark Mode](https://tailwindcss.com/docs/dark-mode)
- [MDN: prefers-color-scheme](https://developer.mozilla.org/en-US/docs/Web/CSS/@media/prefers-color-scheme)
- [MDN: Window.matchMedia()](https://developer.mozilla.org/en-US/docs/Web/API/Window/matchMedia)
- [React Context API](https://react.dev/reference/react/useContext)
