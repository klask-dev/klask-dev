import { create } from 'zustand';
import { persist } from 'zustand/middleware';
import type { User } from '../types';
import { UserRole } from '../types';
import { apiClient } from '../lib/api';

interface AuthState {
  user: User | null;
  token: string | null;
  isAuthenticated: boolean;
  isLoading: boolean;
  
  // Actions
  setUser: (user: User | null) => void;
  setToken: (token: string | null) => void;
  login: (token: string, user: User) => void;
  logout: () => void;
  refreshUser: () => Promise<void>;
  checkTokenValidity: () => boolean;
  clearAuth: () => void;
}

export const useAuthStore = create<AuthState>()(
  persist(
    (set, get) => ({
      user: null,
      token: null,
      isAuthenticated: false,
      isLoading: false,

      setUser: (user) => {
        set({ user, isAuthenticated: !!user });
      },

      setToken: (token) => {
        set({ token });
        apiClient.setToken(token);
        
        // Decode token to check validity
        if (token) {
          const isValid = get().checkTokenValidity();
          if (!isValid) {
            get().logout();
          }
        }
      },

      login: (token, user) => {
        set({
          token,
          user,
          isAuthenticated: true,
          isLoading: false,
        });
        apiClient.setToken(token);
      },

      logout: async () => {
        // Clear the HttpOnly cookie via server-side logout endpoint.
        await apiClient.auth.logout();
        set({
          user: null,
          token: null,
          isAuthenticated: false,
          isLoading: false,
        });
        // Clean up any leftover localStorage keys from the old auth scheme.
        localStorage.removeItem('authToken');
        localStorage.removeItem('csrfToken');
      },

      refreshUser: async () => {
        // Cookie-based auth: no token needed to refresh — browser sends the cookie.
        try {
          set({ isLoading: true });
          const user = await apiClient.auth.getProfile();
          set({ user, isAuthenticated: true });
        } catch (error) {
          console.error('Failed to refresh user:', error);
          get().logout();
        } finally {
          set({ isLoading: false });
        }
      },

      checkTokenValidity: () => {
        // With cookie-based auth the token is not accessible from JS.
        // Return true if user info is present (session validated on rehydrate).
        const { user } = get();
        return !!user;
      },

      clearAuth: () => {
        // Best-effort: ask server to clear the cookie.
        apiClient.auth.logout().catch(() => {/* ignore */});
        set({
          user: null,
          token: null,
          isAuthenticated: false,
          isLoading: false,
        });
        localStorage.removeItem('authToken');
        localStorage.removeItem('csrfToken');
      },
    }),
    {
      name: 'klask-auth',
      // Do NOT persist the token: the browser stores the HttpOnly cookie.
      // Only persist user info for instant UI rendering on load.
      partialize: (state) => ({ 
        user: state.user 
      }),
      onRehydrateStorage: () => (state) => {
        if (state?.user) {
          // Validate the session by refreshing user data from the server.
          // The HttpOnly cookie is sent automatically by the browser.
          state.refreshUser();
        }
      },
    }
  )
);

// Selectors for convenient access to auth state (use getState() — safe outside React components)
export const authSelectors = {
  isAuthenticated: () => useAuthStore.getState().isAuthenticated,
  user: () => useAuthStore.getState().user,
  token: () => useAuthStore.getState().token,
  isLoading: () => useAuthStore.getState().isLoading,
  isAdmin: () => useAuthStore.getState().user?.role === UserRole.ADMIN,
  hasRole: (role: UserRole) => useAuthStore.getState().user?.role === role,
};

// Initialize auth on app start
export const initializeAuth = () => {
  // Clean up any leftover localStorage data from the old cookie-less auth scheme.
  localStorage.removeItem('authToken');
  localStorage.removeItem('csrfToken');
};