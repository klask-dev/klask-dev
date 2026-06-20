import React, { useEffect, useState } from 'react';
import { Navigate, useLocation } from 'react-router-dom';
import { useAuthStore } from '../../stores/auth-store';
import { FullPageSpinner } from '../ui/LoadingSpinner';

interface ProtectedRouteProps {
  children: React.ReactNode;
}

export const ProtectedRoute: React.FC<ProtectedRouteProps> = ({ children }) => {
  const user = useAuthStore((state) => state.user);
  const isAuthenticated = useAuthStore((state) => state.isAuthenticated);
  const isLoading = useAuthStore((state) => state.isLoading);
  const refreshUser = useAuthStore((state) => state.refreshUser);
  const location = useLocation();

  const [hasCheckedSession, setHasCheckedSession] = useState(false);

  useEffect(() => {
    // On initial mount, validate the session by trying to refresh user data
    // This ensures the HttpOnly cookie is still valid
    if (!hasCheckedSession && user && !isLoading) {
      setHasCheckedSession(true);
      refreshUser();
    } else if (!hasCheckedSession && !user) {
      // No cached user, mark as checked
      setHasCheckedSession(true);
    }
  }, []);

  if (isLoading || !hasCheckedSession) {
    return <FullPageSpinner message="Authenticating..." />;
  }

  if (!isAuthenticated) {
    // Redirect to login page with return url
    return (
      <Navigate
        to="/login"
        state={{ from: location }}
        replace
      />
    );
  }

  return <>{children}</>;
};