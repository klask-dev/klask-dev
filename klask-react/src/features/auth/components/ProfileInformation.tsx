import React, { useState, useCallback } from 'react';
import { useDebouncedCallback } from 'use-debounce';
import type { UpdateProfileRequest } from '../../../types';
import { useProfile } from '../../../hooks/useProfile';
import { validateFullName, validateBio, validatePhone } from '../../../lib/profileValidation';
import { getAvailableTimezones } from '../../../lib/profileValidation';

const ProfileInformation: React.FC = () => {
  const { user, updateProfile, isUpdating } = useProfile();
  const [formData, setFormData] = useState({
    full_name: user?.full_name || '',
    bio: user?.bio || '',
    phone: user?.phone || '',
    timezone: user?.timezone || 'UTC',
  });

  const [errors, setErrors] = useState<Record<string, string>>({});
  const [hasChanges, setHasChanges] = useState(false);

  const validateBeforeSubmit = (data: UpdateProfileRequest): boolean => {
    // Final validation before API submission
    const errors: Record<string, string> = {};

    if (data.full_name && data.full_name.length > 255) {
      errors.full_name = 'Full name must be 255 characters or less';
    }

    if (data.bio && data.bio.length > 2000) {
      errors.bio = 'Bio must be 2000 characters or less';
    }

    if (data.phone && !validatePhone(data.phone)) {
      errors.phone = 'Phone must be in E.164 format (e.g., +1234567890)';
    }

    if (Object.keys(errors).length > 0) {
      setErrors(errors);
      return false;
    }

    return true;
  };

  const debouncedUpdate = useDebouncedCallback((data: UpdateProfileRequest) => {
    if (validateBeforeSubmit(data)) {
      updateProfile(data);
      setHasChanges(false);
    }
  }, 1000);

  const handleChange = useCallback(
    (field: keyof typeof formData, value: string) => {
      const newData = { ...formData, [field]: value };
      setFormData(newData);
      setHasChanges(true);

      // Validate field
      const fieldErrors: Record<string, string> = { ...errors };

      if (field === 'full_name' && value && !validateFullName(value)) {
        fieldErrors[field] =
          'Full name can only contain letters, spaces, hyphens, and apostrophes (max 255 characters)';
      } else if (field === 'bio' && value && !validateBio(value)) {
        fieldErrors[field] = 'Bio must be less than 2000 characters';
      } else if (field === 'phone' && value && !validatePhone(value)) {
        fieldErrors[field] = 'Phone must be in E.164 format (e.g., +1234567890)';
      } else {
        delete fieldErrors[field];
      }

      setErrors(fieldErrors);

      // Trigger debounced update if valid
      if (Object.keys(fieldErrors).length === 0) {
        debouncedUpdate(newData);
      }
    },
    [formData, errors, debouncedUpdate]
  );

  return (
    <div className="space-y-6">
      {/* Full Name */}
      <div>
        <label htmlFor="full_name" className="block text-sm font-medium text-gray-700 mb-2">
          Full Name
        </label>
        <input
          id="full_name"
          type="text"
          value={formData.full_name}
          onChange={(e) => handleChange('full_name', e.target.value)}
          placeholder="Enter your full name"
          className={`w-full px-4 py-2 border rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-transparent outline-none transition ${
            errors.full_name ? 'border-red-500' : 'border-gray-300'
          }`}
          disabled={isUpdating}
        />
        {errors.full_name && <p className="mt-1 text-sm text-red-600">{errors.full_name}</p>}
        <p className="mt-1 text-xs text-gray-500">{formData.full_name.length}/255</p>
      </div>

      {/* Bio */}
      <div>
        <label htmlFor="bio" className="block text-sm font-medium text-gray-700 mb-2">
          Bio
        </label>
        <textarea
          id="bio"
          value={formData.bio}
          onChange={(e) => handleChange('bio', e.target.value)}
          placeholder="Tell us about yourself"
          rows={4}
          className={`w-full px-4 py-2 border rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-transparent outline-none transition resize-none ${
            errors.bio ? 'border-red-500' : 'border-gray-300'
          }`}
          disabled={isUpdating}
        />
        {errors.bio && <p className="mt-1 text-sm text-red-600">{errors.bio}</p>}
        <p className="mt-1 text-xs text-gray-500">{formData.bio.length}/2000</p>
      </div>

      {/* Phone */}
      <div>
        <label htmlFor="phone" className="block text-sm font-medium text-gray-700 mb-2">
          Phone Number
        </label>
        <input
          id="phone"
          type="tel"
          value={formData.phone}
          onChange={(e) => handleChange('phone', e.target.value)}
          placeholder="+1 (555) 123-4567"
          className={`w-full px-4 py-2 border rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-transparent outline-none transition ${
            errors.phone ? 'border-red-500' : 'border-gray-300'
          }`}
          disabled={isUpdating}
        />
        {errors.phone && <p className="mt-1 text-sm text-red-600">{errors.phone}</p>}
      </div>

      {/* Timezone */}
      <div>
        <label htmlFor="timezone" className="block text-sm font-medium text-gray-700 mb-2">
          Timezone
        </label>
        <select
          id="timezone"
          value={formData.timezone}
          onChange={(e) => handleChange('timezone', e.target.value)}
          className="w-full px-4 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-transparent outline-none transition"
          disabled={isUpdating}
        >
          {getAvailableTimezones().map((tz) => (
            <option key={tz} value={tz}>
              {tz}
            </option>
          ))}
        </select>
      </div>

      {/* Status */}
      {hasChanges && (
        <div className="p-3 bg-blue-50 border border-blue-200 rounded-lg flex items-center gap-2">
          <svg className="w-5 h-5 text-blue-600" fill="currentColor" viewBox="0 0 20 20">
            <path
              fillRule="evenodd"
              d="M18 5v8a2 2 0 01-2 2h-5l-5 4v-4H4a2 2 0 01-2-2V5a2 2 0 012-2h12a2 2 0 012 2zm-11-1a1 1 0 11-2 0 1 1 0 012 0z"
              clipRule="evenodd"
            />
          </svg>
          <span className="text-sm text-blue-700">
            {isUpdating ? 'Saving changes...' : 'Unsaved changes'}
          </span>
        </div>
      )}

      {isUpdating && (
        <div className="p-3 bg-gray-50 border border-gray-200 rounded-lg flex items-center gap-2">
          <div className="w-4 h-4 border-2 border-blue-500 border-t-transparent rounded-full animate-spin" />
          <span className="text-sm text-gray-700">Updating your profile...</span>
        </div>
      )}
    </div>
  );
};

export default ProfileInformation;
