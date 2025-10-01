import { invoke } from '@tauri-apps/api/core';
import { useCallback, useEffect, useState } from 'react';
import type { Channel } from '../components/ChannelList';
import { useProfileStore } from '../stores/profileStore';
import type { XtreamChannel, XtreamMoviesListing, XtreamShow } from '../types/types';

export interface ContentItem {
  type: 'channel' | 'xtream-channel' | 'xtream-movie' | 'xtream-series';
  data: Channel | XtreamChannel | XtreamMoviesListing | XtreamShow;
  url?: string;
  metadata?: {
    title?: string;
    description?: string;
    duration?: number;
    genre?: string;
    rating?: number;
    year?: string;
    cast?: string;
    director?: string;
    episodeId?: string;
    seasonNumber?: number;
    episodeNumber?: number;
  };
}

export interface PlaybackHistory {
  id: string;
  profileId: string;
  contentType: string;
  contentId: string;
  contentData: any;
  watchedAt: string;
  position?: number; // Playback position in seconds
  duration?: number; // Total duration in seconds
}

export interface PlaybackState {
  currentContent: ContentItem | null;
  isPlaying: boolean;
  position: number;
  duration: number;
  volume: number;
  muted: boolean;
  fullscreen: boolean;
}

export const useContentPlayback = () => {
  const { activeProfile } = useProfileStore();
  const [playbackState, setPlaybackState] = useState<PlaybackState>({
    currentContent: null,
    isPlaying: false,
    position: 0,
    duration: 0,
    volume: 1,
    muted: false,
    fullscreen: false,
  });

  const [playbackHistory, setPlaybackHistory] = useState<PlaybackHistory[]>([]);
  const [isLoadingHistory, setIsLoadingHistory] = useState(false);

  // Load playback history for the active profile
  useEffect(() => {
    const loadPlaybackHistory = async () => {
      if (!activeProfile) {
        setPlaybackHistory([]);
        return;
      }

      setIsLoadingHistory(true);
      try {
        const history = await invoke<PlaybackHistory[]>('get_xtream_playback_history', {
          profileId: activeProfile.id
        });
        setPlaybackHistory(history);
      } catch (error) {
        console.error('Failed to load playback history:', error);
        setPlaybackHistory([]);
      } finally {
        setIsLoadingHistory(false);
      }
    };

    loadPlaybackHistory();
  }, [activeProfile]);

  // Play content
  const playContent = useCallback(async (content: ContentItem) => {
    if (!activeProfile) {
      throw new Error('No active profile selected');
    }

    try {
      // Generate stream URL if not already present
      let streamUrl = content.url;
      if (!streamUrl && content.type.startsWith('xtream-')) {
        const contentId = getContentId(content);
        const contentType = getXtreamContentType(content.type);

        if (contentId && contentType) {
          streamUrl = await invoke<string>('generate_xtream_stream_url', {
            profileId: activeProfile.id,
            contentType,
            contentId,
            extension: getDefaultExtension(content.type)
          });
        }
      }

      if (!streamUrl) {
        throw new Error('Unable to generate stream URL for content');
      }

      // Update playback state
      setPlaybackState(prev => ({
        ...prev,
        currentContent: { ...content, url: streamUrl },
        isPlaying: true,
        position: 0,
      }));

      // Add to playback history
      await addToPlaybackHistory(content);

    } catch (error) {
      console.error('Failed to play content:', error);
      throw error;
    }
  }, [activeProfile]);

  // Add content to playback history
  const addToPlaybackHistory = useCallback(async (content: ContentItem) => {
    if (!activeProfile) return;

    try {
      const contentId = getContentId(content);
      if (!contentId) return;

      const historyItem = {
        profileId: activeProfile.id,
        contentType: content.type,
        contentId,
        contentData: content.data,
        position: playbackState.position,
        duration: playbackState.duration,
      };

      await invoke('add_to_xtream_playback_history', historyItem);

      // Refresh history
      const updatedHistory = await invoke<PlaybackHistory[]>('get_xtream_playback_history', {
        profileId: activeProfile.id
      });
      setPlaybackHistory(updatedHistory);
    } catch (error) {
      console.error('Failed to add to playback history:', error);
    }
  }, [activeProfile, playbackState.position, playbackState.duration]);

  // Update playback position (for resume functionality)
  const updatePlaybackPosition = useCallback(async (position: number, duration?: number) => {
    setPlaybackState(prev => ({
      ...prev,
      position,
      duration: duration || prev.duration,
    }));

    // Save position to history for resume functionality
    if (playbackState.currentContent && activeProfile) {
      try {
        const contentId = getContentId(playbackState.currentContent);
        if (contentId) {
          await invoke('update_xtream_playback_position', {
            profileId: activeProfile.id,
            contentType: playbackState.currentContent.type,
            contentId,
            position,
            duration: duration || playbackState.duration,
          });
        }
      } catch (error) {
        console.error('Failed to update playback position:', error);
      }
    }
  }, [playbackState.currentContent, activeProfile, playbackState.duration]);

  // Get resume position for content
  const getResumePosition = useCallback((content: ContentItem): number => {
    if (!activeProfile) return 0;

    const contentId = getContentId(content);
    if (!contentId) return 0;

    const historyItem = playbackHistory.find(item =>
      item.profileId === activeProfile.id &&
      item.contentType === content.type &&
      item.contentId === contentId
    );

    return historyItem?.position || 0;
  }, [activeProfile, playbackHistory]);

  // Control functions
  const pause = useCallback(() => {
    setPlaybackState(prev => ({ ...prev, isPlaying: false }));
  }, []);

  const resume = useCallback(() => {
    setPlaybackState(prev => ({ ...prev, isPlaying: true }));
  }, []);

  const stop = useCallback(() => {
    setPlaybackState(prev => ({
      ...prev,
      currentContent: null,
      isPlaying: false,
      position: 0,
      duration: 0,
    }));
  }, []);

  const setVolume = useCallback((volume: number) => {
    setPlaybackState(prev => ({ ...prev, volume: Math.max(0, Math.min(1, volume)) }));
  }, []);

  const toggleMute = useCallback(() => {
    setPlaybackState(prev => ({ ...prev, muted: !prev.muted }));
  }, []);

  const toggleFullscreen = useCallback(() => {
    setPlaybackState(prev => ({ ...prev, fullscreen: !prev.fullscreen }));
  }, []);

  const seek = useCallback((position: number) => {
    setPlaybackState(prev => ({
      ...prev,
      position: Math.max(0, Math.min(prev.duration, position))
    }));
  }, []);

  // Helper functions
  const getContentId = (content: ContentItem): string | null => {
    switch (content.type) {
      case 'channel':
        return (content.data as Channel).name;
      case 'xtream-channel':
        return (content.data as XtreamChannel).stream_id.toString();
      case 'xtream-movie':
        return (content.data as XtreamMoviesListing).stream_id.toString();
      case 'xtream-series':
        return (content.data as XtreamShow).info?.series_id?.toString() || null;
      default:
        return null;
    }
  };

  const getXtreamContentType = (type: string): string | null => {
    switch (type) {
      case 'xtream-channel':
        return 'Channel';
      case 'xtream-movie':
        return 'Movie';
      case 'xtream-series':
        return 'Series';
      default:
        return null;
    }
  };

  const getDefaultExtension = (type: string): string => {
    switch (type) {
      case 'xtream-channel':
        return 'm3u8';
      case 'xtream-movie':
      case 'xtream-series':
        return 'mp4';
      default:
        return 'm3u8';
    }
  };

  return {
    playbackState,
    playbackHistory,
    isLoadingHistory,
    playContent,
    updatePlaybackPosition,
    getResumePosition,
    pause,
    resume,
    stop,
    setVolume,
    toggleMute,
    toggleFullscreen,
    seek,
  };
};