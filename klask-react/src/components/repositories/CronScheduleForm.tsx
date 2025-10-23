import React, { useState, useEffect } from 'react';
import { ClockIcon, ExclamationTriangleIcon, CheckCircleIcon } from '@heroicons/react/24/outline';
import { convertCronToUTC, convertCronToLocal, describeCronExpression, getBrowserTimezoneName } from '../../utils/cronTimezone';

interface CronScheduleFormProps {
  autoCrawlEnabled: boolean;
  cronSchedule?: string;
  crawlFrequencyHours?: number;
  maxCrawlDurationMinutes?: number;
  onScheduleChange: (schedule: {
    autoCrawlEnabled: boolean;
    cronSchedule?: string;
    crawlFrequencyHours?: number;
    maxCrawlDurationMinutes?: number;
  }) => void;
  className?: string;
  repositoryId?: string; // Add this to track repository changes
}

const FREQUENCY_OPTIONS = [
  { value: 1, label: 'Every hour' },
  { value: 6, label: 'Every 6 hours' },
  { value: 12, label: 'Every 12 hours' },
  { value: 24, label: 'Daily' },
  { value: 168, label: 'Weekly (7 days)' },
] as const;

const DURATION_OPTIONS = [
  { value: 15, label: '15 minutes' },
  { value: 30, label: '30 minutes' },
  { value: 60, label: '1 hour' },
  { value: 120, label: '2 hours' },
  { value: 240, label: '4 hours' },
  { value: 480, label: '8 hours' },
  { value: 1440, label: '24 hours' },
] as const;

