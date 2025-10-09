import { useCallback, useEffect, useRef } from 'react';
import { useProfileStore } from '../stores/profileStore';
import { useXtreamContentStore } from '../stores/xtreamContentStore';

interface UseHistoryTrackingOptions {
  contentType: 'channel' | 'movie' | 'series';
  contentId: string;
  contentData: any;
  enabled?: boolean;
  updateInterval?: number; // How often to update position (in seconds)
}

/**
 * Hook to automatically track viewing history and playback position
 * 
 * @example
 * ```tsx
 * const { updatePosition } = useHistoryTracking({
 *   contentType: 'movie',
 *   contentId: movie.stream_id.toString(),
 *   contentData: movie,
 *   enabled: isPlaying
 * });
 * 
 * // In your video player's onTimeUpdate:
 * updatePosition(currentTime, duration);
 * ```
 */
export const useHistoryTracking = ({
  contentType,
  contentId,
  contentData,
  enabled = true,
  updateInterval = 10, // Update every 10 seconds by default
}: UseHistoryTrackingOptions) => {
  const { activeProfile } = useProfileStore();
  const { addToHistory, updatePlaybackPosition, getHistoryItem } = useXtreamContentStore();

  const lastUpdateRef = useRef<number>(0);
  const hasAddedToHistoryRef = useRef<boolean>(false);
  const updateTimeoutRef = useRef<ReturnType<typeof setTimeout> | null>(null);

  // Add to history when playback starts
  useEffect(() => {
    if (!enabled || !activeProfile?.id || hasAddedToHistoryRef.current) return;

    const addInitialHistory = async () => {
      try {
        // Check if item already exists in history
        const existingItem = getHistoryItem(activeProfile.id, contentType, contentId);

        if (!existingItem) {
          // Add new history item
          await addToHistory(
            activeProfile.id,
            contentType,
            contentId,
            contentData,
            0,
            undefined
          );
        }

        hasAddedToHistoryRef.current = true;
      } catch (error) {
        console.error('Failed to add to history:', error);
      }
    };

    addInitialHistory();
  }, [enabled, activeProfile?.id, contentType, contentId, contentData, addToHistory, getHistoryItem]);

  // Update playback position
  const updatePosition = useCallback(
    (position: number, duration?: number) => {
      if (!enabled || !activeProfile?.id) return;

      const now = Date.now();
      const timeSinceLastUpdate = (now - lastUpdateRef.current) / 1000;

      // Only update if enough time has passed
      if (timeSinceLastUpdate >= updateInterval) {
        lastUpdateRef.current = now;

        // Clear any pending update
        if (updateTimeoutRef.current) {
          clearTimeout(updateTimeoutRef.current);
        }

        // Debounce the update to avoid too many calls
        updateTimeoutRef.current = setTimeout(async () => {
          try {
            await updatePlaybackPosition(
              activeProfile.id,
              contentType,
              contentId,
              position,
              duration
            );
          } catch (error) {
            console.error('Failed to update playback position:', error);
          }
        }, 500);
      }
    },
    [enabled, activeProfile?.id, contentType, contentId, updateInterval, updatePlaybackPosition]
  );

  // Save final position when component unmounts or playback stops
  useEffect(() => {
    return () => {
      if (updateTimeoutRef.current) {
        clearTimeout(updateTimeoutRef.current);
      }
    };
  }, []);

  // Reset when content changes
  useEffect(() => {
    hasAddedToHistoryRef.current = false;
    lastUpdateRef.current = 0;
  }, [contentId]);

  return {
    updatePosition,
  };
};

/**
 * Hook to get resume position for content
 */
export const useResumePosition = (
  contentType: 'channel' | 'movie' | 'series',
  contentId: string
) => {
  const { activeProfile } = useProfileStore();
  const { getHistoryItem } = useXtreamContentStore();

  if (!activeProfile?.id) return null;

  const historyItem = getHistoryItem(activeProfile.id, contentType, contentId);

  if (!historyItem || !historyItem.position || !historyItem.duration) {
    return null;
  }

  // Only suggest resume if watched more than 5% but less than 95%
  const progress = (historyItem.position / historyItem.duration) * 100;
  if (progress < 5 || progress > 95) {
    return null;
  }

  return {
    position: historyItem.position,
    duration: historyItem.duration,
    progress,
  };
};
