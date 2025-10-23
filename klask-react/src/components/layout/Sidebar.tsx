import React from 'react';
import { useLocation } from 'react-router-dom';

import { SidebarFilters } from '../search/SidebarFilters';

export const Sidebar: React.FC = () => {
  const location = useLocation();

  // Check if we're on a search page to conditionally render filters
  const isSearchPage = location.pathname.includes('/search');

  return (
    <div className="flex grow flex-col gap-y-5 overflow-y-auto bg-white px-6 pb-4 pt-4 border-r border-gray-200">
      <nav className="flex flex-1 flex-col">
        <ul role="list" className="flex flex-1 flex-col gap-y-7">
          {/* Search Filters */}
          {isSearchPage && (
            <li>
              <SidebarFilters />
            </li>
          )}
        </ul>
      </nav>
    </div>
  );
};