export const CronScheduleForm: React.FC<CronScheduleFormProps> = ({
  autoCrawlEnabled,
  cronSchedule,
  crawlFrequencyHours,
  maxCrawlDurationMinutes,
  onScheduleChange,
  className = '',
  repositoryId,
}) => {
  // Determine mode from props
  const currentMode = cronSchedule && cronSchedule.trim() !== '' ? 'cron' : 'frequency';

  // Local state only for UI interactions
  // Store the cron expression in LOCAL timezone for display/editing
  const [localCronSchedule, setLocalCronSchedule] = useState(
    cronSchedule ? convertCronToLocal(cronSchedule) : ''
  );
  const [cronError, setCronError] = useState<string | null>(null);

  // Reset local cron when repository changes
  useEffect(() => {
    // Convert from UTC (backend) to local timezone for display
    setLocalCronSchedule(cronSchedule ? convertCronToLocal(cronSchedule) : '');
    setCronError(null);
  }, [repositoryId, cronSchedule]);


  const validateCronExpression = (cron: string): string | null => {
    if (!cron.trim()) return null;
    
    // Basic cron validation (should have 6 parts for seconds-based cron)
    const parts = cron.trim().split(/\s+/);
    if (parts.length !== 6) {
      return 'Cron expression must have 6 parts (seconds minutes hours day month weekday)';
    }

    // Additional basic validation could be added here
    const validParts = parts.every(part => {
      return /^[\d*\-,/?]+$/.test(part) || part === '?';
    });

    if (!validParts) {
      return 'Invalid characters in cron expression';
    }

    return null;
  };

  const handleCronChange = (value: string) => {
    setLocalCronSchedule(value);
    const error = validateCronExpression(value);
    setCronError(error);
  };

  // Handle changes to scheduling mode
  const handleScheduleModeChange = (mode: 'frequency' | 'cron') => {
    if (mode === 'cron') {
      // Switch to cron mode
      const defaultLocalCron = '0 0 2 * * *'; // Default: 2am local time
      const cronToUse = localCronSchedule || defaultLocalCron;
      const utcCron = convertCronToUTC(cronToUse);

      onScheduleChange({
        autoCrawlEnabled,
        maxCrawlDurationMinutes: maxCrawlDurationMinutes || 60,
        cronSchedule: utcCron, // Send UTC version
        crawlFrequencyHours: undefined,
      });
    } else {
      // Switch to frequency mode
      onScheduleChange({
        autoCrawlEnabled,
        maxCrawlDurationMinutes: maxCrawlDurationMinutes || 60,
        crawlFrequencyHours: crawlFrequencyHours || 24, // Default frequency
        cronSchedule: undefined,
      });
    }
  };

  // Handle frequency changes
  const handleFrequencyChange = (value: number) => {
    onScheduleChange({
      autoCrawlEnabled,
      maxCrawlDurationMinutes: maxCrawlDurationMinutes || 60,
      crawlFrequencyHours: value,
      cronSchedule: undefined,
    });
  };

  // Handle duration changes  
  const handleDurationChange = (value: number) => {
    onScheduleChange({
      autoCrawlEnabled,
      maxCrawlDurationMinutes: value,
      ...(currentMode === 'cron' 
        ? { cronSchedule: localCronSchedule, crawlFrequencyHours: undefined }
        : { crawlFrequencyHours: crawlFrequencyHours || 24, cronSchedule: undefined }
      ),
    });
  };

  // Handle cron input changes
  const handleCronInputChange = (value: string) => {
    handleCronChange(value);
    if (!validateCronExpression(value)) {
      // Convert from local timezone to UTC before sending to backend
      const utcCron = convertCronToUTC(value);
      onScheduleChange({
        autoCrawlEnabled,
        maxCrawlDurationMinutes: maxCrawlDurationMinutes || 60,
        cronSchedule: utcCron, // Send UTC version to backend
        crawlFrequencyHours: undefined,
      });
    }
  };

  // Handle auto-crawl toggle
  const handleAutoCrawlToggle = (enabled: boolean) => {
    onScheduleChange({
      autoCrawlEnabled: enabled,
      maxCrawlDurationMinutes: maxCrawlDurationMinutes || 60,
      ...(currentMode === 'cron'
        ? { cronSchedule: convertCronToUTC(localCronSchedule), crawlFrequencyHours: undefined }
        : { crawlFrequencyHours: crawlFrequencyHours || 24, cronSchedule: undefined }
      ),
    });
  };

  const getNextRunDescription = () => {
    if (!autoCrawlEnabled) return null;

    if (currentMode === 'frequency' && crawlFrequencyHours) {
      return `Will run every ${crawlFrequencyHours} hour${crawlFrequencyHours !== 1 ? 's' : ''}`;
    } else if (localCronSchedule && !cronError) {
      const description = describeCronExpression(convertCronToUTC(localCronSchedule));
      const timezone = getBrowserTimezoneName();
      return `${description} (${timezone})`;
    }

    return null;
  };

  const getCommonCronExamples = () => [
    { expression: '0 0 */6 * * *', description: 'Every 6 hours' },
    { expression: '0 0 2 * * *', description: 'Daily at 2:00 AM' },
    { expression: '0 0 2 * * 1', description: 'Weekly on Monday at 2:00 AM' },
    { expression: '0 30 1 * * *', description: 'Daily at 1:30 AM' },
  ];

  return (
    <div className={`space-y-6 ${className}`}>
      {/* Auto-crawl Toggle */}
      <div className="flex items-center space-x-3">
        <input
          type="checkbox"
          id="autoCrawlEnabled"
          checked={autoCrawlEnabled}
          onChange={(e) => handleAutoCrawlToggle(e.target.checked)}
          className="h-4 w-4 text-blue-600 focus:ring-blue-500 border-gray-300 rounded"
        />
        <label htmlFor="autoCrawlEnabled" className="text-sm font-medium text-gray-900">
          Enable automatic crawling
        </label>
      </div>

      {autoCrawlEnabled && (
        <>
          {/* Schedule Mode Selection */}
          <div>
            <label className="block text-sm font-medium text-gray-700 mb-3">
              Schedule Type
            </label>
            <div className="grid grid-cols-2 gap-3">
              <label className={`relative flex items-center justify-center p-3 border rounded-lg cursor-pointer transition-colors ${
                currentMode === 'frequency'
                  ? 'border-blue-500 bg-blue-50 text-blue-700'
                  : 'border-gray-300 hover:border-gray-400'
              }`}>
                <input
                  type="radio"
                  value="frequency"
                  checked={currentMode === 'frequency'}
                  onChange={(e) => {
                    if (e.target.checked) {
                      handleScheduleModeChange('frequency');
                    }
                  }}
                  className="sr-only"
                />
                <div className="flex flex-col items-center space-y-1">
                  <ClockIcon className="h-5 w-5" />
                  <span className="text-xs font-medium">Simple Frequency</span>
                </div>
              </label>

              <label className={`relative flex items-center justify-center p-3 border rounded-lg cursor-pointer transition-colors ${
                currentMode === 'cron'
                  ? 'border-blue-500 bg-blue-50 text-blue-700'
                  : 'border-gray-300 hover:border-gray-400'
              }`}>
                <input
                  type="radio"
                  value="cron"
                  checked={currentMode === 'cron'}
                  onChange={(e) => {
                    if (e.target.checked) {
                      handleScheduleModeChange('cron');
                    }
                  }}
                  className="sr-only"
                />
                <div className="flex flex-col items-center space-y-1">
                  <ClockIcon className="h-5 w-5" />
                  <span className="text-xs font-medium">Cron Expression</span>
                </div>
              </label>
            </div>
          </div>

          {/* Frequency Configuration */}
          {currentMode === 'frequency' && (
            <div>
              <label htmlFor="frequency" className="block text-sm font-medium text-gray-700 mb-1">
                Crawl Frequency
              </label>
              <select
                id="frequency"
                value={crawlFrequencyHours || 24}
                onChange={(e) => handleFrequencyChange(Number(e.target.value))}
                className="block w-full px-3 py-2 border border-gray-300 rounded-md shadow-sm focus:outline-none focus:ring-blue-500 focus:border-blue-500"
              >
                {FREQUENCY_OPTIONS.map((option) => (
                  <option key={option.value} value={option.value}>
                    {option.label}
                  </option>
                ))}
              </select>
              <p className="mt-1 text-xs text-gray-500">
                How often should this repository be crawled automatically?
              </p>
            </div>
          )}

          {/* Cron Configuration */}
          {currentMode === 'cron' && (
            <div>
              <label htmlFor="cronSchedule" className="block text-sm font-medium text-gray-700 mb-1">
                Cron Expression
              </label>
              <input
                type="text"
                id="cronSchedule"
                value={localCronSchedule}
                onChange={(e) => handleCronInputChange(e.target.value)}
                className={`block w-full px-3 py-2 border rounded-md shadow-sm focus:outline-none focus:ring-blue-500 focus:border-blue-500 ${
                  cronError ? 'border-red-300' : 'border-gray-300'
                }`}
                placeholder="0 0 2 * * * (daily at 2:00 AM)"
              />
              {cronError && (
                <div className="mt-1 flex items-center space-x-1 text-sm text-red-600">
                  <ExclamationTriangleIcon className="h-4 w-4" />
                  <span>{cronError}</span>
                </div>
              )}
              {!cronError && localCronSchedule && (
                <div className="mt-1 flex items-center space-x-1 text-sm text-green-600">
                  <CheckCircleIcon className="h-4 w-4" />
                  <span>Valid cron expression</span>
                </div>
              )}
              <p className="mt-1 text-xs text-gray-500">
                Format: seconds minutes hours day month weekday
              </p>
              <p className="mt-1 text-xs text-blue-600">
                ℹ️ Times are in your local timezone ({getBrowserTimezoneName()})
              </p>

              {/* Common Examples */}
              <div className="mt-3">
                <p className="text-xs font-medium text-gray-700 mb-2">Common Examples:</p>
                <div className="grid grid-cols-1 gap-1">
                  {getCommonCronExamples().map((example, index) => (
                    <button
                      key={index}
                      type="button"
                      onClick={() => handleCronInputChange(example.expression)}
                      className="text-left px-2 py-1 text-xs bg-gray-50 hover:bg-gray-100 rounded border-l-2 border-blue-500"
                    >
                      <code className="font-mono text-blue-600">{example.expression}</code>
                      <span className="ml-2 text-gray-600">- {example.description}</span>
                    </button>
                  ))}
                </div>
              </div>
            </div>
          )}

          {/* Max Duration */}
          <div>
            <label htmlFor="maxDuration" className="block text-sm font-medium text-gray-700 mb-1">
              Maximum Crawl Duration
            </label>
            <select
              id="maxDuration"
              value={maxCrawlDurationMinutes || 60}
              onChange={(e) => handleDurationChange(Number(e.target.value))}
              className="block w-full px-3 py-2 border border-gray-300 rounded-md shadow-sm focus:outline-none focus:ring-blue-500 focus:border-blue-500"
            >
              {DURATION_OPTIONS.map((option) => (
                <option key={option.value} value={option.value}>
                  {option.label}
                </option>
              ))}
            </select>
            <p className="mt-1 text-xs text-gray-500">
              Crawl will be terminated if it exceeds this duration
            </p>
          </div>

          {/* Next Run Info */}
          {getNextRunDescription() && (
            <div className="p-3 bg-blue-50 border border-blue-200 rounded-lg">
              <div className="flex items-center space-x-2">
                <ClockIcon className="h-4 w-4 text-blue-600" />
                <span className="text-sm text-blue-800">{getNextRunDescription()}</span>
              </div>
            </div>
          )}
        </>
      )}
    </div>
  );
};