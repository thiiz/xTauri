import { invoke } from '@tauri-apps/api/core';
import { create } from 'zustand';
import { createJSONStorage, persist } from 'zustand/middleware';

// Profile types based on the design document
export interface XtreamProfile {
  id: string;
  name: string;
  url: string;
  username: string;
  // password is stored encrypted and not exposed in this interface
  created_at: string;
  updated_at: string;
  last_used: string | null;
  is_active: boolean;
}

export interface CreateProfileRequest {
  name: string;
  url: string;
  username: string;
  password: string;
}

export interface UpdateProfileRequest {
  name?: string;
  url?: string;
  username?: string;
  password?: string;
}

export interface ProfileCredentials {
  url: string;
  username: string;
  password: string;
}

export interface ProfileValidationResult {
  success: boolean;
  error_message?: string;
  error_type: string;
  server_info?: any;
}

interface ProfileState {
  // Profile data
  profiles: XtreamProfile[];
  activeProfile: XtreamProfile | null;
  previousActiveProfileId: string | null;

  // Loading states
  isLoading: boolean;
  isAuthenticating: boolean;
  isValidating: boolean;
  isSaving: boolean;
  isDeleting: boolean;
  isSwitching: boolean;

  // Error states
  error: string | null;
  authError: string | null;
  validationError: string | null;

  // Authentication retry state
  authRetryCount: number;
  maxAuthRetries: number;
  authRetryDelay: number;

  // Profile CRUD actions
  createProfile: (profile: CreateProfileRequest) => Promise<string>;
  updateProfile: (id: string, profile: UpdateProfileRequest) => Promise<void>;
  deleteProfile: (id: string) => Promise<void>;
  fetchProfiles: () => Promise<void>;
  getProfile: (id: string) => Promise<XtreamProfile | null>;

  // Profile validation
  validateProfile: (credentials: ProfileCredentials) => Promise<ProfileValidationResult>;

  // Profile activation and authentication
  setActiveProfile: (id: string) => Promise<void>;
  switchProfile: (id: string) => Promise<void>;
  authenticateProfile: (id: string) => Promise<void>;
  authenticateWithRetry: (id: string) => Promise<void>;
  clearActiveProfile: () => void;

  // State cleanup
  cleanupProfileState: () => void;

  // Error handling
  clearError: () => void;
  clearAuthError: () => void;
  clearValidationError: () => void;
  resetAuthRetry: () => void;

  // Utility actions
  refreshActiveProfile: () => Promise<void>;
  getActiveProfileCredentials: () => Promise<ProfileCredentials | null>;
  restorePreviousProfile: () => Promise<void>;
}

