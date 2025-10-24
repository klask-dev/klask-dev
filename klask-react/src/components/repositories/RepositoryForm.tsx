import React, { useState, useCallback } from 'react';
import { useForm } from 'react-hook-form';
import { zodResolver } from '@hookform/resolvers/zod';
import { z } from 'zod';
import { XMarkIcon, FolderIcon, GlobeAltIcon, ServerIcon, EyeIcon, EyeSlashIcon, CogIcon, FunnelIcon, ClockIcon, ExclamationTriangleIcon, CheckCircleIcon } from '@heroicons/react/24/outline';
import type { Repository, RepositoryType } from '../../types';
import { LoadingSpinner } from '../ui/LoadingSpinner';
import { CronScheduleForm } from './CronScheduleForm';

const createRepositorySchema = (isEditing: boolean, hasExistingToken: boolean) => z.object({
  name: z
    .string()
    .min(1, 'Repository name is required')
    .min(2, 'Repository name must be at least 2 characters')
    .max(100, 'Repository name must be less than 100 characters'),
  url: z
    .string()
    .optional(),
  repositoryType: z.enum(['Git', 'GitLab', 'GitHub', 'FileSystem'] as const),
  branch: z
    .string()
    .optional()
    .refine((val) => !val || val.length >= 1, 'Branch name cannot be empty if provided'),
  accessToken: z
    .string()
    .optional()
    .refine(() => {
      // Only validate as required for new GitLab/GitHub repositories in create mode
      return true; // Let the main refine handle the validation contextually
    }),
  gitlabNamespace: z
    .string()
    .optional(),
  gitlabExcludedProjects: z
    .string()
    .optional(),
  gitlabExcludedPatterns: z
    .string()
    .optional(),
  isGroup: z.boolean().optional(),
  githubNamespace: z
    .string()
    .optional(),
  githubExcludedRepositories: z
    .string()
    .optional(),
  githubExcludedPatterns: z
    .string()
    .optional(),
  // Enhanced filtering fields for branch and project selection
  includedBranches: z
    .string()
    .optional(),
  includedBranchesPatterns: z
    .string()
    .optional(),
  excludedBranches: z
    .string()
    .optional(),
  excludedBranchesPatterns: z
    .string()
    .optional(),
  includedProjects: z
    .string()
    .optional(),
  includedProjectsPatterns: z
    .string()
    .optional(),
  enabled: z.boolean(),
}).refine((data) => {
  // For GitLab, accessToken is required only for new repositories
  // For editing, we allow empty token if it was previously set
  if (data.repositoryType === 'GitLab') {
    // For new repositories, accessToken is required
    if (!isEditing && (!data.accessToken || data.accessToken.trim() === '')) {
      return false;
    }
    // For editing, accessToken is optional if it was previously set
    if (isEditing && !hasExistingToken && (!data.accessToken || data.accessToken.trim() === '')) {
      return false;
    }
    // If URL is provided, validate it
    if (data.url && data.url.trim() !== '') {
      try {
        new URL(data.url);
      } catch {
        return false;
      }
    }
    return true;
  }
  // For GitHub, same logic as GitLab
  if (data.repositoryType === 'GitHub') {
    // For new repositories, accessToken is required
    if (!isEditing && (!data.accessToken || data.accessToken.trim() === '')) {
      return false;
    }
    // For editing, accessToken is optional if it was previously set
    if (isEditing && !hasExistingToken && (!data.accessToken || data.accessToken.trim() === '')) {
      return false;
    }
    // If URL is provided, validate it
    if (data.url && data.url.trim() !== '') {
      try {
        new URL(data.url);
      } catch {
        return false;
      }
    }
    return true;
  }
  // For Git, URL is required and must be valid
  if (data.repositoryType === 'Git') {
    if (!data.url || data.url.trim() === '') return false;
    try {
      new URL(data.url);
      return true;
    } catch {
      return false;
    }
  }
  // For FileSystem, validate as path
  if (data.repositoryType === 'FileSystem') {
    if (!data.url || data.url.trim() === '') return false;
    return data.url.startsWith('/') || data.url.match(/^[a-zA-Z]:[\\//]/);
  }
  return true;
}, {
  message: 'Please provide valid URL/path for the selected repository type',
  path: ['url'],
}).refine((data) => {
  // Additional validation for GitLab/GitHub access token
  if ((data.repositoryType === 'GitLab' || data.repositoryType === 'GitHub') && (!data.accessToken || data.accessToken.trim() === '')) {
    return false;
  }
  return true;
}, {
  message: 'Access token is required for GitLab and GitHub repositories',
  path: ['accessToken'],
});

// Create base type from the schema creation function
type BaseRepositoryFormData = z.infer<ReturnType<typeof createRepositorySchema>>;

// Define a type that includes scheduling data
type RepositoryFormSubmitData = BaseRepositoryFormData & {
  autoCrawlEnabled: boolean;
  cronSchedule?: string;
  crawlFrequencyHours?: number;
  maxCrawlDurationMinutes: number;
};

interface RepositoryFormProps {
  repository?: Repository;
  isOpen: boolean;
  onClose: () => void;
  onSubmit: (data: RepositoryFormSubmitData) => void;
  isLoading?: boolean;
  title?: string;
}

export const RepositoryForm: React.FC<RepositoryFormProps> = ({
  repository,
  isOpen,
  onClose,
  onSubmit,
  isLoading = false,
  title,
}) => {
  const isEditing = !!repository;
  const formTitle = title || (isEditing ? 'Edit Repository' : 'Add Repository');

  // Scheduling state
  const [schedulingData, setSchedulingData] = useState({
    autoCrawlEnabled: repository?.autoCrawlEnabled || false,
    cronSchedule: repository?.cronSchedule,
    crawlFrequencyHours: repository?.crawlFrequencyHours,
    maxCrawlDurationMinutes: repository?.maxCrawlDurationMinutes || 60,
  });

  // Track if scheduling data has changed for edit mode
  const [hasSchedulingChanged, setHasSchedulingChanged] = useState(false);

  // Track if user wants to change the access token in edit mode
  const [showTokenField, setShowTokenField] = useState(!isEditing);
  const hasExistingToken = isEditing && !!repository?.accessToken;

  // Password visibility state for tokens
  const [showGitLabToken, setShowGitLabToken] = useState(false);
  const [showGitHubToken, setShowGitHubToken] = useState(false);

  // Tab management
  const [activeTab, setActiveTab] = useState('basic');

  const repositorySchema = createRepositorySchema(isEditing, hasExistingToken);
  type RepositoryFormData = z.infer<typeof repositorySchema>;

  const {
    register,
    handleSubmit,
    watch,
    reset,
    formState: { errors, isValid, isDirty },
  } = useForm<RepositoryFormData>({
    resolver: zodResolver(repositorySchema),
    defaultValues: repository ? {
      name: repository.name,
      url: repository.url,
      repositoryType: repository.repositoryType,
      branch: repository.branch || '',
      enabled: repository.enabled,
      accessToken: repository.accessToken || '',
      gitlabNamespace: repository.gitlabNamespace || '',
      gitlabExcludedProjects: repository.gitlabExcludedProjects || '',
      gitlabExcludedPatterns: repository.gitlabExcludedPatterns || '',
      isGroup: repository.isGroup || false,
      githubNamespace: repository.githubNamespace || '',
      githubExcludedRepositories: repository.githubExcludedRepositories || '',
      githubExcludedPatterns: repository.githubExcludedPatterns || '',
      includedBranches: repository.includedBranches || '',
      includedBranchesPatterns: repository.includedBranchesPatterns || '',
      excludedBranches: repository.excludedBranches || '',
      excludedBranchesPatterns: repository.excludedBranchesPatterns || '',
      includedProjects: repository.includedProjects || '',
      includedProjectsPatterns: repository.includedProjectsPatterns || '',
    } : {
      name: '',
      url: '',
      repositoryType: 'Git',
      branch: '',
      enabled: true,
      accessToken: '',
      gitlabNamespace: '',
      gitlabExcludedProjects: '',
      gitlabExcludedPatterns: '',
      isGroup: false,
      githubNamespace: '',
      githubExcludedRepositories: '',
      githubExcludedPatterns: '',
      includedBranches: '',
      includedBranchesPatterns: '',
      excludedBranches: '',
      excludedBranchesPatterns: '',
      includedProjects: '',
      includedProjectsPatterns: '',
    },
  });

  const watchedType = watch('repositoryType');

  // Auto-switch to appropriate tab when repository type changes
  React.useEffect(() => {
    const newTabs = getTabsForType(watchedType);
    // If current tab doesn't exist for the new repository type, switch to basic
    if (!newTabs.find(tab => tab.id === activeTab)) {
      setActiveTab('basic');
    }
  }, [watchedType, activeTab]);

  // Tab configuration based on repository type
  const getTabsForType = (type: RepositoryType) => {
    const baseTabs = [
      {
        id: 'basic',
        name: 'Basic Configuration',
        icon: CogIcon,
        required: true,
        fields: ['name', 'repositoryType', 'url', 'branch']
      }
    ];

    if (type === 'GitLab' || type === 'GitHub') {
      baseTabs.push({
        id: 'provider',
        name: `${type} Settings`,
        icon: GlobeAltIcon,
        required: true,
        fields: ['accessToken', `${type.toLowerCase()}Namespace`]
      });
    }

    baseTabs.push(
      {
        id: 'filters',
        name: 'Filters & Exclusions',
        icon: FunnelIcon,
        required: false,
        fields: type === 'GitLab'
          ? ['gitlabExcludedProjects', 'gitlabExcludedPatterns']
          : type === 'GitHub'
          ? ['githubExcludedRepositories', 'githubExcludedPatterns']
          : []
      },
      {
        id: 'scheduling',
        name: 'Auto Crawling',
        icon: ClockIcon,
        required: false,
        fields: ['autoCrawlEnabled', 'cronSchedule', 'crawlFrequencyHours', 'maxCrawlDurationMinutes']
      }
    );

    return baseTabs;
  };

  const tabs = getTabsForType(watchedType);

  // Check if tab has validation errors
  const getTabValidationState = (tabId: string) => {
    const tab = tabs.find(t => t.id === tabId);
    if (!tab) return { hasErrors: false, hasWarnings: false };

    // Check for actual validation errors in this tab's fields
    const tabErrors = tab.fields.filter(field => {
      // Handle nested field names like 'gitlabNamespace' -> 'gitlabNamespace'
      const fieldName = field as keyof typeof errors;
      return errors[fieldName];
    });

    // Check for required fields that are empty
    const hasRequiredEmpty = tab.required && tab.fields.some(field => {
      const value = watch(field as keyof RepositoryFormData);

      // Special cases for conditional requirements
      if (field === 'accessToken' && hasExistingToken && !showTokenField) return false;
      if (field === 'url' && (watchedType === 'GitLab' || watchedType === 'GitHub')) return false;
      if (field === 'repositoryType') return false; // Always has a default value

      // Required fields should have values
      if (field === 'name') return !value || (typeof value === 'string' && value.trim() === '');
      if (field === 'accessToken' && (watchedType === 'GitLab' || watchedType === 'GitHub')) {
        return !value || (typeof value === 'string' && value.trim() === '');
      }
      if (field === 'url' && watchedType === 'Git') {
        return !value || (typeof value === 'string' && value.trim() === '');
      }
      if (field === 'url' && watchedType === 'FileSystem') {
        return !value || (typeof value === 'string' && value.trim() === '');
      }

      return false;
    });

    return {
      hasErrors: tabErrors.length > 0,
      hasWarnings: hasRequiredEmpty,
      errorCount: tabErrors.length
    };
  };

  // Monitor button state for validation
  React.useEffect(() => {
    // Button state is computed dynamically in the submit button element
  }, [isValid, isLoading, isEditing, isDirty, hasSchedulingChanged, errors]);

  // Handle scheduling data changes
  const handleScheduleChange = useCallback((newSchedulingData: {
    autoCrawlEnabled: boolean;
    cronSchedule?: string;
    crawlFrequencyHours?: number;
    maxCrawlDurationMinutes?: number;
  }) => {
    setSchedulingData({
      autoCrawlEnabled: newSchedulingData.autoCrawlEnabled,
      cronSchedule: newSchedulingData.cronSchedule || undefined,
      crawlFrequencyHours: newSchedulingData.crawlFrequencyHours || undefined,
      maxCrawlDurationMinutes: newSchedulingData.maxCrawlDurationMinutes || 60,
    });

    if (isEditing && repository) {
      // Check if scheduling data has changed
      // For comparison, treat undefined, null, and empty string as equivalent
      const areEqual = (a: string | null | undefined, b: string | null | undefined) => {
        if ((a === undefined || a === null || a === '') &&
            (b === undefined || b === null || b === '')) {
          return true;
        }
        return a === b;
      };

      const hasChanged =
        repository.autoCrawlEnabled !== newSchedulingData.autoCrawlEnabled ||
        !areEqual(repository.cronSchedule, newSchedulingData.cronSchedule) ||
        repository.crawlFrequencyHours !== newSchedulingData.crawlFrequencyHours ||
        (repository.maxCrawlDurationMinutes || 60) !== newSchedulingData.maxCrawlDurationMinutes;

      // Only log when there's an actual change
      if (hasChanged) {
        console.log('Schedule changed:', hasChanged);
      }

      setHasSchedulingChanged(hasChanged);
    }
  }, [isEditing, repository]);

  // Track the repository ID to detect when we switch repositories
  const [currentRepositoryId, setCurrentRepositoryId] = React.useState(repository?.id);

  React.useEffect(() => {
    // Only reset when we actually switch to a different repository
    if (repository?.id !== currentRepositoryId) {
      setCurrentRepositoryId(repository?.id);

      if (repository) {
        const formData = {
          name: repository.name,
          url: repository.url,
          repositoryType: repository.repositoryType,
          branch: repository.branch || '',
          enabled: repository.enabled,
          accessToken: repository.accessToken || '',
          gitlabNamespace: repository.gitlabNamespace || '',
          gitlabExcludedProjects: repository.gitlabExcludedProjects || '',
          gitlabExcludedPatterns: repository.gitlabExcludedPatterns || '',
          isGroup: repository.isGroup || false,
          githubNamespace: repository.githubNamespace || '',
          githubExcludedRepositories: repository.githubExcludedRepositories || '',
          githubExcludedPatterns: repository.githubExcludedPatterns || '',
          includedBranches: repository.includedBranches || '',
          includedBranchesPatterns: repository.includedBranchesPatterns || '',
          excludedBranches: repository.excludedBranches || '',
          excludedBranchesPatterns: repository.excludedBranchesPatterns || '',
          includedProjects: repository.includedProjects || '',
          includedProjectsPatterns: repository.includedProjectsPatterns || '',
        };
        reset(formData);
        setSchedulingData({
          autoCrawlEnabled: repository.autoCrawlEnabled,
          cronSchedule: repository.cronSchedule,
          crawlFrequencyHours: repository.crawlFrequencyHours,
          maxCrawlDurationMinutes: repository.maxCrawlDurationMinutes || 60,
        });
        setHasSchedulingChanged(false);
      } else {
        const formData = {
          name: '',
          url: '',
          repositoryType: 'Git' as RepositoryType,
          branch: '',
          enabled: true,
          accessToken: '',
          gitlabNamespace: '',
          gitlabExcludedProjects: '',
          gitlabExcludedPatterns: '',
          isGroup: false,
          githubNamespace: '',
          githubExcludedRepositories: '',
          githubExcludedPatterns: '',
          includedBranches: '',
          includedBranchesPatterns: '',
          excludedBranches: '',
          excludedBranchesPatterns: '',
          includedProjects: '',
          includedProjectsPatterns: '',
        };
        reset(formData);
        setSchedulingData({
          autoCrawlEnabled: false,
          cronSchedule: undefined,
          crawlFrequencyHours: undefined,
          maxCrawlDurationMinutes: 60,
        });
        setHasSchedulingChanged(false);
      }
    }
  }, [repository?.id, repository, reset, currentRepositoryId]);

  const handleFormSubmit = (data: RepositoryFormData) => {
    // Clean up empty strings from all filter fields and merge scheduling data
    const submitData: RepositoryFormSubmitData = {
      ...data,
      branch: data.branch?.trim() || undefined,
      includedBranches: data.includedBranches?.trim() || undefined,
      includedBranchesPatterns: data.includedBranchesPatterns?.trim() || undefined,
      excludedBranches: data.excludedBranches?.trim() || undefined,
      excludedBranchesPatterns: data.excludedBranchesPatterns?.trim() || undefined,
      includedProjects: data.includedProjects?.trim() || undefined,
      includedProjectsPatterns: data.includedProjectsPatterns?.trim() || undefined,
      // For GitLab repositories, default to gitlab.com if URL is empty
      url: data.repositoryType === 'GitLab' && (!data.url || data.url.trim() === '')
        ? 'https://gitlab.com'
        : data.repositoryType === 'GitHub' && (!data.url || data.url.trim() === '')
        ? 'https://api.github.com'
        : data.url,
      // If we're editing and not showing the token field, preserve the existing token
      accessToken: hasExistingToken && !showTokenField
        ? repository?.accessToken
        : data.accessToken,
      ...schedulingData,
    };
    onSubmit(submitData);
  };

  // Auto-navigate to tab with errors on form submission failure
  const handleSubmitWithErrorNavigation = (e: React.FormEvent) => {
    e.preventDefault();

    // Check if there are validation errors and navigate to first tab with errors
    if (Object.keys(errors).length > 0) {
      const firstTabWithError = tabs.find(tab => {
        const validation = getTabValidationState(tab.id);
        return validation.hasErrors;
      });

      if (firstTabWithError && firstTabWithError.id !== activeTab) {
        setActiveTab(firstTabWithError.id);
        return;
      }
    }

    handleSubmit(handleFormSubmit)(e);
  };

  const getTypeIcon = (type: RepositoryType) => {
    switch (type) {
      case 'Git':
        return <GlobeAltIcon className="h-5 w-5" />;
      case 'GitLab':
        return <GlobeAltIcon className="h-5 w-5" />;
      case 'FileSystem':
        return <ServerIcon className="h-5 w-5" />;
      default:
        return <FolderIcon className="h-5 w-5" />;
    }
  };

  const getPlaceholderUrl = (type: RepositoryType) => {
    switch (type) {
      case 'Git':
        return 'https://github.com/user/repository.git';
      case 'GitLab':
        return 'https://gitlab.com/user/repository.git';
      case 'FileSystem':
        return '/path/to/local/directory';
      default:
        return 'Enter repository URL';
    }
  };

  if (!isOpen) return null;

  return (
    <div className="fixed inset-0 z-50 overflow-y-auto">
      <div className="flex items-center justify-center min-h-screen px-4 pt-4 pb-20 text-center sm:block sm:p-0">
        {/* Backdrop */}
        <div className="fixed inset-0 transition-opacity bg-gray-500 bg-opacity-75" onClick={onClose} />

        {/* Modal */}
        <div className="inline-block w-full max-w-4xl p-6 my-8 overflow-hidden text-left align-middle transition-all transform bg-white dark:bg-gray-800 shadow-xl rounded-lg">
          {/* Header */}
          <div className="flex items-center justify-between mb-6">
            <h3 className="text-lg font-semibold text-gray-900 dark:text-white">
              {formTitle}
            </h3>
            <button
              onClick={onClose}
              className="text-gray-400 dark:text-gray-500 hover:text-gray-600 dark:hover:text-gray-300 transition-colors"
            >
              <XMarkIcon className="h-6 w-6" />
            </button>
          </div>

          {/* Tab Navigation */}
          <div className="border-b border-gray-200 dark:border-gray-700 mb-6">
            <nav className="-mb-px flex space-x-8">
              {tabs.map((tab) => {
                const validation = getTabValidationState(tab.id);
                const isActive = activeTab === tab.id;

                return (
                  <button
                    key={tab.id}
                    type="button"
                    onClick={() => setActiveTab(tab.id)}
                    className={`group inline-flex items-center py-2 px-1 border-b-2 font-medium text-sm transition-colors ${
                      isActive
                        ? 'border-blue-500 text-blue-600 dark:text-blue-400'
                        : 'border-transparent text-gray-500 dark:text-gray-400 hover:text-gray-700 dark:hover:text-gray-300 hover:border-gray-300 dark:hover:border-gray-600'
                    }`}
                  >
                    <tab.icon className={`mr-2 h-5 w-5 ${
                      isActive ? 'text-blue-500 dark:text-blue-400' : 'text-gray-400 dark:text-gray-500 group-hover:text-gray-500 dark:group-hover:text-gray-400'
                    }`} />
                    {tab.name}
                    {validation.hasErrors && (
                      <ExclamationTriangleIcon className="ml-2 h-4 w-4 text-red-500 dark:text-red-400" />
                    )}
                    {!validation.hasErrors && validation.hasWarnings && (
                      <div className="ml-2 h-2 w-2 bg-amber-400 dark:bg-amber-500 rounded-full" />
                    )}
                    {!validation.hasErrors && !validation.hasWarnings && tab.required && (
                      <CheckCircleIcon className="ml-2 h-4 w-4 text-green-500 dark:text-green-400" />
                    )}
                  </button>
                );
              })}
            </nav>
          </div>

          {/* Form */}
          <form onSubmit={handleSubmitWithErrorNavigation} className="space-y-6">
            {/* Tab Content */}
            <div className="min-h-[400px]">
              {/* Basic Configuration Tab */}
              {activeTab === 'basic' && (
                <div className="space-y-6">
                  {/* Repository Name */}
                  <div>
                    <label htmlFor="name" className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">
                      Repository Name *
                    </label>
                    <input
                      {...register('name')}
                      type="text"
                      className={`input-field ${errors.name ? 'border-red-300 focus:ring-red-500 focus:border-red-500' : ''}`}
                      placeholder="My Repository"
                    />
                    {errors.name && (
                      <p className="mt-1 text-sm text-red-600">{errors.name.message}</p>
                    )}
                  </div>

                  {/* Repository Type */}
                  <div>
                    <label htmlFor="repositoryType" className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">
                      Repository Type *
                    </label>
                    <div className="grid grid-cols-2 sm:grid-cols-4 gap-3">
                      {(['Git', 'GitLab', 'GitHub', 'FileSystem'] as const).map((type) => (
                        <label
                          key={type}
                          className={`relative flex items-center justify-center p-3 border rounded-lg cursor-pointer transition-colors ${
                            watchedType === type
                              ? 'border-blue-500 bg-blue-50 dark:bg-blue-900/20 text-blue-700 dark:text-blue-400'
                              : 'border-gray-300 dark:border-gray-600 hover:border-gray-400 dark:hover:border-gray-500'
                          }`}
                        >
                          <input
                            {...register('repositoryType')}
                            type="radio"
                            value={type}
                            className="sr-only"
                          />
                          <div className="flex flex-col items-center space-y-1">
                            <div className={watchedType === type ? 'text-blue-600 dark:text-blue-400' : 'text-gray-600 dark:text-gray-400'}>
                              {getTypeIcon(type)}
                            </div>
                            <span className="text-xs font-medium text-gray-900 dark:text-gray-100">{type}</span>
                          </div>
                        </label>
                      ))}
                    </div>
                    {errors.repositoryType && (
                      <p className="mt-1 text-sm text-red-600">{errors.repositoryType.message}</p>
                    )}
                  </div>

                  {/* Repository URL - Not for GitLab/GitHub types */}
                  {watchedType !== 'GitLab' && watchedType !== 'GitHub' && (
                    <div>
                      <label htmlFor="url" className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">
                        Repository URL *
                      </label>
                      <input
                        {...register('url')}
                        type={watchedType === 'FileSystem' ? 'text' : 'url'}
                        className={`input-field ${errors.url ? 'border-red-300 focus:ring-red-500 focus:border-red-500' : ''}`}
                        placeholder={getPlaceholderUrl(watchedType)}
                      />
                      {errors.url && (
                        <p className="mt-1 text-sm text-red-600">{errors.url.message}</p>
                      )}
                      <p className="mt-1 text-xs text-gray-500">
                        {watchedType === 'FileSystem'
                          ? 'Enter the absolute path to the directory'
                          : 'Enter the full URL to the repository'
                        }
                      </p>
                    </div>
                  )}

                  {/* Enabled Toggle */}
                  <div className="flex items-center">
                    <input
                      {...register('enabled')}
                      type="checkbox"
                      className="h-4 w-4 text-blue-600 focus:ring-blue-500 border-gray-300 dark:border-gray-600 dark:bg-gray-700 rounded"
                    />
                    <label htmlFor="enabled" className="ml-2 block text-sm text-gray-900 dark:text-gray-300">
                      Enable this repository for crawling
                    </label>
                  </div>
                </div>
              )}

              {/* GitLab/GitHub Settings Tab */}
              {activeTab === 'provider' && (watchedType === 'GitLab' || watchedType === 'GitHub') && (
                <div className="space-y-6">
                  {watchedType === 'GitLab' && (
                    <>
                      <div className="p-3 bg-blue-50 dark:bg-blue-800/20 border border-blue-200 dark:border-blue-700/50 rounded-lg">
                        <p className="text-sm text-blue-800 dark:text-blue-300">
                          GitLab repositories will be automatically discovered and imported using your access token.
                        </p>
                      </div>

                      <div>
                        <label htmlFor="url" className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">
                          GitLab Server URL (Optional)
                        </label>
                        <input
                          {...register('url')}
                          type="url"
                          className={`input-field ${errors.url ? 'border-red-300 focus:ring-red-500 focus:border-red-500' : ''}`}
                          placeholder="https://gitlab.com"
                        />
                        {errors.url && (
                          <p className="mt-1 text-sm text-red-600">{errors.url.message}</p>
                        )}
                        <p className="mt-1 text-xs text-gray-500 dark:text-gray-400">
                          Leave empty to use gitlab.com, or enter your self-hosted GitLab URL
                        </p>
                      </div>

                      <div>
                        <label htmlFor="accessToken" className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">
                          Personal Access Token {!isEditing || !hasExistingToken ? '*' : ''}
                        </label>

                        {hasExistingToken && !showTokenField ? (
                          <div className="space-y-2">
                            <div className="flex items-center justify-between p-3 bg-green-50 dark:bg-green-900/20 border border-green-200 dark:border-green-700/50 rounded-md">
                              <div className="flex items-center">
                                <div className="w-2 h-2 bg-green-400 dark:bg-green-500 rounded-full mr-2"></div>
                                <span className="text-sm text-green-700 dark:text-green-300">Access token configured</span>
                              </div>
                              <button
                                type="button"
                                onClick={() => setShowTokenField(true)}
                                className="text-sm text-blue-600 dark:text-blue-400 hover:text-blue-800 dark:hover:text-blue-300 underline"
                              >
                                Change token
                              </button>
                            </div>
                          </div>
                        ) : (
                          <div className="space-y-2">
                            <div className="relative">
                              <input
                                {...register('accessToken')}
                                type={showGitLabToken ? 'text' : 'password'}
                                className={`input-field pr-10 ${errors.accessToken ? 'border-red-300 focus:ring-red-500 focus:border-red-500' : ''}`}
                                placeholder="glpat-..."
                                required={!isEditing || !hasExistingToken}
                              />
                              <button
                                type="button"
                                onClick={() => setShowGitLabToken(!showGitLabToken)}
                                className="absolute inset-y-0 right-0 flex items-center pr-3"
                                aria-label={showGitLabToken ? 'Hide token' : 'Show token'}
                                title={showGitLabToken ? 'Hide token' : 'Show token'}
                              >
                                {showGitLabToken ? (
                                  <EyeSlashIcon className="h-5 w-5 text-gray-400 dark:text-gray-500 hover:text-gray-600 dark:hover:text-gray-400" />
                                ) : (
                                  <EyeIcon className="h-5 w-5 text-gray-400 dark:text-gray-500 hover:text-gray-600 dark:hover:text-gray-400" />
                                )}
                              </button>
                            </div>
                            {hasExistingToken && showTokenField && (
                              <button
                                type="button"
                                onClick={() => setShowTokenField(false)}
                                className="text-sm text-gray-600 dark:text-gray-400 hover:text-gray-800 dark:hover:text-gray-300 underline"
                              >
                                Keep existing token
                              </button>
                            )}
                          </div>
                        )}

                        {errors.accessToken && (
                          <p className="mt-1 text-sm text-red-600 dark:text-red-400">{errors.accessToken.message}</p>
                        )}
                        <p className="mt-1 text-xs text-gray-500 dark:text-gray-400">
                          Create a token with 'read_repository' scope in GitLab settings
                        </p>
                      </div>

                    </>
                  )}

                  {watchedType === 'GitHub' && (
                    <>
                      <div className="p-3 bg-blue-50 dark:bg-blue-800/20 border border-blue-200 dark:border-blue-700/50 rounded-lg">
                        <p className="text-sm text-blue-800 dark:text-blue-300">
                          GitHub repositories will be automatically discovered and imported using your Personal Access Token.
                        </p>
                      </div>

                      <div>
                        <label htmlFor="url" className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">
                          GitHub API URL (Optional)
                        </label>
                        <input
                          {...register('url')}
                          type="url"
                          className={`input-field ${errors.url ? 'border-red-300 focus:ring-red-500 focus:border-red-500' : ''}`}
                          placeholder="https://api.github.com"
                        />
                        {errors.url && (
                          <p className="mt-1 text-sm text-red-600">{errors.url.message}</p>
                        )}
                        <p className="mt-1 text-xs text-gray-500">
                          Leave empty to use github.com, or enter your GitHub Enterprise API URL
                        </p>
                      </div>

                      <div>
                        <label htmlFor="accessToken" className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">
                          Personal Access Token {!isEditing || !hasExistingToken ? '*' : ''}
                        </label>

                        {hasExistingToken && !showTokenField ? (
                          <div className="space-y-2">
                            <div className="flex items-center justify-between p-3 bg-green-50 dark:bg-green-900/20 border border-green-200 dark:border-green-700/50 rounded-md">
                              <div className="flex items-center">
                                <div className="w-2 h-2 bg-green-400 dark:bg-green-500 rounded-full mr-2"></div>
                                <span className="text-sm text-green-700 dark:text-green-300">Access token configured</span>
                              </div>
                              <button
                                type="button"
                                onClick={() => setShowTokenField(true)}
                                className="text-sm text-blue-600 dark:text-blue-400 hover:text-blue-800 dark:hover:text-blue-300 underline"
                              >
                                Change token
                              </button>
                            </div>
                          </div>
                        ) : (
                          <div className="space-y-2">
                            <div className="relative">
                              <input
                                {...register('accessToken')}
                                type={showGitHubToken ? 'text' : 'password'}
                                className={`input-field pr-10 ${errors.accessToken ? 'border-red-300 focus:ring-red-500 focus:border-red-500' : ''}`}
                                placeholder="ghp_..."
                                required={!isEditing || !hasExistingToken}
                              />
                              <button
                                type="button"
                                onClick={() => setShowGitHubToken(!showGitHubToken)}
                                className="absolute inset-y-0 right-0 flex items-center pr-3"
                                aria-label={showGitHubToken ? 'Hide token' : 'Show token'}
                                title={showGitHubToken ? 'Hide token' : 'Show token'}
                              >
                                {showGitHubToken ? (
                                  <EyeSlashIcon className="h-5 w-5 text-gray-400 dark:text-gray-500 hover:text-gray-600 dark:hover:text-gray-400" />
                                ) : (
                                  <EyeIcon className="h-5 w-5 text-gray-400 dark:text-gray-500 hover:text-gray-600 dark:hover:text-gray-400" />
                                )}
                              </button>
                            </div>
                            {hasExistingToken && showTokenField && (
                              <button
                                type="button"
                                onClick={() => setShowTokenField(false)}
                                className="text-sm text-gray-600 dark:text-gray-400 hover:text-gray-800 dark:hover:text-gray-300 underline"
                              >
                                Keep existing token
                              </button>
                            )}
                          </div>
                        )}

                        {errors.accessToken && (
                          <p className="mt-1 text-sm text-red-600 dark:text-red-400">{errors.accessToken.message}</p>
                        )}
                        <p className="mt-1 text-xs text-gray-500 dark:text-gray-400">
                          Create a token with 'repo' scope in GitHub settings
                        </p>
                      </div>

                    </>
                  )}
                </div>
              )}

              {/* Filters & Exclusions Tab */}
              {activeTab === 'filters' && (
                <div className="space-y-6">
                  {/* Branch Selection Section */}
                  <div>
                    <h3 className="text-lg font-semibold text-gray-900 dark:text-white mb-4">Branch Selection</h3>
                    <div className="space-y-4">
                      {/* Included Branches */}
                      <div>
                        <label htmlFor="includedBranches" className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">
                          Included Branches (Optional)
                        </label>
                        <input
                          {...register('includedBranches')}
                          type="text"
                          className={`input-field ${errors.includedBranches ? 'border-red-300 focus:ring-red-500 focus:border-red-500' : ''}`}
                          placeholder="main, develop, release-*"
                        />
                        {errors.includedBranches && (
                          <p className="mt-1 text-sm text-red-600">{errors.includedBranches.message}</p>
                        )}
                        <p className="mt-1 text-xs text-gray-500 dark:text-gray-400">
                          Comma-separated list of exact branch names or patterns to include
                        </p>
                      </div>

                      {/* Included Branches Patterns */}
                      <div>
                        <label htmlFor="includedBranchesPatterns" className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">
                          Included Branches Patterns (Optional)
                        </label>
                        <input
                          {...register('includedBranchesPatterns')}
                          type="text"
                          className={`input-field ${errors.includedBranchesPatterns ? 'border-red-300 focus:ring-red-500 focus:border-red-500' : ''}`}
                          placeholder="release-*, hotfix-*, feature-*"
                        />
                        {errors.includedBranchesPatterns && (
                          <p className="mt-1 text-sm text-red-600">{errors.includedBranchesPatterns.message}</p>
                        )}
                        <p className="mt-1 text-xs text-gray-500 dark:text-gray-400">
                          Comma-separated list of wildcard patterns (e.g., release-*, hotfix-*)
                        </p>
                      </div>

                      {/* Excluded Branches */}
                      <div>
                        <label htmlFor="excludedBranches" className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">
                          Excluded Branches (Optional)
                        </label>
                        <input
                          {...register('excludedBranches')}
                          type="text"
                          className={`input-field ${errors.excludedBranches ? 'border-red-300 focus:ring-red-500 focus:border-red-500' : ''}`}
                          placeholder="temp-branch, wip-branch"
                        />
                        {errors.excludedBranches && (
                          <p className="mt-1 text-sm text-red-600">{errors.excludedBranches.message}</p>
                        )}
                        <p className="mt-1 text-xs text-gray-500 dark:text-gray-400">
                          Comma-separated list of exact branch names to exclude
                        </p>
                      </div>

                      {/* Excluded Branches Patterns */}
                      <div>
                        <label htmlFor="excludedBranchesPatterns" className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">
                          Excluded Branches Patterns (Optional)
                        </label>
                        <input
                          {...register('excludedBranchesPatterns')}
                          type="text"
                          className={`input-field ${errors.excludedBranchesPatterns ? 'border-red-300 focus:ring-red-500 focus:border-red-500' : ''}`}
                          placeholder="archive-*, backup-*, test-*"
                        />
                        {errors.excludedBranchesPatterns && (
                          <p className="mt-1 text-sm text-red-600">{errors.excludedBranchesPatterns.message}</p>
                        )}
                        <p className="mt-1 text-xs text-gray-500 dark:text-gray-400">
                          Comma-separated list of wildcard patterns to exclude
                        </p>
                      </div>
                    </div>
                  </div>

                  {/* Project/Repository Selection Section - GitLab and GitHub only */}
                  {(watchedType === 'GitLab' || watchedType === 'GitHub') && (
                    <div className="border-t border-gray-200 dark:border-gray-700 pt-6">
                      <h3 className="text-lg font-semibold text-gray-900 dark:text-white mb-4">
                        {watchedType === 'GitLab' ? 'Project' : 'Repository'} Selection
                      </h3>
                      <div className="space-y-4">
                        {/* Included Projects/Repositories */}
                        <div>
                          <label
                            htmlFor="includedProjects"
                            className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1"
                          >
                            {watchedType === 'GitLab' ? 'Included Projects' : 'Included Repositories'} (Optional)
                          </label>
                          <input
                            {...register('includedProjects')}
                            type="text"
                            className={`input-field ${errors.includedProjects ? 'border-red-300 focus:ring-red-500 focus:border-red-500' : ''}`}
                            placeholder={watchedType === 'GitLab' ? 'team/project1, org/repo2' : 'org/repo1, user/repo2'}
                          />
                          {errors.includedProjects && (
                            <p className="mt-1 text-sm text-red-600">{errors.includedProjects.message}</p>
                          )}
                          <p className="mt-1 text-xs text-gray-500 dark:text-gray-400">
                            Comma-separated list of exact {watchedType === 'GitLab' ? 'project' : 'repository'} paths to include
                          </p>
                        </div>

                        {/* Included Projects/Repositories Patterns */}
                        <div>
                          <label
                            htmlFor="includedProjectsPatterns"
                            className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1"
                          >
                            {watchedType === 'GitLab' ? 'Included Projects Patterns' : 'Included Repositories Patterns'} (Optional)
                          </label>
                          <input
                            {...register('includedProjectsPatterns')}
                            type="text"
                            className={`input-field ${errors.includedProjectsPatterns ? 'border-red-300 focus:ring-red-500 focus:border-red-500' : ''}`}
                            placeholder={watchedType === 'GitLab' ? 'team/*, team-*/core-*' : 'org/*, user/core-*'}
                          />
                          {errors.includedProjectsPatterns && (
                            <p className="mt-1 text-sm text-red-600">{errors.includedProjectsPatterns.message}</p>
                          )}
                          <p className="mt-1 text-xs text-gray-500 dark:text-gray-400">
                            Comma-separated list of wildcard patterns (replaces old namespace filter)
                          </p>
                        </div>

                        {/* Excluded Projects/Repositories */}
                        <div>
                          <label
                            htmlFor={watchedType === 'GitLab' ? 'gitlabExcludedProjects' : 'githubExcludedRepositories'}
                            className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1"
                          >
                            {watchedType === 'GitLab' ? 'Excluded Projects' : 'Excluded Repositories'} (Optional)
                          </label>
                          <input
                            {...register(watchedType === 'GitLab' ? 'gitlabExcludedProjects' : 'githubExcludedRepositories')}
                            type="text"
                            className={`input-field ${
                              watchedType === 'GitLab'
                                ? errors.gitlabExcludedProjects ? 'border-red-300 focus:ring-red-500 focus:border-red-500' : ''
                                : errors.githubExcludedRepositories ? 'border-red-300 focus:ring-red-500 focus:border-red-500' : ''
                            }`}
                            placeholder={watchedType === 'GitLab' ? 'team/project-archive, old/legacy-system' : 'org/repo-archive, user/legacy-project'}
                          />
                          {watchedType === 'GitLab' && errors.gitlabExcludedProjects && (
                            <p className="mt-1 text-sm text-red-600">{errors.gitlabExcludedProjects.message}</p>
                          )}
                          {watchedType === 'GitHub' && errors.githubExcludedRepositories && (
                            <p className="mt-1 text-sm text-red-600">{errors.githubExcludedRepositories.message}</p>
                          )}
                          <p className="mt-1 text-xs text-gray-500 dark:text-gray-400">
                            Comma-separated list of exact {watchedType === 'GitLab' ? 'project' : 'repository'} paths to exclude
                          </p>
                        </div>

                        {/* Excluded Patterns */}
                        <div>
                          <label
                            htmlFor={watchedType === 'GitLab' ? 'gitlabExcludedPatterns' : 'githubExcludedPatterns'}
                            className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1"
                          >
                            Excluded Patterns (Optional)
                          </label>
                          <input
                            {...register(watchedType === 'GitLab' ? 'gitlabExcludedPatterns' : 'githubExcludedPatterns')}
                            type="text"
                            className={`input-field ${
                              watchedType === 'GitLab'
                                ? errors.gitlabExcludedPatterns ? 'border-red-300 focus:ring-red-500 focus:border-red-500' : ''
                                : errors.githubExcludedPatterns ? 'border-red-300 focus:ring-red-500 focus:border-red-500' : ''
                            }`}
                            placeholder="*-archive, test-*, *-temp"
                          />
                          {watchedType === 'GitLab' && errors.gitlabExcludedPatterns && (
                            <p className="mt-1 text-sm text-red-600 dark:text-red-400">{errors.gitlabExcludedPatterns.message}</p>
                          )}
                          {watchedType === 'GitHub' && errors.githubExcludedPatterns && (
                            <p className="mt-1 text-sm text-red-600 dark:text-red-400">{errors.githubExcludedPatterns.message}</p>
                          )}
                          <p className="mt-1 text-xs text-gray-500 dark:text-gray-400">
                            Comma-separated patterns with wildcards (*) to exclude {watchedType === 'GitLab' ? 'projects' : 'repositories'}
                          </p>
                        </div>
                      </div>
                    </div>
                  )}

                  {watchedType === 'FileSystem' && (
                    <div className="text-center py-8 text-gray-500 dark:text-gray-400">
                      <FunnelIcon className="h-12 w-12 mx-auto mb-4 text-gray-300 dark:text-gray-600" />
                      <p>No filter options available for FileSystem repositories.</p>
                    </div>
                  )}
                </div>
              )}

              {/* Auto Crawling / Scheduling Tab */}
              {activeTab === 'scheduling' && (
                <div className="space-y-6">
                  <div className="p-4 bg-gray-50 dark:bg-gray-800 border border-gray-200 dark:border-gray-600 rounded-lg">
                    <h4 className="text-md font-medium text-gray-900 dark:text-white mb-4">Automatic Crawling Schedule</h4>
                    <CronScheduleForm
                      autoCrawlEnabled={schedulingData.autoCrawlEnabled}
                      cronSchedule={schedulingData.cronSchedule}
                      crawlFrequencyHours={schedulingData.crawlFrequencyHours}
                      maxCrawlDurationMinutes={schedulingData.maxCrawlDurationMinutes}
                      onScheduleChange={handleScheduleChange}
                      repositoryId={repository?.id}
                    />
                  </div>
                </div>
              )}
            </div>

            {/* Tab Navigation & Actions */}
            <div className="flex items-center justify-between pt-6 border-t border-gray-200 dark:border-gray-700">
              {/* Tab Navigation Buttons */}
              <div className="flex space-x-3">
                {/* Previous Tab Button */}
                {tabs.findIndex(tab => tab.id === activeTab) > 0 && (
                  <button
                    type="button"
                    onClick={() => {
                      const currentIndex = tabs.findIndex(tab => tab.id === activeTab);
                      if (currentIndex > 0) {
                        setActiveTab(tabs[currentIndex - 1].id);
                      }
                    }}
                    className="btn-secondary"
                  >
                    Previous
                  </button>
                )}

                {/* Next Tab Button */}
                {tabs.findIndex(tab => tab.id === activeTab) < tabs.length - 1 && (
                  <button
                    type="button"
                    onClick={() => {
                      const currentIndex = tabs.findIndex(tab => tab.id === activeTab);
                      if (currentIndex < tabs.length - 1) {
                        setActiveTab(tabs[currentIndex + 1].id);
                      }
                    }}
                    className="btn-primary"
                  >
                    Next
                  </button>
                )}
              </div>

              {/* Main Actions */}
              <div className="flex items-center space-x-3">
                <button
                  type="button"
                  onClick={onClose}
                  className="btn-secondary"
                  disabled={isLoading}
                >
                  Cancel
                </button>
                <button
                  type="submit"
                  disabled={(() => {
                    // Always disable if invalid or loading
                    if (!isValid || isLoading) return true;

                    // For new repositories, enable if valid
                    if (!isEditing) return false;

                    // For editing, enable if anything changed
                    return !isDirty && !hasSchedulingChanged;
                  })()}
                  className="btn-primary disabled:opacity-50 disabled:cursor-not-allowed flex items-center"
                >
                  {isLoading ? (
                    <>
                      <LoadingSpinner size="sm" className="mr-2" />
                      {isEditing ? 'Updating...' : 'Creating...'}
                    </>
                  ) : (
                    <>
                      {isEditing ? 'Update Repository' : 'Create Repository'}
                    </>
                  )}
                </button>
              </div>
            </div>
          </form>
        </div>
      </div>
    </div>
  );
};
