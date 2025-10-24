/**
 * Profile-related validation utilities
 */

export interface PasswordStrengthResult {
  isValid: boolean;
  strength: 'weak' | 'fair' | 'good' | 'strong';
  errors: string[];
  score: number;
}

/**
 * Validate email format
 */
export function validateEmail(email: string): boolean {
  const emailRegex = /^[^\s@]+@[^\s@]+\.[^\s@]+$/;
  return emailRegex.test(email);
}

/**
 * Validate phone number (E.164 format for international numbers)
 * Supports: +1234567890, +44 20 7946 0958, +33 1 42 68 53 00
 */
export function validatePhone(phone: string): boolean {
  // E.164 format: +[country code][number]
  // Must start with +, followed by 1-3 digit country code, then 4-14 digits
  const e164Regex = /^\+[1-9]\d{1,14}$/;
  // Remove common separators for validation
  const cleanedPhone = phone.replace(/[\s\-().]/g, '');
  return e164Regex.test(cleanedPhone);
}

/**
 * Validate password strength
 */
export function validatePassword(password: string): PasswordStrengthResult {
  const errors: string[] = [];
  let score = 0;

  // Check minimum length
  if (password.length < 8) {
    errors.push('Password must be at least 8 characters long');
  } else {
    score += 1;
  }

  // Check for uppercase
  if (!/[A-Z]/.test(password)) {
    errors.push('Password must contain at least one uppercase letter');
  } else {
    score += 1;
  }

  // Check for lowercase
  if (!/[a-z]/.test(password)) {
    errors.push('Password must contain at least one lowercase letter');
  } else {
    score += 1;
  }

  // Check for numbers
  if (!/[0-9]/.test(password)) {
    errors.push('Password must contain at least one number');
  } else {
    score += 1;
  }

  // Check for special characters
  if (!/[!@#$%^&*()_+=[\]{};':"\\|,.<>/?-]/.test(password)) {
    errors.push('Password should contain at least one special character');
  } else {
    score += 1;
  }

  // Bonus for length > 12
  if (password.length > 12) {
    score += 0.5;
  }

  // Determine strength
  let strength: 'weak' | 'fair' | 'good' | 'strong';
  if (score < 2) {
    strength = 'weak';
  } else if (score < 3) {
    strength = 'fair';
  } else if (score < 4) {
    strength = 'good';
  } else {
    strength = 'strong';
  }

  return {
    isValid: errors.length === 0 || errors.length <= 1, // Allow missing special char
    strength,
    errors,
    score: Math.min(score, 5),
  };
}

/**
 * Validate full name
 */
export function validateFullName(name: string): boolean {
  if (!name || name.trim().length === 0) return true; // Optional field
  if (name.length > 255) return false;
  // Allow letters, spaces, hyphens, apostrophes
  return /^[a-zA-Z\s\-']+$/.test(name);
}

/**
 * Validate bio
 */
export function validateBio(bio: string): boolean {
  if (!bio || bio.trim().length === 0) return true; // Optional field
  return bio.length <= 2000;
}

/**
 * Validate timezone (IANA timezone format)
 */
export function validateTimezone(timezone: string): boolean {
  if (!timezone) return true; // Optional field

  const validTimezones = [
    'UTC',
    'America/New_York',
    'America/Chicago',
    'America/Denver',
    'America/Los_Angeles',
    'Europe/London',
    'Europe/Paris',
    'Europe/Berlin',
    'Europe/Amsterdam',
    'Europe/Madrid',
    'Europe/Rome',
    'Europe/Vienna',
    'Asia/Dubai',
    'Asia/Kolkata',
    'Asia/Bangkok',
    'Asia/Singapore',
    'Asia/Hong_Kong',
    'Asia/Tokyo',
    'Asia/Shanghai',
    'Australia/Sydney',
    'Australia/Melbourne',
    'Australia/Brisbane',
    'Pacific/Auckland',
  ];

  return validTimezones.includes(timezone);
}

/**
 * Get all available IANA timezones (subset for common ones)
 */
export function getAvailableTimezones(): string[] {
  return [
    'UTC',
    'America/New_York',
    'America/Chicago',
    'America/Denver',
    'America/Los_Angeles',
    'America/Toronto',
    'America/Mexico_City',
    'America/Buenos_Aires',
    'America/Sao_Paulo',
    'Europe/London',
    'Europe/Paris',
    'Europe/Berlin',
    'Europe/Amsterdam',
    'Europe/Madrid',
    'Europe/Rome',
    'Europe/Vienna',
    'Europe/Prague',
    'Europe/Warsaw',
    'Europe/Istanbul',
    'Asia/Dubai',
    'Asia/Kolkata',
    'Asia/Bangkok',
    'Asia/Singapore',
    'Asia/Hong_Kong',
    'Asia/Tokyo',
    'Asia/Shanghai',
    'Asia/Seoul',
    'Australia/Sydney',
    'Australia/Melbourne',
    'Australia/Brisbane',
    'Pacific/Auckland',
  ];
}

/**
 * Get password strength color for UI
 */
export function getPasswordStrengthColor(strength: string): string {
  switch (strength) {
    case 'weak':
      return 'bg-red-500';
    case 'fair':
      return 'bg-yellow-500';
    case 'good':
      return 'bg-blue-500';
    case 'strong':
      return 'bg-green-500';
    default:
      return 'bg-gray-300';
  }
}

/**
 * Get password strength label for UI
 */
export function getPasswordStrengthLabel(strength: string): string {
  switch (strength) {
    case 'weak':
      return 'Weak';
    case 'fair':
      return 'Fair';
    case 'good':
      return 'Good';
    case 'strong':
      return 'Strong';
    default:
      return 'Unknown';
  }
}
