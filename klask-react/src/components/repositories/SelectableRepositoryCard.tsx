import React from 'react';
import { RepositoryCard } from './RepositoryCard';
import { CheckIcon } from '@heroicons/react/24/outline';
import type { RepositoryWithStats } from '../../types';
import type { CrawlProgressInfo } from '../../hooks/useProgress';
import { clsx } from 'clsx';

interface SelectableRepositoryCardProps {
  repository: RepositoryWithStats;
  selected: boolean;
  onSelect: (selected: boolean) => void;
  onEdit: (repository: RepositoryWithStats) => void;
  onDelete: (repository: RepositoryWithStats) => void;
  onCrawl: (repository: RepositoryWithStats) => void;
  onToggleEnabled: (repository: RepositoryWithStats) => void;
  activeProgress: CrawlProgressInfo[];
  isLoading?: boolean;
  isCrawling?: boolean;
  className?: string;
}

export const SelectableRepositoryCard: React.FC<SelectableRepositoryCardProps> = ({
  repository,
  selected,
  onSelect,
  onEdit,
  onDelete,
  onCrawl,
  onToggleEnabled,
  activeProgress,
  isLoading = false,
  isCrawling = false,
  className = '',
}) => {
  return (
    <div className={clsx(
      'group relative rounded-lg',
      selected ? 'ring-2 ring-blue-200 bg-blue-50 shadow-lg transform translate-y-0.5' : '',
      'transition-all duration-200',
      className
    )}
    style={{ isolation: 'isolate' }}>
      
      {/* Simple checkbox */}
      <div className="absolute top-3 left-3 z-20">
        <div
          className={clsx(
            'w-5 h-5 rounded border-2 transition-all duration-200 cursor-pointer flex items-center justify-center',
            selected 
              ? 'bg-blue-600 border-blue-600' 
              : 'bg-white border-gray-300 opacity-0 group-hover:opacity-100',
            'hover:border-blue-400'
          )}
          onClick={(e) => {
            e.stopPropagation();
            onSelect(!selected);
          }}
        >
          {selected && <CheckIcon className="w-3 h-3 text-white" />}
        </div>
      </div>

      {/* Repository card */}
      <div
        className="transition-all duration-200"
        onClick={(e) => {
          // Check if clicked element is interactive
          const target = e.target as HTMLElement;
          const isInteractive = target.closest('button') || 
                               target.closest('a') || 
                               target.closest('input') ||
                               target.closest('select') ||
                               target.closest('[role="button"]') ||
                               target.closest('[role="menuitem"]');
          
          // Only select if clicking on non-interactive areas
          if (!isInteractive) {
            onSelect(!selected);
          }
        }}
      >
        <RepositoryCard
          repository={repository}
          onEdit={onEdit}
          onDelete={onDelete}
          onCrawl={onCrawl}
          onToggleEnabled={onToggleEnabled}
          activeProgress={activeProgress}
          isLoading={isLoading}
          isCrawling={isCrawling}
          className={clsx(
            'transition-all duration-200',
            selected 
              ? 'shadow-lg' 
              : 'hover:shadow-md'
          )}
        />
      </div>

    </div>
  );
};