export const useProfileStore = create<ProfileState>()(
  persist(
    (set, get) => ({
      // Initial state
      profiles: [],
      activeProfile: null,
      previousActiveProfileId: null,

      // Loading states
      isLoading: false,
      isAuthenticating: false,
      isValidating: false,
      isSaving: false,
      isDeleting: false,
      isSwitching: false,

      // Error states
      error: null,
      authError: null,
      validationError: null,

      // Authentication retry state
      authRetryCount: 0,
      maxAuthRetries: 3,
      authRetryDelay: 1000, // 1 second base delay

      // Profile CRUD actions
      createProfile: async (profile: CreateProfileRequest) => {
        set({ isSaving: true, error: null, validationError: null });

        try {
          // First validate the profile credentials
          const validationResult = await get().validateProfile({
            url: profile.url,
            username: profile.username,
            password: profile.password
          });

          if (!validationResult.success) {
            throw new Error(validationResult.error_message || 'Profile validation failed');
          }

          // Create the profile if validation succeeds
          const profileId = await invoke<string>('create_xtream_profile', { request: profile });

          // Refresh the profiles list
          await get().fetchProfiles();

          set({ isSaving: false });
          return profileId;
        } catch (error) {
          const errorMessage = error instanceof Error ? error.message : String(error);
          set({
            error: errorMessage,
            validationError: errorMessage,
            isSaving: false
          });
          throw error;
        }
      },

      updateProfile: async (id: string, profile: UpdateProfileRequest) => {
        set({ isSaving: true, error: null, validationError: null });

        try {
          // If credentials are being updated, validate them first
          if (profile.url || profile.username || profile.password) {
            const currentProfile = get().profiles.find(p => p.id === id);
            if (!currentProfile) {
              throw new Error('Profile not found');
            }

            const credentialsToValidate: ProfileCredentials = {
              url: profile.url || currentProfile.url,
              username: profile.username || currentProfile.username,
              password: profile.password || '' // Will need to get from backend if not provided
            };

            // Only validate if we have a password (either new or existing)
            if (profile.password) {
              const validationResult = await get().validateProfile(credentialsToValidate);

              if (!validationResult.success) {
                throw new Error(validationResult.error_message || 'Profile validation failed');
              }
            }
          }

          await invoke('update_xtream_profile', { id, request: profile });

          // Refresh the profiles list
          await get().fetchProfiles();

          // If the updated profile is the active one, refresh it
          const { activeProfile } = get();
          if (activeProfile && activeProfile.id === id) {
            await get().refreshActiveProfile();
          }

          set({ isSaving: false });
        } catch (error) {
          const errorMessage = error instanceof Error ? error.message : String(error);
          set({
            error: errorMessage,
            validationError: errorMessage,
            isSaving: false
          });
          throw error;
        }
      },

      deleteProfile: async (id: string) => {
        set({ isDeleting: true, error: null });

        try {
          await invoke('delete_xtream_profile', { id });

          // Clear active profile if it was the deleted one
          const { activeProfile } = get();
          if (activeProfile && activeProfile.id === id) {
            set({ activeProfile: null });
          }

          // Refresh the profiles list
          await get().fetchProfiles();

          set({ isDeleting: false });
        } catch (error) {
          const errorMessage = error instanceof Error ? error.message : String(error);
          set({
            error: errorMessage,
            isDeleting: false
          });
          throw error;
        }
      },

      fetchProfiles: async () => {
        set({ isLoading: true, error: null });

        try {
          const profiles = await invoke<XtreamProfile[]>('get_xtream_profiles');
          set({ profiles, isLoading: false });
        } catch (error) {
          const errorMessage = error instanceof Error ? error.message : String(error);
          set({
            error: errorMessage,
            isLoading: false,
            profiles: []
          });
        }
      },

      getProfile: async (id: string) => {
        try {
          const profile = await invoke<XtreamProfile | null>('get_xtream_profile', { id });
          return profile;
        } catch (error) {
          const errorMessage = error instanceof Error ? error.message : String(error);
          set({ error: errorMessage });
          return null;
        }
      },

      // Profile validation
      validateProfile: async (credentials: ProfileCredentials) => {
        set({ isValidating: true, validationError: null });

        try {
          const result = await invoke<ProfileValidationResult>('validate_xtream_credentials', {
            credentials
          });

          set({ isValidating: false });

          if (!result.success && result.error_message) {
            set({ validationError: result.error_message });
          }

          return result;
        } catch (error) {
          const errorMessage = error instanceof Error ? error.message : String(error);
          set({
            validationError: errorMessage,
            isValidating: false
          });

          return {
            success: false,
            error_message: errorMessage,
            error_type: 'ValidationError'
          };
        }
      },

      // Profile activation and authentication
      setActiveProfile: async (id: string) => {
        set({ isAuthenticating: true, authError: null });

        try {
          // Get the profile first
          const profile = await get().getProfile(id);
          if (!profile) {
            throw new Error('Profile not found');
          }

          // Authenticate with the profile using retry logic
          await get().authenticateWithRetry(id);

          // Set as active profile
          set({
            activeProfile: profile,
            isAuthenticating: false
          });

        } catch (error) {
          const errorMessage = error instanceof Error ? error.message : String(error);
          set({
            authError: errorMessage,
            isAuthenticating: false
          });
          throw error;
        }
      },

      authenticateProfile: async (id: string) => {
        try {
          await invoke('authenticate_xtream_profile', { profileId: id });
        } catch (error) {
          const errorMessage = error instanceof Error ? error.message : String(error);
          set({ authError: errorMessage });
          throw error;
        }
      },

      clearActiveProfile: () => {
        set({
          activeProfile: null,
          authError: null
        });
      },

      // Enhanced profile switching with state cleanup
      switchProfile: async (id: string) => {
        const { activeProfile } = get();

        set({
          isSwitching: true,
          authError: null,
          previousActiveProfileId: activeProfile?.id || null
        });

        try {
          // Clean up current profile state
          get().cleanupProfileState();

          // Set the new active profile
          await get().setActiveProfile(id);

          set({ isSwitching: false });
        } catch (error) {
          const errorMessage = error instanceof Error ? error.message : String(error);
          set({
            authError: errorMessage,
            isSwitching: false
          });

          // Try to restore previous profile on failure
          const { previousActiveProfileId } = get();
          if (previousActiveProfileId) {
            try {
              await get().restorePreviousProfile();
            } catch (restoreError) {
              console.error('Failed to restore previous profile:', restoreError);
            }
          }

          throw error;
        }
      },

      // Authentication with exponential backoff retry
      authenticateWithRetry: async (id: string) => {
        const { authRetryCount, maxAuthRetries, authRetryDelay } = get();

        try {
          await get().authenticateProfile(id);
          get().resetAuthRetry();
        } catch (error) {
          if (authRetryCount < maxAuthRetries) {
            const delay = authRetryDelay * Math.pow(2, authRetryCount);

            set({ authRetryCount: authRetryCount + 1 });

            // Wait before retrying
            await new Promise(resolve => setTimeout(resolve, delay));

            // Retry authentication
            return get().authenticateWithRetry(id);
          } else {
            // Max retries reached, reset and throw error
            get().resetAuthRetry();
            throw error;
          }
        }
      },

      // State cleanup when switching profiles
      cleanupProfileState: () => {
        // Clear any cached content from the previous profile
        set({
          authError: null,
          error: null,
          validationError: null
        });

        // Note: In a real implementation, this would also clear
        // content from XtreamContentStore and other related stores
        // For now, we'll add a comment about this integration point

        // TODO: Integrate with XtreamContentStore to clear content:
        // const xtreamStore = useXtreamContentStore.getState();
        // xtreamStore.clearAll();
      },

      // Error handling
      clearError: () => {
        set({ error: null });
      },

      clearAuthError: () => {
        set({ authError: null });
      },

      clearValidationError: () => {
        set({ validationError: null });
      },

      resetAuthRetry: () => {
        set({ authRetryCount: 0 });
      },

      // Utility actions
      refreshActiveProfile: async () => {
        const { activeProfile } = get();
        if (!activeProfile) return;

        try {
          const updatedProfile = await get().getProfile(activeProfile.id);
          if (updatedProfile) {
            set({ activeProfile: updatedProfile });
          }
        } catch (error) {
          console.error('Failed to refresh active profile:', error);
        }
      },

      restorePreviousProfile: async () => {
        const { previousActiveProfileId } = get();
        if (!previousActiveProfileId) {
          throw new Error('No previous profile to restore');
        }

        try {
          await get().setActiveProfile(previousActiveProfileId);
          set({ previousActiveProfileId: null });
        } catch (error) {
          console.error('Failed to restore previous profile:', error);
          throw error;
        }
      },

      getActiveProfileCredentials: async () => {
        const { activeProfile } = get();
        if (!activeProfile) return null;

        // TODO: Implement get_xtream_profile_credentials command in backend
        console.warn('getActiveProfileCredentials not implemented - missing backend command');
        return null;
      },
    }),
    {
      name: 'profile-storage',
      storage: createJSONStorage(() => localStorage),
      partialize: (state) => ({
        profiles: state.profiles,
        activeProfile: state.activeProfile,
        previousActiveProfileId: state.previousActiveProfileId,
      }),
      skipHydration: false,
    }
  )
);