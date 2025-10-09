import { invoke } from "@tauri-apps/api/core";
import Hls from "hls.js";
import React, { forwardRef, useCallback, useEffect, useRef, useState } from "react";
import { useContentPlayback } from "../hooks/useContentPlayback";
import { useChannelStore, useSettingsStore } from "../stores";
import { useProfileStore } from "../stores/profileStore";
import { useXtreamContentStore } from "../stores/xtreamContentStore";
import type { EnhancedEPGListing, XtreamChannel, XtreamMoviesListing, XtreamShow } from "../types/types";
import type { Channel } from "./ChannelList";
import { PlayIcon } from "./Icons";
import NextEpisodeCountdown from "./NextEpisodeCountdown";
import VideoControls from "./VideoControls";
import VideoMetadataOverlay from "./VideoMetadataOverlay";
import VideoResumePrompt from "./VideoResumePrompt";

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

interface ModernVideoPlayerProps {
  selectedContent?: ContentItem | null;
  onContentChange?: (content: ContentItem | null) => void;
  nextEpisode?: {
    episode: any;
    series: any;
  } | null;
  onPlayNextEpisode?: () => void;
}

const ModernVideoPlayer = forwardRef<HTMLVideoElement, ModernVideoPlayerProps>(
  ({ selectedContent, nextEpisode, onPlayNextEpisode }, ref) => {
    const { selectedChannel } = useChannelStore();
    const { activeProfile } = useProfileStore();
    const { currentAndNextEPG, epgData } = useXtreamContentStore();
    const {
      muteOnStart,
      autoplay,
      volume: savedVolume,
      isMuted: savedIsMuted,
      setVolume: setSavedVolume,
      setIsMuted: setSavedIsMuted,
      saveVolume,
      saveIsMuted
    } = useSettingsStore();

    // Video state
    const [codecWarning, setCodecWarning] = useState(false);
    const [currentEPG, setCurrentEPG] = useState<EnhancedEPGListing | null>(null);
    const [nextEPG, setNextEPG] = useState<EnhancedEPGListing | null>(null);
    const [isGeneratingUrl, setIsGeneratingUrl] = useState(false);
    const [streamUrl, setStreamUrl] = useState<string | null>(null);
    const [showResumePrompt, setShowResumePrompt] = useState(false);
    const [resumePosition, setResumePosition] = useState(0);
    const [showMetadata, setShowMetadata] = useState(false);

    // Playback state - Use settings store values
    const [isPlaying, setIsPlaying] = useState(false);
    const [isMuted, setIsMuted] = useState(savedIsMuted);
    const [volume, setVolume] = useState(savedVolume);
    const [currentTime, setCurrentTime] = useState(0);
    const [duration, setDuration] = useState(0);
    const [playbackRate, setPlaybackRate] = useState(1);
    const [isFullscreen, setIsFullscreen] = useState(false);
    const [isPiP, setIsPiP] = useState(false);
    const [showControlsOverlay, setShowControlsOverlay] = useState(true);
    const [buffering, setBuffering] = useState(false);

    // Next episode countdown state
    const [showNextEpisodeCountdown, setShowNextEpisodeCountdown] = useState(false);
    const [countdownTriggered, setCountdownTriggered] = useState(false);

    // Subtitle and audio tracks
    const [subtitles, setSubtitles] = useState<TextTrack[]>([]);
    const [audioTracks, setAudioTracks] = useState<any[]>([]);
    const [selectedSubtitle, setSelectedSubtitle] = useState<number>(-1);
    const [selectedAudioTrack, setSelectedAudioTrack] = useState<number>(0);

    const hlsRef = useRef<Hls | null>(null);
    const controlsTimeoutRef = useRef<number | null>(null);
    const containerRef = useRef<HTMLDivElement>(null);

    const { getResumePosition, updatePlaybackPosition } = useContentPlayback();

    const activeContent = selectedContent || (selectedChannel ? {
      type: 'channel' as const,
      data: selectedChannel,
      url: selectedChannel.url
    } : null);

    // Sync local state with settings store
    useEffect(() => {
      setVolume(savedVolume);
      setIsMuted(savedIsMuted);
    }, [savedVolume, savedIsMuted]);

    // Generate stream URL - Use ref to track content ID to prevent unnecessary regeneration
    const lastContentIdRef = useRef<string | null>(null);

    useEffect(() => {
      const generateStreamUrl = async () => {
        if (!activeContent || !activeProfile) {
          setStreamUrl(null);
          setShowResumePrompt(false);
          lastContentIdRef.current = null;
          return;
        }

        // Generate a unique content identifier
        const contentId = getContentId(activeContent);
        const currentContentId = `${activeContent.type}-${contentId}`;

        // Skip if same content (prevents re-generation on re-renders)
        if (lastContentIdRef.current === currentContentId) {
          return;
        }

        lastContentIdRef.current = currentContentId;

        if (activeContent.type === 'xtream-movie' || activeContent.type === 'xtream-series') {
          const resumePos = getResumePosition(activeContent);
          if (resumePos > 30) {
            setResumePosition(resumePos);
            setShowResumePrompt(true);
          }
        }

        if (activeContent.type === 'channel') {
          setStreamUrl(activeContent.url || null);
          return;
        }

        if (activeContent.url) {
          setStreamUrl(activeContent.url);
          return;
        }

        setIsGeneratingUrl(true);
        try {
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
    }, [activeContent, activeProfile]);

    // Fetch EPG data
    useEffect(() => {
      const fetchEPGData = async () => {
        if (!activeContent || !activeProfile || activeContent.type !== 'xtream-channel') {
          setCurrentEPG(null);
          setNextEPG(null);
          return;
        }

        const channelData = activeContent.data as XtreamChannel;
        const channelId = channelData.stream_id.toString();

        try {
          const existingEPG = currentAndNextEPG[channelId];
          if (existingEPG?.current) {
            setCurrentEPG(existingEPG.current);
            setNextEPG(existingEPG.next || null);
            return;
          }

          const channelEPG = epgData[channelId];
          if (channelEPG && channelEPG.length > 0) {
            const now = Date.now() / 1000;
            const currentProgram = channelEPG.find(program =>
              (program.start_timestamp || 0) <= now && (program.stop_timestamp || 0) >= now
            );
            if (currentProgram) {
              setCurrentEPG(currentProgram);

              // Find next program
              const nextProgram = channelEPG.find(program =>
                (program.start_timestamp || 0) > now
              );
              setNextEPG(nextProgram || null);
              return;
            }
          }

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

    // Apply volume and mute state to video element
    useEffect(() => {
      if (!ref || typeof ref === 'function') return;
      const video = ref.current;
      if (!video) return;

      video.volume = volume;
      video.muted = isMuted;
    }, [streamUrl, ref, volume, isMuted]);

    // Save playback position when content changes or component unmounts
    useEffect(() => {
      return () => {
        if (!ref || typeof ref === 'function') return;
        const video = ref.current;
        if (!video || !activeContent) return;

        // Save final position for VOD content
        if (activeContent.type === 'xtream-movie' || activeContent.type === 'xtream-series') {
          if (video.currentTime > 0 && video.duration > 0) {
            updatePlaybackPosition(video.currentTime, video.duration);
          }
        }
      };
    }, [activeContent, ref, updatePlaybackPosition]);

    // Setup HLS.js - Use ref to track stream URL to prevent unnecessary recreation
    const lastStreamUrlRef = useRef<string | null>(null);

    useEffect(() => {
      // Skip if same stream URL (prevents recreation on re-renders)
      if (streamUrl && lastStreamUrlRef.current === streamUrl) {
        return;
      }

      // Cleanup previous HLS instance only when URL actually changes
      if (hlsRef.current && lastStreamUrlRef.current !== streamUrl) {
        hlsRef.current.destroy();
        hlsRef.current = null;
      }

      lastStreamUrlRef.current = streamUrl;

      if (!streamUrl || !ref || typeof ref === 'function') {
        return;
      }

      const video = ref.current;
      if (!video) return;

      const isM3u8 = streamUrl.includes('.m3u8') || streamUrl.includes('m3u8');
      const isTsStream = streamUrl.includes('.ts') || streamUrl.endsWith('.ts');
      const isLiveChannel = activeContent?.type === 'channel' || activeContent?.type === 'xtream-channel';
      const needsHlsJs = isM3u8 || isTsStream || isLiveChannel;

      if (needsHlsJs && Hls.isSupported()) {
        const hls = new Hls({
          enableWorker: false,
          lowLatencyMode: isLiveChannel,
          backBufferLength: 90,
          maxBufferLength: 30,
          maxMaxBufferLength: 60,
          progressive: isTsStream,
          xhrSetup: (xhr: XMLHttpRequest) => {
            xhr.withCredentials = false;
          }
        });

        hlsRef.current = hls;
        hls.loadSource(streamUrl);
        hls.attachMedia(video);

        hls.on(Hls.Events.MANIFEST_PARSED, () => {
          setCodecWarning(false);
          setBuffering(false);

          // Extract audio tracks
          if (hls.audioTracks && hls.audioTracks.length > 0) {
            setAudioTracks(hls.audioTracks);
          }

          if (autoplay) {
            video.play().catch(err => {
              console.error('Autoplay failed:', err);
              setCodecWarning(true);
            });
          }
        });

        hls.on(Hls.Events.ERROR, (_event, data) => {
          if (data.fatal) {
            switch (data.type) {
              case Hls.ErrorTypes.NETWORK_ERROR:
                console.log('Network error, attempting recovery...');
                hls.startLoad();
                break;
              case Hls.ErrorTypes.MEDIA_ERROR:
                console.log('Media error, attempting recovery...');
                hls.recoverMediaError();
                break;
              default:
                console.error('Fatal HLS error:', data);
                setCodecWarning(true);
                hls.destroy();
                break;
            }
          }
        });

      } else if (needsHlsJs && video.canPlayType('application/vnd.apple.mpegurl')) {
        video.src = streamUrl;
        const handleLoadedMetadata = () => {
          setCodecWarning(false);
          if (autoplay) video.play().catch(console.error);
        };
        const handleError = () => setCodecWarning(true);

        video.addEventListener('loadedmetadata', handleLoadedMetadata);
        video.addEventListener('error', handleError);
        video.load();

        return () => {
          video.removeEventListener('loadedmetadata', handleLoadedMetadata);
          video.removeEventListener('error', handleError);
        };
      } else if (!needsHlsJs) {
        video.src = streamUrl;
        const handleLoadedMetadata = () => {
          setCodecWarning(false);
          if (autoplay) video.play().catch(console.error);
        };
        const handleError = () => setCodecWarning(true);

        video.addEventListener('loadedmetadata', handleLoadedMetadata);
        video.addEventListener('error', handleError);
        video.load();

        return () => {
          video.removeEventListener('loadedmetadata', handleLoadedMetadata);
          video.removeEventListener('error', handleError);
        };
      } else {
        setCodecWarning(true);
      }

      return () => {
        // Only destroy on unmount, not on every re-render
        if (hlsRef.current && !lastStreamUrlRef.current) {
          hlsRef.current.destroy();
          hlsRef.current = null;
        }
      };
    }, [streamUrl, autoplay, ref]);

    // Throttle playback position updates to avoid excessive backend calls
    const lastPositionUpdateRef = useRef<number>(0);
    const POSITION_UPDATE_INTERVAL = 5000; // Update every 5 seconds

    // Video event handlers
    const handleVideoTimeUpdate = useCallback((event: React.SyntheticEvent<HTMLVideoElement>) => {
      const video = event.currentTarget;
      setCurrentTime(video.currentTime);

      // Throttle playback position updates for VOD content
      if (activeContent && (activeContent.type === 'xtream-movie' || activeContent.type === 'xtream-series')) {
        const now = Date.now();
        if (now - lastPositionUpdateRef.current >= POSITION_UPDATE_INTERVAL) {
          updatePlaybackPosition(video.currentTime, video.duration);
          lastPositionUpdateRef.current = now;
        }
      }

      // Check if we should show next episode countdown (30 seconds before end)
      const timeRemaining = video.duration - video.currentTime;
      if (
        nextEpisode &&
        activeContent?.type === 'xtream-series' &&
        timeRemaining <= 30 &&
        timeRemaining > 0 &&
        !countdownTriggered &&
        !video.paused
      ) {
        setShowNextEpisodeCountdown(true);
        setCountdownTriggered(true);
      }
    }, [activeContent, updatePlaybackPosition, nextEpisode, countdownTriggered]);

    const handleVideoLoadedMetadata = useCallback((event: React.SyntheticEvent<HTMLVideoElement>) => {
      const video = event.currentTarget;
      setDuration(video.duration);

      if (resumePosition > 0 && !showResumePrompt) {
        video.currentTime = resumePosition;
      }

      // Extract text tracks (subtitles)
      const tracks = Array.from(video.textTracks);
      setSubtitles(tracks);

      // Reset countdown state for new video
      setShowNextEpisodeCountdown(false);
      setCountdownTriggered(false);
    }, [resumePosition, showResumePrompt]);

    const handlePlay = useCallback(() => setIsPlaying(true), []);
    const handlePause = useCallback((event: React.SyntheticEvent<HTMLVideoElement>) => {
      setIsPlaying(false);

      // Save playback position when paused
      const video = event.currentTarget;
      if (activeContent && (activeContent.type === 'xtream-movie' || activeContent.type === 'xtream-series')) {
        updatePlaybackPosition(video.currentTime, video.duration);
      }
    }, [activeContent, updatePlaybackPosition]);
    const handleVolumeChange = useCallback((event: React.SyntheticEvent<HTMLVideoElement>) => {
      const video = event.currentTarget;
      const newVolume = video.volume;
      const newMuted = video.muted;

      setVolume(newVolume);
      setIsMuted(newMuted);

      // Persist to settings
      setSavedVolume(newVolume);
      setSavedIsMuted(newMuted);
      saveVolume();
      saveIsMuted();
    }, [setSavedVolume, setSavedIsMuted, saveVolume, saveIsMuted]);

    const handleWaiting = useCallback(() => setBuffering(true), []);
    const handleCanPlay = useCallback(() => setBuffering(false), []);

    // Control handlers
    const togglePlayPause = useCallback(() => {
      if (!ref || typeof ref === 'function') return;
      const video = ref.current;
      if (!video) return;

      if (video.paused) {
        video.play();
      } else {
        video.pause();
      }
    }, [ref]);

    const toggleMute = useCallback(() => {
      if (!ref || typeof ref === 'function') return;
      const video = ref.current;
      if (!video) return;

      const newMuted = !video.muted;
      video.muted = newMuted;
      setIsMuted(newMuted);

      // Persist to settings
      setSavedIsMuted(newMuted);
      saveIsMuted();
    }, [ref, setSavedIsMuted, saveIsMuted]);

    const handleVolumeSliderChange = useCallback((newVolume: number) => {
      if (!ref || typeof ref === 'function') return;
      const video = ref.current;
      if (!video) return;

      video.volume = newVolume;
      setVolume(newVolume);

      if (newVolume > 0 && video.muted) {
        video.muted = false;
        setIsMuted(false);
        setSavedIsMuted(false);
        saveIsMuted();
      }

      // Persist to settings
      setSavedVolume(newVolume);
      saveVolume();
    }, [ref, setSavedVolume, setSavedIsMuted, saveVolume, saveIsMuted]);

    const handleSeek = useCallback((time: number) => {
      if (!ref || typeof ref === 'function') return;
      const video = ref.current;
      if (!video) return;

      video.currentTime = time;
    }, [ref]);

    const changePlaybackRate = useCallback((rate: number) => {
      if (!ref || typeof ref === 'function') return;
      const video = ref.current;
      if (!video) return;

      video.playbackRate = rate;
      setPlaybackRate(rate);
    }, [ref]);

    const toggleFullscreen = useCallback(async () => {
      if (!containerRef.current) return;

      if (!document.fullscreenElement) {
        await containerRef.current.requestFullscreen();
        setIsFullscreen(true);
      } else {
        await document.exitFullscreen();
        setIsFullscreen(false);
      }
    }, []);

    const togglePiP = useCallback(async () => {
      if (!ref || typeof ref === 'function') return;
      const video = ref.current;
      if (!video) return;

      try {
        if (document.pictureInPictureElement) {
          await document.exitPictureInPicture();
          setIsPiP(false);
        } else {
          await video.requestPictureInPicture();
          setIsPiP(true);
        }
      } catch (error) {
        console.error('PiP error:', error);
      }
    }, [ref]);

    const handleSubtitleChange = useCallback((index: number) => {
      subtitles.forEach((track, i) => {
        track.mode = i === index ? 'showing' : 'hidden';
      });
      setSelectedSubtitle(index);
    }, [subtitles]);

    const handleAudioTrackChange = useCallback((index: number) => {
      if (hlsRef.current && hlsRef.current.audioTracks) {
        hlsRef.current.audioTrack = index;
        setSelectedAudioTrack(index);
      }
    }, []);

    const handleResumePlayback = useCallback(() => {
      setShowResumePrompt(false);
    }, []);

    const handleStartFromBeginning = useCallback(() => {
      setShowResumePrompt(false);
      setResumePosition(0);
    }, []);

    const toggleMetadataDisplay = useCallback(() => {
      setShowMetadata(!showMetadata);
    }, [showMetadata]);

    const handlePlayNextEpisode = useCallback(() => {
      setShowNextEpisodeCountdown(false);
      onPlayNextEpisode?.();
    }, [onPlayNextEpisode]);

    const handleCancelNextEpisode = useCallback(() => {
      setShowNextEpisodeCountdown(false);
    }, []);

    // Auto-hide controls
    const resetControlsTimeout = useCallback(() => {
      if (controlsTimeoutRef.current) {
        clearTimeout(controlsTimeoutRef.current);
      }

      setShowControlsOverlay(true);

      if (isPlaying) {
        controlsTimeoutRef.current = setTimeout(() => {
          setShowControlsOverlay(false);
        }, 3000);
      }
    }, [isPlaying]);

    useEffect(() => {
      resetControlsTimeout();
      return () => {
        if (controlsTimeoutRef.current) {
          clearTimeout(controlsTimeoutRef.current);
        }
      };
    }, [resetControlsTimeout]);

    // Keyboard shortcuts
    useEffect(() => {
      const handleKeyDown = (e: KeyboardEvent) => {
        if (!ref || typeof ref === 'function') return;
        const video = ref.current;
        if (!video) return;

        switch (e.key) {
          case ' ':
          case 'k':
            e.preventDefault();
            togglePlayPause();
            break;
          case 'ArrowLeft':
            e.preventDefault();
            handleSeek(Math.max(0, video.currentTime - 10));
            break;
          case 'ArrowRight':
            e.preventDefault();
            handleSeek(Math.min(video.duration, video.currentTime + 10));
            break;
          case 'ArrowUp':
            e.preventDefault();
            handleVolumeSliderChange(Math.min(1, video.volume + 0.1));
            break;
          case 'ArrowDown':
            e.preventDefault();
            handleVolumeSliderChange(Math.max(0, video.volume - 0.1));
            break;
          case 'm':
            e.preventDefault();
            toggleMute();
            break;
          case 'f':
            e.preventDefault();
            toggleFullscreen();
            break;
          case 'p':
            e.preventDefault();
            togglePiP();
            break;
          case 'i':
            e.preventDefault();
            toggleMetadataDisplay();
            break;
        }
      };

      window.addEventListener('keydown', handleKeyDown);
      return () => window.removeEventListener('keydown', handleKeyDown);
    }, [ref, togglePlayPause, handleSeek, handleVolumeSliderChange, toggleMute, toggleFullscreen, togglePiP, toggleMetadataDisplay]);

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
          return 'HD';
        case 'xtream-movie':
        case 'xtream-series':
          return 'HD';
        default:
          return 'HD';
      }
    };

    const metadata = getContentMetadata();

    return (
      <div className="video-preview">
        <div
          ref={containerRef}
          className="video-container modern-player"
          onMouseMove={resetControlsTimeout}
          onMouseLeave={() => isPlaying && setShowControlsOverlay(false)}
        >
          {activeContent ? (
            <>
              {isGeneratingUrl ? (
                <div className="video-placeholder">
                  <div className="loading-spinner"></div>
                  <div className="video-placeholder-text">Generating Stream URL...</div>
                </div>
              ) : streamUrl ? (
                <>
                  <video
                    ref={ref}
                    className="video-player"
                    controls={false}
                    muted={muteOnStart}
                    autoPlay={autoplay}
                    onTimeUpdate={handleVideoTimeUpdate}
                    onLoadedMetadata={handleVideoLoadedMetadata}
                    onPlay={handlePlay}
                    onPause={handlePause}
                    onVolumeChange={handleVolumeChange}
                    onWaiting={handleWaiting}
                    onCanPlay={handleCanPlay}
                  />

                  {buffering && (
                    <div className="buffering-indicator">
                      <div className="loading-spinner"></div>
                    </div>
                  )}

                  <VideoMetadataOverlay
                    show={showMetadata || showControlsOverlay}
                    title={getContentTitle()}
                    statusText={getStatusText()}
                    qualityBadge={getQualityBadge()}
                    currentEPG={currentEPG}
                    nextEPG={nextEPG}
                    metadata={metadata}
                    onToggle={toggleMetadataDisplay}
                  />

                  <VideoControls
                    show={showControlsOverlay}
                    isPlaying={isPlaying}
                    isMuted={isMuted}
                    volume={volume}
                    currentTime={currentTime}
                    duration={duration}
                    playbackRate={playbackRate}
                    isFullscreen={isFullscreen}
                    isPiP={isPiP}
                    subtitles={subtitles}
                    audioTracks={audioTracks}
                    selectedSubtitle={selectedSubtitle}
                    selectedAudioTrack={selectedAudioTrack}
                    isLive={activeContent?.type === 'channel' || activeContent?.type === 'xtream-channel'}
                    onPlayPause={togglePlayPause}
                    onMute={toggleMute}
                    onVolumeChange={handleVolumeSliderChange}
                    onSeek={handleSeek}
                    onPlaybackRateChange={changePlaybackRate}
                    onFullscreen={toggleFullscreen}
                    onPiP={togglePiP}
                    onSubtitleChange={handleSubtitleChange}
                    onAudioTrackChange={handleAudioTrackChange}
                  />
                </>
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
                  ⚠️ Unable to play this stream. The video format may not be supported by your browser.
                </div>
              )}

              {showResumePrompt && (
                <VideoResumePrompt
                  resumePosition={resumePosition}
                  onResume={handleResumePlayback}
                  onStartOver={handleStartFromBeginning}
                />
              )}

              {showNextEpisodeCountdown && nextEpisode && (
                <NextEpisodeCountdown
                  show={showNextEpisodeCountdown}
                  timeRemaining={10}
                  nextEpisodeTitle={nextEpisode.episode.title || 'Próximo Episódio'}
                  nextEpisodeNumber={nextEpisode.episode.episode_num || ''}
                  onConfirm={handlePlayNextEpisode}
                  onCancel={handleCancelNextEpisode}
                />
              )}
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

ModernVideoPlayer.displayName = "ModernVideoPlayer";

// Memoize to prevent unnecessary re-renders
export default React.memo(ModernVideoPlayer, (prevProps, nextProps) => {
  // Only re-render if these specific props change
  return (
    prevProps.selectedContent?.type === nextProps.selectedContent?.type &&
    prevProps.selectedContent?.data === nextProps.selectedContent?.data &&
    prevProps.nextEpisode === nextProps.nextEpisode &&
    prevProps.onPlayNextEpisode === nextProps.onPlayNextEpisode &&
    prevProps.onContentChange === nextProps.onContentChange
  );
});
