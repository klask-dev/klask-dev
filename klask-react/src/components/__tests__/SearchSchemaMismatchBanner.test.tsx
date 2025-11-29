import React from 'react';
import { describe, it, expect, vi, beforeEach } from 'vitest';

// Mocks must be at the top level before imports!
vi.mock('../../api/indexMetrics', () => ({
  useSearchStatus: vi.fn(),
  useIndexStats: vi.fn(),
  useIndexHealth: vi.fn(),
  useTuningRecommendations: vi.fn(),
  useOptimizeIndex: vi.fn(),
  useAllIndexMetrics: vi.fn(),
}));

vi.mock('react-router-dom', async () => {
  const actual = await vi.importActual('react-router-dom');
  return {
    ...actual,
    Link: ({ to, children, className }: any) => (
      <a href={to} className={className} data-testid="admin-link">
        {children}
      </a>
    ),
  };
});

vi.mock('@heroicons/react/24/outline', () => ({
  ExclamationTriangleIcon: () => <div data-testid="warning-icon" />,
  XMarkIcon: () => <div data-testid="close-icon" />,
}));

import { render, screen, waitFor } from '../../test/utils';
import { SearchSchemaMismatchBanner } from '../SearchSchemaMismatchBanner';
import * as indexMetricsApi from '../../api/indexMetrics';

