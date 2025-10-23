import React, { Fragment } from 'react';
import { Link, useLocation, useNavigate } from 'react-router-dom';
import { Menu, Transition } from '@headlessui/react';
import {
  Bars3Icon,
  MagnifyingGlassIcon,
  FolderIcon,
  UserIcon,
  Cog6ToothIcon,
  ArrowRightOnRectangleIcon,
  XMarkIcon,
  UsersIcon,
  ChevronDownIcon
} from '@heroicons/react/24/outline';
import { clsx } from 'clsx';

import { authSelectors, useAuthStore } from '../../stores/auth-store';
import { IconButton } from '../ui/Button';

export const Navbar: React.FC = () => {
  const location = useLocation();
  const navigate = useNavigate();
  const [mobileMenuOpen, setMobileMenuOpen] = React.useState(false);

  const user = authSelectors.user();
  const isAdmin = authSelectors.isAdmin();

  const logout = useAuthStore((state) => state.logout);

  const handleLogout = () => {
    logout();
    navigate('/login');
  };

  const navigation = [
    { name: 'Search', href: '/search', icon: MagnifyingGlassIcon, current: location.pathname.startsWith('/search') },
  ];

  const adminNavigation = [
    { name: 'Dashboard', href: '/admin', icon: Cog6ToothIcon },
    { name: 'Repositories', href: '/admin/repositories', icon: FolderIcon },
    { name: 'Users', href: '/admin/users', icon: UsersIcon },
    { name: 'Index', href: '/admin/index', icon: MagnifyingGlassIcon },
  ];

  return (
    <nav className="fixed top-0 z-50 w-full bg-white border-b border-gray-200">
      <div className="px-3 py-3 lg:px-5 lg:pl-3">
        <div className="flex items-center justify-between">
          <div className="flex items-center justify-start">
            {/* Mobile menu toggle */}
            <IconButton
              variant="ghost"
              size="md"
              icon={mobileMenuOpen ? <XMarkIcon className="w-5 h-5" /> : <Bars3Icon className="w-5 h-5" />}
              onClick={() => setMobileMenuOpen(!mobileMenuOpen)}
              aria-label={mobileMenuOpen ? 'Close menu' : 'Open menu'}
              className="md:hidden mr-2"
            />

            {/* Logo and brand */}
            <Link to="/home" className="flex items-center ml-2 md:mr-24">
              <div className="w-8 h-8 bg-blue-600 rounded-lg flex items-center justify-center mr-3">
                <MagnifyingGlassIcon className="w-5 h-5 text-white" />
              </div>
              <span className="self-center text-xl font-semibold sm:text-2xl whitespace-nowrap">
                Klask
              </span>
              <span className="ml-2 px-2 py-1 text-xs font-medium bg-blue-100 text-blue-800 rounded-full">
                v2.0
              </span>
            </Link>
          </div>

          {/* Navigation */}
          <div className="hidden md:flex items-center space-x-4">
            {navigation.map((item) => (
              <Link
                key={item.name}
                to={item.href}
                className={clsx(
                  'flex items-center px-3 py-2 text-sm font-medium rounded-md transition-colors',
                  item.current
                    ? 'bg-blue-100 text-blue-700'
                    : 'text-gray-700 hover:bg-gray-100'
                )}
              >
                <item.icon className="w-4 h-4 mr-2" />
                {item.name}
              </Link>
            ))}

            {/* Admin navigation buttons */}
            {isAdmin && adminNavigation.map((item) => {
              const isActive = location.pathname === item.href;
              return (
                <Link
                  key={item.name}
                  to={item.href}
                  className={clsx(
                    'flex items-center px-3 py-2 text-sm font-medium rounded-md transition-colors',
                    isActive
                      ? 'bg-blue-100 text-blue-700'
                      : 'text-gray-700 hover:bg-gray-100'
                  )}
                >
                  <item.icon className="w-4 h-4 mr-2" />
                  {item.name}
                </Link>
              );
            })}
          </div>

          {/* User menu */}
          <div className="flex items-center">
            <div className="flex items-center ml-3">
              <div className="hidden md:block">
                <div className="flex items-center">

                  {/* User dropdown */}
                  <Menu as="div" className="relative ml-3">
                    <div>
                      <Menu.Button className="flex items-center text-sm rounded-full focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-blue-500">
                        <span className="sr-only">Open user menu</span>
                        <div className="w-8 h-8 bg-gray-300 rounded-full flex items-center justify-center">
                          <UserIcon className="w-5 h-5 text-gray-600" />
                        </div>
                        <span className="ml-3 text-gray-700 text-sm font-medium hidden lg:block">
                          {user?.username}
                        </span>
                        <ChevronDownIcon className="w-4 h-4 ml-1 text-gray-600 hidden lg:block" />
                      </Menu.Button>
                    </div>
                    <Transition
                      as={Fragment}
                      enter="transition ease-out duration-100"
                      enterFrom="transform opacity-0 scale-95"
                      enterTo="transform opacity-100 scale-100"
                      leave="transition ease-in duration-75"
                      leaveFrom="transform opacity-100 scale-100"
                      leaveTo="transform opacity-0 scale-95"
                    >
                      <Menu.Items className="absolute right-0 z-10 mt-2 w-48 origin-top-right rounded-md bg-white py-1 shadow-lg ring-1 ring-black ring-opacity-5 focus:outline-none">
                        <div className="px-4 py-2 text-sm text-gray-900 border-b border-gray-100">
                          <div className="font-medium">{user?.username}</div>
                          <div className="text-gray-500">{user?.email}</div>
                        </div>

                        <Menu.Item>
                          {({ active }) => (
                            <Link
                              to="/profile"
                              className={clsx(
                                active ? 'bg-gray-100' : '',
                                'block px-4 py-2 text-sm text-gray-700'
                              )}
                            >
                              Your Profile
                            </Link>
                          )}
                        </Menu.Item>

                        <Menu.Item>
                          {({ active }) => (
                            <button
                              onClick={handleLogout}
                              className={clsx(
                                active ? 'bg-gray-100' : '',
                                'block w-full text-left px-4 py-2 text-sm text-gray-700'
                              )}
                            >
                              <ArrowRightOnRectangleIcon className="w-4 h-4 inline mr-2" />
                              Sign out
                            </button>
                          )}
                        </Menu.Item>
                      </Menu.Items>
                    </Transition>
                  </Menu>
                </div>
              </div>

              {/* Mobile user menu */}
              <div className="md:hidden">
                <IconButton
                  variant="ghost"
                  size="md"
                  icon={<UserIcon className="w-5 h-5" />}
                  onClick={() => navigate('/profile')}
                  aria-label="User profile"
                />
              </div>
            </div>
          </div>
        </div>
      </div>

      {/* Mobile Navigation Menu - Outside main container */}
      {mobileMenuOpen && (
        <div className="absolute top-full left-0 right-0 md:hidden bg-white shadow-lg border-t border-gray-200">
          <div className="px-3 py-3 space-y-1">
              {navigation.map((item) => (
                <Link
                  key={item.name}
                  to={item.href}
                  onClick={() => setMobileMenuOpen(false)}
                  className={clsx(
                    'flex items-center px-3 py-2 text-sm font-medium rounded-md transition-colors',
                    item.current
                      ? 'bg-blue-100 text-blue-700'
                      : 'text-gray-700 hover:bg-gray-100'
                  )}
                >
                  <item.icon className="w-5 h-5 mr-3" />
                  {item.name}
                </Link>
              ))}
            </div>

            {isAdmin && (
              <>
                <div className="border-t border-gray-200 px-3 py-3">
                  <div className="px-3 py-1 text-xs font-semibold text-gray-500 uppercase tracking-wide">
                    Administration
                  </div>
                </div>
                <div className="px-3 pb-3 space-y-1">
                  {adminNavigation.map((item) => {
                    const isActive = location.pathname === item.href;
                    return (
                      <Link
                        key={item.name}
                        to={item.href}
                        onClick={() => setMobileMenuOpen(false)}
                        className={clsx(
                          'flex items-center px-3 py-2 text-sm font-medium rounded-md transition-colors',
                          isActive
                            ? 'bg-blue-100 text-blue-700'
                            : 'text-gray-700 hover:bg-gray-100'
                        )}
                      >
                        <item.icon className="w-5 h-5 mr-3" />
                        {item.name}
                      </Link>
                    );
                  })}
                </div>
              </>
            )}
        </div>
      )}
    </nav>
  );
};
