import { describe, it, expect } from 'vitest';
import {
  validateEmail,
  validatePhone,
  validatePassword,
  validateFullName,
  validateBio,
  validateTimezone,
  getAvailableTimezones,
  getPasswordStrengthColor,
  getPasswordStrengthLabel,
} from '../profileValidation';

describe('profileValidation', () => {
  describe('validateEmail', () => {
    it('should validate correct email addresses', () => {
      expect(validateEmail('test@example.com')).toBe(true);
      expect(validateEmail('user.name@domain.co.uk')).toBe(true);
      expect(validateEmail('user+tag@example.com')).toBe(true);
    });

    it('should reject invalid email addresses', () => {
      expect(validateEmail('invalid')).toBe(false);
      expect(validateEmail('invalid@')).toBe(false);
      expect(validateEmail('@example.com')).toBe(false);
      expect(validateEmail('user @example.com')).toBe(false);
    });
  });

  describe('validatePhone', () => {
    it('should validate phone numbers', () => {
      // E.164 format requires + prefix
      expect(validatePhone('+1234567890')).toBe(true);
      expect(validatePhone('+12345678901')).toBe(true);
      expect(validatePhone('+123456789012')).toBe(true);

      // Formatted numbers are cleaned but still need + prefix
      expect(validatePhone('+1 (234) 567-890')).toBe(true);
    });

    it('should reject invalid phone numbers', () => {
      expect(validatePhone('123')).toBe(false);
      expect(validatePhone('abc-def-ghij')).toBe(false);
    });
  });

  describe('validatePassword', () => {
    it('should validate strong passwords', () => {
      const result = validatePassword('StrongPass123!');
      expect(result.isValid).toBe(true);
      expect(result.strength).toBe('strong');
    });

    it('should mark weak passwords', () => {
      const result = validatePassword('weak');
      expect(result.isValid).toBe(false);
      expect(result.strength).toBe('weak');
      expect(result.errors.length).toBeGreaterThan(0);
    });

    it('should return password strength levels', () => {
      const weak = validatePassword('weak');
      expect(['weak', 'fair'].includes(weak.strength)).toBe(true);

      const strong = validatePassword('VeryStrongPassword123!');
      expect(strong.strength).toBe('strong');
    });

    it('should check for uppercase letters', () => {
      const result = validatePassword('lowercase123!');
      expect(result.errors.some((e) => e.includes('uppercase'))).toBe(true);
    });

    it('should check for lowercase letters', () => {
      const result = validatePassword('UPPERCASE123!');
      expect(result.errors.some((e) => e.includes('lowercase'))).toBe(true);
    });

    it('should check for numbers', () => {
      const result = validatePassword('NoNumbers!');
      expect(result.errors.some((e) => e.includes('number'))).toBe(true);
    });

    it('should check for minimum length', () => {
      const result = validatePassword('Short1');
      expect(result.isValid).toBe(false);
    });
  });

  describe('validateFullName', () => {
    it('should validate full names', () => {
      expect(validateFullName('John Doe')).toBe(true);
      expect(validateFullName('Mary-Jane Watson')).toBe(true);
      expect(validateFullName("O'Brien")).toBe(true);
      expect(validateFullName('')).toBe(true); // Optional field
    });

    it('should reject names with special characters', () => {
      expect(validateFullName('John@Doe')).toBe(false);
      expect(validateFullName('Jane#Doe')).toBe(false);
    });

    it('should enforce length limit', () => {
      const longName = 'A'.repeat(256);
      expect(validateFullName(longName)).toBe(false);
    });
  });

  describe('validateBio', () => {
    it('should validate bio text', () => {
      expect(validateBio('I am a software developer')).toBe(true);
      expect(validateBio('')).toBe(true); // Optional field
    });

    it('should enforce length limit', () => {
      const longBio = 'A'.repeat(2001);
      expect(validateBio(longBio)).toBe(false);
    });
  });

  describe('validateTimezone', () => {
    it('should validate known timezones', () => {
      expect(validateTimezone('UTC')).toBe(true);
      expect(validateTimezone('America/New_York')).toBe(true);
      expect(validateTimezone('Europe/London')).toBe(true);
      expect(validateTimezone('Asia/Tokyo')).toBe(true);
    });

    it('should reject unknown timezones', () => {
      expect(validateTimezone('Invalid/Timezone')).toBe(false);
    });

    it('should allow empty timezone', () => {
      expect(validateTimezone('')).toBe(true);
    });
  });

  describe('getAvailableTimezones', () => {
    it('should return array of timezones', () => {
      const timezones = getAvailableTimezones();
      expect(Array.isArray(timezones)).toBe(true);
      expect(timezones.length).toBeGreaterThan(0);
      expect(timezones.includes('UTC')).toBe(true);
    });
  });

  describe('getPasswordStrengthColor', () => {
    it('should return correct colors', () => {
      expect(getPasswordStrengthColor('weak')).toBe('bg-red-500');
      expect(getPasswordStrengthColor('fair')).toBe('bg-yellow-500');
      expect(getPasswordStrengthColor('good')).toBe('bg-blue-500');
      expect(getPasswordStrengthColor('strong')).toBe('bg-green-500');
      expect(getPasswordStrengthColor('unknown')).toBe('bg-gray-300');
    });
  });

  describe('getPasswordStrengthLabel', () => {
    it('should return correct labels', () => {
      expect(getPasswordStrengthLabel('weak')).toBe('Weak');
      expect(getPasswordStrengthLabel('fair')).toBe('Fair');
      expect(getPasswordStrengthLabel('good')).toBe('Good');
      expect(getPasswordStrengthLabel('strong')).toBe('Strong');
    });
  });
});
