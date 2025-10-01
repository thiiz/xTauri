import { useEffect, useRef, useState } from 'react';
import { useProfileStore } from '../stores/profileStore';

export default function ProfileSelector() {
  const {
    profiles,
    activeProfile,
    isAuthenticating,
    isSwitching,
    authError,
    switchProfile,
    fetchProfiles,
    clearAuthError
  } = useProfileStore();

  const [isDropdownOpen, setIsDropdownOpen] = useState(false);
  const dropdownRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    fetchProfiles();
  }, [fetchProfiles]);

  // Close dropdown when clicking outside
  useEffect(() => {
    const handleClickOutside = (event: MouseEvent) => {
      if (dropdownRef.current && !dropdownRef.current.contains(event.target as Node)) {
        setIsDropdownOpen(false);
      }
    };

    document.addEventListener('mousedown', handleClickOutside);
    return () => {
      document.removeEventListener('mousedown', handleClickOutside);
    };
  }, []);

  const handleProfileSelect = async (profileId: string) => {
    if (profileId === activeProfile?.id) {
      setIsDropdownOpen(false);
      return;
    }

    try {
      await switchProfile(profileId);
      setIsDropdownOpen(false);
    } catch (error) {
      console.error('Failed to switch profile:', error);
      // Error is handled by the store
    }
  };

  const toggleDropdown = () => {
    setIsDropdownOpen(!isDropdownOpen);
    if (authError) {
      clearAuthError();
    }
  };

  const getStatusIcon = () => {
    if (isAuthenticating || isSwitching) {
      return <span className="status-icon loading"></span>;
    }

    if (authError) {
      return <span className="status-icon error" title={authError}></span>;
    }

    if (activeProfile) {
      return <span className="status-icon connected"></span>;
    }

    return <span className="status-icon disconnected"></span>;
  };

  const getStatusText = () => {
    if (isAuthenticating) {
      return 'Authenticating...';
    }

    if (isSwitching) {
      return 'Switching profile...';
    }

    if (authError) {
      return 'Authentication failed';
    }

    if (activeProfile) {
      return activeProfile.name;
    }

    return 'No profile selected';
  };

  const formatLastUsed = (dateString: string | null) => {
    if (!dateString) return 'Never used';

    const date = new Date(dateString);
    const now = new Date();
    const diffMs = now.getTime() - date.getTime();
    const diffDays = Math.floor(diffMs / (1000 * 60 * 60 * 24));

    if (diffDays === 0) {
      return 'Today';
    } else if (diffDays === 1) {
      return 'Yesterday';
    } else if (diffDays < 7) {
      return `${diffDays} days ago`;
    } else {
      return date.toLocaleDateString();
    }
  };

  return (
    <div className="profile-selector" ref={dropdownRef}>
      <button
        className={`profile-selector-button ${isDropdownOpen ? 'open' : ''} ${authError ? 'error' : ''}`}
        onClick={toggleDropdown}
        disabled={isAuthenticating || isSwitching}
        title={authError || getStatusText()}
      >
        <div className="profile-selector-content">
          {getStatusIcon()}
          <div className="profile-selector-text">
            <span className="profile-name">{getStatusText()}</span>
            {activeProfile && !isAuthenticating && !isSwitching && (
              <span className="profile-url">{activeProfile.url}</span>
            )}
          </div>
          <span className={`dropdown-arrow ${isDropdownOpen ? 'up' : 'down'}`}>
            ▼
          </span>
        </div>
      </button>

      {authError && (
        <div className="profile-selector-error">
          <span className="error-icon">⚠</span>
          <span className="error-text">{authError}</span>
          <button
            className="error-close"
            onClick={clearAuthError}
            title="Dismiss error"
          >
            ×
          </button>
        </div>
      )}

      {isDropdownOpen && (
        <div className="profile-selector-dropdown">
          <div className="dropdown-header">
            <span>Select Profile</span>
          </div>

          <div className="dropdown-content">
            {profiles.length === 0 ? (
              <div className="dropdown-empty">
                <p>No profiles available</p>
                <p className="dropdown-empty-hint">
                  Create a profile in Settings to get started
                </p>
              </div>
            ) : (
              profiles.map(profile => (
                <button
                  key={profile.id}
                  className={`dropdown-item ${profile.is_active ? 'active' : ''}`}
                  onClick={() => handleProfileSelect(profile.id)}
                  disabled={isAuthenticating || isSwitching}
                >
                  <div className="dropdown-item-content">
                    <div className="dropdown-item-main">
                      <span className="dropdown-item-name">
                        {profile.name}
                        {profile.is_active && (
                          <span className="active-indicator">●</span>
                        )}
                      </span>
                      <span className="dropdown-item-url">{profile.url}</span>
                    </div>
                    <div className="dropdown-item-meta">
                      <span className="dropdown-item-username">
                        {profile.username}
                      </span>
                      <span className="dropdown-item-last-used">
                        {formatLastUsed(profile.last_used)}
                      </span>
                    </div>
                  </div>

                  {profile.is_active && (
                    <span className="dropdown-item-check">✓</span>
                  )}
                </button>
              ))
            )}
          </div>

          <div className="dropdown-footer">
            <div className="dropdown-footer-text">
              Manage profiles in Settings
            </div>
          </div>
        </div>
      )}
    </div>
  );
}