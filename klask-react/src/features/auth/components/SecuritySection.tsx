import React, { useState } from 'react';
import toast from 'react-hot-toast';
import type { ChangePasswordRequest } from '../../../types';
import { useChangePassword } from '../../../hooks/useProfile';
import {
  validatePassword,
  getPasswordStrengthColor,
  getPasswordStrengthLabel,
} from '../../../lib/profileValidation';

const SecuritySection: React.FC = () => {
  const changePasswordMutation = useChangePassword();

  const [formData, setFormData] = useState({
    current_password: '',
    new_password: '',
    new_password_confirm: '',
  });

  const [showPasswords, setShowPasswords] = useState({
    current: false,
    new: false,
    confirm: false,
  });

  const [errors, setErrors] = useState<Record<string, string>>({});

  const passwordValidation = validatePassword(formData.new_password);

  const handleChange = (field: string, value: string) => {
    setFormData({ ...formData, [field]: value });
    // Clear error for this field
    const newErrors = { ...errors };
    delete newErrors[field];
    setErrors(newErrors);
  };

  const validateForm = (): boolean => {
    const newErrors: Record<string, string> = {};

    if (!formData.current_password) {
      newErrors.current_password = 'Current password is required';
    }

    if (!formData.new_password) {
      newErrors.new_password = 'New password is required';
    } else if (formData.new_password.length < 8) {
      newErrors.new_password = 'Password must be at least 8 characters';
    } else if (!passwordValidation.isValid) {
      newErrors.new_password = passwordValidation.errors[0] || 'Password does not meet requirements';
    }

    if (formData.new_password !== formData.new_password_confirm) {
      newErrors.new_password_confirm = 'Passwords do not match';
    }

    setErrors(newErrors);
    return Object.keys(newErrors).length === 0;
  };

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();

    if (!validateForm()) {
      return;
    }

    const data: ChangePasswordRequest = {
      current_password: formData.current_password,
      new_password: formData.new_password,
      new_password_confirm: formData.new_password_confirm,
    };

    changePasswordMutation.mutate(data, {
      onSuccess: () => {
        setFormData({
          current_password: '',
          new_password: '',
          new_password_confirm: '',
        });
        toast.success('Password changed successfully');
      },
    });
  };

  return (
    <div className="space-y-6">
      <form onSubmit={handleSubmit} className="space-y-6">
        {/* Current Password */}
        <div>
          <label htmlFor="current_password" className="block text-sm font-medium text-gray-700 mb-2">
            Current Password
          </label>
          <div className="relative">
            <input
              id="current_password"
              type={showPasswords.current ? 'text' : 'password'}
              value={formData.current_password}
              onChange={(e) => handleChange('current_password', e.target.value)}
              className={`w-full px-4 py-2 border rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-transparent outline-none transition pr-10 ${
                errors.current_password ? 'border-red-500' : 'border-gray-300'
              }`}
              disabled={changePasswordMutation.isPending}
            />
            <button
              type="button"
              onClick={() => setShowPasswords({ ...showPasswords, current: !showPasswords.current })}
              className="absolute right-3 top-1/2 transform -translate-y-1/2 text-gray-500 hover:text-gray-700"
            >
              {showPasswords.current ? (
                <svg className="w-5 h-5" fill="currentColor" viewBox="0 0 20 20">
                  <path d="M10 12a2 2 0 100-4 2 2 0 000 4z" />
                  <path
                    fillRule="evenodd"
                    d="M.458 10C1.732 5.943 5.522 3 10 3s8.268 2.943 9.542 7c-1.274 4.057-5.064 7-9.542 7S1.732 14.057.458 10zM14 10a4 4 0 11-8 0 4 4 0 018 0z"
                    clipRule="evenodd"
                  />
                </svg>
              ) : (
                <svg className="w-5 h-5" fill="currentColor" viewBox="0 0 20 20">
                  <path
                    fillRule="evenodd"
                    d="M3.707 2.293a1 1 0 00-1.414 1.414l14 14a1 1 0 001.414-1.414l-14-14zM10 4a6 6 0 100 12 6 6 0 000-12zM7.5 10.5a2.5 2.5 0 115 0 2.5 2.5 0 01-5 0z"
                    clipRule="evenodd"
                  />
                </svg>
              )}
            </button>
          </div>
          {errors.current_password && (
            <p className="mt-1 text-sm text-red-600">{errors.current_password}</p>
          )}
        </div>

        {/* New Password */}
        <div>
          <label htmlFor="new_password" className="block text-sm font-medium text-gray-700 mb-2">
            New Password
          </label>
          <div className="relative">
            <input
              id="new_password"
              type={showPasswords.new ? 'text' : 'password'}
              value={formData.new_password}
              onChange={(e) => handleChange('new_password', e.target.value)}
              className={`w-full px-4 py-2 border rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-transparent outline-none transition pr-10 ${
                errors.new_password ? 'border-red-500' : 'border-gray-300'
              }`}
              disabled={changePasswordMutation.isPending}
            />
            <button
              type="button"
              onClick={() => setShowPasswords({ ...showPasswords, new: !showPasswords.new })}
              className="absolute right-3 top-1/2 transform -translate-y-1/2 text-gray-500 hover:text-gray-700"
            >
              {showPasswords.new ? (
                <svg className="w-5 h-5" fill="currentColor" viewBox="0 0 20 20">
                  <path d="M10 12a2 2 0 100-4 2 2 0 000 4z" />
                  <path
                    fillRule="evenodd"
                    d="M.458 10C1.732 5.943 5.522 3 10 3s8.268 2.943 9.542 7c-1.274 4.057-5.064 7-9.542 7S1.732 14.057.458 10zM14 10a4 4 0 11-8 0 4 4 0 018 0z"
                    clipRule="evenodd"
                  />
                </svg>
              ) : (
                <svg className="w-5 h-5" fill="currentColor" viewBox="0 0 20 20">
                  <path
                    fillRule="evenodd"
                    d="M3.707 2.293a1 1 0 00-1.414 1.414l14 14a1 1 0 001.414-1.414l-14-14zM10 4a6 6 0 100 12 6 6 0 000-12zM7.5 10.5a2.5 2.5 0 115 0 2.5 2.5 0 01-5 0z"
                    clipRule="evenodd"
                  />
                </svg>
              )}
            </button>
          </div>
          {errors.new_password && (
            <p className="mt-1 text-sm text-red-600">{errors.new_password}</p>
          )}

          {/* Password Strength Indicator */}
          {formData.new_password && (
            <div className="mt-3 space-y-2">
              <div className="flex items-center justify-between">
                <p className="text-sm text-gray-600">Password Strength:</p>
                <span
                  className={`text-sm font-medium ${
                    passwordValidation.strength === 'weak'
                      ? 'text-red-600'
                      : passwordValidation.strength === 'fair'
                        ? 'text-yellow-600'
                        : passwordValidation.strength === 'good'
                          ? 'text-blue-600'
                          : 'text-green-600'
                  }`}
                >
                  {getPasswordStrengthLabel(passwordValidation.strength)}
                </span>
              </div>

              <div className="w-full bg-gray-200 rounded-full h-2">
                <div
                  className={`h-2 rounded-full transition-all ${getPasswordStrengthColor(
                    passwordValidation.strength
                  )}`}
                  style={{
                    width: `${(passwordValidation.score / 5) * 100}%`,
                  }}
                />
              </div>

              {passwordValidation.errors.length > 0 && (
                <ul className="text-xs text-gray-600 space-y-1">
                  {passwordValidation.errors.map((error, idx) => (
                    <li key={idx} className="flex items-center gap-1">
                      <svg className="w-3 h-3 text-orange-500" fill="currentColor" viewBox="0 0 20 20">
                        <path
                          fillRule="evenodd"
                          d="M8.257 3.099c.765-1.36 2.722-1.36 3.486 0l5.58 9.92c.75 1.334-.213 2.98-1.742 2.98H4.42c-1.53 0-2.493-1.646-1.743-2.98l5.58-9.92zM11 13a1 1 0 11-2 0 1 1 0 012 0zm-1-8a1 1 0 00-1 1v3a1 1 0 002 0V6a1 1 0 00-1-1z"
                          clipRule="evenodd"
                        />
                      </svg>
                      {error}
                    </li>
                  ))}
                </ul>
              )}
            </div>
          )}
        </div>

        {/* Confirm Password */}
        <div>
          <label htmlFor="new_password_confirm" className="block text-sm font-medium text-gray-700 mb-2">
            Confirm New Password
          </label>
          <div className="relative">
            <input
              id="new_password_confirm"
              type={showPasswords.confirm ? 'text' : 'password'}
              value={formData.new_password_confirm}
              onChange={(e) => handleChange('new_password_confirm', e.target.value)}
              className={`w-full px-4 py-2 border rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-transparent outline-none transition pr-10 ${
                errors.new_password_confirm ? 'border-red-500' : 'border-gray-300'
              }`}
              disabled={changePasswordMutation.isPending}
            />
            <button
              type="button"
              onClick={() => setShowPasswords({ ...showPasswords, confirm: !showPasswords.confirm })}
              className="absolute right-3 top-1/2 transform -translate-y-1/2 text-gray-500 hover:text-gray-700"
            >
              {showPasswords.confirm ? (
                <svg className="w-5 h-5" fill="currentColor" viewBox="0 0 20 20">
                  <path d="M10 12a2 2 0 100-4 2 2 0 000 4z" />
                  <path
                    fillRule="evenodd"
                    d="M.458 10C1.732 5.943 5.522 3 10 3s8.268 2.943 9.542 7c-1.274 4.057-5.064 7-9.542 7S1.732 14.057.458 10zM14 10a4 4 0 11-8 0 4 4 0 018 0z"
                    clipRule="evenodd"
                  />
                </svg>
              ) : (
                <svg className="w-5 h-5" fill="currentColor" viewBox="0 0 20 20">
                  <path
                    fillRule="evenodd"
                    d="M3.707 2.293a1 1 0 00-1.414 1.414l14 14a1 1 0 001.414-1.414l-14-14zM10 4a6 6 0 100 12 6 6 0 000-12zM7.5 10.5a2.5 2.5 0 115 0 2.5 2.5 0 01-5 0z"
                    clipRule="evenodd"
                  />
                </svg>
              )}
            </button>
          </div>
          {errors.new_password_confirm && (
            <p className="mt-1 text-sm text-red-600">{errors.new_password_confirm}</p>
          )}
        </div>

        {/* Submit Button */}
        <div className="flex justify-end pt-4">
          <button
            type="submit"
            disabled={changePasswordMutation.isPending || !formData.current_password || !formData.new_password}
            className="px-6 py-2 bg-green-600 text-white rounded-lg hover:bg-green-700 disabled:bg-gray-400 disabled:cursor-not-allowed transition flex items-center gap-2"
          >
            {changePasswordMutation.isPending && (
              <div className="w-4 h-4 border-2 border-white border-t-transparent rounded-full animate-spin" />
            )}
            {changePasswordMutation.isPending ? 'Updating...' : 'Update Password'}
          </button>
        </div>
      </form>
    </div>
  );
};

export default SecuritySection;
