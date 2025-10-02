import { invoke } from "@tauri-apps/api/core";
import { forwardRef, useEffect, useRef, useState } from "react";
import { useContentPlayback } from "../hooks/useContentPlayback";
import { useChannelStore, useSettingsStore } from "../stores";
import { useProfileStore } from "../stores/profileStore";
import { useXtreamContentStore } from "../stores/xtreamContentStore";
import type { EnhancedEPGListing, XtreamChannel, XtreamMoviesListing, XtreamShow } from "../types/types";
import type { Channel } from "./ChannelList";
import { PlayIcon } from "./Icons";

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
  };
}

interface EnhancedVideoPlayerProps {
  selectedContent?: ContentItem | null;
  onContentChange?: (content: ContentItem | null) => void;
}

const EnhancedVideoPlayer = forwardRef<HTMLVideoElement, EnhancedVideoPlayerProps>(
  ({ selectedContent }, ref) => {
    const {
      selectedChannel,
      isExternalPlayerPlaying,
      setIsExternalPlayerPlaying,
    } = useChannelStore();

    const { activeProfile } = useProfileStore();
    const { currentAndNextEPG, epgData } = useXtreamContentStore();
    const { muteOnStart, showControls, autoplay } = useSettingsStore();

    const previousContentRef = useRef(selectedContent);
    const [codecWarning, setCodecWarning] = useState(false);
    const [currentEPG, setCurrentEPG] = useState<EnhancedEPGListing | null>(null);
    const [isGeneratingUrl, setIsGeneratingUrl] = useState(false);
    const [streamUrl, setStreamUrl] = useState<string | null>(null);
    const [showResumePrompt, setShowResumePrompt] = useState(false);
    const [resumePosition, setResumePosition] = useState(0);
    const [showMetadata, setShowMetadata] = useState(false);

    // Use content playback hook for enhanced features
    const {
      getResumePosition,
      updatePlaybackPosition
    } = useContentPlayback();

    // Determine the active content (either from props or legacy channel store)
    const activeContent = selectedContent || (selectedChannel ? {
      type: 'channel' as const,
      data: selectedChannel,
      url: selectedChannel.url
    } : null);

    // Reset external player playing state when content changes
    useEffect(() => {
      if (
        activeContent &&
        previousContentRef.current &&
        getContentId(activeContent) !== getContentId(previousContentRef.current) &&
        isExternalPlayerPlaying
      ) {
        setIsExternalPlayerPlaying(false);
      }
      previousContentRef.current = activeContent;
    }, [activeContent, isExternalPlayerPlaying, setIsExternalPlayerPlaying]);

    // Generate stream URL for Xtream content and check for resume position
    useEffect(() => {
      const generateStreamUrl = async () => {
        if (!activeContent || !activeProfile) {
          setStreamUrl(null);
          setShowResumePrompt(false);
          return;
        }

        // Check for resume position for VOD content (not live channels)
        if (activeContent.type === 'xtream-movie' || activeContent.type === 'xtream-series') {
          const resumePos = getResumePosition(activeContent);
          if (resumePos > 30) { // Only show resume prompt if more than 30 seconds
            setResumePosition(resumePos);
            setShowResumePrompt(true);
          }
        }

        // For legacy channels, use existing URL
        if (activeContent.type === 'channel') {
          setStreamUrl(activeContent.url || null);
          return;
        }

        // For Xtream content, generate URL if not already present
        if (activeContent.url) {
          setStreamUrl(activeContent.url);
          return;
        }

        setIsGeneratingUrl(true);
        try {
          const contentId = getContentId(activeContent);
          const contentType = getXtreamContentType(activeContent.type);

          if (contentId && contentType) {
            const url = await invoke<string>('generate_xtream_stream_url', {
              profileId: activeProfile.id,
              contentType,
              contentId,
              extension: getDefaultExtension(activeContent.type)
            });
            setStreamUrl(url);
          }
        } catch (error) {
          console.error('Failed to generate stream URL:', error);
          setStreamUrl(null);
        } finally {
          setIsGeneratingUrl(false);
        }
      };

      generateStreamUrl();
    }, [activeContent, activeProfile, getResumePosition]);

    // Fetch EPG data for live channels
    useEffect(() => {
      const fetchEPGData = async () => {
        if (!activeContent || !activeProfile || activeContent.type !== 'xtream-channel') {
          setCurrentEPG(null);
          return;
        }

        const channelData = activeContent.data as XtreamChannel;
        const channelId = channelData.stream_id.toString();

        try {
          // Check if we already have current/next EPG data
          const existingEPG = currentAndNextEPG[channelId];
          if (existingEPG?.current) {
            setCurrentEPG(existingEPG.current);
            return;
          }

          // Check if we have EPG data in the store
          const channelEPG = epgData[channelId];
          if (channelEPG && channelEPG.length > 0) {
            // Find current program
            const now = Date.now() / 1000;
            const currentProgram = channelEPG.find(program =>
              (program.start_timestamp || 0) <= now && (program.stop_timestamp || 0) >= now
            );
            if (currentProgram) {
              setCurrentEPG(currentProgram);
              return;
            }
          }

          // Fetch fresh EPG data
          await invoke('get_xtream_current_and_next_epg', {
            profileId: activeProfile.id,
            channelId
          });
        } catch (error) {
          console.error('Failed to fetch EPG data:', error);
        }
      };

      fetchEPGData();
    }, [activeContent, activeProfile, currentAndNextEPG, epgData]);

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
          // For series episodes, use the episode stream_id if available
          const seriesData = content.data as any;
          return seriesData.stream_id?.toString() || seriesData.info?.series_id?.toString() || null;
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
          return 'Movie'; // Episodes are treated as movies in the Xtream API
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

    const getContentTitle = (): string => {
      if (!activeContent) return '';

      switch (activeContent.type) {
        case 'channel':
          return (activeContent.data as Channel).name;
        case 'xtream-channel':
          return (activeContent.data as XtreamChannel).name;
        case 'xtream-movie':
          return (activeContent.data as XtreamMoviesListing).title || (activeContent.data as XtreamMoviesListing).name;
        case 'xtream-series':
          return (activeContent.data as XtreamShow).info?.title || (activeContent.data as XtreamShow).info?.name || '';
        default:
          return '';
      }
    };

    const getContentMetadata = () => {
      if (!activeContent) return null;

      switch (activeContent.type) {
        case 'xtream-movie':
          const movie = activeContent.data as XtreamMoviesListing;
          return {
            genre: movie.genre,
            year: movie.year,
            rating: movie.rating,
            duration: movie.episode_run_time,
            cast: movie.cast,
            director: movie.director,
            description: movie.plot
          };
        case 'xtream-series':
          const series = activeContent.data as XtreamShow;
          return {
            genre: series.info?.genre,
            year: series.info?.year,
            rating: parseFloat(series.info?.rating || '0'),
            cast: series.info?.cast,
            director: series.info?.director,
            description: series.info?.plot
          };
        default:
          return null;
      }
    };

    const getStatusText = (): string => {
      if (!activeContent) return '';

      switch (activeContent.type) {
        case 'channel':
        case 'xtream-channel':
          return 'Live';
        case 'xtream-movie':
          return 'Movie';
        case 'xtream-series':
          return 'Series';
        default:
          return '';
      }
    };

    const getQualityBadge = (): string => {
      if (!activeContent) return 'HD';

      switch (activeContent.type) {
        case 'channel':
          return (activeContent.data as Channel).resolution || 'HD';
        case 'xtream-channel':
          // Xtream channels don't typically have resolution info, so we'll default to HD
          return 'HD';
        case 'xtream-movie':
        case 'xtream-series':
          return 'HD'; // Could be enhanced with actual quality info if available
        default:
          return 'HD';
      }
    };

    // Video event handlers for enhanced playback features
    const handleVideoTimeUpdate = (event: React.SyntheticEvent<HTMLVideoElement>) => {
      const video = event.currentTarget;
      if (activeContent && (activeContent.type === 'xtream-movie' || activeContent.type === 'xtream-series')) {
        updatePlaybackPosition(video.currentTime, video.duration);
      }
    };

    const handleVideoLoadedMetadata = (event: React.SyntheticEvent<HTMLVideoElement>) => {
      const video = event.currentTarget;
      // If we have a resume position and user chose to resume, seek to that position
      if (resumePosition > 0 && !showResumePrompt) {
        video.currentTime = resumePosition;
      }
    };

    const handleResumePlayback = () => {
      setShowResumePrompt(false);
      // The video will seek to resume position in handleVideoLoadedMetadata
    };

    const handleStartFromBeginning = () => {
      setShowResumePrompt(false);
      setResumePosition(0);
    };

    const toggleMetadataDisplay = () => {
      setShowMetadata(!showMetadata);
    };

    const formatTime = (seconds: number): string => {
      const hours = Math.floor(seconds / 3600);
      const minutes = Math.floor((seconds % 3600) / 60);
      const secs = Math.floor(seconds % 60);

      if (hours > 0) {
        return `${hours}:${minutes.toString().padStart(2, '0')}:${secs.toString().padStart(2, '0')}`;
      }
      return `${minutes}:${secs.toString().padStart(2, '0')}`;
    };

    const metadata = getContentMetadata();

    return (
      <div className="video-preview">
        <div className="video-container">
          {activeContent && !isExternalPlayerPlaying ? (
            <>
              {isGeneratingUrl ? (
                <div className="video-placeholder">
                  <div className="loading-spinner"></div>
                  <div className="video-placeholder-text">Generating Stream URL...</div>
                </div>
              ) : streamUrl ? (
                <video
                  ref={ref}
                  className="video-player"
                  controls={showControls}
                  muted={muteOnStart}
                  autoPlay={autoplay}
                  src={streamUrl}
                  onError={() => setCodecWarning(true)}
                  onLoadStart={() => setCodecWarning(false)}
                  onTimeUpdate={handleVideoTimeUpdate}
                  onLoadedMetadata={handleVideoLoadedMetadata}
                />
              ) : (
                <div className="video-placeholder">
                  <PlayIcon />
                  <div className="video-placeholder-text">Stream Unavailable</div>
                  <div className="video-placeholder-channel">
                    Unable to generate stream URL
                  </div>
                </div>
              )}

              {codecWarning && (
                <div className="codec-warning">
                  ⚠️ Video codec issue detected. Install GStreamer plugins: gstreamer1.0-plugins-bad gstreamer1.0-libav
                </div>
              )}

              {/* Resume playback prompt */}
              {showResumePrompt && (
                <div className="resume-prompt">
                  <div className="resume-content">
                    <h3>Resume Playback</h3>
                    <p>Continue watching from {formatTime(resumePosition)}?</p>
                    <div className="resume-buttons">
                      <button onClick={handleResumePlayback} className="resume-btn">
                        Resume
                      </button>
                      <button onClick={handleStartFromBeginning} className="restart-btn">
                        Start Over
                      </button>
                    </div>
                  </div>
                </div>
              )}

              <div className="video-controls">
                <div className="video-status">
                  <div className="status-dot"></div>
                  <span className="status-text">{getStatusText()}</span>
                </div>
                <div className="quality-badge">
                  {getQualityBadge()}
                </div>
              </div>

              {/* Content metadata overlay */}
              <div className={`content-metadata ${showMetadata ? 'expanded' : ''}`}>
                <button
                  className="metadata-toggle"
                  onClick={toggleMetadataDisplay}
                  title="Toggle metadata display"
                >
                  ℹ️
                </button>
                <div className="content-title">{getContentTitle()}</div>

                {/* EPG information for live channels */}
                {(activeContent.type === 'xtream-channel') && currentEPG && (
                  <div className="epg-info">
                    <div className="current-program">
                      <span className="program-title">{currentEPG.title}</span>
                      <span className="program-time">
                        {new Date((currentEPG.start_timestamp || 0) * 1000).toLocaleTimeString()} -
                        {new Date((currentEPG.stop_timestamp || 0) * 1000).toLocaleTimeString()}
                      </span>
                    </div>
                    {currentEPG.description && (
                      <div className="program-description">{currentEPG.description}</div>
                    )}
                  </div>
                )}

                {/* Metadata for movies and series */}
                {metadata && (
                  <div className="content-info">
                    {metadata.year && <span className="content-year">{metadata.year}</span>}
                    {metadata.genre && <span className="content-genre">{metadata.genre}</span>}
                    {metadata.rating && <span className="content-rating">★ {metadata.rating}</span>}
                    {metadata.duration && (
                      <span className="content-duration">{Math.floor(metadata.duration / 60)}h {metadata.duration % 60}m</span>
                    )}
                    {metadata.description && (
                      <div className="content-description">{metadata.description}</div>
                    )}
                  </div>
                )}
              </div>
            </>
          ) : (
            <div className="video-placeholder">
              <PlayIcon />
              <div className="video-placeholder-text">Preview Window</div>
              <div className="video-placeholder-channel">
                Select content to start watching
              </div>
            </div>
          )}
        </div>
      </div>
    );
  }
);

EnhancedVideoPlayer.displayName = "EnhancedVideoPlayer";

export default EnhancedVideoPlayer;