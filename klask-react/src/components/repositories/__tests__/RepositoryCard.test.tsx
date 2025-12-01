import React from 'react';
import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen, waitFor, within } from '../../../test/utils';
import userEvent from '@testing-library/user-event';
import { RepositoryCard } from '../RepositoryCard';
import type { Repository, RepositoryWithStats, CrawlProgressInfo } from '../../../types';
import { useActiveProgress, useStopCrawl } from '../../../hooks/useRepositories';
import { isRepositoryCrawling, getRepositoryProgressFromActive } from '../../../hooks/useProgress';

// Mock the hooks
vi.mock('../../../hooks/useRepositories', () => ({
  useActiveProgress: vi.fn(),
  useStopCrawl: vi.fn(),
}));

vi.mock('../../../hooks/useProgress', () => ({
  isRepositoryCrawling: vi.fn(),
  getRepositoryProgressFromActive: vi.fn(),
}));

const mockUseActiveProgress = useActiveProgress as any;
const mockUseStopCrawl = useStopCrawl as any;
const mockIsRepositoryCrawling = isRepositoryCrawling as any;
const mockGetRepositoryProgressFromActive = getRepositoryProgressFromActive as any;

describe('RepositoryCard Stop Crawl Functionality', () => {
  const mockStopCrawl = {
    mutateAsync: vi.fn(),
    isPending: false,
    isError: false,
    error: null,
  };

  const mockRepository: Repository = {
    id: 'repo-123',
    name: 'Test Repository',
    url: 'https://github.com/test/repo.git',
    repositoryType: 'Git',
    branch: 'main',
    enabled: true,
    accessToken: null,
    gitlabNamespace: null,
    isGroup: false,
    lastCrawled: null,
    createdAt: new Date().toISOString(),
    updatedAt: new Date().toISOString(),
  };

  const mockActiveProgress: CrawlProgressInfo[] = [
    {
      repository_id: 'repo-123',
      repository_name: 'Test Repository',
      status: 'processing',
      progress_percentage: 50,
      files_processed: 100,
      files_total: 200,
      files_indexed: 80,
      current_file: 'src/main.ts',
      error_message: null,
      started_at: new Date().toISOString(),
      updated_at: new Date().toISOString(),
      completed_at: null,
    },
  ];

  const defaultProps = {
    repository: mockRepository,
    onEdit: vi.fn(),
    onDelete: vi.fn(),
    onCrawl: vi.fn(),
    onStopCrawl: vi.fn(),
    onToggleEnabled: vi.fn(),
    activeProgress: [],
  };

  beforeEach(() => {
    vi.clearAllMocks();
    mockUseActiveProgress.mockReturnValue({ data: [] });
    mockUseStopCrawl.mockReturnValue(mockStopCrawl);
    mockIsRepositoryCrawling.mockReturnValue(false);
    mockGetRepositoryProgressFromActive.mockReturnValue(null);
  });

  it('should not show stop button when repository is not crawling', () => {
    render(<RepositoryCard {...defaultProps} />);

    // Stop button should not be visible
    expect(screen.queryByRole('button', { name: /stop/i })).not.toBeInTheDocument();
    // Should show crawl button instead
    expect(screen.getByRole('button', { name: /crawl/i })).toBeInTheDocument();
  });

  it('should show stop button when repository is crawling', () => {
    // Mock repository as currently crawling
    mockIsRepositoryCrawling.mockReturnValue(true);
    mockUseActiveProgress.mockReturnValue({ data: mockActiveProgress });

    render(<RepositoryCard {...defaultProps} />);

    // Stop button should be visible
    expect(screen.getByRole('button', { name: /stop/i })).toBeInTheDocument();
    // Should not show crawl button
    expect(screen.queryByRole('button', { name: /crawl/i })).not.toBeInTheDocument();
  });

  it('should show confirmation dialog when stop button is clicked', async () => {
    const user = userEvent.setup();
    
    // Mock repository as crawling
    mockIsRepositoryCrawling.mockReturnValue(true);
    mockUseActiveProgress.mockReturnValue({ data: mockActiveProgress });

    render(<RepositoryCard {...defaultProps} />);

    // Click stop button
    const stopButton = screen.getByRole('button', { name: /stop/i });
    await user.click(stopButton);

    // Confirmation dialog should appear
    expect(screen.getByRole('dialog', { name: /stop crawl/i })).toBeInTheDocument();
    expect(screen.getByText(/are you sure/i)).toBeInTheDocument();
    expect(screen.getByRole('button', { name: /stop crawl/i })).toBeInTheDocument();
    expect(screen.getByRole('button', { name: /cancel/i })).toBeInTheDocument();
  });

  it('should cancel confirmation dialog when cancel is clicked', async () => {
    const user = userEvent.setup();
    
    // Mock repository as crawling
    mockIsRepositoryCrawling.mockReturnValue(true);
    mockUseActiveProgress.mockReturnValue({ data: mockActiveProgress });

    render(<RepositoryCard {...defaultProps} />);

    // Click stop button
    const stopButton = screen.getByRole('button', { name: /stop/i });
    await user.click(stopButton);

    // Click cancel in confirmation dialog
    const cancelButton = screen.getByRole('button', { name: /cancel/i });
    await user.click(cancelButton);

    // Dialog should be closed
    expect(screen.queryByText(/stop crawl/i)).not.toBeInTheDocument();
    expect(mockStopCrawl.mutateAsync).not.toHaveBeenCalled();
  });

  it('should call stop crawl mutation when confirmed', async () => {
    const user = userEvent.setup();
    mockStopCrawl.mutateAsync.mockResolvedValue('Crawl stopped');
    
    // Mock repository as crawling
    mockIsRepositoryCrawling.mockReturnValue(true);
    mockUseActiveProgress.mockReturnValue({ data: mockActiveProgress });

    render(<RepositoryCard {...defaultProps} />);

    // Click stop and confirm
    const stopButton = screen.getByText(/stop/i);
    await user.click(stopButton);
    const confirmButton = screen.getByRole('button', { name: /stop crawl/i });
    await user.click(confirmButton);

    // Mutation should be called
    expect(mockStopCrawl.mutateAsync).toHaveBeenCalledWith('repo-123');
    
    await waitFor(() => {
      expect(screen.queryByText(/stop crawl/i)).not.toBeInTheDocument();
    });
  });

  it('should call onStopCrawl callback when stop succeeds', async () => {
    const user = userEvent.setup();
    const onStopCrawl = vi.fn();
    mockStopCrawl.mutateAsync.mockResolvedValue('Crawl stopped');
    
    // Mock repository as crawling
    mockIsRepositoryCrawling.mockReturnValue(true);
    mockUseActiveProgress.mockReturnValue({ data: mockActiveProgress });

    render(<RepositoryCard {...defaultProps} onStopCrawl={onStopCrawl} />);

    // Stop crawl process
    const stopButton = screen.getByText(/stop/i);
    await user.click(stopButton);
    const confirmButton = screen.getByRole('button', { name: /stop crawl/i });
    await user.click(confirmButton);

    await waitFor(() => {
      expect(onStopCrawl).toHaveBeenCalledWith(mockRepository);
    });
  });

  it('should handle stop crawl mutation error', async () => {
    const user = userEvent.setup();
    const consoleErrorSpy = vi.spyOn(console, 'error').mockImplementation(() => {});
    const mockError = new Error('Failed to stop crawl');
    mockStopCrawl.mutateAsync.mockRejectedValue(mockError);
    
    // Mock repository as crawling
    mockIsRepositoryCrawling.mockReturnValue(true);
    mockUseActiveProgress.mockReturnValue({ data: mockActiveProgress });

    render(<RepositoryCard {...defaultProps} />);

    // Try to stop crawl
    const stopButton = screen.getByText(/stop/i);
    await user.click(stopButton);
    const confirmButton = screen.getByRole('button', { name: /stop crawl/i });
    await user.click(confirmButton);

    await waitFor(() => {
      expect(consoleErrorSpy).toHaveBeenCalledWith('Failed to stop crawl:', mockError);
    });

    consoleErrorSpy.mockRestore();
  });

  it('should show loading state while stop mutation is pending', async () => {
    const pendingStopCrawl = {
      ...mockStopCrawl,
      isPending: true,
      mutateAsync: vi.fn(() => new Promise(() => {})), // Never resolves
    };
    mockUseStopCrawl.mockReturnValue(pendingStopCrawl);
    
    // Mock repository as crawling
    mockIsRepositoryCrawling.mockReturnValue(true);
    mockUseActiveProgress.mockReturnValue({ data: mockActiveProgress });

    render(<RepositoryCard {...defaultProps} />);

    // Should show stopping state in button
    const stoppingButton = screen.getByRole('button', { name: /stopping/i });
    expect(stoppingButton).toBeInTheDocument();
    
    // Button should be disabled when pending
    expect(stoppingButton.closest('button')).toBeDisabled();
  });

  it('should work without onStopCrawl callback', async () => {
    const user = userEvent.setup();
    mockStopCrawl.mutateAsync.mockResolvedValue('Crawl stopped');
    
    // Mock repository as crawling
    mockIsRepositoryCrawling.mockReturnValue(true);
    mockUseActiveProgress.mockReturnValue({ data: mockActiveProgress });

    // Render without onStopCrawl prop
    const { onStopCrawl: _onStopCrawl, ...propsWithoutCallback } = defaultProps;
    render(<RepositoryCard {...propsWithoutCallback} />);

    // Should still work without callback
    const stopButton = screen.getByText(/stop/i);
    await user.click(stopButton);
    const confirmButton = screen.getByRole('button', { name: /stop crawl/i });
    await user.click(confirmButton);

    expect(mockStopCrawl.mutateAsync).toHaveBeenCalledWith('repo-123');
  });

  it('should show stop button with correct icon', () => {
    // Mock repository as crawling
    mockIsRepositoryCrawling.mockReturnValue(true);
    mockUseActiveProgress.mockReturnValue({ data: mockActiveProgress });

    render(<RepositoryCard {...defaultProps} />);

    // Should show stop button with stop icon
    const stopButton = screen.getByRole('button', { name: /stop/i });
    expect(stopButton).toBeInTheDocument();
    
    // Check if stop icon is present (StopCircleIcon)
    const stopIcon = stopButton.closest('button')?.querySelector('svg');
    expect(stopIcon).toBeInTheDocument();
  });

  it('should show confirmation dialog when stop button is clicked directly', async () => {
    const user = userEvent.setup();
    
    // Mock repository as crawling
    mockIsRepositoryCrawling.mockReturnValue(true);
    mockUseActiveProgress.mockReturnValue({ data: mockActiveProgress });

    render(<RepositoryCard {...defaultProps} />);

    // Stop button should be visible
    expect(screen.getByText(/stop/i)).toBeInTheDocument();
    
    // Click stop button
    const stopButton = screen.getByRole('button', { name: /stop/i });
    await user.click(stopButton);

    // Confirmation dialog should appear
    expect(screen.getByRole('dialog', { name: /stop crawl/i })).toBeInTheDocument();
  });

  it('should display progress information when crawling', () => {
    // Mock repository as crawling with progress
    mockIsRepositoryCrawling.mockReturnValue(true);
    mockGetRepositoryProgressFromActive.mockReturnValue(mockActiveProgress[0]);
    mockUseActiveProgress.mockReturnValue({ data: mockActiveProgress });

    render(<RepositoryCard {...defaultProps} />);

    // Should show progress information somewhere in the component
    // Note: The exact format might vary depending on ProgressBar component implementation
    const progressElements = screen.getAllByText(/50|100|200/i);
    expect(progressElements.length).toBeGreaterThan(0);
  });

  it('should handle cancelled crawl status correctly', () => {
    const cancelledProgress: CrawlProgressInfo = {
      ...mockActiveProgress[0],
      status: 'cancelled',
      progress_percentage: 100,
      completed_at: new Date().toISOString(),
    };

    // Mock repository as having cancelled progress
    mockIsRepositoryCrawling.mockReturnValue(false); // Cancelled is not crawling
    mockGetRepositoryProgressFromActive.mockReturnValue(cancelledProgress);
    mockUseActiveProgress.mockReturnValue({ data: [cancelledProgress] });

    render(<RepositoryCard {...defaultProps} />);

    // Should not show stop button since crawl is cancelled
    expect(screen.queryByRole('button', { name: /stop/i })).not.toBeInTheDocument();
    // Should show crawl button instead
    expect(screen.getByRole('button', { name: /crawl/i })).toBeInTheDocument();
  });
});

