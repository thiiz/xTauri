import { invoke } from '@tauri-apps/api/core';
import { useEffect, useState } from 'react';
import type { Channel } from '../components/ChannelList';
import type { ContentItem } from '../components/VideoPlayerWrapper';
import type { XtreamChannel, XtreamMoviesListing } from '../types/types';

interface UseStreamUrlResult {
  streamUrl: string | null;
  isGenerating: boolean;
  error: string | null;
}

/**
 * Custom hook to generate and manage stream URLs
 * Handles different content types and caches results
 */
export const useStreamUrl = (
  activeContent: ContentItem | null,
  profileId: string | null
): UseStreamUrlResult => {
  const [streamUrl, setStreamUrl] = useState<string | null>(null);
  const [isGenerating, setIsGenerating] = useState(false);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    const generateStreamUrl = async () => {
      // Reset state
      setError(null);

      if (!activeContent || !profileId) {
        setStreamUrl(null);
        setIsGenerating(false);
        return;
      }

      // Handle channel type - direct URL
      if (activeContent.type === 'channel') {
        setStreamUrl(activeContent.url || null);
        setIsGenerating(false);
        return;
      }

      // Use pre-generated URL if available
      if (activeContent.url) {
        setStreamUrl(activeContent.url);
        setIsGenerating(false);
        return;
      }

      // Generate URL for Xtream content
      setIsGenerating(true);
      try {
        const contentId = getContentId(activeContent);
        const contentType = getXtreamContentType(activeContent.type);

        if (!contentId || !contentType) {
          throw new Error('Invalid content ID or type');
        }

        const url = await invoke<string>('generate_xtream_stream_url', {
          profileId,
          contentType,
          contentId,
          extension: getDefaultExtension(activeContent.type)
        });

        setStreamUrl(url);
        setError(null);
      } catch (err) {
        const errorMessage = err instanceof Error ? err.message : 'Failed to generate stream URL';
        console.error('Stream URL generation error:', errorMessage);
        setError(errorMessage);
        setStreamUrl(null);
      } finally {
        setIsGenerating(false);
      }
    };

    generateStreamUrl();
  }, [activeContent, profileId]);

  return { streamUrl, isGenerating, error };
};

// Helper functions
function getContentId(content: ContentItem): string | null {
  switch (content.type) {
    case 'channel':
      return (content.data as Channel).name;
    case 'xtream-channel':
      return (content.data as XtreamChannel).stream_id.toString();
    case 'xtream-movie':
      return (content.data as XtreamMoviesListing).stream_id.toString();
    case 'xtream-series': {
      const seriesData = content.data as any;
      return seriesData.stream_id?.toString() || seriesData.info?.series_id?.toString() || null;
    }
    default:
      return null;
  }
}

function getXtreamContentType(type: string): string | null {
  switch (type) {
    case 'xtream-channel':
      return 'Channel';
    case 'xtream-movie':
    case 'xtream-series':
      return 'Movie';
    default:
      return null;
  }
}

function getDefaultExtension(type: string): string {
  switch (type) {
    case 'xtream-channel':
      return 'm3u8';
    case 'xtream-movie':
    case 'xtream-series':
      return 'mp4';
    default:
      return 'm3u8';
  }
}
