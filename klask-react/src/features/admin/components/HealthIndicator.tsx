import React from 'react';
import {
  CheckCircleIcon,
  ExclamationTriangleIcon,
  XCircleIcon,
  InformationCircleIcon
} from '@heroicons/react/24/outline';
import type { IndexHealthResponse, HealthIssue } from '../../../types/tantivy';

interface HealthIndicatorProps {
  health: IndexHealthResponse;
  className?: string;
}

/**
 * Displays index health status with warnings and recommendations
 * Shows overall score, status badge, and actionable warnings
 */
export const HealthIndicator: React.FC<HealthIndicatorProps> = ({
  health,
  className = '',
}) => {
  const getStatusIcon = () => {
    switch (health.status) {
      case 'Healthy':
        return <CheckCircleIcon className="h-6 w-6 text-green-600 dark:text-green-400" />;
      case 'Warning':
        return <ExclamationTriangleIcon className="h-6 w-6 text-yellow-600 dark:text-yellow-400" />;
      case 'Degraded':
      default:
        return <ExclamationTriangleIcon className="h-6 w-6 text-orange-600 dark:text-orange-400" />;
    }
  };

  const getStatusColors = () => {
    switch (health.status) {
      case 'Healthy':
        return {
          bg: 'bg-green-50 dark:bg-green-900/20',
          border: 'border-green-200 dark:border-green-800',
          text: 'text-green-900 dark:text-green-200',
          badge: 'bg-green-100 dark:bg-green-900/40 text-green-800 dark:text-green-200',
        };
      case 'Warning':
        return {
          bg: 'bg-yellow-50 dark:bg-yellow-900/20',
          border: 'border-yellow-200 dark:border-yellow-800',
          text: 'text-yellow-900 dark:text-yellow-200',
          badge: 'bg-yellow-100 dark:bg-yellow-900/40 text-yellow-800 dark:text-yellow-200',
        };
      case 'Degraded':
      default:
        return {
          bg: 'bg-orange-50 dark:bg-orange-900/20',
          border: 'border-orange-200 dark:border-orange-800',
          text: 'text-orange-900 dark:text-orange-200',
          badge: 'bg-orange-100 dark:bg-orange-900/40 text-orange-800 dark:text-orange-200',
        };
    }
  };

  const getIssueSeverityIcon = (severity: 'High' | 'Medium' | 'Low') => {
    switch (severity) {
      case 'High':
        return <XCircleIcon className="h-5 w-5 text-red-600 dark:text-red-400" />;
      case 'Medium':
        return <ExclamationTriangleIcon className="h-5 w-5 text-yellow-600 dark:text-yellow-400" />;
      case 'Low':
      default:
        return <InformationCircleIcon className="h-5 w-5 text-blue-600 dark:text-blue-400" />;
    }
  };

  const colors = getStatusColors();

  return (
    <div className={`space-y-6 ${className}`}>
      {/* Status Header */}
      <div className={`${colors.bg} ${colors.border} border-2 rounded-lg p-6`}>
        <div className="flex items-start justify-between">
          <div className="flex items-center gap-4">
            {getStatusIcon()}
            <div>
              <h3 className={`text-2xl font-bold ${colors.text}`}>
                {health.status}
              </h3>
              <p className="text-sm text-gray-600 mt-1">
                Index health status
              </p>
            </div>
          </div>
          <div className="text-right">
            <div className={`${colors.badge} inline-block px-4 py-2 rounded-lg font-bold text-lg`}>
              {health.status}
            </div>
            <p className="text-xs text-gray-500 mt-2">
              Status
            </p>
          </div>
        </div>

        {/* Status Message */}
        <div className="mt-6">
          <p className="text-sm text-gray-700 dark:text-gray-300">
            {health.status_message}
          </p>
        </div>

        {/* Check Timestamp */}
        <p className="text-xs text-gray-600 dark:text-gray-400 mt-4">
          Checked:{' '}
          <span className="font-mono font-medium">
            {new Date(health.checked_at).toLocaleString()}
          </span>
        </p>
      </div>

      {/* Issues Section */}
      {health.issues.length > 0 && (
        <div className="space-y-3">
          <h3 className="text-sm font-semibold text-gray-900 dark:text-white">
            Issues ({health.issues.length})
          </h3>
          <div className="space-y-2">
            {health.issues.map((issue: HealthIssue, idx) => (
              <div
                key={idx}
                className={`${
                  issue.severity === 'High'
                    ? 'bg-red-50 dark:bg-red-900/20 border-red-200 dark:border-red-800'
                    : issue.severity === 'Medium'
                    ? 'bg-yellow-50 dark:bg-yellow-900/20 border-yellow-200 dark:border-yellow-800'
                    : 'bg-blue-50 dark:bg-blue-900/20 border-blue-200 dark:border-blue-800'
                } border border-l-4 rounded-lg p-4`}
              >
                <div className="flex items-start gap-3">
                  {getIssueSeverityIcon(issue.severity)}
                  <div className="flex-1 min-w-0">
                    <p className="text-sm font-medium text-gray-900 dark:text-white">
                      {issue.description}
                    </p>
                    <p className="text-xs text-gray-600 dark:text-gray-400 mt-1">
                      Current: <span className="font-mono">{issue.metric_value}</span> Threshold: <span className="font-mono">{issue.threshold}</span>
                    </p>
                  </div>
                </div>
              </div>
            ))}
          </div>
        </div>
      )}

      {/* No Issues State */}
      {health.issues.length === 0 && (
        <div className="bg-green-50 dark:bg-green-900/20 border border-green-200 dark:border-green-800 rounded-lg p-4 text-center">
          <CheckCircleIcon className="h-8 w-8 text-green-600 dark:text-green-400 mx-auto mb-2" />
          <p className="text-sm font-medium text-green-900 dark:text-green-200">
            No issues detected
          </p>
          <p className="text-xs text-green-700 dark:text-green-300 mt-1">
            Your index is healthy and performing optimally
          </p>
        </div>
      )}
    </div>
  );
};
