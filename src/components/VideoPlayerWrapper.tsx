import { forwardRef } from "react";
import ModernVideoPlayer, { type ContentItem } from "./ModernVideoPlayer";

interface VideoPlayerWrapperProps {
  selectedXtreamContent?: ContentItem | null;
  onContentChange?: (content: ContentItem | null) => void;
}

/**
 * VideoPlayerWrapper - Provides backward compatibility while using the modern player
 * This component wraps the ModernVideoPlayer and can be used as a drop-in replacement
 * for the EnhancedVideoPlayer component.
 */
const VideoPlayerWrapper = forwardRef<HTMLVideoElement, VideoPlayerWrapperProps>(
  ({ selectedXtreamContent, onContentChange }, ref) => {
    return (
      <ModernVideoPlayer
        ref={ref}
        selectedContent={selectedXtreamContent}
        onContentChange={onContentChange}
      />
    );
  }
);

VideoPlayerWrapper.displayName = "VideoPlayerWrapper";

export default VideoPlayerWrapper;
export type { ContentItem };

