import { useCallback, useState } from "react";

interface VideoControlsProps {
  show: boolean;
  isPlaying: boolean;
  isMuted: boolean;
  volume: number;
  currentTime: number;
  duration: number;
  playbackRate: number;
  isFullscreen: boolean;
  isPiP: boolean;
  subtitles: TextTrack[];
  audioTracks: any[];
  selectedSubtitle: number;
  selectedAudioTrack: number;
  isLive?: boolean; // New prop to indicate live content
  onPlayPause: () => void;
  onMute: () => void;
  onVolumeChange: (volume: number) => void;
  onSeek: (time: number) => void;
  onPlaybackRateChange: (rate: number) => void;
  onFullscreen: () => void;
  onPiP: () => void;
  onSubtitleChange: (index: number) => void;
  onAudioTrackChange: (index: number) => void;
}

const VideoControls: React.FC<VideoControlsProps> = ({
  show,
  isPlaying,
  isMuted,
  volume,
  currentTime,
  duration,
  playbackRate,
  isFullscreen,
  isPiP,
  subtitles,
  audioTracks,
  selectedSubtitle,
  selectedAudioTrack,
  isLive = false,
  onPlayPause,
  onMute,
  onVolumeChange,
  onSeek,
  onPlaybackRateChange,
  onFullscreen,
  onPiP,
  onSubtitleChange,
  onAudioTrackChange,
}) => {
  const [showVolumeSlider, setShowVolumeSlider] = useState(false);
  const [showPlaybackRateMenu, setShowPlaybackRateMenu] = useState(false);
  const [showSubtitleMenu, setShowSubtitleMenu] = useState(false);
  const [showAudioMenu, setShowAudioMenu] = useState(false);
  const [isDragging, setIsDragging] = useState(false);

  const formatTime = (seconds: number): string => {
    if (!isFinite(seconds)) return "0:00";

    const hours = Math.floor(seconds / 3600);
    const minutes = Math.floor((seconds % 3600) / 60);
    const secs = Math.floor(seconds % 60);

    if (hours > 0) {
      return `${hours}:${minutes.toString().padStart(2, '0')}:${secs.toString().padStart(2, '0')}`;
    }
    return `${minutes}:${secs.toString().padStart(2, '0')}`;
  };

  const handleProgressClick = useCallback((e: React.MouseEvent<HTMLDivElement>) => {
    const rect = e.currentTarget.getBoundingClientRect();
    const pos = (e.clientX - rect.left) / rect.width;
    onSeek(pos * duration);
  }, [duration, onSeek]);

  const handleProgressMouseDown = useCallback(() => {
    setIsDragging(true);
  }, []);

  const handleProgressMouseMove = useCallback((e: React.MouseEvent<HTMLDivElement>) => {
    if (!isDragging) return;
    const rect = e.currentTarget.getBoundingClientRect();
    const pos = Math.max(0, Math.min(1, (e.clientX - rect.left) / rect.width));
    onSeek(pos * duration);
  }, [isDragging, duration, onSeek]);

  const handleProgressMouseUp = useCallback(() => {
    setIsDragging(false);
  }, []);

  const playbackRates = [0.25, 0.5, 0.75, 1, 1.25, 1.5, 1.75, 2];

  const progress = duration > 0 ? (currentTime / duration) * 100 : 0;

  return (
    <div className={`modern-video-controls ${show ? 'show' : ''} ${isLive ? 'live-controls' : ''}`}>
      {/* Progress Bar - Hidden for live content */}
      {!isLive && (
        <div
          className="progress-bar-container"
          onClick={handleProgressClick}
          onMouseDown={handleProgressMouseDown}
          onMouseMove={handleProgressMouseMove}
          onMouseUp={handleProgressMouseUp}
          onMouseLeave={handleProgressMouseUp}
        >
          <div className="progress-bar">
            <div
              className="progress-bar-filled"
              style={{ width: `${progress}%` }}
            >
              <div className="progress-bar-handle"></div>
            </div>
          </div>
        </div>
      )}

      {/* Controls Row */}
      <div className="controls-row">
        {/* Left Controls */}
        <div className="controls-left">
          <button
            className="control-btn play-pause-btn"
            onClick={onPlayPause}
            title={isPlaying ? "Pause (Space)" : "Play (Space)"}
          >
            {isPlaying ? (
              <svg width="24" height="24" viewBox="0 0 24 24" fill="currentColor">
                <path d="M6 4h4v16H6V4zm8 0h4v16h-4V4z" />
              </svg>
            ) : (
              <svg width="24" height="24" viewBox="0 0 24 24" fill="currentColor">
                <path d="M8 5v14l11-7z" />
              </svg>
            )}
          </button>

          <div
            className="volume-control"
            onMouseEnter={() => setShowVolumeSlider(true)}
            onMouseLeave={() => setShowVolumeSlider(false)}
          >
            <button
              className="control-btn volume-btn"
              onClick={onMute}
              title={isMuted ? "Unmute (M)" : "Mute (M)"}
            >
              {isMuted || volume === 0 ? (
                <svg width="24" height="24" viewBox="0 0 24 24" fill="currentColor">
                  <path d="M16.5 12c0-1.77-1.02-3.29-2.5-4.03v2.21l2.45 2.45c.03-.2.05-.41.05-.63zm2.5 0c0 .94-.2 1.82-.54 2.64l1.51 1.51C20.63 14.91 21 13.5 21 12c0-4.28-2.99-7.86-7-8.77v2.06c2.89.86 5 3.54 5 6.71zM4.27 3L3 4.27 7.73 9H3v6h4l5 5v-6.73l4.25 4.25c-.67.52-1.42.93-2.25 1.18v2.06c1.38-.31 2.63-.95 3.69-1.81L19.73 21 21 19.73l-9-9L4.27 3zM12 4L9.91 6.09 12 8.18V4z" />
                </svg>
              ) : volume < 0.5 ? (
                <svg width="24" height="24" viewBox="0 0 24 24" fill="currentColor">
                  <path d="M7 9v6h4l5 5V4l-5 5H7z" />
                </svg>
              ) : (
                <svg width="24" height="24" viewBox="0 0 24 24" fill="currentColor">
                  <path d="M3 9v6h4l5 5V4L7 9H3zm13.5 3c0-1.77-1.02-3.29-2.5-4.03v8.05c1.48-.73 2.5-2.25 2.5-4.02z" />
                </svg>
              )}
            </button>

            {showVolumeSlider && (
              <div className="volume-slider-container">
                <input
                  type="range"
                  className="volume-slider"
                  min="0"
                  max="1"
                  step="0.01"
                  value={isMuted ? 0 : volume}
                  onChange={(e) => onVolumeChange(parseFloat(e.target.value))}
                />
              </div>
            )}
          </div>

          {/* Time Display - Show "LIVE" badge for live content */}
          <div className="time-display">
            {isLive ? (
              <span className="live-badge">● LIVE</span>
            ) : (
              <>
                <span className="current-time">{formatTime(currentTime)}</span>
                <span className="time-separator"> / </span>
                <span className="duration-time">{formatTime(duration)}</span>
              </>
            )}
          </div>
        </div>

        {/* Right Controls */}
        <div className="controls-right">
          {/* Subtitles */}
          {subtitles.length > 0 && (
            <div className="control-menu-wrapper">
              <button
                className="control-btn"
                onClick={() => setShowSubtitleMenu(!showSubtitleMenu)}
                title="Subtitles"
              >
                <svg width="24" height="24" viewBox="0 0 24 24" fill="currentColor">
                  <path d="M20 4H4c-1.1 0-2 .9-2 2v12c0 1.1.9 2 2 2h16c1.1 0 2-.9 2-2V6c0-1.1-.9-2-2-2zM4 12h4v2H4v-2zm10 6H4v-2h10v2zm6 0h-4v-2h4v2zm0-4H10v-2h10v2z" />
                </svg>
              </button>

              {showSubtitleMenu && (
                <div className="control-menu subtitle-menu">
                  <div className="menu-header">Subtitles</div>
                  <button
                    className={`menu-item ${selectedSubtitle === -1 ? 'active' : ''}`}
                    onClick={() => {
                      onSubtitleChange(-1);
                      setShowSubtitleMenu(false);
                    }}
                  >
                    Off
                  </button>
                  {subtitles.map((track, index) => (
                    <button
                      key={index}
                      className={`menu-item ${selectedSubtitle === index ? 'active' : ''}`}
                      onClick={() => {
                        onSubtitleChange(index);
                        setShowSubtitleMenu(false);
                      }}
                    >
                      {track.label || `Track ${index + 1}`}
                    </button>
                  ))}
                </div>
              )}
            </div>
          )}

          {/* Audio Tracks */}
          {audioTracks.length > 1 && (
            <div className="control-menu-wrapper">
              <button
                className="control-btn"
                onClick={() => setShowAudioMenu(!showAudioMenu)}
                title="Audio Track"
              >
                <svg width="24" height="24" viewBox="0 0 24 24" fill="currentColor">
                  <path d="M12 3v9.28c-.47-.17-.97-.28-1.5-.28C8.01 12 6 14.01 6 16.5S8.01 21 10.5 21c2.31 0 4.2-1.75 4.45-4H15V6h4V3h-7z" />
                </svg>
              </button>

              {showAudioMenu && (
                <div className="control-menu audio-menu">
                  <div className="menu-header">Audio Track</div>
                  {audioTracks.map((track, index) => (
                    <button
                      key={index}
                      className={`menu-item ${selectedAudioTrack === index ? 'active' : ''}`}
                      onClick={() => {
                        onAudioTrackChange(index);
                        setShowAudioMenu(false);
                      }}
                    >
                      {track.name || `Track ${index + 1}`}
                    </button>
                  ))}
                </div>
              )}
            </div>
          )}

          {/* Playback Rate - Hidden for live content */}
          {!isLive && (
            <div className="control-menu-wrapper">
              <button
                className="control-btn playback-rate-btn"
                onClick={() => setShowPlaybackRateMenu(!showPlaybackRateMenu)}
                title="Playback Speed"
              >
                <span className="playback-rate-text">{playbackRate}x</span>
              </button>

              {showPlaybackRateMenu && (
                <div className="control-menu playback-rate-menu">
                  <div className="menu-header">Playback Speed</div>
                  {playbackRates.map((rate) => (
                    <button
                      key={rate}
                      className={`menu-item ${playbackRate === rate ? 'active' : ''}`}
                      onClick={() => {
                        onPlaybackRateChange(rate);
                        setShowPlaybackRateMenu(false);
                      }}
                    >
                      {rate === 1 ? 'Normal' : `${rate}x`}
                    </button>
                  ))}
                </div>
              )}
            </div>
          )}

          {/* Picture-in-Picture */}
          <button
            className="control-btn"
            onClick={onPiP}
            title={isPiP ? "Exit Picture-in-Picture (P)" : "Picture-in-Picture (P)"}
          >
            <svg width="24" height="24" viewBox="0 0 24 24" fill="currentColor">
              <path d="M19 7h-8v6h8V7zm2-4H3c-1.1 0-2 .9-2 2v14c0 1.1.9 1.98 2 1.98h18c1.1 0 2-.88 2-1.98V5c0-1.1-.9-2-2-2zm0 16.01H3V4.98h18v14.03z" />
            </svg>
          </button>

          {/* Fullscreen */}
          <button
            className="control-btn"
            onClick={onFullscreen}
            title={isFullscreen ? "Exit Fullscreen (F)" : "Fullscreen (F)"}
          >
            {isFullscreen ? (
              <svg width="24" height="24" viewBox="0 0 24 24" fill="currentColor">
                <path d="M5 16h3v3h2v-5H5v2zm3-8H5v2h5V5H8v3zm6 11h2v-3h3v-2h-5v5zm2-11V5h-2v5h5V8h-3z" />
              </svg>
            ) : (
              <svg width="24" height="24" viewBox="0 0 24 24" fill="currentColor">
                <path d="M7 14H5v5h5v-2H7v-3zm-2-4h2V7h3V5H5v5zm12 7h-3v2h5v-5h-2v3zM14 5v2h3v3h2V5h-5z" />
              </svg>
            )}
          </button>
        </div>
      </div>
    </div>
  );
};

export default VideoControls;
