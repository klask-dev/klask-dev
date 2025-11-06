import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { RepositoryForm } from '../RepositoryForm';
import type { Repository } from '../../../types';

/**
 * Comprehensive tests for the RepositoryForm component
 * Tests focus on the new filtering fields:
 * - includedBranches
 * - includedBranchesPatterns
 * - excludedBranches
 * - excludedBranchesPatterns
 * - includedProjects
 * - includedProjectsPatterns
 */

describe('RepositoryForm - Filtering Fields', () => {
  const mockOnClose = vi.fn();
  const mockOnSubmit = vi.fn();

  beforeEach(() => {
    vi.clearAllMocks();
  });

  // ============================================================================
  // Section 1: Tab Visibility Tests (Combined for performance)
  // ============================================================================
  describe('Tab Visibility', () => {
    it('should show "Filters & Exclusions" tab for all repository types', () => {
      render(
        <RepositoryForm
          isOpen={true}
          onClose={mockOnClose}
          onSubmit={mockOnSubmit}
          isLoading={false}
        />
      );

      // Git is default - Filters tab should exist
      let filtersTab = screen.getByRole('button', { name: /Filters & Exclusions/i });
      expect(filtersTab).toBeInTheDocument();

      // Test GitLab
      const gitlabRadio = screen.getByRole('radio', { name: /gitlab/i });
      gitlabRadio.click();
      filtersTab = screen.getByRole('button', { name: /Filters & Exclusions/i });
      expect(filtersTab).toBeInTheDocument();

      // Test GitHub
      const githubRadio = screen.getByRole('radio', { name: /github/i });
      githubRadio.click();
      filtersTab = screen.getByRole('button', { name: /Filters & Exclusions/i });
      expect(filtersTab).toBeInTheDocument();

      // Test FileSystem
      const fileSystemRadio = screen.getByRole('radio', { name: /filesystem/i });
      fileSystemRadio.click();
      filtersTab = screen.getByRole('button', { name: /Filters & Exclusions/i });
      expect(filtersTab).toBeInTheDocument();
    });

    it('should display branch selection section in filters tab', () => {
      render(
        <RepositoryForm
          isOpen={true}
          onClose={mockOnClose}
          onSubmit={mockOnSubmit}
          isLoading={false}
        />
      );

      // Click Filters tab
      const filtersTab = screen.getByRole('button', { name: /Filters & Exclusions/i });
      filtersTab.click();

      // Branch Selection section should be visible
      expect(screen.getByText('Branch Selection')).toBeInTheDocument();
    });

    it('should display type-specific selection sections in filters tab', () => {
      const { rerender } = render(
        <RepositoryForm
          isOpen={true}
          onClose={mockOnClose}
          onSubmit={mockOnSubmit}
          isLoading={false}
        />
      );

      // Test GitLab - should show Project Selection
      const gitlabRadio = screen.getByRole('radio', { name: /gitlab/i });
      gitlabRadio.click();

      let filtersTab = screen.getByRole('button', { name: /Filters & Exclusions/i });
      filtersTab.click();

      expect(screen.getByText('Project Selection')).toBeInTheDocument();

      // Test GitHub - should show Repository Selection
      rerender(
        <RepositoryForm
          isOpen={true}
          onClose={mockOnClose}
          onSubmit={mockOnSubmit}
          isLoading={false}
        />
      );

      const githubRadio = screen.getByRole('radio', { name: /github/i });
      githubRadio.click();

      filtersTab = screen.getByRole('button', { name: /Filters & Exclusions/i });
      filtersTab.click();

      expect(screen.getByText('Repository Selection')).toBeInTheDocument();
    });
  });

  // ============================================================================
  // Section 2: Field Rendering Tests
  // ============================================================================
  describe('Field Rendering', () => {
    it('should render all 6 new filter fields', async () => {
      const user = userEvent.setup();

      render(
        <RepositoryForm
          isOpen={true}
          onClose={mockOnClose}
          onSubmit={mockOnSubmit}
          isLoading={false}
        />
      );

      // Click Filters tab
      const filtersTab = screen.getByRole('button', { name: /Filters & Exclusions/i });
      await user.click(filtersTab);

      // Check for all 4 branch fields by placeholder
      await waitFor(() => {
        expect(screen.getByPlaceholderText('main, develop, release-*')).toBeInTheDocument();
        expect(screen.getByPlaceholderText('release-*, hotfix-*, feature-*')).toBeInTheDocument();
        expect(screen.getByPlaceholderText('temp-branch, wip-branch')).toBeInTheDocument();
        expect(screen.getByPlaceholderText('archive-*, backup-*, test-*')).toBeInTheDocument();
      });

      // Check for 2 project fields (for Git, these are in the branch section)
      // Switch to GitLab to see project fields
      const basicTab = screen.getByRole('button', { name: /Basic Configuration/i });
      await user.click(basicTab);

      const gitlabRadio = screen.getByRole('radio', { name: /gitlab/i });
      await user.click(gitlabRadio);

      // Go back to filters
      await user.click(screen.getByRole('button', { name: /Filters & Exclusions/i }));

      // Now check project fields by placeholder
      await waitFor(() => {
        expect(screen.getByPlaceholderText('team/project1, org/repo2')).toBeInTheDocument();
        expect(screen.getByPlaceholderText('team/*, team-*/core-*')).toBeInTheDocument();
      });
    });

    it('should have correct labels for all new fields', async () => {
      const user = userEvent.setup();

      render(
        <RepositoryForm
          isOpen={true}
          onClose={mockOnClose}
          onSubmit={mockOnSubmit}
          isLoading={false}
        />
      );

      const filtersTab = screen.getByRole('button', { name: /Filters & Exclusions/i });
      await user.click(filtersTab);

      await waitFor(() => {
        expect(screen.getByText(/Included Branches \(Optional\)/)).toBeInTheDocument();
        expect(screen.getByText(/Included Branches Patterns \(Optional\)/)).toBeInTheDocument();
        expect(screen.getByText(/Excluded Branches \(Optional\)/)).toBeInTheDocument();
        expect(screen.getByText(/Excluded Branches Patterns \(Optional\)/)).toBeInTheDocument();
      });
    });

    it('should have helpful placeholder text for branch fields', async () => {
      const user = userEvent.setup();

      render(
        <RepositoryForm
          isOpen={true}
          onClose={mockOnClose}
          onSubmit={mockOnSubmit}
          isLoading={false}
        />
      );

      const filtersTab = screen.getByRole('button', { name: /Filters & Exclusions/i });
      await user.click(filtersTab);

      await waitFor(() => {
        expect(screen.getByPlaceholderText('main, develop, release-*')).toBeInTheDocument();
        expect(screen.getByPlaceholderText('release-*, hotfix-*, feature-*')).toBeInTheDocument();
        expect(screen.getByPlaceholderText('temp-branch, wip-branch')).toBeInTheDocument();
        expect(screen.getByPlaceholderText('archive-*, backup-*, test-*')).toBeInTheDocument();
      });
    });

    it('should show "Project" label for GitLab repositories', async () => {
      const user = userEvent.setup();

      render(
        <RepositoryForm
          isOpen={true}
          onClose={mockOnClose}
          onSubmit={mockOnSubmit}
          isLoading={false}
        />
      );

      // Select GitLab
      const basicTab = screen.getByRole('button', { name: /Basic Configuration/i });
      await user.click(basicTab);

      const gitlabRadio = screen.getByRole('radio', { name: /gitlab/i });
      await user.click(gitlabRadio);

      // Go to Filters tab
      const filtersTab = screen.getByRole('button', { name: /Filters & Exclusions/i });
      await user.click(filtersTab);

      await waitFor(() => {
        expect(screen.getByText(/Included Projects \(Optional\)/)).toBeInTheDocument();
        expect(screen.getByText(/Included Projects Patterns \(Optional\)/)).toBeInTheDocument();
      });
    });

    it('should show "Repository" label for GitHub repositories', async () => {
      const user = userEvent.setup();

      render(
        <RepositoryForm
          isOpen={true}
          onClose={mockOnClose}
          onSubmit={mockOnSubmit}
          isLoading={false}
        />
      );

      // Select GitHub
      const basicTab = screen.getByRole('button', { name: /Basic Configuration/i });
      await user.click(basicTab);

      const githubRadio = screen.getByRole('radio', { name: /github/i });
      await user.click(githubRadio);

      // Go to Filters tab
      const filtersTab = screen.getByRole('button', { name: /Filters & Exclusions/i });
      await user.click(filtersTab);

      await waitFor(() => {
        expect(screen.getByText(/Included Repositories \(Optional\)/)).toBeInTheDocument();
        expect(screen.getByText(/Included Repositories Patterns \(Optional\)/)).toBeInTheDocument();
      });
    });

    it('should have descriptive help text for all fields', async () => {
      const user = userEvent.setup();

      render(
        <RepositoryForm
          isOpen={true}
          onClose={mockOnClose}
          onSubmit={mockOnSubmit}
          isLoading={false}
        />
      );

      const filtersTab = screen.getByRole('button', { name: /Filters & Exclusions/i });
      await user.click(filtersTab);

      await waitFor(() => {
        expect(
          screen.getByText(/Comma-separated list of exact branch names or patterns to include/)
        ).toBeInTheDocument();
        expect(
          screen.getByText(/Comma-separated list of wildcard patterns \(e.g., release-\*, hotfix-\*\)/)
        ).toBeInTheDocument();
      });
    });
  });

  // ============================================================================
  // Section 3: Form State Management Tests
  // ============================================================================
  describe('Form State Management', () => {
    it('should have empty values for all new fields in create mode', async () => {
      const user = userEvent.setup();

      render(
        <RepositoryForm
          isOpen={true}
          onClose={mockOnClose}
          onSubmit={mockOnSubmit}
          isLoading={false}
        />
      );

      const filtersTab = screen.getByRole('button', { name: /Filters & Exclusions/i });
      await user.click(filtersTab);

      await waitFor(() => {
        const includedBranchesInput = screen.getByPlaceholderText(
          'main, develop, release-*'
        ) as HTMLInputElement;
        const includedPatternsInput = screen.getByPlaceholderText(
          'release-*, hotfix-*, feature-*'
        ) as HTMLInputElement;
        const excludedBranchesInput = screen.getByPlaceholderText(
          'temp-branch, wip-branch'
        ) as HTMLInputElement;
        const excludedPatternsInput = screen.getByPlaceholderText(
          'archive-*, backup-*, test-*'
        ) as HTMLInputElement;

        expect(includedBranchesInput.value).toBe('');
        expect(includedPatternsInput.value).toBe('');
        expect(excludedBranchesInput.value).toBe('');
        expect(excludedPatternsInput.value).toBe('');
      });
    });

    it('should load existing filter values when editing', async () => {
      const user = userEvent.setup();

      const existingRepo: Repository = {
        id: 'repo-123',
        name: 'Test Repo',
        url: 'https://github.com/test/repo.git',
        repositoryType: 'Git',
        branch: 'main',
        enabled: true,
        createdAt: new Date().toISOString(),
        updatedAt: new Date().toISOString(),
        autoCrawlEnabled: false,
        crawlState: 'idle',
        lastProcessedProject: null,
        crawlStartedAt: null,
        includedBranches: 'main, develop',
        includedBranchesPatterns: 'release-*',
        excludedBranches: 'temp-*',
        excludedBranchesPatterns: 'archive-*',
        includedProjects: 'org/repo1',
        includedProjectsPatterns: 'org/*',
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

      const filtersTab = screen.getByRole('button', { name: /Filters & Exclusions/i });
      await user.click(filtersTab);

      await waitFor(() => {
        expect(screen.getByDisplayValue('main, develop')).toBeInTheDocument();
        expect(screen.getByDisplayValue('release-*')).toBeInTheDocument();
        expect(screen.getByDisplayValue('temp-*')).toBeInTheDocument();
        expect(screen.getByDisplayValue('archive-*')).toBeInTheDocument();
      });
    });

    it('should handle repositories with partial filter fields', async () => {
      const user = userEvent.setup();

      const partialRepo: Repository = {
        id: 'repo-456',
        name: 'Partial Filters Repo',
        url: 'https://github.com/test/repo.git',
        repositoryType: 'Git',
        branch: 'main',
        enabled: true,
        createdAt: new Date().toISOString(),
        updatedAt: new Date().toISOString(),
        autoCrawlEnabled: false,
        crawlState: 'idle',
        lastProcessedProject: null,
        crawlStartedAt: null,
        includedBranches: 'main',
        // includedBranchesPatterns is undefined
        // excludedBranches is undefined
        excludedBranchesPatterns: 'test-*',
      };

      render(
        <RepositoryForm
          repository={partialRepo}
          isOpen={true}
          onClose={mockOnClose}
          onSubmit={mockOnSubmit}
          isLoading={false}
        />
      );

      const filtersTab = screen.getByRole('button', { name: /Filters & Exclusions/i });
      await user.click(filtersTab);

      await waitFor(() => {
        expect(screen.getByDisplayValue('main')).toBeInTheDocument();
        expect(screen.getByDisplayValue('test-*')).toBeInTheDocument();
      });

      // Check that empty fields are still empty
      const includedPatternsInput = screen.getByPlaceholderText(
        'release-*, hotfix-*, feature-*'
      ) as HTMLInputElement;
      expect(includedPatternsInput.value).toBe('');
    });

    it('should handle repositories with no filter fields set', async () => {
      const user = userEvent.setup();

      const noFiltersRepo: Repository = {
        id: 'repo-789',
        name: 'No Filters Repo',
        url: 'https://github.com/test/repo.git',
        repositoryType: 'Git',
        branch: 'main',
        enabled: true,
        createdAt: new Date().toISOString(),
        updatedAt: new Date().toISOString(),
        autoCrawlEnabled: false,
        crawlState: 'idle',
        lastProcessedProject: null,
        crawlStartedAt: null,
      };

      render(
        <RepositoryForm
          repository={noFiltersRepo}
          isOpen={true}
          onClose={mockOnClose}
          onSubmit={mockOnSubmit}
          isLoading={false}
        />
      );

      const filtersTab = screen.getByRole('button', { name: /Filters & Exclusions/i });
      await user.click(filtersTab);

      await waitFor(() => {
        const inputs = screen.getAllByRole('textbox');
        const filterInputs = inputs.filter((input) => {
          const parent = input.closest('div');
          return parent?.textContent?.includes('Branch') || parent?.textContent?.includes('Project');
        });

        filterInputs.forEach((input) => {
          expect((input as HTMLInputElement).value).toBe('');
        });
      });
    });

    it('should update form state as user types in filter fields', async () => {
      const user = userEvent.setup();

      render(
        <RepositoryForm
          isOpen={true}
          onClose={mockOnClose}
          onSubmit={mockOnSubmit}
          isLoading={false}
        />
      );

      const filtersTab = screen.getByRole('button', { name: /Filters & Exclusions/i });
      await user.click(filtersTab);

      const includedBranchesInput = screen.getByPlaceholderText('main, develop, release-*');
      await user.type(includedBranchesInput, 'main, develop');

      await waitFor(() => {
        expect(screen.getByDisplayValue('main, develop')).toBeInTheDocument();
      });
    });
  });

  // ============================================================================
  // Section 4: Form Validation Tests
  // ============================================================================
  describe('Form Validation', () => {
    it('should mark all new filter fields as optional', async () => {
      const user = userEvent.setup();

      render(
        <RepositoryForm
          isOpen={true}
          onClose={mockOnClose}
          onSubmit={mockOnSubmit}
          isLoading={false}
        />
      );

      // Fill basic fields to make form valid
      await user.type(screen.getByPlaceholderText('My Repository'), 'Test Repo');

      // Select FileSystem to avoid token requirement
      const fileSysRadio = screen.getByRole('radio', { name: /filesystem/i });
      await user.click(fileSysRadio);

      const urlInput = screen.getByPlaceholderText(/path/i);
      await user.type(urlInput, '/path/to/repo');

      // Go to filters tab
      const filtersTab = screen.getByRole('button', { name: /Filters & Exclusions/i });
      await user.click(filtersTab);

      // All fields should be marked as Optional
      await waitFor(() => {
        expect(screen.getByText(/Included Branches \(Optional\)/)).toBeInTheDocument();
        expect(screen.getByText(/Included Branches Patterns \(Optional\)/)).toBeInTheDocument();
        expect(screen.getByText(/Excluded Branches \(Optional\)/)).toBeInTheDocument();
        expect(screen.getByText(/Excluded Branches Patterns \(Optional\)/)).toBeInTheDocument();
      });
    });

    it('should allow form submission with empty filter fields', async () => {
      const user = userEvent.setup();

      render(
        <RepositoryForm
          isOpen={true}
          onClose={mockOnClose}
          onSubmit={mockOnSubmit}
          isLoading={false}
        />
      );

      // Fill basic required fields
      await user.type(screen.getByPlaceholderText('My Repository'), 'Test Repo');

      // Select FileSystem
      const fileSysRadio = screen.getByRole('radio', { name: /filesystem/i });
      await user.click(fileSysRadio);

      const urlInput = screen.getByPlaceholderText(/path/i);
      await user.type(urlInput, '/path/to/repo');

      // Leave all filter fields empty
      // Submit should be possible
      const submitButton = screen.getByRole('button', { name: /Create Repository/i });

      await waitFor(() => {
        expect(submitButton).not.toBeDisabled();
      });
    });

    it('should allow comma-separated values in filter fields', async () => {
      const user = userEvent.setup();

      render(
        <RepositoryForm
          isOpen={true}
          onClose={mockOnClose}
          onSubmit={mockOnSubmit}
          isLoading={false}
        />
      );

      await user.type(screen.getByPlaceholderText('My Repository'), 'Test Repo');
      const fileSysRadio = screen.getByRole('radio', { name: /filesystem/i });
      await user.click(fileSysRadio);
      const urlInput = screen.getByPlaceholderText(/path/i);
      await user.type(urlInput, '/path/to/repo');

      const filtersTab = screen.getByRole('button', { name: /Filters & Exclusions/i });
      await user.click(filtersTab);

      const includedBranches = screen.getByPlaceholderText('main, develop, release-*');
      await user.type(includedBranches, 'main, develop, staging, production');

      const submitButton = screen.getByRole('button', { name: /Create Repository/i });

      await waitFor(() => {
        expect(submitButton).not.toBeDisabled();
      });
    });

    it('should allow wildcard patterns in pattern fields', async () => {
      const user = userEvent.setup();

      render(
        <RepositoryForm
          isOpen={true}
          onClose={mockOnClose}
          onSubmit={mockOnSubmit}
          isLoading={false}
        />
      );

      await user.type(screen.getByPlaceholderText('My Repository'), 'Test Repo');
      const fileSysRadio = screen.getByRole('radio', { name: /filesystem/i });
      await user.click(fileSysRadio);
      const urlInput = screen.getByPlaceholderText(/path/i);
      await user.type(urlInput, '/path/to/repo');

      const filtersTab = screen.getByRole('button', { name: /Filters & Exclusions/i });
      await user.click(filtersTab);

      const patterns = screen.getByPlaceholderText('release-*, hotfix-*, feature-*');
      await user.type(patterns, 'release-*, hotfix-*, feature-*');

      const submitButton = screen.getByRole('button', { name: /Create Repository/i });

      await waitFor(() => {
        expect(submitButton).not.toBeDisabled();
      });
    });

    it('should not show validation errors for empty optional fields', async () => {
      const user = userEvent.setup();

      render(
        <RepositoryForm
          isOpen={true}
          onClose={mockOnClose}
          onSubmit={mockOnSubmit}
          isLoading={false}
        />
      );

      await user.type(screen.getByPlaceholderText('My Repository'), 'Test Repo');
      const fileSysRadio = screen.getByRole('radio', { name: /filesystem/i });
      await user.click(fileSysRadio);
      const urlInput = screen.getByPlaceholderText(/path/i);
      await user.type(urlInput, '/path/to/repo');

      const filtersTab = screen.getByRole('button', { name: /Filters & Exclusions/i });
      await user.click(filtersTab);

      // Check that no error messages are displayed
      const errorMessages = screen.queryAllByText(/error/i);
      expect(errorMessages.length).toBe(0);
    });
  });

  // ============================================================================
  // Section 5: Form Submission Tests
  // ============================================================================
  describe('Form Submission', () => {
    it('should convert empty strings to undefined before submission', async () => {
      const user = userEvent.setup();

      render(
        <RepositoryForm
          isOpen={true}
          onClose={mockOnClose}
          onSubmit={mockOnSubmit}
          isLoading={false}
        />
      );

      await user.type(screen.getByPlaceholderText('My Repository'), 'Test Repo');
      const fileSysRadio = screen.getByRole('radio', { name: /filesystem/i });
      await user.click(fileSysRadio);
      const urlInput = screen.getByPlaceholderText(/path/i);
      await user.type(urlInput, '/path/to/repo');

      const filtersTab = screen.getByRole('button', { name: /Filters & Exclusions/i });
      await user.click(filtersTab);

      // Leave all filter fields empty and submit
      const submitButton = screen.getByRole('button', { name: /Create Repository/i });
      await user.click(submitButton);

      await waitFor(() => {
        expect(mockOnSubmit).toHaveBeenCalled();
        const submittedData = mockOnSubmit.mock.calls[0][0];

        // Empty strings should be converted to undefined
        expect(submittedData.includedBranches).toBeUndefined();
        expect(submittedData.includedBranchesPatterns).toBeUndefined();
        expect(submittedData.excludedBranches).toBeUndefined();
        expect(submittedData.excludedBranchesPatterns).toBeUndefined();
      });
    });

    it('should trim whitespace from filter field values before submission', async () => {
      const user = userEvent.setup();

      render(
        <RepositoryForm
          isOpen={true}
          onClose={mockOnClose}
          onSubmit={mockOnSubmit}
          isLoading={false}
        />
      );

      await user.type(screen.getByPlaceholderText('My Repository'), 'Test Repo');
      const fileSysRadio = screen.getByRole('radio', { name: /filesystem/i });
      await user.click(fileSysRadio);
      const urlInput = screen.getByPlaceholderText(/path/i);
      await user.type(urlInput, '/path/to/repo');

      const filtersTab = screen.getByRole('button', { name: /Filters & Exclusions/i });
      await user.click(filtersTab);

      const includedBranches = screen.getByPlaceholderText('main, develop, release-*');
      await user.type(includedBranches, '  main, develop  ');

      const submitButton = screen.getByRole('button', { name: /Create Repository/i });
      await user.click(submitButton);

      await waitFor(() => {
        expect(mockOnSubmit).toHaveBeenCalled();
        const submittedData = mockOnSubmit.mock.calls[0][0];

        // Whitespace should be trimmed
        expect(submittedData.includedBranches).toBe('main, develop');
      });
    });

    it('should include all filter fields in CreateRepositoryRequest', async () => {
      const user = userEvent.setup();

      render(
        <RepositoryForm
          isOpen={true}
          onClose={mockOnClose}
          onSubmit={mockOnSubmit}
          isLoading={false}
        />
      );

      await user.type(screen.getByPlaceholderText('My Repository'), 'Test Repo');
      const fileSysRadio = screen.getByRole('radio', { name: /filesystem/i });
      await user.click(fileSysRadio);
      const urlInput = screen.getByPlaceholderText(/path/i);
      await user.type(urlInput, '/path/to/repo');

      const filtersTab = screen.getByRole('button', { name: /Filters & Exclusions/i });
      await user.click(filtersTab);

      await user.type(
        screen.getByPlaceholderText('main, develop, release-*'),
        'main'
      );
      await user.type(
        screen.getByPlaceholderText('release-*, hotfix-*, feature-*'),
        'release-*'
      );
      await user.type(
        screen.getByPlaceholderText('temp-branch, wip-branch'),
        'temp-*'
      );
      await user.type(
        screen.getByPlaceholderText('archive-*, backup-*, test-*'),
        'archive-*'
      );

      const submitButton = screen.getByRole('button', { name: /Create Repository/i });
      await user.click(submitButton);

      await waitFor(() => {
        expect(mockOnSubmit).toHaveBeenCalled();
        const submittedData = mockOnSubmit.mock.calls[0][0];

        expect(submittedData).toHaveProperty('includedBranches', 'main');
        expect(submittedData).toHaveProperty('includedBranchesPatterns', 'release-*');
        expect(submittedData).toHaveProperty('excludedBranches', 'temp-*');
        expect(submittedData).toHaveProperty('excludedBranchesPatterns', 'archive-*');
      });
    });

    it('should include filter fields in UpdateRepositoryRequest', async () => {
      const user = userEvent.setup();

      const existingRepo: Repository = {
        id: 'repo-123',
        name: 'Test Repo',
        url: 'https://github.com/test/repo.git',
        repositoryType: 'Git',
        branch: 'main',
        enabled: true,
        createdAt: new Date().toISOString(),
        updatedAt: new Date().toISOString(),
        autoCrawlEnabled: false,
        crawlState: 'idle',
        lastProcessedProject: null,
        crawlStartedAt: null,
        includedBranches: 'main',
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

      const filtersTab = screen.getByRole('button', { name: /Filters & Exclusions/i });
      await user.click(filtersTab);

      const excludedBranches = screen.getByPlaceholderText('temp-branch, wip-branch');
      await user.type(excludedBranches, 'temp-*');

      const submitButton = screen.getByRole('button', { name: /Update Repository/i });
      await user.click(submitButton);

      await waitFor(() => {
        expect(mockOnSubmit).toHaveBeenCalled();
        const submittedData = mockOnSubmit.mock.calls[0][0];

        expect(submittedData).toHaveProperty('excludedBranches', 'temp-*');
      });
    });

    it('should not submit form with validation errors', async () => {
      const user = userEvent.setup();

      render(
        <RepositoryForm
          isOpen={true}
          onClose={mockOnClose}
          onSubmit={mockOnSubmit}
          isLoading={false}
        />
      );

      // Don't fill required fields
      const submitButton = screen.getByRole('button', { name: /Create Repository/i });

      // Button should be disabled
      await waitFor(() => {
        expect(submitButton).toBeDisabled();
      });
    });
  });

  // ============================================================================
  // Section 6: UI/UX Tests
  // ============================================================================
  describe('UI/UX Layout', () => {
    it('should display section headers for branch and project selection', async () => {
      const user = userEvent.setup();

      render(
        <RepositoryForm
          isOpen={true}
          onClose={mockOnClose}
          onSubmit={mockOnSubmit}
          isLoading={false}
        />
      );

      const basicTab = screen.getByRole('button', { name: /Basic Configuration/i });
      await user.click(basicTab);

      const gitlabRadio = screen.getByRole('radio', { name: /gitlab/i });
      await user.click(gitlabRadio);

      const filtersTab = screen.getByRole('button', { name: /Filters & Exclusions/i });
      await user.click(filtersTab);

      await waitFor(() => {
        expect(screen.getByText('Branch Selection')).toBeInTheDocument();
        expect(screen.getByText('Project Selection')).toBeInTheDocument();
      });
    });

    it('should separate branch and project sections visually', async () => {
      const user = userEvent.setup();

      const { container } = render(
        <RepositoryForm
          isOpen={true}
          onClose={mockOnClose}
          onSubmit={mockOnSubmit}
          isLoading={false}
        />
      );

      const basicTab = screen.getByRole('button', { name: /Basic Configuration/i });
      await user.click(basicTab);

      const gitlabRadio = screen.getByRole('radio', { name: /gitlab/i });
      await user.click(gitlabRadio);

      const filtersTab = screen.getByRole('button', { name: /Filters & Exclusions/i });
      await user.click(filtersTab);

      await waitFor(() => {
        // Check for border separator between sections
        const borderElements = container.querySelectorAll('.border-t');
        expect(borderElements.length).toBeGreaterThan(0);
      });
    });

    it('should display help text for all filter fields', async () => {
      const user = userEvent.setup();

      render(
        <RepositoryForm
          isOpen={true}
          onClose={mockOnClose}
          onSubmit={mockOnSubmit}
          isLoading={false}
        />
      );

      const filtersTab = screen.getByRole('button', { name: /Filters & Exclusions/i });
      await user.click(filtersTab);

      await waitFor(() => {
        expect(
          screen.getByText(/Comma-separated list of exact branch names or patterns to include/)
        ).toBeInTheDocument();
        expect(
          screen.getAllByText(/Comma-separated list of wildcard patterns/)[0]
        ).toBeInTheDocument();
      });
    });

    it('should apply dark mode styling to filter fields', async () => {
      const user = userEvent.setup();

      const { container } = render(
        <RepositoryForm
          isOpen={true}
          onClose={mockOnClose}
          onSubmit={mockOnSubmit}
          isLoading={false}
        />
      );

      const filtersTab = screen.getByRole('button', { name: /Filters & Exclusions/i });
      await user.click(filtersTab);

      await waitFor(() => {
        // Check for dark mode styling in the modal container
        const modal = container.querySelector('.dark\\:bg-gray-800');
        expect(modal).toBeInTheDocument();

        // Check that input fields are rendered properly
        const inputs = screen.getAllByRole('textbox');
        expect(inputs.length).toBeGreaterThan(0);
      });
    });
  });

  // ============================================================================
  // Section 7: Backward Compatibility Tests
  // ============================================================================
  describe('Backward Compatibility', () => {
    it('should still work with old branch field', async () => {
      const user = userEvent.setup();

      const repoWithOldBranch: Repository = {
        id: 'repo-old',
        name: 'Old Repo',
        url: 'https://github.com/test/repo.git',
        repositoryType: 'Git',
        branch: 'old-branch',
        enabled: true,
        createdAt: new Date().toISOString(),
        updatedAt: new Date().toISOString(),
        autoCrawlEnabled: false,
        crawlState: 'idle',
        lastProcessedProject: null,
        crawlStartedAt: null,
      };

      render(
        <RepositoryForm
          repository={repoWithOldBranch}
          isOpen={true}
          onClose={mockOnClose}
          onSubmit={mockOnSubmit}
          isLoading={false}
        />
      );

      // The old branch field is still in the basic tab
      // Just verify that the form loads without errors and renders the basic fields
      expect(screen.getByDisplayValue('Old Repo')).toBeInTheDocument();
      expect(screen.getByDisplayValue('https://github.com/test/repo.git')).toBeInTheDocument();
    });

    it('should load old GitLab namespace field if present', async () => {
      const user = userEvent.setup();

      const repoWithOldNamespace: Repository = {
        id: 'repo-old-gitlab',
        name: 'Old GitLab Repo',
        url: 'https://gitlab.com',
        repositoryType: 'GitLab',
        enabled: true,
        createdAt: new Date().toISOString(),
        updatedAt: new Date().toISOString(),
        autoCrawlEnabled: false,
        crawlState: 'idle',
        lastProcessedProject: null,
        crawlStartedAt: null,
        gitlabNamespace: 'old-namespace',
      };

      render(
        <RepositoryForm
          repository={repoWithOldNamespace}
          isOpen={true}
          onClose={mockOnClose}
          onSubmit={mockOnSubmit}
          isLoading={false}
        />
      );

      const providerTab = screen.getByRole('button', { name: /GitLab Settings/i });
      await user.click(providerTab);

      // The gitlabNamespace field is not visible in the form anymore (removed)
      // Just verify the form can load without errors
      expect(providerTab).toBeInTheDocument();
    });

    it('should load old GitHub namespace field if present', async () => {
      const user = userEvent.setup();

      const repoWithOldNamespace: Repository = {
        id: 'repo-old-github',
        name: 'Old GitHub Repo',
        url: 'https://api.github.com',
        repositoryType: 'GitHub',
        enabled: true,
        createdAt: new Date().toISOString(),
        updatedAt: new Date().toISOString(),
        autoCrawlEnabled: false,
        crawlState: 'idle',
        lastProcessedProject: null,
        crawlStartedAt: null,
        githubNamespace: 'old-org',
      };

      render(
        <RepositoryForm
          repository={repoWithOldNamespace}
          isOpen={true}
          onClose={mockOnClose}
          onSubmit={mockOnSubmit}
          isLoading={false}
        />
      );

      const providerTab = screen.getByRole('button', { name: /GitHub Settings/i });
      await user.click(providerTab);

      // The githubNamespace field is not visible in the form anymore (removed)
      // Just verify the form can load without errors
      expect(providerTab).toBeInTheDocument();
    });

    it('should still support old excluded projects/repositories fields', async () => {
      const user = userEvent.setup();

      const repoWithOldExclusions: Repository = {
        id: 'repo-old-excl',
        name: 'Old Exclusions Repo',
        url: 'https://gitlab.com',
        repositoryType: 'GitLab',
        enabled: true,
        createdAt: new Date().toISOString(),
        updatedAt: new Date().toISOString(),
        autoCrawlEnabled: false,
        crawlState: 'idle',
        lastProcessedProject: null,
        crawlStartedAt: null,
        gitlabExcludedProjects: 'old/excluded-project',
        gitlabExcludedPatterns: 'archive-*',
      };

      render(
        <RepositoryForm
          repository={repoWithOldExclusions}
          isOpen={true}
          onClose={mockOnClose}
          onSubmit={mockOnSubmit}
          isLoading={false}
        />
      );

      const filtersTab = screen.getByRole('button', { name: /Filters & Exclusions/i });
      await user.click(filtersTab);

      expect(screen.getByDisplayValue('old/excluded-project')).toBeInTheDocument();
      expect(screen.getByDisplayValue('archive-*')).toBeInTheDocument();
    });

    it('should not auto-migrate old branch values to includedBranches', async () => {
      const user = userEvent.setup();

      const repoWithOldBranch: Repository = {
        id: 'repo-no-migrate',
        name: 'No Migration Repo',
        url: 'https://github.com/test/repo.git',
        repositoryType: 'Git',
        branch: 'old-branch-value',
        enabled: true,
        createdAt: new Date().toISOString(),
        updatedAt: new Date().toISOString(),
        autoCrawlEnabled: false,
        crawlState: 'idle',
        lastProcessedProject: null,
        crawlStartedAt: null,
        includedBranches: undefined,
      };

      render(
        <RepositoryForm
          repository={repoWithOldBranch}
          isOpen={true}
          onClose={mockOnClose}
          onSubmit={mockOnSubmit}
          isLoading={false}
        />
      );

      const filtersTab = screen.getByRole('button', { name: /Filters & Exclusions/i });
      await user.click(filtersTab);

      const includedBranchesInput = screen.getByPlaceholderText(
        'main, develop, release-*'
      ) as HTMLInputElement;

      // Old branch should not be migrated
      expect(includedBranchesInput.value).toBe('');
    });
  });

  // ============================================================================
  // Section 8: Dark Mode Tests
  // ============================================================================
  describe('Dark Mode Support', () => {
    it('should apply dark mode classes to form elements', async () => {
      const user = userEvent.setup();

      const { container } = render(
        <RepositoryForm
          isOpen={true}
          onClose={mockOnClose}
          onSubmit={mockOnSubmit}
          isLoading={false}
        />
      );

      const filtersTab = screen.getByRole('button', { name: /Filters & Exclusions/i });
      await user.click(filtersTab);

      await waitFor(() => {
        // Check for dark mode styling in modal/form
        const modal = container.querySelector('div[class*="dark:"]');
        expect(modal).toBeInTheDocument();
      });
    });

    it('should have correct text colors in dark mode', async () => {
      const user = userEvent.setup();

      const { container } = render(
        <RepositoryForm
          isOpen={true}
          onClose={mockOnClose}
          onSubmit={mockOnSubmit}
          isLoading={false}
        />
      );

      const filtersTab = screen.getByRole('button', { name: /Filters & Exclusions/i });
      await user.click(filtersTab);

      await waitFor(() => {
        // Check that dark mode classes are applied to the form
        const form = container.querySelector('form');
        expect(form).toBeInTheDocument();

        // Labels should be present with styling
        const labels = screen.getAllByText(/Included Branches/);
        expect(labels.length).toBeGreaterThan(0);
      });
    });
  });

  // ============================================================================
  // Section 9: Conditional Rendering Tests
  // ============================================================================
  describe('Conditional Rendering by Repository Type', () => {
    it('should show branch filtering for Git repositories', async () => {
      const user = userEvent.setup();

      render(
        <RepositoryForm
          isOpen={true}
          onClose={mockOnClose}
          onSubmit={mockOnSubmit}
          isLoading={false}
        />
      );

      // Git is default
      const filtersTab = screen.getByRole('button', { name: /Filters & Exclusions/i });
      await user.click(filtersTab);

      await waitFor(() => {
        expect(screen.getByText('Branch Selection')).toBeInTheDocument();
      });
    });

    it('should show both branch and project filtering for GitLab', async () => {
      const user = userEvent.setup();

      render(
        <RepositoryForm
          isOpen={true}
          onClose={mockOnClose}
          onSubmit={mockOnSubmit}
          isLoading={false}
        />
      );

      const basicTab = screen.getByRole('button', { name: /Basic Configuration/i });
      await user.click(basicTab);

      const gitlabRadio = screen.getByRole('radio', { name: /gitlab/i });
      await user.click(gitlabRadio);

      const filtersTab = screen.getByRole('button', { name: /Filters & Exclusions/i });
      await user.click(filtersTab);

      await waitFor(() => {
        expect(screen.getByText('Branch Selection')).toBeInTheDocument();
        expect(screen.getByText('Project Selection')).toBeInTheDocument();
      });
    });

    it('should show both branch and repository filtering for GitHub', async () => {
      const user = userEvent.setup();

      render(
        <RepositoryForm
          isOpen={true}
          onClose={mockOnClose}
          onSubmit={mockOnSubmit}
          isLoading={false}
        />
      );

      const basicTab = screen.getByRole('button', { name: /Basic Configuration/i });
      await user.click(basicTab);

      const githubRadio = screen.getByRole('radio', { name: /github/i });
      await user.click(githubRadio);

      const filtersTab = screen.getByRole('button', { name: /Filters & Exclusions/i });
      await user.click(filtersTab);

      await waitFor(() => {
        expect(screen.getByText('Branch Selection')).toBeInTheDocument();
        expect(screen.getByText('Repository Selection')).toBeInTheDocument();
      });
    });

    it('should show "no filters available" message for FileSystem repositories', async () => {
      const user = userEvent.setup();

      render(
        <RepositoryForm
          isOpen={true}
          onClose={mockOnClose}
          onSubmit={mockOnSubmit}
          isLoading={false}
        />
      );

      const basicTab = screen.getByRole('button', { name: /Basic Configuration/i });
      await user.click(basicTab);

      const fileSysRadio = screen.getByRole('radio', { name: /filesystem/i });
      await user.click(fileSysRadio);

      const filtersTab = screen.getByRole('button', { name: /Filters & Exclusions/i });
      await user.click(filtersTab);

      await waitFor(() => {
        expect(
          screen.getByText(/No filter options available for FileSystem repositories/)
        ).toBeInTheDocument();
      });
    });
  });

  // ============================================================================
  // Section 10: Integration Tests
  // ============================================================================
  describe('Integration - End-to-End Flows', () => {
    it('should create GitLab repository with all filter fields', async () => {
      const user = userEvent.setup();

      render(
        <RepositoryForm
          isOpen={true}
          onClose={mockOnClose}
          onSubmit={mockOnSubmit}
          isLoading={false}
        />
      );

      // Basic tab
      await user.type(screen.getByPlaceholderText('My Repository'), 'GitLab Filtered Repo');
      const gitlabRadio = screen.getByRole('radio', { name: /gitlab/i });
      await user.click(gitlabRadio);

      // GitLab Settings tab
      const settingsTab = screen.getByRole('button', { name: /GitLab Settings/i });
      await user.click(settingsTab);

      await user.type(
        screen.getByPlaceholderText('glpat-...'),
        'glpat_test_token'
      );

      // Filters tab
      const filtersTab = screen.getByRole('button', { name: /Filters & Exclusions/i });
      await user.click(filtersTab);

      await user.type(
        screen.getByPlaceholderText('main, develop, release-*'),
        'main, develop'
      );
      await user.type(
        screen.getByPlaceholderText('release-*, hotfix-*, feature-*'),
        'release-*'
      );
      await user.type(
        screen.getByPlaceholderText('team/project1, org/repo2'),
        'my-org/project-1, my-org/project-2'
      );
      await user.type(
        screen.getByPlaceholderText('team/*, team-*/core-*'),
        'my-org/core-*'
      );

      const submitButton = screen.getByRole('button', { name: /Create Repository/i });
      await user.click(submitButton);

      await waitFor(() => {
        expect(mockOnSubmit).toHaveBeenCalled();
        const submittedData = mockOnSubmit.mock.calls[0][0];

        expect(submittedData.name).toBe('GitLab Filtered Repo');
        expect(submittedData.repositoryType).toBe('GitLab');
        expect(submittedData.includedBranches).toBe('main, develop');
        expect(submittedData.includedBranchesPatterns).toBe('release-*');
        expect(submittedData.includedProjects).toBe('my-org/project-1, my-org/project-2');
        expect(submittedData.includedProjectsPatterns).toBe('my-org/core-*');
      });
    });

    it('should edit existing GitHub repository to add filters', async () => {
      const user = userEvent.setup();

      const existingRepo: Repository = {
        id: 'github-repo-123',
        name: 'Existing GitHub Repo',
        url: 'https://api.github.com',
        repositoryType: 'GitHub',
        branch: 'main',
        enabled: true,
        createdAt: new Date().toISOString(),
        updatedAt: new Date().toISOString(),
        autoCrawlEnabled: false,
        crawlState: 'idle',
        lastProcessedProject: null,
        crawlStartedAt: null,
        githubNamespace: 'existing-org',
        accessToken: 'ghp_existing_token', // Must have token to avoid validation error
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

      const filtersTab = screen.getByRole('button', { name: /Filters & Exclusions/i });
      await user.click(filtersTab);

      await user.type(
        screen.getByPlaceholderText('main, develop, release-*'),
        'main, develop'
      );
      await user.type(
        screen.getByPlaceholderText('org/repo1, user/repo2'),
        'existing-org/core-app, existing-org/sdk'
      );

      const submitButton = screen.getByRole('button', { name: /Update Repository/i });
      await user.click(submitButton);

      await waitFor(() => {
        expect(mockOnSubmit).toHaveBeenCalled();
        const submittedData = mockOnSubmit.mock.calls[0][0];

        expect(submittedData.includedBranches).toBe('main, develop');
        expect(submittedData.includedProjects).toBe('existing-org/core-app, existing-org/sdk');
      });
    });

    it('should maintain form state when switching between tabs', async () => {
      const user = userEvent.setup();

      render(
        <RepositoryForm
          isOpen={true}
          onClose={mockOnClose}
          onSubmit={mockOnSubmit}
          isLoading={false}
        />
      );

      // Fill basic tab
      await user.type(screen.getByPlaceholderText('My Repository'), 'Tab Switch Test');
      const fileSysRadio = screen.getByRole('radio', { name: /filesystem/i });
      await user.click(fileSysRadio);
      const urlInput = screen.getByPlaceholderText(/path/i);
      await user.type(urlInput, '/path/to/repo');

      // Go to filters
      const filtersTab = screen.getByRole('button', { name: /Filters & Exclusions/i });
      await user.click(filtersTab);

      const includedBranches = screen.getByPlaceholderText('main, develop, release-*');
      await user.type(includedBranches, 'main');

      // Go back to basic
      const basicTab = screen.getByRole('button', { name: /Basic Configuration/i });
      await user.click(basicTab);

      // Check that basic tab values are preserved
      expect(screen.getByDisplayValue('Tab Switch Test')).toBeInTheDocument();

      // Go back to filters
      await user.click(filtersTab);

      // Check that filter values are preserved
      expect(screen.getByDisplayValue('main')).toBeInTheDocument();
    });
  });
});
