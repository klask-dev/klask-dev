import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import {
  getBrowserTimezoneOffset,
  parseCronExpression,
  convertCronToUTC,
  convertCronToLocal,
  describeCronExpression,
} from '../cronTimezone';

describe('cronTimezone', () => {
  // Save original timezone offset
  const originalGetTimezoneOffset = Date.prototype.getTimezoneOffset;

  beforeEach(() => {
    // Mock timezone to CEST (UTC+2, which is -120 minutes offset)
    Date.prototype.getTimezoneOffset = () => -120;
  });

  afterEach(() => {
    Date.prototype.getTimezoneOffset = originalGetTimezoneOffset;
  });

  describe('getBrowserTimezoneOffset', () => {
    it('should return positive offset for CEST (UTC+2)', () => {
      expect(getBrowserTimezoneOffset()).toBe(2);
    });
  });

  describe('parseCronExpression', () => {
    it('should parse valid 6-field cron expression', () => {
      const result = parseCronExpression('0 0 22 * * *');
      expect(result).toEqual({
        seconds: '0',
        minutes: '0',
        hours: '22',
        day: '*',
        month: '*',
        weekday: '*',
      });
    });

    it('should return null for invalid cron expression', () => {
      expect(parseCronExpression('invalid')).toBeNull();
      expect(parseCronExpression('0 0 22 * *')).toBeNull(); // 5 fields
    });
  });

  describe('convertCronToUTC', () => {
    it('should convert 22:00 local (CEST) to 20:00 UTC', () => {
      // 22:00 CEST (UTC+2) = 20:00 UTC
      const result = convertCronToUTC('0 0 22 * * *');
      expect(result).toBe('0 0 20 * * *');
    });

    it('should convert 2:00 local (CEST) to 0:00 UTC', () => {
      // 2:00 CEST (UTC+2) = 0:00 UTC
      const result = convertCronToUTC('0 0 2 * * *');
      expect(result).toBe('0 0 0 * * *');
    });

    it('should handle midnight crossing (1:00 local to 23:00 previous day UTC)', () => {
      // 1:00 CEST (UTC+2) = 23:00 UTC (previous day)
      const result = convertCronToUTC('0 0 1 * * *');
      expect(result).toBe('0 0 23 * * *');
    });

    it('should leave complex hour expressions unchanged', () => {
      const result = convertCronToUTC('0 0 */6 * * *');
      expect(result).toBe('0 0 */6 * * *');
    });
  });

  describe('convertCronToLocal', () => {
    it('should convert 20:00 UTC to 22:00 local (CEST)', () => {
      const result = convertCronToLocal('0 0 20 * * *');
      expect(result).toBe('0 0 22 * * *');
    });

    it('should convert 0:00 UTC to 2:00 local (CEST)', () => {
      const result = convertCronToLocal('0 0 0 * * *');
      expect(result).toBe('0 0 2 * * *');
    });

    it('should handle midnight crossing (23:00 UTC to 1:00 next day local)', () => {
      const result = convertCronToLocal('0 0 23 * * *');
      expect(result).toBe('0 0 1 * * *');
    });
  });

  describe('describeCronExpression', () => {
    it('should describe daily schedule in local time', () => {
      // 20:00 UTC = 22:00 CEST
      const result = describeCronExpression('0 0 20 * * *');
      expect(result).toContain('22:00');
      expect(result).toContain('local time');
    });

    it('should describe interval schedule', () => {
      const result = describeCronExpression('0 0 */6 * * *');
      expect(result).toBe('Every 6 hours');
    });
  });
});