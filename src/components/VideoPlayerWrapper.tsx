import { forwardRef, useEffect, useState } from "react";
import { useChannelStore } from "../stores";
import { useProfileStore } from "../stores/profileStore";
import type { XtreamChannel, XtreamMoviesListing, XtreamShow } from "../types/types";
import type { Channel } from "./ChannelList";
import EnhancedVideoPlayer from "./EnhancedVideoPlayer";

interface ContentItem {
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

interface VideoPlayerWrapperProps {
  selectedXtreamContent?: ContentItem | null;
  onContentChange?: (content: ContentItem | null) => void;
}

const VideoPlayerWrapper = forwardRef<HTMLVideoElement, VideoPlayerWrapperProps>(
  ({ selectedXtreamContent, onContentChange }, ref) => {
    const { selectedChannel } = useChannelStore();
    const { activeProfile } = useProfileStore();
    const [currentContent, setCurrentContent] = useState<ContentItem | null>(null);

    // Determine which content to display
    useEffect(() => {
      if (selectedXtreamContent) {
        setCurrentContent(selectedXtreamContent);
      } else if (selectedChannel) {
        setCurrentContent({
          type: 'channel',
          data: selectedChannel,
          url: selectedChannel.url
        });
      } else {
        setCurrentContent(null);
      }
    }, [selectedXtreamContent, selectedChannel]);

    // Notify parent of content changes
    useEffect(() => {
      if (onContentChange) {
        onContentChange(currentContent);
      }
    }, [currentContent, onContentChange]);

    // Use enhanced video player if we have an active Xtream profile or Xtream content
    const useEnhancedPlayer = activeProfile || (currentContent && currentContent.type.startsWith('xtream-'));

    if (useEnhancedPlayer) {
      return (
        <EnhancedVideoPlayer
          ref={ref}
          selectedContent={currentContent}
          onContentChange={onContentChange}
        />
      );
    }

    // Fall back to legacy video player for traditional channels
    return <EnhancedVideoPlayer ref={ref} />;
  }
);

VideoPlayerWrapper.displayName = "VideoPlayerWrapper";

export default VideoPlayerWrapper;
export type { ContentItem };

