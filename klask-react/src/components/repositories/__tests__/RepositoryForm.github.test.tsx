import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { RepositoryForm } from '../RepositoryForm';
import type { Repository } from '../../../types';

/**
 * Tests for GitHub integration in RepositoryForm
 * These tests verify that the form correctly handles GitHub-specific fields
 */

describe('RepositoryForm - GitHub Integration', () => {
  const mockOnClose = vi.fn();
  const mockOnSubmit = vi.fn();

  beforeEach(() => {
    vi.clearAllMocks();
  });

  it('should display GitHub-specific fields when GitHub type is selected', async () => {
    const user = userEvent.setup();

    render(
      <RepositoryForm
        isOpen={true}
        onClose={mockOnClose}
        onSubmit={mockOnSubmit}
        isLoading={false}
      />
    );

    // Select GitHub repository type
    const githubRadio = screen.getByRole('radio', { name: /github/i });
    await user.click(githubRadio);

    // Navigate to GitHub Settings tab
    const githubSettingsTab = screen.getByRole('button', { name: /GitHub Settings/i });
    await user.click(githubSettingsTab);

    // Verify GitHub-specific text is shown
    await waitFor(() => {
      expect(screen.getByText(/GitHub repositories will be automatically discovered/i)).toBeInTheDocument();
    });

    // Verify placeholders for GitHub fields exist (first check GitHub Settings tab)
    expect(screen.getByPlaceholderText('ghp_...')).toBeInTheDocument();
    expect(screen.getByPlaceholderText('organization-name or username')).toBeInTheDocument();

    // Navigate to Filters tab for exclusion fields
    const filtersTab = screen.getByRole('button', { name: /Filters & Exclusions/i });
    await user.click(filtersTab);

    expect(screen.getByPlaceholderText('org/repo-archive, user/legacy-project')).toBeInTheDocument();
    expect(screen.getByPlaceholderText('*-archive, test-*, *-temp')).toBeInTheDocument();
  });

  it('should validate required access token for GitHub', async () => {
    const user = userEvent.setup();

    render(
      <RepositoryForm
        isOpen={true}
        onClose={mockOnClose}
        onSubmit={mockOnSubmit}
        isLoading={false}
      />
    );

    // Select GitHub type
    const githubRadio = screen.getByRole('radio', { name: /github/i });
    await user.click(githubRadio);

    // Fill in name but not token
    const nameInput = screen.getByPlaceholderText('My Repository');
    await user.type(nameInput, 'Test GitHub Repo');

    // Try to submit - button should be disabled because token is required
    const submitButton = screen.getByRole('button', { name: /Create Repository/i });

    await waitFor(() => {
      expect(submitButton).toBeDisabled();
    });
  });

  it('should submit GitHub repository with all fields filled', async () => {
    const user = userEvent.setup();

    render(
      <RepositoryForm
        isOpen={true}
        onClose={mockOnClose}
        onSubmit={mockOnSubmit}
        isLoading={false}
      />
    );

    // Select GitHub type
    const githubRadio = screen.getByRole('radio', { name: /github/i });
    await user.click(githubRadio);

    // Fill in repository name in Basic tab
    await user.type(screen.getByPlaceholderText('My Repository'), 'GitHub Test Repo');

    // Navigate to GitHub Settings tab
    const githubSettingsTab = screen.getByRole('button', { name: /GitHub Settings/i });
    await user.click(githubSettingsTab);

    // Fill in GitHub-specific fields
    await user.type(screen.getByPlaceholderText('ghp_...'), 'ghp_test_token');
    await user.type(screen.getByPlaceholderText('organization-name or username'), 'test-org');

    // Navigate to Filters tab
    const filtersTab = screen.getByRole('button', { name: /Filters & Exclusions/i });
    await user.click(filtersTab);

    // Fill in filter fields
    await user.type(screen.getByPlaceholderText('org/repo-archive, user/legacy-project'), 'test-org/archive');
    await user.type(screen.getByPlaceholderText('*-archive, test-*, *-temp'), '*-temp');

    // Submit form
    const submitButton = screen.getByRole('button', { name: /Create Repository/i });
    await user.click(submitButton);

    // Verify submission
    await waitFor(() => {
      expect(mockOnSubmit).toHaveBeenCalled();
      const submittedData = mockOnSubmit.mock.calls[0][0];
      expect(submittedData.name).toBe('GitHub Test Repo');
      expect(submittedData.repositoryType).toBe('GitHub');
      expect(submittedData.accessToken).toBe('ghp_test_token');
      expect(submittedData.githubNamespace).toBe('test-org');
    });
  });

  it('should allow GitHub repository without namespace filter', async () => {
    const user = userEvent.setup();

    render(
      <RepositoryForm
        isOpen={true}
        onClose={mockOnClose}
        onSubmit={mockOnSubmit}
        isLoading={false}
      />
    );

    // Select GitHub type
    const githubRadio = screen.getByRole('radio', { name: /github/i });
    await user.click(githubRadio);

    // Fill in repository name in Basic tab
    await user.type(screen.getByPlaceholderText('My Repository'), 'GitHub All Repos');

    // Navigate to GitHub Settings tab and fill in token
    const githubSettingsTab = screen.getByRole('button', { name: /GitHub Settings/i });
    await user.click(githubSettingsTab);
    await user.type(screen.getByPlaceholderText('ghp_...'), 'ghp_token');

    // Submit without namespace
    const submitButton = screen.getByRole('button', { name: /Create Repository/i });
    await user.click(submitButton);

    await waitFor(() => {
      expect(mockOnSubmit).toHaveBeenCalled();
      const submittedData = mockOnSubmit.mock.calls[0][0];
      expect(submittedData.name).toBe('GitHub All Repos');
      expect(submittedData.accessToken).toBe('ghp_token');
    });
  });

  it('should edit existing GitHub repository', async () => {
    const user = userEvent.setup();

    const existingRepo: Repository = {
      id: 'test-id-123',
      name: 'Existing GitHub Repo',
      url: 'https://api.github.com',
      repositoryType: 'GitHub',
      branch: 'main',
      enabled: true,
      gitlabNamespace: null,
      isGroup: false,
      lastCrawled: null,
      createdAt: new Date().toISOString(),
      updatedAt: new Date().toISOString(),
      autoCrawlEnabled: false,
      cronSchedule: null,
      nextCrawlAt: null,
      crawlFrequencyHours: null,
      maxCrawlDurationMinutes: 60,
      lastCrawlDurationSeconds: null,
      gitlabExcludedProjects: null,
      gitlabExcludedPatterns: null,
      githubNamespace: 'original-org',
      githubExcludedRepositories: 'original-org/old-repo',
      githubExcludedPatterns: '*-old',
      crawlState: 'idle',
      lastProcessedProject: null,
      crawlStartedAt: null,
    };

    render(
      <RepositoryForm
        repository={existingRepo}
        isOpen={true}
        onClose={mockOnClose}
        onSubmit={mockOnSubmit}
        isLoading={false}
      />
    );

    // Verify existing values are populated in Basic tab
    expect(screen.getByDisplayValue('Existing GitHub Repo')).toBeInTheDocument();

    // Navigate to GitHub Settings tab to verify namespace
    const githubSettingsTab = screen.getByRole('button', { name: /GitHub Settings/i });
    await user.click(githubSettingsTab);
    expect(screen.getByDisplayValue('original-org')).toBeInTheDocument();

    // Navigate to Filters tab to verify exclusions
    const filtersTab = screen.getByRole('button', { name: /Filters & Exclusions/i });
    await user.click(filtersTab);
    expect(screen.getByDisplayValue('original-org/old-repo')).toBeInTheDocument();
    expect(screen.getByDisplayValue('*-old')).toBeInTheDocument();
  });

  it('should show token configured indicator when editing with existing token', async () => {
    const user = userEvent.setup();

    const existingRepo: Repository = {
      id: 'test-id-456',
      name: 'Repo With Token',
      url: 'https://api.github.com',
      repositoryType: 'GitHub',
      branch: 'main',
      enabled: true,
      accessToken: 'encrypted_token', // Existing token
      gitlabNamespace: null,
      isGroup: false,
      lastCrawled: null,
      createdAt: new Date().toISOString(),
      updatedAt: new Date().toISOString(),
      autoCrawlEnabled: false,
      cronSchedule: null,
      nextCrawlAt: null,
      crawlFrequencyHours: null,
      maxCrawlDurationMinutes: 60,
      lastCrawlDurationSeconds: null,
      gitlabExcludedProjects: null,
      gitlabExcludedPatterns: null,
      githubNamespace: 'test-org',
      githubExcludedRepositories: null,
      githubExcludedPatterns: null,
      crawlState: 'idle',
      lastProcessedProject: null,
      crawlStartedAt: null,
    };

    render(
      <RepositoryForm
        repository={existingRepo}
        isOpen={true}
        onClose={mockOnClose}
        onSubmit={mockOnSubmit}
        isLoading={false}
      />
    );

    // Navigate to GitHub Settings tab to see token configured indicator
    const githubSettingsTab = screen.getByRole('button', { name: /GitHub Settings/i });
    await user.click(githubSettingsTab);

    // Should show token configured indicator
    expect(screen.getByText(/Access token configured/i)).toBeInTheDocument();

    // Should have "Change token" button
    expect(screen.getByRole('button', { name: /Change token/i })).toBeInTheDocument();
  });

  it('should show placeholder text for GitHub fields', async () => {
    const user = userEvent.setup();

    render(
      <RepositoryForm
        isOpen={true}
        onClose={mockOnClose}
        onSubmit={mockOnSubmit}
        isLoading={false}
      />
    );

    // Select GitHub type
    const githubRadio = screen.getByRole('radio', { name: /github/i });
    await user.click(githubRadio);

    // Navigate to GitHub Settings tab
    const githubSettingsTab = screen.getByRole('button', { name: /GitHub Settings/i });
    await user.click(githubSettingsTab);

    // Check GitHub Settings placeholders
    await waitFor(() => {
      expect(screen.getByPlaceholderText('ghp_...')).toBeInTheDocument();
      expect(screen.getByPlaceholderText('organization-name or username')).toBeInTheDocument();
    });

    // Navigate to Filters tab
    const filtersTab = screen.getByRole('button', { name: /Filters & Exclusions/i });
    await user.click(filtersTab);

    // Check Filters placeholders
    await waitFor(() => {
      expect(screen.getByPlaceholderText('org/repo-archive, user/legacy-project')).toBeInTheDocument();
      expect(screen.getByPlaceholderText('*-archive, test-*, *-temp')).toBeInTheDocument();
    });
  });

  it('should not show GitLab fields when GitHub is selected', async () => {
    const user = userEvent.setup();

    render(
      <RepositoryForm
        isOpen={true}
        onClose={mockOnClose}
        onSubmit={mockOnSubmit}
        isLoading={false}
      />
    );

    // Select GitHub type
    const githubRadio = screen.getByRole('radio', { name: /github/i });
    await user.click(githubRadio);

    // Navigate to GitHub Settings tab
    const githubSettingsTab = screen.getByRole('button', { name: /GitHub Settings/i });
    await user.click(githubSettingsTab);

    await waitFor(() => {
      // GitLab message should not be present
      expect(screen.queryByText(/GitLab repositories will be automatically/i)).not.toBeInTheDocument();

      // GitHub message should be present
      expect(screen.getByText(/GitHub repositories will be automatically/i)).toBeInTheDocument();
    });
  });
});
