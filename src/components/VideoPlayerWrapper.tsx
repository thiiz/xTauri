import { forwardRef } from "react";
import ModernVideoPlayer, { type ContentItem } from "./ModernVideoPlayer";

interface VideoPlayerWrapperProps {
  selectedXtreamContent?: ContentItem | null;
  onContentChange?: (content: ContentItem | null) => void;
  nextEpisode?: {
    episode: any;
    series: any;
  } | null;
  onPlayNextEpisode?: () => void;
}

const VideoPlayerWrapper = forwardRef<HTMLVideoElement, VideoPlayerWrapperProps>(
  ({ selectedXtreamContent, onContentChange, nextEpisode, onPlayNextEpisode }, ref) => {
    return (
      <ModernVideoPlayer
        ref={ref}
        selectedContent={selectedXtreamContent}
        onContentChange={onContentChange}
        nextEpisode={nextEpisode}
        onPlayNextEpisode={onPlayNextEpisode}
      />
    );
  }
);

VideoPlayerWrapper.displayName = "VideoPlayerWrapper";

export default VideoPlayerWrapper;
export type { ContentItem };

