import { useEffect, useState } from 'react';
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

export default function ProfileManager() {
  const {
    profiles,
    isLoading,
    isSaving,
    isDeleting,
    isValidating,
    error,
    validationError,
    createProfile,
    updateProfile,
    deleteProfile,
    fetchProfiles,
    setActiveProfile,
    clearError,
    clearValidationError
  } = useProfileStore();

  const [isFormOpen, setIsFormOpen] = useState(false);
  const [editingProfile, setEditingProfile] = useState<XtreamProfile | null>(null);
  const [formData, setFormData] = useState<ProfileFormData>(initialFormData);
  const [showDeleteConfirm, setShowDeleteConfirm] = useState<string | null>(null);

  useEffect(() => {
    fetchProfiles();
  }, [fetchProfiles]);

  const handleOpenForm = (profile?: XtreamProfile) => {
    if (profile) {
      setEditingProfile(profile);
      setFormData({
        name: profile.name,
        url: profile.url,
        username: profile.username,
        password: '' // Don't populate password for security
      });
    } else {
      setEditingProfile(null);
      setFormData(initialFormData);
    }
    setIsFormOpen(true);
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
    // Clear validation errors when user starts typing
    if (validationError) {
      clearValidationError();
    }
  };

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();

    // Basic validation
    if (!formData.name.trim() || !formData.url.trim() || !formData.username.trim()) {
      return;
    }

    try {
      if (editingProfile) {
        // Update existing profile
        const updateData: UpdateProfileRequest = {
          name: formData.name.trim(),
          url: formData.url.trim(),
          username: formData.username.trim()
        };

        // Only include password if it was changed
        if (formData.password.trim()) {
          updateData.password = formData.password.trim();
        }

        await updateProfile(editingProfile.id, updateData);
      } else {
        // Create new profile
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
      // Error is handled by the store
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

  const handleActivate = async (profileId: string) => {
    try {
      await setActiveProfile(profileId);
    } catch (error) {
      console.error('Failed to activate profile:', error);
    }
  };

  const formatDate = (dateString: string) => {
    return new Date(dateString).toLocaleDateString();
  };

  return (
    <div className="profile-manager">
      <div className="profile-manager-header">
        <h2>Xtream Profiles</h2>
        <button
          className="btn btn-primary"
          onClick={() => handleOpenForm()}
          disabled={isSaving}
        >
          Add Profile
        </button>
      </div>

      {error && (
        <div className="error-message">
          <span>{error}</span>
          <button onClick={clearError} className="error-close">×</button>
        </div>
      )}

      {isLoading ? (
        <div className="loading-container">
          <div className="loading-spinner"></div>
          <p>Loading profiles...</p>
        </div>
      ) : (
        <div className="profile-list">
          {profiles.length === 0 ? (
            <div className="empty-state">
              <p>No profiles configured yet.</p>
              <p>Add your first Xtream Codes profile to get started.</p>
            </div>
          ) : (
            profiles.map(profile => (
              <div
                key={profile.id}
                className={`profile-item ${profile.is_active ? 'active' : ''}`}
              >
                <div className="profile-info">
                  <div className="profile-name">
                    {profile.name}
                    {profile.is_active && <span className="active-badge">Active</span>}
                  </div>
                  <div className="profile-details">
                    <span className="profile-url">{profile.url}</span>
                    <span className="profile-username">User: {profile.username}</span>
                  </div>
                  <div className="profile-meta">
                    <span>Created: {formatDate(profile.created_at)}</span>
                    {profile.last_used && (
                      <span>Last used: {formatDate(profile.last_used)}</span>
                    )}
                  </div>
                </div>

                <div className="profile-actions">
                  {!profile.is_active && (
                    <button
                      className="btn btn-secondary"
                      onClick={() => handleActivate(profile.id)}
                      disabled={isSaving}
                    >
                      Activate
                    </button>
                  )}

                  <button
                    className="btn btn-outline"
                    onClick={() => handleOpenForm(profile)}
                    disabled={isSaving || isDeleting}
                  >
                    Edit
                  </button>

                  <button
                    className="btn btn-danger"
                    onClick={() => setShowDeleteConfirm(profile.id)}
                    disabled={isSaving || isDeleting}
                  >
                    Delete
                  </button>
                </div>
              </div>
            ))
          )}
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
                  placeholder="http://your-server.com:8080"
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