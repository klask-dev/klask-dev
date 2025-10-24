import React from 'react';
import {
  ChartBarIcon,
  ArrowTrendingUpIcon,
  ArrowTrendingDownIcon,
  ArrowRightIcon
} from '@heroicons/react/24/outline';

interface MetricCardProps {
  title: string;
  value: string | number;
  description?: string;
  trend?: {
    value: number;
    direction: 'up' | 'down';
    label: string;
  };
  icon?: React.ComponentType<React.SVGProps<SVGSVGElement>>;
  color?: 'blue' | 'green' | 'yellow' | 'red' | 'purple' | 'indigo';
  onClick?: () => void;
}

const colorClasses = {
  blue: {
    bg: 'bg-blue-50 dark:bg-blue-900/20',
    icon: 'text-blue-600',
    trend: {
      up: 'text-green-600 dark:text-green-400',
      down: 'text-red-600 dark:text-red-400'
    }
  },
  green: {
    bg: 'bg-green-50 dark:bg-green-900/20',
    icon: 'text-green-600',
    trend: {
      up: 'text-green-600 dark:text-green-400',
      down: 'text-red-600 dark:text-red-400'
    }
  },
  yellow: {
    bg: 'bg-yellow-50 dark:bg-yellow-900/20',
    icon: 'text-yellow-600',
    trend: {
      up: 'text-green-600 dark:text-green-400',
      down: 'text-red-600 dark:text-red-400'
    }
  },
  red: {
    bg: 'bg-red-50 dark:bg-red-900/20',
    icon: 'text-red-600',
    trend: {
      up: 'text-green-600 dark:text-green-400',
      down: 'text-red-600 dark:text-red-400'
    }
  },
  purple: {
    bg: 'bg-purple-50 dark:bg-purple-900/20',
    icon: 'text-purple-600',
    trend: {
      up: 'text-green-600 dark:text-green-400',
      down: 'text-red-600 dark:text-red-400'
    }
  },
  indigo: {
    bg: 'bg-indigo-50 dark:bg-indigo-900/20',
    icon: 'text-indigo-600',
    trend: {
      up: 'text-green-600 dark:text-green-400',
      down: 'text-red-600 dark:text-red-400'
    }
  }
};

export const MetricCard: React.FC<MetricCardProps> = ({
  title,
  value,
  description,
  trend,
  icon: Icon = ChartBarIcon,
  color = 'blue',
  onClick
}) => {
  const colors = colorClasses[color];
  const isClickable = !!onClick;

  const formatValue = (val: string | number) => {
    if (typeof val === 'number') {
      return val.toLocaleString();
    }
    return val;
  };

  return (
    <div
      className={`
        relative bg-white dark:bg-gray-800 overflow-hidden shadow rounded-lg
        ${isClickable ? 'cursor-pointer hover:shadow-md dark:hover:shadow-xl transition-shadow duration-200' : ''}
      `}
      onClick={onClick}
    >
      <div className="p-5">
        <div className="flex items-center">
          <div className="flex-shrink-0">
            <div className={`inline-flex items-center justify-center p-3 ${colors.bg} rounded-md`}>
              <Icon className={`h-6 w-6 ${colors.icon}`} />
            </div>
          </div>
          <div className="ml-5 w-0 flex-1">
            <dl>
              <dt className="text-sm font-medium text-gray-500 dark:text-gray-400 truncate">
                {title}
              </dt>
              <dd>
                <div className="text-lg font-medium text-gray-900 dark:text-white">
                  {formatValue(value)}
                </div>
              </dd>
            </dl>
          </div>
          {isClickable && (
            <div className="flex-shrink-0">
              <ArrowRightIcon className="h-5 w-5 text-gray-400 dark:text-gray-500" />
            </div>
          )}
        </div>

        {(description || trend) && (
          <div className="mt-3 flex items-center justify-between">
            {description && (
              <div className="text-sm text-gray-500 dark:text-gray-400">
                {description}
              </div>
            )}
            {trend && (
              <div className={`flex items-center text-sm ${colors.trend[trend.direction]}`}>
                {trend.direction === 'up' ? (
                  <ArrowTrendingUpIcon className="flex-shrink-0 h-4 w-4 mr-1" />
                ) : (
                  <ArrowTrendingDownIcon className="flex-shrink-0 h-4 w-4 mr-1" />
                )}
                <span className="font-medium">
                  {trend.direction === 'up' ? '+' : '-'}{Math.abs(trend.value)}%
                </span>
                <span className="ml-1 text-gray-500 dark:text-gray-400">
                  {trend.label}
                </span>
              </div>
            )}
          </div>
        )}
      </div>
    </div>
  );
};