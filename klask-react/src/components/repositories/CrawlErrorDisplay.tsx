import React, { useState } from 'react';
import {
  ExclamationCircleIcon,
  ChevronDownIcon,
  ChevronUpIcon,
  ClockIcon,
  XMarkIcon,
} from '@heroicons/react/24/outline';
import { formatDistanceToNow } from 'date-fns';

interface CrawlErrorDisplayProps {
  errorMessage: string;
  occurredAt?: string;
  repositoryName?: string;
  onDismiss?: () => void;
  className?: string;
  compact?: boolean;
}

export const CrawlErrorDisplay: React.FC<CrawlErrorDisplayProps> = ({
  errorMessage,
  occurredAt,
  repositoryName,
  onDismiss,
  className = '',
  compact = false,
}) => {
  const [isExpanded, setIsExpanded] = useState(!compact);

  const formatTime = (date: string | undefined) => {
    if (!date) return null;
    try {
      return formatDistanceToNow(new Date(date), { addSuffix: true });
    } catch {
      return null;
    }
  };

  const timeAgo = formatTime(occurredAt);

  // Truncate error message for compact view
  const truncatedMessage = errorMessage.length > 100
    ? `${errorMessage.substring(0, 100)}...`
    : errorMessage;

  const displayMessage = compact && !isExpanded ? truncatedMessage : errorMessage;

  return (
    <div
      className={`bg-red-50 dark:bg-red-900/20 border border-red-200 dark:border-red-800 rounded-lg overflow-hidden ${className}`}
    >
      {/* Header */}
      <div className="px-4 py-3">
        <div className="flex items-start justify-between gap-3">
          <div className="flex items-start gap-3 flex-1 min-w-0">
            <ExclamationCircleIcon className="h-5 w-5 text-red-600 dark:text-red-400 flex-shrink-0 mt-0.5" />

            <div className="flex-1 min-w-0">
              <div className="flex items-center gap-2 mb-1">
                <h4 className="text-sm font-semibold text-red-800 dark:text-red-300">
                  Crawl Error
                </h4>
                {timeAgo && (
                  <div className="flex items-center gap-1 text-xs text-red-600 dark:text-red-400">
                    <ClockIcon className="h-3 w-3" />
                    <span>{timeAgo}</span>
                  </div>
                )}
              </div>

              {repositoryName && (
                <p className="text-xs text-red-700 dark:text-red-300 mb-2">
                  Repository: <span className="font-medium">{repositoryName}</span>
                </p>
              )}

              <p className="text-sm text-red-700 dark:text-red-200 whitespace-pre-wrap break-words">
                {displayMessage}
              </p>

              {compact && errorMessage.length > 100 && (
                <button
                  onClick={() => setIsExpanded(!isExpanded)}
                  className="mt-2 inline-flex items-center gap-1 text-xs font-medium text-red-600 dark:text-red-400 hover:text-red-800 dark:hover:text-red-300"
                >
                  {isExpanded ? (
                    <>
                      <ChevronUpIcon className="h-3 w-3" />
                      Show less
                    </>
                  ) : (
                    <>
                      <ChevronDownIcon className="h-3 w-3" />
                      Show more
                    </>
                  )}
                </button>
              )}
            </div>
          </div>

          {onDismiss && (
            <button
              onClick={onDismiss}
              className="flex-shrink-0 p-1 rounded hover:bg-red-100 dark:hover:bg-red-900/40 text-red-600 dark:text-red-400"
              title="Dismiss error"
            >
              <XMarkIcon className="h-4 w-4" />
            </button>
          )}
        </div>
      </div>

      {/* Error Details (if expanded and there's more info) */}
      {isExpanded && (
        <div className="px-4 py-3 border-t border-red-200 dark:border-red-800 bg-red-100 dark:bg-red-900/30">
          <h5 className="text-xs font-semibold text-red-800 dark:text-red-300 mb-2">
            Troubleshooting Tips:
          </h5>
          <ul className="text-xs text-red-700 dark:text-red-200 space-y-1 list-disc list-inside">
            <li>Check if the repository URL is accessible</li>
            <li>Verify access token permissions if using private repositories</li>
            <li>Ensure network connectivity to the repository host</li>
            <li>Check repository logs for more details</li>
          </ul>
        </div>
      )}
    </div>
  );
};

// Compact inline error display for cards
export const InlineCrawlError: React.FC<{
  errorMessage: string;
  onClick?: () => void;
}> = ({ errorMessage, onClick }) => {
  const truncated = errorMessage.length > 60
    ? `${errorMessage.substring(0, 60)}...`
    : errorMessage;

  return (
    <div
      onClick={onClick}
      className={`flex items-start gap-2 p-2 bg-red-50 dark:bg-red-900/20 border border-red-200 dark:border-red-800 rounded text-xs ${
        onClick ? 'cursor-pointer hover:bg-red-100 dark:hover:bg-red-900/30' : ''
      }`}
    >
      <ExclamationCircleIcon className="h-4 w-4 text-red-600 dark:text-red-400 flex-shrink-0 mt-0.5" />
      <div className="flex-1 min-w-0">
        <p className="font-medium text-red-800 dark:text-red-300 mb-0.5">
          Crawl failed
        </p>
        <p className="text-red-700 dark:text-red-200 break-words">
          {truncated}
        </p>
        {onClick && (
          <p className="mt-1 text-red-600 dark:text-red-400 font-medium">
            Click for details →
          </p>
        )}
      </div>
    </div>
  );
};
