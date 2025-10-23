import React from 'react';
import { FolderIcon } from '@heroicons/react/24/outline';
import { clsx } from 'clsx';
import type { RepositoryType } from '../../types';

interface RepositoryBadgeProps {
  name: string;
  type?: RepositoryType;
  size?: 'sm' | 'md' | 'lg';
  clickable?: boolean;
  onClick?: () => void;
  className?: string;
}

export const RepositoryBadge: React.FC<RepositoryBadgeProps> = ({
  name,
  type,
  size = 'md',
  clickable = false,
  onClick,
  className,
}) => {
  // Color scheme based on repository type
  const getTypeColor = (repoType?: RepositoryType): string => {
    switch (repoType) {
      case 'Git':
        return 'bg-orange-50 text-orange-700 border-orange-200';
      case 'GitLab':
        return 'bg-purple-50 text-purple-700 border-purple-200';
      case 'GitHub':
        return 'bg-gray-50 text-gray-700 border-gray-300';
      case 'FileSystem':
        return 'bg-blue-50 text-blue-700 border-blue-200';
      default:
        return 'bg-gray-50 text-gray-700 border-gray-200';
    }
  };

  const sizeClasses = {
    sm: 'text-xs px-2 py-0.5',
    md: 'text-sm px-2.5 py-1',
    lg: 'text-base px-3 py-1.5',
  };

  const iconSizes = {
    sm: 'h-3 w-3',
    md: 'h-4 w-4',
    lg: 'h-5 w-5',
  };

  const baseClasses = clsx(
    'inline-flex items-center gap-1.5 font-medium rounded-md border',
    sizeClasses[size],
    getTypeColor(type),
    clickable && 'cursor-pointer hover:opacity-80 transition-opacity',
    className
  );

  const content = (
    <>
      <FolderIcon className={iconSizes[size]} />
      <span className="font-mono truncate max-w-[200px]" title={name}>
        {name}
      </span>
    </>
  );

  if (clickable && onClick) {
    return (
      <button onClick={onClick} className={baseClasses} type="button">
        {content}
      </button>
    );
  }

  return <div className={baseClasses}>{content}</div>;
};

// Repository type indicator badge (small version for inline use)
interface RepositoryTypeBadgeProps {
  type: RepositoryType;
  size?: 'sm' | 'md';
  className?: string;
}

export const RepositoryTypeBadge: React.FC<RepositoryTypeBadgeProps> = ({
  type,
  size = 'sm',
  className,
}) => {
  const getTypeStyles = (repoType: RepositoryType): { color: string; label: string } => {
    switch (repoType) {
      case 'Git':
        return { color: 'bg-orange-100 text-orange-800', label: 'Git' };
      case 'GitLab':
        return { color: 'bg-purple-100 text-purple-800', label: 'GitLab' };
      case 'GitHub':
        return { color: 'bg-gray-100 text-gray-800', label: 'GitHub' };
      case 'FileSystem':
        return { color: 'bg-blue-100 text-blue-800', label: 'FS' };
    }
  };

  const styles = getTypeStyles(type);
  const sizeClass = size === 'sm' ? 'text-xs px-1.5 py-0.5' : 'text-sm px-2 py-1';

  return (
    <span className={clsx('inline-flex items-center font-medium rounded', sizeClass, styles.color, className)}>
      {styles.label}
    </span>
  );
};
