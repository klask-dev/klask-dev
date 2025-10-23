import React from 'react';
import { BrowserRouter, Routes, Route } from 'react-router-dom';
import { QueryClientProvider } from '@tanstack/react-query';
import { ReactQueryDevtools } from '@tanstack/react-query-devtools';
import { Toaster } from 'react-hot-toast';

import { queryClient } from './lib/react-query';
import { initializeAuth } from './stores/auth-store';
import { AppLayout } from './components/layout/AppLayout';
import { ProtectedRoute } from './components/common/ProtectedRoute';
import { AdminRoute } from './components/common/AdminRoute';
import { SearchFiltersProvider } from './contexts/SearchFiltersContext';

// Lazy load pages for better performance
import { Suspense } from 'react';
import { LoadingSpinner } from './components/ui/LoadingSpinner';

const HomePage = React.lazy(() => import('./components/home/HomePage'));
const SearchPage = React.lazy(() => import('./features/search/SearchPage'));
const FileDetailPage = React.lazy(() => import('./features/files/FileDetailPage'));
const RepositoriesPage = React.lazy(() => import('./features/repositories/RepositoriesPage'));
const RepositoryDetailPage = React.lazy(() => import('./features/repositories/RepositoryDetailPage'));
const FileBrowserPage = React.lazy(() => import('./features/files/FileBrowserPage'));
const AdminDashboard = React.lazy(() => import('./features/admin/AdminDashboard'));
const UserManagement = React.lazy(() => import('./features/admin/UserManagement'));
const IndexManagement = React.lazy(() => import('./features/admin/IndexManagement'));
const LoginPage = React.lazy(() => import('./features/auth/LoginPage'));
const RegisterPage = React.lazy(() => import('./features/auth/RegisterPage'));
const ProfilePage = React.lazy(() => import('./features/auth/ProfilePage'));
const SetupPage = React.lazy(() => import('./features/auth/SetupPage'));
const SetupRedirect = React.lazy(() => import('./components/setup/SetupRedirect'));
const SyntaxHighlighterTest = React.lazy(() => import('./components/test/SyntaxHighlighterTest'));

// Initialize auth on app start
initializeAuth();

function App() {
  return (
    <QueryClientProvider client={queryClient}>
      <BrowserRouter>
        <div className="min-h-screen bg-gray-50">
          <Toaster
            position="top-right"
            toastOptions={{
              duration: 4000,
              style: {
                background: '#363636',
                color: '#fff',
              },
              success: {
                duration: 3000,
                iconTheme: {
                  primary: '#10b981',
                  secondary: '#fff',
                },
              },
              error: {
                duration: 5000,
                iconTheme: {
                  primary: '#ef4444',
                  secondary: '#fff',
                },
              },
            }}
          />
          
          <Routes>
            {/* Public routes */}
            <Route 
              path="/setup" 
              element={
                <Suspense fallback={<LoadingSpinner />}>
                  <SetupPage />
                </Suspense>
              } 
            />
            <Route 
              path="/login" 
              element={
                <Suspense fallback={<LoadingSpinner />}>
                  <LoginPage />
                </Suspense>
              } 
            />
            <Route 
              path="/register" 
              element={
                <Suspense fallback={<LoadingSpinner />}>
                  <RegisterPage />
                </Suspense>
              } 
            />
            
            {/* Protected routes with layout */}
            <Route
              path="/"
              element={
                <ProtectedRoute>
                  <SearchFiltersProvider>
                    <AppLayout />
                  </SearchFiltersProvider>
                </ProtectedRoute>
              }
            >
              {/* Default route - setup check and redirect */}
              <Route index element={
                <Suspense fallback={<LoadingSpinner />}>
                  <SetupRedirect />
                </Suspense>
              } />
              
              {/* Home route */}
              <Route 
                path="home" 
                element={
                  <Suspense fallback={<LoadingSpinner />}>
                    <HomePage />
                  </Suspense>
                } 
              />
              
              {/* Search routes */}
              <Route 
                path="search" 
                element={
                  <Suspense fallback={<LoadingSpinner />}>
                    <SearchPage />
                  </Suspense>
                } 
              />
              
              {/* File detail routes */}
              <Route
                path="files/:id"
                element={
                  <Suspense fallback={<LoadingSpinner />}>
                    <FileDetailPage />
                  </Suspense>
                }
              />
              <Route
                path="files/doc/:docAddress"
                element={
                  <Suspense fallback={<LoadingSpinner />}>
                    <FileDetailPage />
                  </Suspense>
                }
              />
              
              {/* Repository file browser routes */}
              <Route 
                path="repositories/:id/browse" 
                element={
                  <Suspense fallback={<LoadingSpinner />}>
                    <FileBrowserPage />
                  </Suspense>
                } 
              />
              
              {/* Profile route */}
              <Route 
                path="profile" 
                element={
                  <Suspense fallback={<LoadingSpinner />}>
                    <ProfilePage />
                  </Suspense>
                } 
              />
              
              {/* Test route for syntax highlighter */}
              <Route 
                path="test/syntax-highlighter" 
                element={
                  <Suspense fallback={<LoadingSpinner />}>
                    <SyntaxHighlighterTest />
                  </Suspense>
                } 
              />
              
              {/* Admin routes */}
              <Route 
                path="admin" 
                element={
                  <AdminRoute>
                    <Suspense fallback={<LoadingSpinner />}>
                      <AdminDashboard />
                    </Suspense>
                  </AdminRoute>
                } 
              />
              <Route 
                path="admin/users" 
                element={
                  <AdminRoute>
                    <Suspense fallback={<LoadingSpinner />}>
                      <UserManagement />
                    </Suspense>
                  </AdminRoute>
                } 
              />
              <Route 
                path="admin/repositories" 
                element={
                  <AdminRoute>
                    <Suspense fallback={<LoadingSpinner />}>
                      <RepositoriesPage />
                    </Suspense>
                  </AdminRoute>
                } 
              />
              <Route 
                path="admin/repositories/:id" 
                element={
                  <AdminRoute>
                    <Suspense fallback={<LoadingSpinner />}>
                      <RepositoryDetailPage />
                    </Suspense>
                  </AdminRoute>
                } 
              />
              <Route 
                path="admin/index" 
                element={
                  <AdminRoute>
                    <Suspense fallback={<LoadingSpinner />}>
                      <IndexManagement />
                    </Suspense>
                  </AdminRoute>
                } 
              />
            </Route>
            
            {/* 404 route */}
            <Route 
              path="*" 
              element={
                <div className="min-h-screen flex items-center justify-center">
                  <div className="text-center">
                    <h1 className="text-4xl font-bold text-gray-900 mb-4">404</h1>
                    <p className="text-gray-600 mb-8">Page not found</p>
                    <a 
                      href="/home" 
                      className="btn-primary"
                    >
                      Go to Home
                    </a>
                  </div>
                </div>
              } 
            />
          </Routes>
        </div>
      </BrowserRouter>
      
      {/* React Query DevTools (only in development) */}
      {import.meta.env.DEV && (
        <ReactQueryDevtools 
          initialIsOpen={false} 
        />
      )}
    </QueryClientProvider>
  );
}

export default App;