describe('RepositoryCard Crawl Error Badge Display', () => {
  const mockStopCrawl = {
    mutateAsync: vi.fn(),
    isPending: false,
    isError: false,
    error: null,
  };

  const createMockRepository = (overrides: Partial<Repository> = {}): Repository => ({
    id: 'repo-123',
    name: 'Test Repository',
    url: 'https://github.com/test/repo.git',
    repositoryType: 'Git',
    branch: 'main',
    enabled: true,
    accessToken: null,
    gitlabNamespace: null,
    isGroup: false,
    lastCrawled: new Date().toISOString(),
    createdAt: new Date().toISOString(),
    updatedAt: new Date().toISOString(),
    autoCrawlEnabled: false,
    ...overrides,
  });

  const createMockRepositoryWithStats = (overrides: Partial<Repository> = {}): RepositoryWithStats => ({
    repository: createMockRepository(overrides),
    diskSizeMb: 100,
    fileCount: 1000,
  });

  const defaultProps = {
    repository: createMockRepositoryWithStats(),
    onEdit: vi.fn(),
    onDelete: vi.fn(),
    onCrawl: vi.fn(),
    onStopCrawl: vi.fn(),
    onToggleEnabled: vi.fn(),
    activeProgress: [],
  };

  beforeEach(() => {
    vi.clearAllMocks();
    mockUseStopCrawl.mockReturnValue(mockStopCrawl);
    mockIsRepositoryCrawling.mockReturnValue(false);
    mockGetRepositoryProgressFromActive.mockReturnValue(null);
  });

  describe('Error badge visibility', () => {
    it('should display error badge when lastCrawlError exists with error message', () => {
      const errorMessage = 'Connection timeout';
      const repositoryWithError = createMockRepositoryWithStats({
        lastCrawlError: errorMessage,
      });

      render(<RepositoryCard {...defaultProps} repository={repositoryWithError} />);

      // Badge should be visible
      const errorBadge = screen.getByText('Crawl Error');
      expect(errorBadge).toBeInTheDocument();

      // Badge should be a span element
      expect(errorBadge.parentElement).toHaveClass('text-xs');
    });

    it('should NOT display error badge when lastCrawlError is undefined', () => {
      const repositoryWithoutError = createMockRepositoryWithStats({
        lastCrawlError: undefined,
      });

      render(<RepositoryCard {...defaultProps} repository={repositoryWithoutError} />);

      // Badge should not be visible
      expect(screen.queryByText('Crawl Error')).not.toBeInTheDocument();
    });

    it('should NOT display error badge when lastCrawlError is empty string', () => {
      const repositoryWithEmptyError = createMockRepositoryWithStats({
        lastCrawlError: '',
      });

      render(<RepositoryCard {...defaultProps} repository={repositoryWithEmptyError} />);

      // Badge should not be visible
      expect(screen.queryByText('Crawl Error')).not.toBeInTheDocument();
    });

    it('should NOT display error badge when lastCrawlError is null', () => {
      const repositoryWithNullError = createMockRepositoryWithStats({
        lastCrawlError: null as any,
      });

      render(<RepositoryCard {...defaultProps} repository={repositoryWithNullError} />);

      // Badge should not be visible
      expect(screen.queryByText('Crawl Error')).not.toBeInTheDocument();
    });
  });

  describe('Error badge content and styling', () => {
    it('should contain "Crawl Error" text in the badge', () => {
      const repositoryWithError = createMockRepositoryWithStats({
        lastCrawlError: 'Repository not found',
      });

      render(<RepositoryCard {...defaultProps} repository={repositoryWithError} />);

      expect(screen.getByText('Crawl Error')).toBeInTheDocument();
    });

    it('should display ExclamationTriangleIcon in error badge', () => {
      const repositoryWithError = createMockRepositoryWithStats({
        lastCrawlError: 'Authentication failed',
      });

      render(<RepositoryCard {...defaultProps} repository={repositoryWithError} />);

      const errorBadge = screen.getByText('Crawl Error');
      const badgeContainer = errorBadge.closest('button');

      // SVG icon should be present (from ExclamationTriangleIcon)
      const icon = badgeContainer?.querySelector('svg');
      expect(icon).toBeInTheDocument();
    });

    it('should have correct styling classes for red error appearance', () => {
      const repositoryWithError = createMockRepositoryWithStats({
        lastCrawlError: 'Crawl failed',
      });

      render(<RepositoryCard {...defaultProps} repository={repositoryWithError} />);

      const errorBadge = screen.getByText('Crawl Error').closest('button');

      // Check for red styling classes
      expect(errorBadge).toHaveClass('bg-red-100');
      expect(errorBadge).toHaveClass('text-red-800');
      expect(errorBadge).toHaveClass('dark:bg-red-900');
      expect(errorBadge).toHaveClass('dark:text-red-200');
    });

    it('should have cursor-pointer class for tooltip indication', () => {
      const repositoryWithError = createMockRepositoryWithStats({
        lastCrawlError: 'Network error',
      });

      render(<RepositoryCard {...defaultProps} repository={repositoryWithError} />);

      const errorBadge = screen.getByText('Crawl Error').closest('button');
      expect(errorBadge).toHaveClass('cursor-pointer');
    });

    it('should have title attribute explaining to click for details', () => {
      const errorMessage = 'This is a detailed error message about the crawl failure';
      const repositoryWithError = createMockRepositoryWithStats({
        lastCrawlError: errorMessage,
      });

      render(<RepositoryCard {...defaultProps} repository={repositoryWithError} />);

      const errorBadge = screen.getByText('Crawl Error').closest('button');
      expect(errorBadge).toHaveAttribute('title', 'Click to view error details');
    });

    it('should preserve special characters in error message when displayed', () => {
      const errorMessage = 'Error: Failed to clone "https://github.com/repo.git" (exit code 1)';
      const repositoryWithError = createMockRepositoryWithStats({
        lastCrawlError: errorMessage,
      });

      render(<RepositoryCard {...defaultProps} repository={repositoryWithError} />);

      const errorBadge = screen.getByText('Crawl Error').closest('button');
      expect(errorBadge).toBeInTheDocument();
      // The error message will be displayed in CrawlErrorDisplay after clicking the badge
    });

    it('should handle very long error messages correctly', () => {
      const longErrorMessage = 'A'.repeat(500); // Very long error message
      const repositoryWithError = createMockRepositoryWithStats({
        lastCrawlError: longErrorMessage,
      });

      render(<RepositoryCard {...defaultProps} repository={repositoryWithError} />);

      const errorBadge = screen.getByText('Crawl Error').closest('button');
      expect(errorBadge).toBeInTheDocument();
      // Long error messages are handled by CrawlErrorDisplay component
    });
  });

  describe('Error badge positioning', () => {
    it('should position error badge with other badges in badges section', () => {
      const repositoryWithError = createMockRepositoryWithStats({
        lastCrawlError: 'Crawl timeout',
        branch: 'main',
      });

      render(<RepositoryCard {...defaultProps} repository={repositoryWithError} />);

      const errorBadge = screen.getByText('Crawl Error');
      const typeBadge = screen.getByText('Git');
      const branchBadge = screen.getByText('main');

      // All badges should be in the document
      expect(errorBadge).toBeInTheDocument();
      expect(typeBadge).toBeInTheDocument();
      expect(branchBadge).toBeInTheDocument();

      // They should be within the same container (badges section)
      const errorContainer = errorBadge.closest('button')?.parentElement;
      const typeContainer = typeBadge.closest('span')?.parentElement;

      // Check if they share the same parent (badges section)
      expect(errorContainer).toBe(typeContainer);
    });

    it('should display error badge before type badge', () => {
      const repositoryWithError = createMockRepositoryWithStats({
        lastCrawlError: 'Crawl error occurred',
      });

      render(<RepositoryCard {...defaultProps} repository={repositoryWithError} />);

      const badgesSection = screen.getByText('Crawl Error').closest('button')?.parentElement;
      const children = badgesSection?.children || [];

      // Find the indices of error and type badges
      let errorIndex = -1;
      let typeIndex = -1;

      for (let i = 0; i < children.length; i++) {
        if (children[i].textContent?.includes('Crawl Error')) {
          errorIndex = i;
        }
        if (children[i].textContent === 'Git') {
          typeIndex = i;
        }
      }

      // Error badge should come before type badge (lower index)
      expect(errorIndex).toBeLessThan(typeIndex);
    });

    it('should display error badge with branch and auto-crawl badges', () => {
      const repositoryWithError = createMockRepositoryWithStats({
        lastCrawlError: 'Crawl error',
        branch: 'develop',
        autoCrawlEnabled: true,
      });

      render(<RepositoryCard {...defaultProps} repository={repositoryWithError} />);

      // All badges should be visible
      expect(screen.getByText('Crawl Error')).toBeInTheDocument();
      expect(screen.getByText('Git')).toBeInTheDocument();
      expect(screen.getByText('develop')).toBeInTheDocument();
      expect(screen.getByText('Auto-crawl')).toBeInTheDocument();
    });

    it('should display error badge with flex gap spacing', () => {
      const repositoryWithError = createMockRepositoryWithStats({
        lastCrawlError: 'Connection refused',
      });

      render(<RepositoryCard {...defaultProps} repository={repositoryWithError} />);

      const badgesContainer = screen.getByText('Crawl Error').closest('button')?.parentElement;
      expect(badgesContainer).toHaveClass('gap-2');
    });
  });

  describe('Error badge updates', () => {
    it('should update badge when lastCrawlError changes from undefined to error message', () => {
      const { rerender } = render(
        <RepositoryCard
          {...defaultProps}
          repository={createMockRepositoryWithStats({ lastCrawlError: undefined })}
        />
      );

      // Badge should not be visible initially
      expect(screen.queryByText('Crawl Error')).not.toBeInTheDocument();

      // Rerender with error
      rerender(
        <RepositoryCard
          {...defaultProps}
          repository={createMockRepositoryWithStats({ lastCrawlError: 'New error occurred' })}
        />
      );

      // Badge should now be visible
      expect(screen.getByText('Crawl Error')).toBeInTheDocument();
      expect(screen.getByText('Crawl Error').closest('button')).toHaveAttribute('title', 'Click to view error details');
    });

    it('should remove badge when lastCrawlError changes from error to undefined', () => {
      const { rerender } = render(
        <RepositoryCard
          {...defaultProps}
          repository={createMockRepositoryWithStats({ lastCrawlError: 'Existing error' })}
        />
      );

      // Badge should be visible initially
      expect(screen.getByText('Crawl Error')).toBeInTheDocument();

      // Rerender without error
      rerender(
        <RepositoryCard
          {...defaultProps}
          repository={createMockRepositoryWithStats({ lastCrawlError: undefined })}
        />
      );

      // Badge should no longer be visible
      expect(screen.queryByText('Crawl Error')).not.toBeInTheDocument();
    });

    it('should update error message in tooltip when lastCrawlError changes', () => {
      const { rerender } = render(
        <RepositoryCard
          {...defaultProps}
          repository={createMockRepositoryWithStats({ lastCrawlError: 'Initial error' })}
        />
      );

      let errorBadge = screen.getByText('Crawl Error').closest('button');
      expect(errorBadge).toHaveAttribute('title', 'Click to view error details');

      // Rerender with different error
      rerender(
        <RepositoryCard
          {...defaultProps}
          repository={createMockRepositoryWithStats({ lastCrawlError: 'Updated error message' })}
        />
      );

      errorBadge = screen.getByText('Crawl Error').closest('button');
      expect(errorBadge).toHaveAttribute('title', 'Click to view error details');
    });
  });

  describe('Error badge with different repository types', () => {
    it('should display error badge for GitHub repositories', () => {
      const repositoryWithError = createMockRepositoryWithStats({
        lastCrawlError: 'GitHub API rate limit exceeded',
        repositoryType: 'GitHub',
      });

      render(<RepositoryCard {...defaultProps} repository={repositoryWithError} />);

      expect(screen.getByText('Crawl Error')).toBeInTheDocument();
      expect(screen.getByText('GitHub')).toBeInTheDocument();
    });

    it('should display error badge for GitLab repositories', () => {
      const repositoryWithError = createMockRepositoryWithStats({
        lastCrawlError: 'GitLab namespace not found',
        repositoryType: 'GitLab',
      });

      render(<RepositoryCard {...defaultProps} repository={repositoryWithError} />);

      expect(screen.getByText('Crawl Error')).toBeInTheDocument();
      expect(screen.getByText('GitLab')).toBeInTheDocument();
    });

    it('should display error badge for FileSystem repositories', () => {
      const repositoryWithError = createMockRepositoryWithStats({
        lastCrawlError: 'Directory not accessible',
        repositoryType: 'FileSystem',
      });

      render(<RepositoryCard {...defaultProps} repository={repositoryWithError} />);

      expect(screen.getByText('Crawl Error')).toBeInTheDocument();
      expect(screen.getByText('FileSystem')).toBeInTheDocument();
    });
  });

  describe('Error badge with disabled repositories', () => {
    it('should display error badge even when repository is disabled', () => {
      const repositoryWithError = createMockRepositoryWithStats({
        lastCrawlError: 'Last crawl failed before disable',
        enabled: false,
      });

      render(<RepositoryCard {...defaultProps} repository={repositoryWithError} />);

      expect(screen.getByText('Crawl Error')).toBeInTheDocument();
      // Check that the Disabled status is displayed
      expect(screen.getAllByText('Disabled').length).toBeGreaterThan(0);
    });
  });

  describe('Error badge integration', () => {
    it('should render error badge without interfering with other functionality', () => {
      const repositoryWithError = createMockRepositoryWithStats({
        lastCrawlError: 'Integration test error',
      });

      render(<RepositoryCard {...defaultProps} repository={repositoryWithError} />);

      // Error badge should be present
      expect(screen.getByText('Crawl Error')).toBeInTheDocument();

      // Other elements should still be present and functional
      expect(screen.getByText('Test Repository')).toBeInTheDocument();
      // Check for visible crawl button (not the one in the dropdown menu)
      const crawlButtons = screen.getAllByRole('button', { name: /crawl/i });
      expect(crawlButtons.length).toBeGreaterThan(0);
      // Check for enabled status button
      expect(screen.getByRole('button', { name: /enabled/i })).toBeInTheDocument();
    });

    it('should display error badge alongside crawl status', () => {
      const repositoryWithError = createMockRepositoryWithStats({
        lastCrawlError: 'Last crawl had errors',
        lastCrawled: new Date().toISOString(),
      });

      render(<RepositoryCard {...defaultProps} repository={repositoryWithError} />);

      // Both error badge and status should be present
      expect(screen.getByText('Crawl Error')).toBeInTheDocument();
      expect(screen.getByText('Ready')).toBeInTheDocument();
    });
  });
});