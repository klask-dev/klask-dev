import React, { useState } from 'react';
import { ArrowPathIcon } from '@heroicons/react/24/outline';
import { useUserActivity } from '../../../hooks/useProfile';
import { useProfile } from '../../../hooks/useProfile';

const ActivitySection: React.FC = () => {
  const { user } = useProfile();
  const [page, setPage] = useState(1);
  const limit = 10;
  const { data: activity, isLoading, error, refetch } = useUserActivity(page, limit);

  if (isLoading) {
    return (
      <div className="flex justify-center items-center py-8">
        <div className="w-6 h-6 border-3 border-blue-500 border-t-transparent rounded-full animate-spin" />
      </div>
    );
  }

  if (error) {
    return (
      <div className="p-4 bg-red-50 dark:bg-red-900/20 border border-red-200 dark:border-red-800 rounded-lg text-red-700 dark:text-red-400">
        <p className="font-medium">Failed to load activity data</p>
        <button
          onClick={() => refetch()}
          className="mt-2 text-sm underline hover:no-underline"
        >
          Try again
        </button>
      </div>
    );
  }

  const formatDate = (dateString: string | undefined) => {
    if (!dateString) return 'Never';
    try {
      return new Date(dateString).toLocaleDateString('en-US', {
        year: 'numeric',
        month: 'long',
        day: 'numeric',
        hour: '2-digit',
        minute: '2-digit',
      });
    } catch {
      return 'Unknown';
    }
  };

  const getDeviceIcon = (userAgent: string) => {
    if (userAgent.includes('Windows')) return 'Windows';
    if (userAgent.includes('Mac')) return 'Mac';
    if (userAgent.includes('Linux')) return 'Linux';
    if (userAgent.includes('iPhone')) return 'iPhone';
    if (userAgent.includes('Android')) return 'Android';
    return 'Device';
  };

  return (
    <div className="space-y-8">
      {/* Account Overview */}
      <div>
        <h3 className="text-lg font-medium text-gray-900 dark:text-white mb-4">Account Overview</h3>
        <div className="grid grid-cols-2 md:grid-cols-3 gap-4">
          <div className="p-4 bg-blue-50 dark:bg-blue-900/20 rounded-lg border border-blue-200 dark:border-blue-800">
            <p className="text-sm text-blue-600 dark:text-blue-400 font-medium">Member Since</p>
            <p className="text-lg font-bold text-blue-900 dark:text-blue-300 mt-1">
              {formatDate(user?.created_at)}
            </p>
          </div>

          <div className="p-4 bg-green-50 dark:bg-green-900/20 rounded-lg border border-green-200 dark:border-green-800">
            <p className="text-sm text-green-600 dark:text-green-400 font-medium">Total Logins</p>
            <p className="text-lg font-bold text-green-900 dark:text-green-300 mt-1">
              {activity?.login_count || 0}
            </p>
          </div>

          <div className="p-4 bg-purple-50 dark:bg-purple-900/20 rounded-lg border border-purple-200 dark:border-purple-800">
            <p className="text-sm text-purple-600 dark:text-purple-400 font-medium">Last Login</p>
            <p className="text-sm font-bold text-purple-900 dark:text-purple-300 mt-1">
              {formatDate(activity?.last_login || user?.last_login)}
            </p>
          </div>
        </div>
      </div>

      {/* Active Devices */}
      {activity?.devices && activity.devices.length > 0 && (
        <div>
          <h3 className="text-lg font-medium text-gray-900 dark:text-white mb-4">Active Devices</h3>
          <div className="overflow-x-auto">
            <table className="w-full text-sm">
              <thead className="bg-gray-50 dark:bg-gray-800 border-b border-gray-200 dark:border-gray-700">
                <tr>
                  <th className="px-4 py-3 text-left font-semibold text-gray-700 dark:text-gray-300">Device</th>
                  <th className="px-4 py-3 text-left font-semibold text-gray-700 dark:text-gray-300">IP Address</th>
                  <th className="px-4 py-3 text-left font-semibold text-gray-700 dark:text-gray-300">Last Seen</th>
                </tr>
              </thead>
              <tbody className="divide-y divide-gray-200 dark:divide-gray-700">
                {activity.devices.map((device, idx) => (
                  <tr key={idx} className="hover:bg-gray-50 dark:hover:bg-gray-800 transition">
                    <td className="px-4 py-3">
                      <div className="flex items-center gap-2">
                        <div className="w-8 h-8 rounded-full bg-gray-200 dark:bg-gray-700 flex items-center justify-center">
                          <span className="text-xs font-bold text-gray-700 dark:text-gray-300">
                            {getDeviceIcon(device.user_agent).charAt(0)}
                          </span>
                        </div>
                        <div>
                          <p className="font-medium text-gray-900 dark:text-white">
                            {device.device_name || getDeviceIcon(device.user_agent)}
                          </p>
                          <p className="text-xs text-gray-500 dark:text-gray-400 truncate max-w-xs">
                            {device.user_agent}
                          </p>
                        </div>
                      </div>
                    </td>
                    <td className="px-4 py-3">
                      <code className="text-gray-900 dark:text-gray-300 font-mono text-xs bg-gray-100 dark:bg-gray-800 px-2 py-1 rounded">
                        {device.ip}
                      </code>
                    </td>
                    <td className="px-4 py-3 text-gray-600 dark:text-gray-400">
                      {formatDate(device.last_seen)}
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        </div>
      )}

      {(!activity?.devices || activity.devices.length === 0) && (
        <div className="p-4 bg-gray-50 dark:bg-gray-800 border border-gray-200 dark:border-gray-700 rounded-lg text-gray-700 dark:text-gray-300 text-center">
          <p>No device activity recorded</p>
        </div>
      )}

      {/* Pagination Controls */}
      {activity?.devices &&
        activity.devices.length > 0 &&
        activity.total_devices &&
        activity.total_devices > limit && (
          <div className="flex items-center justify-between pt-4 border-t border-gray-200 dark:border-gray-700">
            <p className="text-sm text-gray-600 dark:text-gray-400">
              Showing {(page - 1) * limit + 1} to {Math.min(page * limit, activity.total_devices)} of{' '}
              {activity.total_devices} devices
            </p>
            <div className="flex gap-2">
              <button
                onClick={() => setPage(Math.max(1, page - 1))}
                disabled={page === 1}
                className="px-3 py-1 text-sm border border-gray-300 dark:border-gray-600 rounded-lg hover:bg-gray-50 dark:hover:bg-gray-800 disabled:bg-gray-100 dark:disabled:bg-gray-700 disabled:cursor-not-allowed transition text-gray-700 dark:text-gray-300"
              >
                Previous
              </button>
              <button
                onClick={() => setPage(page + 1)}
                disabled={activity.total_devices ? page * limit >= activity.total_devices : false}
                className="px-3 py-1 text-sm border border-gray-300 dark:border-gray-600 rounded-lg hover:bg-gray-50 dark:hover:bg-gray-800 disabled:bg-gray-100 dark:disabled:bg-gray-700 disabled:cursor-not-allowed transition text-gray-700 dark:text-gray-300"
              >
                Next
              </button>
            </div>
          </div>
        )}

      {/* Refresh Button */}
      <div className="flex justify-end">
        <button
          onClick={() => {
            setPage(1);
            refetch();
          }}
          disabled={isLoading}
          className="px-4 py-2 text-gray-700 dark:text-gray-300 bg-gray-100 dark:bg-gray-800 hover:bg-gray-200 dark:hover:bg-gray-700 rounded-lg transition flex items-center gap-2 disabled:opacity-50"
        >
          <ArrowPathIcon className={`h-4 w-4 ${isLoading ? 'animate-spin' : ''}`} />
          Refresh
        </button>
      </div>
    </div>
  );
};

export default ActivitySection;
