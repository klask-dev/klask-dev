import React, { useState } from 'react';
import ErrorBoundary from '../../components/ErrorBoundary';
import { useProfile } from '../../hooks/useProfile';
import ProfileHeader from './components/ProfileHeader';
import ProfileInformation from './components/ProfileInformation';
import PreferencesSection from './components/PreferencesSection';
import SecuritySection from './components/SecuritySection';
import ActivitySection from './components/ActivitySection';
import DeleteAccountModal from './components/DeleteAccountModal';

type TabType = 'info' | 'prefs' | 'security' | 'activity';

interface TabConfig {
  id: TabType;
  label: string;
  icon: React.ReactNode;
}

const ProfilePage: React.FC = () => {
  const { user } = useProfile();
  const [activeTab, setActiveTab] = useState<TabType>('info');
  const [deleteModalOpen, setDeleteModalOpen] = useState(false);

  const tabs: TabConfig[] = [
    {
      id: 'info',
      label: 'Information',
      icon: (
        <svg className="w-5 h-5" fill="currentColor" viewBox="0 0 20 20">
          <path
            fillRule="evenodd"
            d="M10 9a3 3 0 100-6 3 3 0 000 6zm-7 9a7 7 0 1114 0H3z"
            clipRule="evenodd"
          />
        </svg>
      ),
    },
    {
      id: 'prefs',
      label: 'Preferences',
      icon: (
        <svg className="w-5 h-5" fill="currentColor" viewBox="0 0 20 20">
          <path d="M10.5 1.5H5.75A2.25 2.25 0 003.5 3.75v12.5A2.25 2.25 0 005.75 18.5h8.5a2.25 2.25 0 002.25-2.25V9.5m-12-4h12m-12 4v8m12-8v2" />
        </svg>
      ),
    },
    {
      id: 'security',
      label: 'Security',
      icon: (
        <svg className="w-5 h-5" fill="currentColor" viewBox="0 0 20 20">
          <path
            fillRule="evenodd"
            d="M5 9V7a5 5 0 0110 0v2a2 2 0 012 2v5a2 2 0 01-2 2H5a2 2 0 01-2-2v-5a2 2 0 012-2zm8-2v2H7V7a3 3 0 016 0z"
            clipRule="evenodd"
          />
        </svg>
      ),
    },
    {
      id: 'activity',
      label: 'Activity',
      icon: (
        <svg className="w-5 h-5" fill="currentColor" viewBox="0 0 20 20">
          <path d="M2 11a1 1 0 011-1h2a1 1 0 011 1v5a1 1 0 01-1 1H3a1 1 0 01-1-1v-5zM8 7a1 1 0 011-1h2a1 1 0 011 1v9a1 1 0 01-1 1H9a1 1 0 01-1-1V7zM14 4a1 1 0 011-1h2a1 1 0 011 1v12a1 1 0 01-1 1h-2a1 1 0 01-1-1V4z" />
        </svg>
      ),
    },
  ];

  if (!user) {
    return (
      <div className="max-w-4xl mx-auto py-8 px-4">
        <div className="bg-blue-50 border border-blue-200 rounded-lg p-6 text-center">
          <p className="text-blue-900">Please log in to view your profile</p>
        </div>
      </div>
    );
  }

  return (
    <ErrorBoundary onError={(error) => console.error('ProfilePage error:', error)}>
      <div className="max-w-4xl mx-auto py-8 px-4 space-y-8">
        {/* Page Header */}
        <div>
          <h1 className="text-3xl font-bold text-gray-900">Profile Settings</h1>
          <p className="text-gray-600 mt-1">Manage your account information and preferences</p>
        </div>

        {/* Profile Header */}
        <ProfileHeader />

        {/* Tabs */}
        <div className="bg-white rounded-lg shadow-sm border border-gray-200">
          {/* Tab Navigation */}
          <div className="border-b border-gray-200">
            <div className="flex flex-wrap md:flex-nowrap">
              {tabs.map((tab) => (
                <button
                  key={tab.id}
                  onClick={() => setActiveTab(tab.id)}
                  className={`flex-1 px-4 py-4 md:px-6 font-medium text-sm md:text-base flex items-center justify-center gap-2 border-b-2 transition ${
                    activeTab === tab.id
                      ? 'border-blue-600 text-blue-600 bg-blue-50'
                      : 'border-transparent text-gray-600 hover:text-gray-900 hover:bg-gray-50'
                  }`}
                >
                  {tab.icon}
                  <span className="hidden sm:inline">{tab.label}</span>
                </button>
              ))}
            </div>
          </div>

          {/* Tab Content */}
          <div className="p-6">
            {activeTab === 'info' && <ProfileInformation />}
            {activeTab === 'prefs' && <PreferencesSection />}
            {activeTab === 'security' && <SecuritySection />}
            {activeTab === 'activity' && <ActivitySection />}
          </div>
        </div>

        {/* Danger Zone */}
        <div className="bg-red-50 border-2 border-red-200 rounded-lg p-6">
          <div className="flex flex-col sm:flex-row items-start sm:items-center justify-between gap-4">
            <div>
              <h3 className="text-lg font-bold text-red-900">Danger Zone</h3>
              <p className="text-sm text-red-700 mt-1">
                Permanently delete your account and all associated data. This action cannot be undone.
              </p>
            </div>
            <button
              onClick={() => setDeleteModalOpen(true)}
              className="px-6 py-2 bg-red-600 text-white rounded-lg hover:bg-red-700 transition font-medium flex-shrink-0"
            >
              Delete Account
            </button>
          </div>
        </div>

        {/* Delete Modal */}
        <DeleteAccountModal isOpen={deleteModalOpen} onClose={() => setDeleteModalOpen(false)} />
      </div>
    </ErrorBoundary>
  );
};

export default ProfilePage;