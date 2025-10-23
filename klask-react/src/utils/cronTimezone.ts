/**
 * Utility functions for handling cron expressions with timezone conversion
 *
 * The backend stores and processes all cron expressions in UTC.
 * The frontend allows users to enter times in their local timezone,
 * then converts to UTC before sending to the backend.
 */

/**
 * Get the browser's timezone offset in hours
 * @returns Timezone offset in hours (e.g., 2 for CEST, -5 for EST)
 */
export function getBrowserTimezoneOffset(): number {
  const offset = new Date().getTimezoneOffset(); // in minutes, negative for ahead of UTC
  return -offset / 60; // convert to hours and flip sign
}

/**
 * Get the browser's timezone name (e.g., "Europe/Paris", "America/New_York")
 */
export function getBrowserTimezoneName(): string {
  return Intl.DateTimeFormat().resolvedOptions().timeZone;
}

/**
 * Parse a cron expression (6 fields: seconds minutes hours day month weekday)
 * @returns Object with parsed fields or null if invalid
 */
export function parseCronExpression(cron: string): {
  seconds: string;
  minutes: string;
  hours: string;
  day: string;
  month: string;
  weekday: string;
} | null {
  const parts = cron.trim().split(/\s+/);
  if (parts.length !== 6) return null;

  return {
    seconds: parts[0],
    minutes: parts[1],
    hours: parts[2],
    day: parts[3],
    month: parts[4],
    weekday: parts[5],
  };
}

/**
 * Convert a cron expression from local timezone to UTC
 *
 * For example, if the user enters "0 0 22 * * *" (10pm local time)
 * and they're in CEST (UTC+2), this will convert to "0 0 20 * * *" (10pm CEST = 8pm UTC)
 *
 * Note: This only handles simple hour offsets. Day boundaries are handled by
 * adjusting the hour field. Complex cases (e.g., "0 0 23 1 * *" = 11pm on 1st of month)
 * might roll over to the next/previous day.
 */
export function convertCronToUTC(localCron: string): string {
  const parsed = parseCronExpression(localCron);
  if (!parsed) return localCron; // Return unchanged if invalid

  // Only convert if hours field is a simple number or wildcard
  const hoursMatch = parsed.hours.match(/^(\d+)$/);
  if (!hoursMatch) {
    // Complex hour expression (e.g., "*/6", "8-17"), can't convert simply
    // Return as-is and let user know it's in UTC
    return localCron;
  }

  const localHour = parseInt(hoursMatch[1], 10);
  const timezoneOffset = getBrowserTimezoneOffset();

  // Convert to UTC
  let utcHour = localHour - timezoneOffset;

  // Handle day boundary crossings
  let dayAdjustment = 0;
  if (utcHour < 0) {
    utcHour += 24;
    dayAdjustment = -1; // Previous day in UTC
  } else if (utcHour >= 24) {
    utcHour -= 24;
    dayAdjustment = 1; // Next day in UTC
  }

  // For simple cases, we only adjust the hour
  // Day adjustment is complex and requires calendar logic, so we skip it for now
  // TODO: Handle day/month boundary crossings properly

  return `${parsed.seconds} ${parsed.minutes} ${utcHour} ${parsed.day} ${parsed.month} ${parsed.weekday}`;
}

/**
 * Convert a cron expression from UTC to local timezone (for display)
 */
export function convertCronToLocal(utcCron: string): string {
  const parsed = parseCronExpression(utcCron);
  if (!parsed) return utcCron;

  const hoursMatch = parsed.hours.match(/^(\d+)$/);
  if (!hoursMatch) return utcCron;

  const utcHour = parseInt(hoursMatch[1], 10);
  const timezoneOffset = getBrowserTimezoneOffset();

  let localHour = utcHour + timezoneOffset;

  if (localHour < 0) {
    localHour += 24;
  } else if (localHour >= 24) {
    localHour -= 24;
  }

  return `${parsed.seconds} ${parsed.minutes} ${localHour} ${parsed.day} ${parsed.month} ${parsed.weekday}`;
}

/**
 * Format an hour in 24h format for display
 */
export function formatHour(hour: number): string {
  return hour.toString().padStart(2, '0') + ':00';
}

/**
 * Get a human-readable description of a cron expression in local time
 */
export function describeCronExpression(utcCron: string): string {
  const localCron = convertCronToLocal(utcCron);
  const parsed = parseCronExpression(localCron);

  if (!parsed) return 'Invalid cron expression';

  const hoursMatch = parsed.hours.match(/^(\d+)$/);
  if (hoursMatch) {
    const hour = parseInt(hoursMatch[1], 10);
    const timeStr = formatHour(hour);

    if (parsed.day === '*' && parsed.month === '*' && parsed.weekday === '*') {
      return `Daily at ${timeStr} (local time)`;
    }
    if (parsed.weekday !== '*') {
      const days = ['Sunday', 'Monday', 'Tuesday', 'Wednesday', 'Thursday', 'Friday', 'Saturday'];
      const dayNum = parseInt(parsed.weekday, 10);
      if (!isNaN(dayNum) && dayNum >= 0 && dayNum <= 6) {
        return `Weekly on ${days[dayNum]} at ${timeStr} (local time)`;
      }
    }
  }

  // Handle interval expressions
  const intervalMatch = parsed.hours.match(/^\*\/(\d+)$/);
  if (intervalMatch) {
    const interval = parseInt(intervalMatch[1], 10);
    return `Every ${interval} hours`;
  }

  return 'Custom schedule';
}