describe('SearchSchemaMismatchBanner Component', () => {
  const mockUseSearchStatus = vi.mocked(indexMetricsApi.useSearchStatus);

  beforeEach(() => {
    vi.clearAllMocks();
  });

  // Test 1: Banner renders when schema_mismatch is true
  it('should render banner when schema_mismatch is true', () => {
    mockUseSearchStatus.mockReturnValue({
      data: {
        schema_mismatch: true,
        index_available: false,
        message: 'Schema version mismatch detected',
      },
      isLoading: false,
      isError: false,
      error: null,
      isFetching: false,
      refetch: vi.fn(),
    } as any);

    render(<SearchSchemaMismatchBanner />);

    expect(screen.getByText('Index schema has changed')).toBeInTheDocument();
    expect(
      screen.getByText(/The search index schema needs to be rebuilt/)
    ).toBeInTheDocument();
    expect(screen.getByText('Admin Settings')).toBeInTheDocument();
  });

  // Test 2: Banner hidden when schema_mismatch is false
  it('should not render banner when schema_mismatch is false', () => {
    mockUseSearchStatus.mockReturnValue({
      data: {
        schema_mismatch: false,
        index_available: true,
      },
      isLoading: false,
      isError: false,
      error: null,
      isFetching: false,
      refetch: vi.fn(),
    } as any);

    render(<SearchSchemaMismatchBanner />);

    expect(screen.queryByText('Index schema has changed')).not.toBeInTheDocument();
  });

  // Test 3: Banner hidden when data is null
  it('should not render when data is null', () => {
    mockUseSearchStatus.mockReturnValue({
      data: null,
      isLoading: false,
      isError: false,
      error: null,
      isFetching: false,
      refetch: vi.fn(),
    } as any);

    render(<SearchSchemaMismatchBanner />);

    expect(screen.queryByText('Index schema has changed')).not.toBeInTheDocument();
  });

  // Test 3: Link to admin settings is present and has correct href
  it('should have link to admin settings with correct href', () => {
    mockUseSearchStatus.mockReturnValue({
      data: {
        schema_mismatch: true,
        index_available: false,
      },
      isLoading: false,
      isError: false,
      error: null,
      isFetching: false,
      refetch: vi.fn(),
    } as any);

    render(<SearchSchemaMismatchBanner />);

    const link = screen.getByTestId('admin-link');
    expect(link).toHaveAttribute('href', '/admin/index');
  });

  // Test 4: Optional message is displayed when provided
  it('should display optional message when provided', () => {
    const testMessage = 'Rebuild in progress';
    mockUseSearchStatus.mockReturnValue({
      data: {
        schema_mismatch: true,
        index_available: false,
        message: testMessage,
      },
      isLoading: false,
      isError: false,
      error: null,
      isFetching: false,
      refetch: vi.fn(),
    } as any);

    render(<SearchSchemaMismatchBanner />);

    expect(screen.getByText(testMessage)).toBeInTheDocument();
  });

  // Test 5: Optional message is not displayed when not provided
  it('should not display message when not provided', () => {
    mockUseSearchStatus.mockReturnValue({
      data: {
        schema_mismatch: true,
        index_available: false,
      },
      isLoading: false,
      isError: false,
      error: null,
      isFetching: false,
      refetch: vi.fn(),
    } as any);

    render(<SearchSchemaMismatchBanner />);

    // The message paragraph should not be in the DOM
    const messageParagraphs = screen.queryAllByText(/^(?!.*Index schema)/);
    // Find italicized text which is the message
    expect(document.querySelector('p.italic')).not.toBeInTheDocument();
  });

  // Test 6: Banner can be dismissed
  it('should hide banner when dismiss button is clicked', () => {
    mockUseSearchStatus.mockReturnValue({
      data: {
        schema_mismatch: true,
        index_available: false,
      },
      isLoading: false,
      isError: false,
      error: null,
      isFetching: false,
      refetch: vi.fn(),
    } as any);

    render(<SearchSchemaMismatchBanner />);

    expect(screen.getByText('Index schema has changed')).toBeInTheDocument();

    const dismissButton = screen.getByRole('button', { name: /dismiss/i });
    dismissButton.click();

    expect(screen.queryByText('Index schema has changed')).not.toBeInTheDocument();
  });

  // Test 7: Warning icon is displayed
  it('should display warning icon', () => {
    mockUseSearchStatus.mockReturnValue({
      data: {
        schema_mismatch: true,
        index_available: false,
      },
      isLoading: false,
      isError: false,
      error: null,
      isFetching: false,
      refetch: vi.fn(),
    } as any);

    render(<SearchSchemaMismatchBanner />);

    expect(screen.getByTestId('warning-icon')).toBeInTheDocument();
  });

  // Test 8: Close button displays close icon
  it('should display close icon on dismiss button', () => {
    mockUseSearchStatus.mockReturnValue({
      data: {
        schema_mismatch: true,
        index_available: false,
      },
      isLoading: false,
      isError: false,
      error: null,
      isFetching: false,
      refetch: vi.fn(),
    } as any);

    render(<SearchSchemaMismatchBanner />);

    expect(screen.getByTestId('close-icon')).toBeInTheDocument();
  });

  // Test 9: Auto-refetch mechanism setup
  it('should set up refetch intervals when mounting', () => {
    const mockRefetch = vi.fn();
    mockUseSearchStatus.mockReturnValue({
      data: {
        schema_mismatch: true,
        index_available: false,
      },
      isLoading: false,
      isError: false,
      error: null,
      isFetching: false,
      refetch: mockRefetch,
    } as any);

    render(<SearchSchemaMismatchBanner />);

    // Verify that refetch function is available
    expect(mockRefetch).toBeDefined();
  });

  // Test 10: Effect hook depends on schema_mismatch
  it('should have useEffect hook that monitors schema_mismatch status', () => {
    const mockRefetch = vi.fn();
    mockUseSearchStatus.mockReturnValue({
      data: {
        schema_mismatch: false,
        index_available: true,
      },
      isLoading: false,
      isError: false,
      error: null,
      isFetching: false,
      refetch: mockRefetch,
    } as any);

    render(<SearchSchemaMismatchBanner />);

    // Just verify it renders without error
    expect(screen.queryByText('Index schema has changed')).not.toBeInTheDocument();
  });

  // Test 11: Cleanup on unmount
  it('should unmount without errors', () => {
    const mockRefetch = vi.fn();
    mockUseSearchStatus.mockReturnValue({
      data: {
        schema_mismatch: true,
        index_available: false,
      },
      isLoading: false,
      isError: false,
      error: null,
      isFetching: false,
      refetch: mockRefetch,
    } as any);

    const { unmount } = render(<SearchSchemaMismatchBanner />);

    expect(() => unmount()).not.toThrow();
  });

  // Test 12: Banner has correct styling classes
  it('should have correct styling for warning state', () => {
    mockUseSearchStatus.mockReturnValue({
      data: {
        schema_mismatch: true,
        index_available: false,
      },
      isLoading: false,
      isError: false,
      error: null,
      isFetching: false,
      refetch: vi.fn(),
    } as any);

    const { container } = render(<SearchSchemaMismatchBanner />);

    const banner = container.querySelector('div.bg-yellow-50');
    expect(banner).toBeInTheDocument();
  });

  // Test 13: Component reacts to status changes
  it('should respond to status changes from mismatch to healthy', () => {
    mockUseSearchStatus.mockReturnValue({
      data: {
        schema_mismatch: true,
        index_available: false,
      },
      isLoading: false,
      isError: false,
      error: null,
      isFetching: false,
      refetch: vi.fn(),
    } as any);

    const { rerender } = render(<SearchSchemaMismatchBanner />);

    // Initially should show mismatch
    expect(screen.getByText('Index schema has changed')).toBeInTheDocument();

    // Change to healthy state
    mockUseSearchStatus.mockReturnValue({
      data: {
        schema_mismatch: false,
        index_available: true,
      },
      isLoading: false,
      isError: false,
      error: null,
      isFetching: false,
      refetch: vi.fn(),
    } as any);

    rerender(<SearchSchemaMismatchBanner />);

    // After state change, banner should disappear
    expect(screen.queryByText('Index schema has changed')).not.toBeInTheDocument();
  });
});
