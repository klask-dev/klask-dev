import React, { useRef, useState } from 'react';
import DOMPurify from 'dompurify';
import { useProfile, useUploadAvatar } from '../../../hooks/useProfile';

const ProfileHeader: React.FC = () => {
  const { user } = useProfile();
  const uploadAvatarMutation = useUploadAvatar();
  const fileInputRef = useRef<HTMLInputElement>(null);
  const [dragOver, setDragOver] = useState(false);

  const handleFileSelect = async (file: File) => {
    if (!file.type.startsWith('image/')) {
      alert('Please select an image file');
      return;
    }

    if (file.size > 5 * 1024 * 1024) {
      alert('File size must be less than 5MB');
      return;
    }

    uploadAvatarMutation.mutate(file);
  };

  const handleDragOver = (e: React.DragEvent) => {
    e.preventDefault();
    setDragOver(true);
  };

  const handleDragLeave = () => {
    setDragOver(false);
  };

  const handleDrop = (e: React.DragEvent) => {
    e.preventDefault();
    setDragOver(false);

    const file = e.dataTransfer.files[0];
    if (file) {
      handleFileSelect(file);
    }
  };

  const handleFileInputChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    const file = e.currentTarget.files?.[0];
    if (file) {
      handleFileSelect(file);
    }
  };

  const getInitials = (name: string): string => {
    return name
      .split(' ')
      .map((n) => n[0])
      .join('')
      .toUpperCase();
  };

  return (
    <div className="bg-white rounded-lg shadow-sm border border-gray-200 p-8">
      <div className="flex flex-col sm:flex-row items-center sm:items-start gap-8">
        {/* Avatar Section */}
        <div className="flex flex-col items-center">
          <div
            className={`relative w-32 h-32 rounded-full mb-4 overflow-hidden border-4 border-gray-200 ${
              dragOver ? 'border-blue-500 bg-blue-50' : ''
            } cursor-pointer transition-colors`}
            onDragOver={handleDragOver}
            onDragLeave={handleDragLeave}
            onDrop={handleDrop}
            onClick={() => fileInputRef.current?.click()}
          >
            {user?.avatar_url ? (
              <img
                src={user.avatar_url}
                alt={user.username}
                className="w-full h-full object-cover"
              />
            ) : (
              <div className="w-full h-full bg-gradient-to-br from-blue-400 to-purple-500 flex items-center justify-center">
                <span className="text-white text-3xl font-bold">
                  {getInitials(user?.full_name || user?.username || 'U')}
                </span>
              </div>
            )}

            {/* Upload overlay */}
            <div className="absolute inset-0 bg-black bg-opacity-0 hover:bg-opacity-40 flex items-center justify-center transition-all">
              <svg
                className="w-8 h-8 text-white opacity-0 hover:opacity-100 transition-opacity"
                fill="none"
                stroke="currentColor"
                viewBox="0 0 24 24"
              >
                <path
                  strokeLinecap="round"
                  strokeLinejoin="round"
                  strokeWidth={2}
                  d="M12 4v16m8-8H4"
                />
              </svg>
            </div>

            {uploadAvatarMutation.isPending && (
              <div className="absolute inset-0 bg-white bg-opacity-80 flex items-center justify-center">
                <div className="w-6 h-6 border-2 border-blue-500 border-t-transparent rounded-full animate-spin" />
              </div>
            )}
          </div>

          <input
            ref={fileInputRef}
            type="file"
            accept="image/*"
            onChange={handleFileInputChange}
            className="hidden"
            disabled={uploadAvatarMutation.isPending}
          />

          <p className="text-xs text-gray-500 text-center">
            Drag and drop or click to upload
            <br />
            Max size: 5MB
          </p>
        </div>

        {/* User Info Section */}
        <div className="flex-1">
          <div className="mb-2">
            <h1 className="text-3xl font-bold text-gray-900">
              {user?.full_name || user?.username}
            </h1>
            <p className="text-gray-600">@{user?.username}</p>
          </div>

          <div className="grid grid-cols-2 gap-4 mt-6">
            <div>
              <p className="text-sm text-gray-600">Email</p>
              <p className="font-medium text-gray-900">{user?.email}</p>
            </div>

            <div>
              <p className="text-sm text-gray-600">Role</p>
              <p className="font-medium text-gray-900">
                <span className="inline-block px-3 py-1 rounded-full text-sm font-semibold bg-blue-100 text-blue-700">
                  {user?.role}
                </span>
              </p>
            </div>

            {user?.phone && (
              <div>
                <p className="text-sm text-gray-600">Phone</p>
                <p className="font-medium text-gray-900">{user.phone}</p>
              </div>
            )}

            {user?.login_count !== undefined && (
              <div>
                <p className="text-sm text-gray-600">Logins</p>
                <p className="font-medium text-gray-900">{user.login_count}</p>
              </div>
            )}
          </div>

          {user?.bio && (
            <div className="mt-4 pt-4 border-t border-gray-200">
              <p className="text-sm text-gray-600">Bio</p>
              <p
                className="text-gray-900 italic"
                dangerouslySetInnerHTML={{
                  __html: DOMPurify.sanitize(user.bio, {
                    ALLOWED_TAGS: [],
                    ALLOWED_ATTR: [],
                    KEEP_CONTENT: true,
                  }),
                }}
              />
            </div>
          )}
        </div>
      </div>
    </div>
  );
};

export default ProfileHeader;
