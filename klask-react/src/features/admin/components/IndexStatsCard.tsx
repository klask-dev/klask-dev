import React from 'react';
import { ArrowTrendingUpIcon, ArrowTrendingDownIcon } from '@heroicons/react/24/outline';

interface IndexStatsCardProps {
  title: string;
  value: number | string;
  unit?: string;
  icon?: React.ComponentType<React.SVGProps<SVGSVGElement>>;
  trend?: {
    value: number;
    label: string;
    direction: 'up' | 'down';
  };
  healthStatus?: 'healthy' | 'warning' | 'critical';
  onClick?: () => void;
  className?: string;
}

const getColorClasses = (status?: 'healthy' | 'warning' | 'critical') => {
  switch (status) {
    case 'healthy':
      return {
        bg: 'bg-green-50 dark:bg-green-900/20',
        border: 'border-green-200 dark:border-green-800',
        text: 'text-green-900 dark:text-green-200',
        icon: 'text-green-600 dark:text-green-400',
      };
    case 'warning':
      return {
        bg: 'bg-yellow-50 dark:bg-yellow-900/20',
        border: 'border-yellow-200 dark:border-yellow-800',
        text: 'text-yellow-900 dark:text-yellow-200',
        icon: 'text-yellow-600 dark:text-yellow-400',
      };
    case 'critical':
      return {
        bg: 'bg-red-50 dark:bg-red-900/20',
        border: 'border-red-200 dark:border-red-800',
        text: 'text-red-900 dark:text-red-200',
        icon: 'text-red-600 dark:text-red-400',
      };
    default:
      return {
        bg: 'bg-blue-50 dark:bg-blue-900/20',
        border: 'border-blue-200 dark:border-blue-800',
        text: 'text-blue-900 dark:text-blue-200',
        icon: 'text-blue-600 dark:text-blue-400',
      };
  }
};

const formatNumber = (value: number | string): string => {
  if (typeof value === 'string') {
    return value;
  }

  if (value >= 1e9) {
    return (value / 1e9).toFixed(2) + 'B';
  }
  if (value >= 1e6) {
    return (value / 1e6).toFixed(2) + 'M';
  }
  if (value >= 1e3) {
    return (value / 1e3).toFixed(2) + 'K';
  }

  return value.toLocaleString();
};

/**
 * Card component for displaying a single index metric
 * Shows metric value, unit, optional trend, and health status
 */
export const IndexStatsCard: React.FC<IndexStatsCardProps> = ({
  title,
  value,
  unit,
  icon: Icon,
  trend,
  healthStatus,
  onClick,
  className = '',
}) => {
  const colors = getColorClasses(healthStatus);
  const isClickable = !!onClick;

  return (
    <div
      className={`
        ${colors.bg} ${colors.border} border rounded-lg p-4
        ${isClickable ? 'cursor-pointer hover:shadow-md transition-shadow' : ''}
        ${className}
      `}
      onClick={onClick}
    >
      <div className="flex items-start justify-between">
        <div className="flex-1">
          <p className="text-sm font-medium text-gray-600 dark:text-gray-400">
            {title}
          </p>
          <div className="mt-2 flex items-baseline gap-2">
            <p className={`text-2xl font-bold ${colors.text}`}>
              {formatNumber(value)}
            </p>
            {unit && (
              <span className="text-sm text-gray-500 dark:text-gray-400 font-medium">
                {unit}
              </span>
            )}
          </div>
        </div>
        {Icon && (
          <div className={`flex-shrink-0 p-2 rounded-lg ${colors.bg}`}>
            <Icon className={`h-6 w-6 ${colors.icon}`} />
          </div>
        )}
      </div>

      {trend && (
        <div className="mt-4 flex items-center gap-2">
          <div className={`flex items-center gap-1 text-sm font-medium ${
            trend.direction === 'up' ? 'text-green-600 dark:text-green-400' : 'text-red-600 dark:text-red-400'
          }`}>
            {trend.direction === 'up' ? (
              <ArrowTrendingUpIcon className="h-4 w-4" />
            ) : (
              <ArrowTrendingDownIcon className="h-4 w-4" />
            )}
            <span>
              {trend.direction === 'up' ? '+' : '-'}{trend.value}%
            </span>
          </div>
          <span className="text-xs text-gray-500 dark:text-gray-400">
            {trend.label}
          </span>
        </div>
      )}
    </div>
  );
};
