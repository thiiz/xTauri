import { forwardRef, useState } from "react";
import { useChannelStore, useSettingsStore } from "../stores";
import { PlayIcon } from "./Icons";

const VideoPlayer = forwardRef<HTMLVideoElement, {}>((_, ref) => {
  const {
    selectedChannel,
  } = useChannelStore();
  const { muteOnStart, showControls, autoplay } = useSettingsStore();
  const [codecWarning, setCodecWarning] = useState(false);

  return (
    <div className="video-preview">
      <div className="video-container">
        {selectedChannel ? (
          <>
            <video
              ref={ref}
              className="video-player"
              controls={showControls}
              muted={muteOnStart}
              autoPlay={autoplay}
              onError={() => setCodecWarning(true)}
              onLoadStart={() => setCodecWarning(false)}
            />
            {codecWarning && (
              <div className="codec-warning">
                ⚠️ Video codec issue detected. Install GStreamer plugins: gstreamer1.0-plugins-bad gstreamer1.0-libav
              </div>
            )}
            <div className="video-controls">
              <div className="video-status">
                <div className="status-dot"></div>
                <span className="status-text">Live</span>
              </div>
              <div className="quality-badge">
                {selectedChannel.resolution || "HD"}
              </div>
            </div>
          </>
        ) : (
          <div className="video-placeholder">
            <PlayIcon />
            <div className="video-placeholder-text">Preview Window</div>
            <div className="video-placeholder-channel">
              Select a channel to start watching
            </div>
          </div>
        )}
      </div>
    </div>
  );
});

VideoPlayer.displayName = "VideoPlayer";

export default VideoPlayer;
