import React from 'react';
import {
  ServerIcon,
  UsersIcon,
  FolderIcon,
  MagnifyingGlassIcon,
  DocumentDuplicateIcon,
  ClockIcon,
  CogIcon,
  ArrowPathIcon
} from '@heroicons/react/24/outline';
import { MetricCard } from '../../components/admin/MetricCard';
import { RepositoryBadge } from '../../components/ui/RepositoryBadge';
import { useAdminDashboard } from '../../hooks/useAdmin';
import { formatDateTime } from '../../lib/utils';

const AdminDashboard: React.FC = () => {
  const { data: dashboardData, isLoading, error, refetch } = useAdminDashboard();

  if (error) {
    return (
      <div className="max-w-7xl mx-auto">
        <h1 className="text-2xl font-bold text-gray-900 mb-8">Admin Dashboard</h1>
        <div className="bg-red-50 border border-red-200 rounded-lg p-6">
          <p className="text-red-600">Error loading dashboard: {error instanceof Error ? error.message : String(error)}</p>
        </div>
      </div>
    );
  }

  const formatUptime = (seconds: number) => {
    const days = Math.floor(seconds / (24 * 60 * 60));
    const hours = Math.floor((seconds % (24 * 60 * 60)) / (60 * 60));
    const minutes = Math.floor((seconds % (60 * 60)) / 60);
    
    if (days > 0) return `${days}d ${hours}h`;
    if (hours > 0) return `${hours}h ${minutes}m`;
    return `${minutes}m`;
  };

  const formatVersion = (version: string) => version.startsWith('v') ? version : `v${version}`;

  return (
    <div className="max-w-7xl mx-auto space-y-6">
      <div className="flex items-center justify-between">
        <h1 className="text-2xl font-bold text-gray-900">Admin Dashboard</h1>
        <div className="flex items-center space-x-4">
          <button
            onClick={() => refetch()}
            disabled={isLoading}
            className="inline-flex items-center px-3 py-2 border border-gray-300 shadow-sm text-sm leading-4 font-medium rounded-md text-gray-700 bg-white hover:bg-gray-50 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-blue-500 disabled:opacity-50"
            title="Refresh dashboard data"
          >
            <ArrowPathIcon className={`h-4 w-4 mr-2 ${isLoading ? 'animate-spin' : ''}`} />
            Refresh
          </button>
          <div className="flex items-center space-x-2">
            <div className="h-2 w-2 bg-green-400 rounded-full"></div>
            <span className="text-sm text-gray-500">System Online</span>
          </div>
        </div>
      </div>

      {isLoading ? (
        <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-6">
          {Array.from({ length: 8 }).map((_, i) => (
            <div key={i} className="bg-white shadow rounded-lg p-6 animate-pulse">
              <div className="flex items-center">
                <div className="flex-shrink-0">
                  <div className="h-12 w-12 bg-gray-200 rounded-md"></div>
                </div>
                <div className="ml-5 w-0 flex-1">
                  <div className="h-4 bg-gray-200 rounded mb-2"></div>
                  <div className="h-6 bg-gray-200 rounded"></div>
                </div>
              </div>
            </div>
          ))}
        </div>
      ) : (
        <>
          {/* System Overview */}
          <div>
            <h2 className="text-lg font-medium text-gray-900 mb-4">System Overview</h2>
            <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-6">
              <MetricCard
                title="System Status"
                value={dashboardData?.system?.database_status || 'Unknown'}
                description={`${formatVersion(dashboardData?.system?.version || '')} • ${dashboardData?.system?.environment || 'Unknown'}`}
                icon={ServerIcon}
                color="green"
              />
              
              <MetricCard
                title="Uptime"
                value={formatUptime(dashboardData?.system?.uptime_seconds || 0)}
                description="System uptime"
                icon={ClockIcon}
                color="blue"
              />
              
              <MetricCard
                title="Total Users"
                value={dashboardData?.users?.total || 0}
                description={`${dashboardData?.users?.active || 0} active, ${dashboardData?.users?.admins || 0} admins`}
                icon={UsersIcon}
                color="purple"
              />
              
              <MetricCard
                title="Repositories"
                value={dashboardData?.repositories?.total_repositories || 0}
                description={`${dashboardData?.repositories?.enabled_repositories || 0} enabled`}
                icon={FolderIcon}
                color="indigo"
              />
            </div>
          </div>

          {/* Search Stats */}
          <div>
            <h2 className="text-lg font-medium text-gray-900 mb-4">Search & Crawling</h2>
            <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">
              <MetricCard
                title="Search Index"
                value={dashboardData?.search?.total_documents || 0}
                description={`${dashboardData?.search?.index_size_mb?.toFixed(1) || '0.0'} MB index`}
                icon={MagnifyingGlassIcon}
                color="blue"
              />
              
              <MetricCard
                title="Recently Crawled"
                value={dashboardData?.repositories?.recently_crawled || 0}
                description="Repositories crawled in last 24h"
                icon={CogIcon}
                color="green"
              />
              
              <MetricCard
                title="Never Crawled"
                value={dashboardData?.repositories?.never_crawled || 0}
                description="Repositories not yet indexed"
                icon={DocumentDuplicateIcon}
                color="yellow"
              />
            </div>
          </div>

          {/* Repository Breakdown */}
          <div>
            <h2 className="text-lg font-medium text-gray-900 mb-4">Repository Types</h2>
            <div className="grid grid-cols-1 md:grid-cols-3 gap-6">
              <MetricCard
                title="Git Repositories"
                value={dashboardData?.repositories?.git_repositories || 0}
                description="Standard Git repositories"
                color="green"
              />

              <MetricCard
                title="GitLab Repositories"
                value={dashboardData?.repositories?.gitlab_repositories || 0}
                description="GitLab hosted repositories"
                color="blue"
              />

              <MetricCard
                title="File System"
                value={dashboardData?.repositories?.filesystem_repositories || 0}
                description="Local file system repositories"
                color="yellow"
              />
            </div>
          </div>

          {/* Search Index Metrics by Repository */}
          {dashboardData?.search?.documents_by_repository && dashboardData.search.documents_by_repository.length > 0 && (
            <div>
              <h2 className="text-lg font-medium text-gray-900 mb-4">Documents by Repository</h2>
              <div className="bg-white shadow rounded-lg overflow-hidden">
                <div className="overflow-x-auto">
                  <table className="min-w-full divide-y divide-gray-200">
                    <thead className="bg-gray-50">
                      <tr>
                        <th scope="col" className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                          Repository
                        </th>
                        <th scope="col" className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                          Type
                        </th>
                        <th scope="col" className="px-6 py-3 text-right text-xs font-medium text-gray-500 uppercase tracking-wider">
                          Documents
                        </th>
                        <th scope="col" className="px-6 py-3 text-right text-xs font-medium text-gray-500 uppercase tracking-wider">
                          Percentage
                        </th>
                      </tr>
                    </thead>
                    <tbody className="bg-white divide-y divide-gray-200">
                      {dashboardData.search.documents_by_repository
                        .sort((a, b) => b.document_count - a.document_count)
                        .slice(0, 10)
                        .map((repo, index) => {
                          const percentage = dashboardData.search.total_documents > 0
                            ? ((repo.document_count / dashboardData.search.total_documents) * 100).toFixed(1)
                            : '0.0';
                          return (
                            <tr key={index} className="hover:bg-gray-50">
                              <td className="px-6 py-4 whitespace-nowrap">
                                <RepositoryBadge
                                  name={repo.repository_name}
                                  type={repo.repository_type as 'Git' | 'GitLab' | 'GitHub' | 'FileSystem' | undefined}
                                  size="sm"
                                />
                              </td>
                              <td className="px-6 py-4 whitespace-nowrap text-sm text-gray-500">
                                {repo.repository_type || 'Unknown'}
                              </td>
                              <td className="px-6 py-4 whitespace-nowrap text-sm font-medium text-gray-900 text-right">
                                {repo.document_count.toLocaleString()}
                              </td>
                              <td className="px-6 py-4 whitespace-nowrap text-sm text-gray-500 text-right">
                                <div className="flex items-center justify-end gap-2">
                                  <div className="w-20 bg-gray-200 rounded-full h-2">
                                    <div
                                      className="bg-blue-600 h-2 rounded-full"
                                      style={{ width: `${percentage}%` }}
                                    />
                                  </div>
                                  <span className="w-12 text-right">{percentage}%</span>
                                </div>
                              </td>
                            </tr>
                          );
                        })}
                    </tbody>
                  </table>
                </div>
                {dashboardData.search.documents_by_repository.length > 10 && (
                  <div className="px-6 py-3 bg-gray-50 border-t border-gray-200 text-sm text-gray-500 text-center">
                    Showing top 10 of {dashboardData.search.documents_by_repository.length} repositories
                  </div>
                )}
              </div>
            </div>
          )}

          {/* Recent Activity */}
          {dashboardData?.recent_activity && (
            <div className="grid grid-cols-1 lg:grid-cols-3 gap-6">
              {/* Recent Users */}
              <div className="bg-white shadow rounded-lg p-6">
                <h3 className="text-lg font-medium text-gray-900 mb-4">Recent Users</h3>
                <div className="space-y-3">
                  {dashboardData.recent_activity.recent_users.length > 0 ? (
                    dashboardData.recent_activity.recent_users.map((user, index) => (
                      <div key={index} className="flex items-center justify-between">
                        <div>
                          <p className="text-sm font-medium text-gray-900">{user.username}</p>
                          <p className="text-xs text-gray-500">{user.role}</p>
                        </div>
                        <div className="text-xs text-gray-400">
                          <time dateTime={user.last_seen}>
                            {formatDateTime(user.last_seen)}
                          </time>
                        </div>
                      </div>
                    ))
                  ) : (
                    <p className="text-sm text-gray-500">No recent users</p>
                  )}
                </div>
              </div>

              {/* Recent Repositories */}
              <div className="bg-white shadow rounded-lg p-6">
                <h3 className="text-lg font-medium text-gray-900 mb-4">Recent Repositories</h3>
                <div className="space-y-3">
                  {dashboardData.recent_activity.recent_repositories.length > 0 ? (
                    dashboardData.recent_activity.recent_repositories.map((repo, index) => (
                      <div key={index} className="flex items-center justify-between">
                        <div>
                          <p className="text-sm font-medium text-gray-900">{repo.name}</p>
                          <p className="text-xs text-gray-500">{repo.repository_type}</p>
                        </div>
                        <div className="text-xs text-gray-400">
                          <time dateTime={repo.created_at}>
                            {formatDateTime(repo.created_at)}
                          </time>
                        </div>
                      </div>
                    ))
                  ) : (
                    <p className="text-sm text-gray-500">No recent repositories</p>
                  )}
                </div>
              </div>

              {/* Recent Crawls */}
              <div className="bg-white shadow rounded-lg p-6">
                <h3 className="text-lg font-medium text-gray-900 mb-4">Recent Crawls</h3>
                <div className="space-y-3">
                  {dashboardData.recent_activity.recent_crawls.length > 0 ? (
                    dashboardData.recent_activity.recent_crawls.map((crawl, index) => (
                      <div key={index} className="flex items-center justify-between">
                        <div>
                          <p className="text-sm font-medium text-gray-900">{crawl.repository_name}</p>
                          <p className="text-xs text-gray-500">{crawl.status}</p>
                        </div>
                        <div className="text-xs text-gray-400">
                          {crawl.last_crawled ? (
                            <time dateTime={crawl.last_crawled}>
                              {formatDateTime(crawl.last_crawled)}
                            </time>
                          ) : (
                            'Never'
                          )}
                        </div>
                      </div>
                    ))
                  ) : (
                    <p className="text-sm text-gray-500">No recent crawls</p>
                  )}
                </div>
              </div>
            </div>
          )}
        </>
      )}
    </div>
  );
};

export default AdminDashboard;