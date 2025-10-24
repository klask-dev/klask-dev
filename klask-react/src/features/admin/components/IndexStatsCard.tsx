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
        bg: 'bg-green-50',
        border: 'border-green-200',
        text: 'text-green-900',
        icon: 'text-green-600',
      };
    case 'warning':
      return {
        bg: 'bg-yellow-50',
        border: 'border-yellow-200',
        text: 'text-yellow-900',
        icon: 'text-yellow-600',
      };
    case 'critical':
      return {
        bg: 'bg-red-50',
        border: 'border-red-200',
        text: 'text-red-900',
        icon: 'text-red-600',
      };
    default:
      return {
        bg: 'bg-blue-50',
        border: 'border-blue-200',
        text: 'text-blue-900',
        icon: 'text-blue-600',
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
          <p className="text-sm font-medium text-gray-600">
            {title}
          </p>
          <div className="mt-2 flex items-baseline gap-2">
            <p className={`text-2xl font-bold ${colors.text}`}>
              {formatNumber(value)}
            </p>
            {unit && (
              <span className="text-sm text-gray-500 font-medium">
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
            trend.direction === 'up' ? 'text-green-600' : 'text-red-600'
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
          <span className="text-xs text-gray-500">
            {trend.label}
          </span>
        </div>
      )}
    </div>
  );
};
