import { useEffect, useRef, useState } from 'react';
import type { CreateProfileRequest, UpdateProfileRequest, XtreamProfile } from '../stores/profileStore';
import { useProfileStore } from '../stores/profileStore';

interface ProfileFormData {
  name: string;
  url: string;
  username: string;
  password: string;
}

const initialFormData: ProfileFormData = {
  name: '',
  url: '',
  username: '',
  password: ''
};

export default function ProfileSelector() {
  const {
    profiles,
    activeProfile,
    isAuthenticating,
    isSwitching,
    isSaving,
    isDeleting,
    isValidating,
    authError,
    error,
    validationError,
    switchProfile,
    fetchProfiles,
    createProfile,
    updateProfile,
    deleteProfile,
    clearAuthError,
    clearError,
    clearValidationError
  } = useProfileStore();

  const [isDropdownOpen, setIsDropdownOpen] = useState(false);
  const [isFormOpen, setIsFormOpen] = useState(false);
  const [editingProfile, setEditingProfile] = useState<XtreamProfile | null>(null);
  const [formData, setFormData] = useState<ProfileFormData>(initialFormData);
  const [showDeleteConfirm, setShowDeleteConfirm] = useState<string | null>(null);
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

  const handleOpenForm = (profile?: XtreamProfile) => {
    if (profile) {
      setEditingProfile(profile);
      setFormData({
        name: profile.name,
        url: profile.url,
        username: profile.username,
        password: ''
      });
    } else {
      setEditingProfile(null);
      setFormData(initialFormData);
    }
    setIsFormOpen(true);
    setIsDropdownOpen(false);
    clearError();
    clearValidationError();
  };

  const handleCloseForm = () => {
    setIsFormOpen(false);
    setEditingProfile(null);
    setFormData(initialFormData);
    clearError();
    clearValidationError();
  };

  const handleInputChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    const { name, value } = e.target;
    setFormData(prev => ({
      ...prev,
      [name]: value
    }));
    if (validationError) {
      clearValidationError();
    }
  };

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();

    if (!formData.name.trim() || !formData.url.trim() || !formData.username.trim()) {
      return;
    }

    try {
      if (editingProfile) {
        const updateData: UpdateProfileRequest = {
          name: formData.name.trim(),
          url: formData.url.trim(),
          username: formData.username.trim()
        };

        if (formData.password.trim()) {
          updateData.password = formData.password.trim();
        }

        await updateProfile(editingProfile.id, updateData);
      } else {
        if (!formData.password.trim()) {
          return;
        }

        const createData: CreateProfileRequest = {
          name: formData.name.trim(),
          url: formData.url.trim(),
          username: formData.username.trim(),
          password: formData.password.trim()
        };

        await createProfile(createData);
      }

      handleCloseForm();
    } catch (error) {
      console.error('Failed to save profile:', error);
    }
  };

  const handleDelete = async (profileId: string) => {
    try {
      await deleteProfile(profileId);
      setShowDeleteConfirm(null);
    } catch (error) {
      console.error('Failed to delete profile:', error);
    }
  };

  const handleEditClick = (e: React.MouseEvent, profile: XtreamProfile) => {
    e.stopPropagation();
    handleOpenForm(profile);
  };

  const handleDeleteClick = (e: React.MouseEvent, profileId: string) => {
    e.stopPropagation();
    setShowDeleteConfirm(profileId);
    setIsDropdownOpen(false);
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
                  Click "Add Profile" below to get started
                </p>
              </div>
            ) : (
              profiles.map(profile => (
                <div
                  key={profile.id}
                  className={`dropdown-item ${profile.is_active ? 'active' : ''}`}
                >
                  <button
                    className="dropdown-item-button"
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

                  <div className="dropdown-item-actions">
                    <button
                      className="dropdown-action-btn edit"
                      onClick={(e) => handleEditClick(e, profile)}
                      title="Edit profile"
                      disabled={isSaving || isDeleting}
                    >
                      ✎
                    </button>
                    <button
                      className="dropdown-action-btn delete"
                      onClick={(e) => handleDeleteClick(e, profile.id)}
                      title="Delete profile"
                      disabled={isSaving || isDeleting}
                    >
                      🗑
                    </button>
                  </div>
                </div>
              ))
            )}
          </div>

          <div className="dropdown-footer">
            <button
              className="dropdown-footer-button"
              onClick={() => handleOpenForm()}
              disabled={isSaving}
            >
              + Add Profile
            </button>
          </div>
        </div>
      )}

      {/* Profile Form Modal */}
      {isFormOpen && (
        <div className="modal-overlay">
          <div className="modal-content">
            <div className="modal-header">
              <h3>{editingProfile ? 'Edit Profile' : 'Add New Profile'}</h3>
              <button
                className="modal-close"
                onClick={handleCloseForm}
                disabled={isSaving}
              >
                ×
              </button>
            </div>

            <form onSubmit={handleSubmit} className="profile-form">
              {validationError && (
                <div className="error-message">
                  <span>{validationError}</span>
                </div>
              )}

              {error && (
                <div className="error-message">
                  <span>{error}</span>
                  <button onClick={clearError} className="error-close">×</button>
                </div>
              )}

              <div className="form-group">
                <label htmlFor="name">Profile Name *</label>
                <input
                  type="text"
                  id="name"
                  name="name"
                  value={formData.name}
                  onChange={handleInputChange}
                  placeholder="Enter a name for this profile"
                  required
                  disabled={isSaving}
                />
              </div>

              <div className="form-group">
                <label htmlFor="url">Server URL *</label>
                <input
                  type="url"
                  id="url"
                  name="url"
                  value={formData.url}
                  onChange={handleInputChange}
                  placeholder="http://your-server.com:80"
                  required
                  disabled={isSaving}
                />
              </div>

              <div className="form-group">
                <label htmlFor="username">Username *</label>
                <input
                  type="text"
                  id="username"
                  name="username"
                  value={formData.username}
                  onChange={handleInputChange}
                  placeholder="Your username"
                  required
                  disabled={isSaving}
                />
              </div>

              <div className="form-group">
                <label htmlFor="password">
                  Password {editingProfile ? '(leave blank to keep current)' : '*'}
                </label>
                <input
                  type="password"
                  id="password"
                  name="password"
                  value={formData.password}
                  onChange={handleInputChange}
                  placeholder="Your password"
                  required={!editingProfile}
                  disabled={isSaving}
                />
              </div>

              <div className="form-actions">
                <button
                  type="button"
                  className="btn btn-outline"
                  onClick={handleCloseForm}
                  disabled={isSaving}
                >
                  Cancel
                </button>
                <button
                  type="submit"
                  className="btn btn-primary"
                  disabled={isSaving || isValidating}
                >
                  {isSaving || isValidating ? (
                    <>
                      <span className="loading-spinner-small"></span>
                      {isValidating ? 'Validating...' : 'Saving...'}
                    </>
                  ) : (
                    editingProfile ? 'Update Profile' : 'Create Profile'
                  )}
                </button>
              </div>
            </form>
          </div>
        </div>
      )}

      {/* Delete Confirmation Modal */}
      {showDeleteConfirm && (
        <div className="modal-overlay">
          <div className="modal-content modal-small">
            <div className="modal-header">
              <h3>Confirm Delete</h3>
            </div>
            <div className="modal-body">
              <p>Are you sure you want to delete this profile?</p>
              <p><strong>{profiles.find(p => p.id === showDeleteConfirm)?.name}</strong></p>
              <p>This action cannot be undone.</p>
            </div>
            <div className="form-actions">
              <button
                className="btn btn-outline"
                onClick={() => setShowDeleteConfirm(null)}
                disabled={isDeleting}
              >
                Cancel
              </button>
              <button
                className="btn btn-danger"
                onClick={() => handleDelete(showDeleteConfirm)}
                disabled={isDeleting}
              >
                {isDeleting ? (
                  <>
                    <span className="loading-spinner-small"></span>
                    Deleting...
                  </>
                ) : (
                  'Delete Profile'
                )}
              </button>
            </div>
          </div>
        </div>
      )}
    </div>
  );
}