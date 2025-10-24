import React from 'react';

interface AvatarDisplayProps {
  /** Avatar image URL (base64 data URI) */
  avatarUrl?: string;
  /** User's full name or username for fallback */
  displayName?: string;
  /** Size of avatar: sm (32px), md (48px), lg (64px), xl (128px) */
  size?: 'sm' | 'md' | 'lg' | 'xl';
  /** Optional CSS class to apply to the avatar container */
  className?: string;
  /** Whether to show as circular or rounded square */
  rounded?: 'full' | 'lg';
}

const AvatarDisplay: React.FC<AvatarDisplayProps> = ({
  avatarUrl,
  displayName = 'U',
  size = 'md',
  className = '',
  rounded = 'full',
}) => {
  const sizeClasses = {
    sm: 'w-8 h-8 text-sm',
    md: 'w-12 h-12 text-base',
    lg: 'w-16 h-16 text-lg',
    xl: 'w-32 h-32 text-3xl',
  };

  const roundedClasses = {
    full: 'rounded-full',
    lg: 'rounded-lg',
  };

  const getInitials = (name: string): string => {
    return name
      .split(' ')
      .map((n) => n[0])
      .join('')
      .toUpperCase()
      .substring(0, 2);
  };

  return (
    <div
      className={`${sizeClasses[size]} ${roundedClasses[rounded]} overflow-hidden border border-gray-200 flex-shrink-0 ${className}`}
    >
      {avatarUrl ? (
        <img
          src={avatarUrl}
          alt={displayName}
          className="w-full h-full object-cover"
        />
      ) : (
        <div className="w-full h-full bg-gradient-to-br from-blue-400 to-purple-500 flex items-center justify-center">
          <span className="text-white font-bold">{getInitials(displayName)}</span>
        </div>
      )}
    </div>
  );
};

export default AvatarDisplay;
