import Hls from "hls.js";
import { useEffect, useRef, useState } from "react";
import Help from "./components/Help";
import MainContent from "./components/MainContent";
import NavigationSidebar from "./components/NavigationSidebar";
import ProfileManager from "./components/ProfileManager";
import Settings from "./components/Settings";

import "./App.css";
import type { Channel } from "./components/ChannelList";
import ContentDetails from "./components/ContentDetails";
import VideoPlayerWrapper, { type ContentItem } from "./components/VideoPlayerWrapper";
import VirtualMovieGrid from "./components/VirtualMovieGrid";
import VirtualSeriesBrowser from "./components/VirtualSeriesBrowser";
import { useKeyboardNavigation } from "./hooks/useKeyboardNavigation";
import {
  GroupDisplayMode,
  useChannelStore,
  useProfileStore,
  useSearchStore,
  useSettingsStore,
  useUIStore,
  useXtreamContentStore,
} from "./stores";
import type { XtreamEpisode, XtreamMoviesListing, XtreamShow } from "./types/types";

function App() {
  // Zustand store hooks
  const {
    channels,
    favorites,
    groups,
    history,
    selectedChannel,
    setChannels,
    setSelectedChannel,
    toggleFavorite,
  } = useChannelStore();

  const {
    activeTab,
    focusedIndex,
    selectedGroup,
    groupDisplayMode,
    enabledGroups,
    groupSearchTerm,
    setActiveTab,
    setFocusedIndex,
    setSelectedGroup,
    setGroupDisplayMode,
    setGroupSearchTerm,
    selectAllGroups,
    unselectAllGroups,
    toggleGroupEnabled,
  } = useUIStore();

  const { searchQuery, clearSearch } = useSearchStore();
  const { activeProfile } = useProfileStore();
  const {
    channels: xtreamChannels,
    fetchChannels: fetchXtreamChannels,
    fetchChannelCategories,
  } = useXtreamContentStore();

  // Get settings
  const {
    enablePreview,
    fetchEnablePreview,
    autoplay,
    fetchAutoplay,
    fetchMuteOnStart,
    fetchShowControls,
    fetchCacheDuration,
  } = useSettingsStore();

  // Refs for video player
  const videoRef = useRef<HTMLVideoElement>(null);
  const hlsRef = useRef<Hls | null>(null);

  // State for Xtream content playback
  const [selectedXtreamContent, setSelectedXtreamContent] = useState<ContentItem | null>(null);

  // Fetch all settings on app load
  useEffect(() => {
    const loadSettings = async () => {
      try {
        await Promise.all([
          fetchEnablePreview(),
          fetchAutoplay(),
          fetchMuteOnStart(),
          fetchShowControls(),
          fetchCacheDuration(),
        ]);
      } catch (error) {
        console.error("Failed to load settings:", error);
      }
    };

    loadSettings();
  }, [fetchEnablePreview, fetchAutoplay, fetchMuteOnStart, fetchShowControls, fetchCacheDuration]);

  // Load Xtream content when active profile changes
  useEffect(() => {
    if (!activeProfile) {
      setChannels([]);
      return;
    }

    const loadXtreamContent = async () => {
      try {
        await Promise.all([
          fetchChannelCategories(activeProfile.id),
          fetchXtreamChannels(activeProfile.id)
        ]);
      } catch (error) {
        console.error("Failed to load Xtream content:", error);
      }
    };

    loadXtreamContent();
  }, [activeProfile, fetchChannelCategories, fetchXtreamChannels, setChannels]);

  // Sync Xtream channels to channel store
  useEffect(() => {
    if (xtreamChannels.length === 0) return;

    const convertedChannels: Channel[] = xtreamChannels.map(ch => ({
      name: ch.name,
      url: ch.url || '',
      group_title: ch.category_id,
      logo: ch.stream_icon,
      tvg_id: ch.epg_channel_id || '',
      resolution: 'HD',
      extra_info: '',
    }));
    setChannels(convertedChannels);
  }, [xtreamChannels, setChannels]);

  useEffect(() => {
    if (hlsRef.current) {
      hlsRef.current.destroy();
    }

    // Only load video if preview is enabled
    if (enablePreview && selectedChannel && videoRef.current) {
      const video = videoRef.current;
      const isHlsUrl =
        selectedChannel.url.includes(".m3u8") ||
        selectedChannel.url.includes("m3u8");

      if (isHlsUrl && Hls.isSupported()) {
        // Use HLS.js for .m3u8 streams when supported
        const hls = new Hls();
        hlsRef.current = hls;
        hls.loadSource(selectedChannel.url);
        hls.attachMedia(video);
        hls.on(Hls.Events.MANIFEST_PARSED, () => {
          if (autoplay) video.play();
        });
      } else if (
        isHlsUrl &&
        video.canPlayType("application/vnd.apple.mpegurl")
      ) {
        // Native HLS support (Safari)
        video.src = selectedChannel.url;
        video.addEventListener("loadedmetadata", () => {
          if (autoplay) video.play();
        });
      } else {
        // Fallback for direct video streams (MP4, WebM, etc.) and other protocols
        video.src = selectedChannel.url;
        video.addEventListener("loadedmetadata", () => {
          if (autoplay) video.play();
        });

        // Handle load errors gracefully
        video.addEventListener("error", (e) => {
          console.warn(`Video load error for ${selectedChannel.name}:`, e);
        });
      }
    }
  }, [selectedChannel, enablePreview, autoplay]);

  const handleSelectGroup = (group: string | null) => {
    setSelectedGroup(group);
    setActiveTab("channels");
  };

  const filteredChannels = (() => {
    let filtered = [...channels];

    // Apply group filtering based on current mode
    if (groupDisplayMode === GroupDisplayMode.EnabledGroups) {
      // Show only channels from enabled groups
      filtered = filtered.filter((channel) =>
        enabledGroups.has(channel.group_title),
      );
    } else if (
      groupDisplayMode === GroupDisplayMode.AllGroups &&
      selectedGroup
    ) {
      // Traditional single group selection from all groups
      filtered = filtered.filter(
        (channel) => channel.group_title === selectedGroup,
      );
    }
    // If AllGroups mode with no selection, show all channels

    return filtered;
  })();

  // Filter groups based on search term for keyboard navigation
  const filteredDisplayedGroups = groups.filter((group: string) =>
    group.toLowerCase().includes(groupSearchTerm.toLowerCase()),
  );

  // Include "All Groups" option for AllGroups mode in keyboard navigation
  const displayedGroups = (() => {
    if (groupDisplayMode === GroupDisplayMode.AllGroups) {
      return [null, ...filteredDisplayedGroups]; // null represents "All Groups"
    }
    return filteredDisplayedGroups;
  })();

  const listItems = (() => {
    switch (activeTab) {
      case "channels":
        return filteredChannels;
      case "favorites":
        return favorites;
      case "groups":
        return displayedGroups;
      case "history":
        return history;
      default:
        return [];
    }
  })();

  const handleClearGroupSearch = () => {
    setGroupSearchTerm("");
    setFocusedIndex(0);
  };

  const handleClearAllFilters = () => {
    clearSearch();
    setSelectedGroup(null);
    setGroupDisplayMode(GroupDisplayMode.EnabledGroups);
    setActiveTab("channels");
    setFocusedIndex(0);
  };

  const handleSelectAllGroups = () => {
    if (activeProfile) {
      selectAllGroups(groups, parseInt(activeProfile.id));
    }
  };

  const handleUnselectAllGroups = () => {
    if (activeProfile) {
      unselectAllGroups(groups, parseInt(activeProfile.id));
    }
  };

  const handleToggleGroupDisplayMode = () => {
    const newMode =
      groupDisplayMode === GroupDisplayMode.EnabledGroups
        ? GroupDisplayMode.AllGroups
        : GroupDisplayMode.EnabledGroups;
    setGroupDisplayMode(newMode);
    setFocusedIndex(0);
  };

  const handleToggleCurrentGroupSelection = () => {
    if (activeTab === "groups" && activeProfile) {
      const currentGroup = listItems[focusedIndex] as string | null;
      if (currentGroup) {
        toggleGroupEnabled(currentGroup, parseInt(activeProfile.id));
      }
    }
  };

  // Video control handlers
  const handleToggleMute = () => {
    if (videoRef.current) {
      videoRef.current.muted = !videoRef.current.muted;
    }
  };

  const handleToggleFullscreen = () => {
    if (videoRef.current) {
      if (document.fullscreenElement) {
        document.exitFullscreen();
      } else {
        videoRef.current.requestFullscreen();
      }
    }
  };

  const handleTogglePlayPause = () => {
    if (videoRef.current) {
      if (videoRef.current.paused) {
        videoRef.current.play();
      } else {
        videoRef.current.pause();
      }
    }
  };

  // Unified content selection handler
  const handleContentSelect = (content: ContentItem | null) => {
    if (content) {
      setSelectedXtreamContent(content);
      setSelectedChannel(null);
    } else {
      setSelectedXtreamContent(null);
    }
  };

  // Handlers for Xtream content
  const handleMovieSelect = (movie: XtreamMoviesListing) => {
    handleContentSelect({
      type: 'xtream-movie',
      data: movie,
      metadata: {
        title: movie.title || movie.name,
        description: movie.plot || undefined,
        duration: movie.episode_run_time || undefined,
        genre: movie.genre || undefined,
        rating: movie.rating || undefined,
        year: movie.year || undefined,
        cast: movie.cast || undefined,
        director: movie.director || undefined,
      }
    });
  };

  const handleMoviePlay = handleMovieSelect;

  const handleEpisodePlay = (episode: XtreamEpisode, series: XtreamShow) => {
    handleContentSelect({
      type: 'xtream-series',
      data: {
        ...series,
        stream_id: parseInt(episode.id),
      } as any,
      metadata: {
        title: episode.title,
        description: episode.info.plot || undefined,
        duration: episode.info.duration_secs ? Math.floor(episode.info.duration_secs / 60) : undefined,
        episodeId: episode.id,
        seasonNumber: episode.season,
        episodeNumber: parseInt(episode.episode_num),
      }
    });
  };

  useKeyboardNavigation({
    activeTab,
    channels,
    favorites,
    groups,
    history,
    selectedGroup,
    selectedChannel,
    focusedIndex,
    listItems,
    searchQuery,
    setFocusedIndex,
    setSelectedChannel,
    setActiveTab,
    handleSelectGroup,
    handleToggleFavorite: toggleFavorite,
    savedFilters: [],
    onSaveFilter: async () => false,
    onApplyFilter: () => { },
    clearSearch,
    clearGroupSearch: handleClearGroupSearch,
    clearAllFilters: handleClearAllFilters,
    refreshCurrentChannelList: () => { },
    selectAllGroups: handleSelectAllGroups,
    unselectAllGroups: handleUnselectAllGroups,
    toggleGroupDisplayMode: handleToggleGroupDisplayMode,
    toggleCurrentGroupSelection: handleToggleCurrentGroupSelection,
    toggleMute: handleToggleMute,
    toggleFullscreen: handleToggleFullscreen,
    togglePlayPause: handleTogglePlayPause,
  });

  // Show profile manager if no active profile
  if (!activeProfile && activeTab !== "settings" && activeTab !== "help") {
    return (
      <div className="container">
        <NavigationSidebar />
        <div className="main-content">
          <div className="settings-full-width">
            <div className="section-header">
              <h2 className="section-title">Welcome to xTauri</h2>
              <p className="section-subtitle">Add your Xtream Codes profile to get started</p>
            </div>
            <div className="settings-container">
              <ProfileManager />
            </div>
          </div>
        </div>
      </div>
    );
  }

  return (
    <div className="container">
      <NavigationSidebar />

      <div className="main-content">
        {activeTab === "settings" ? (
          <div className="settings-full-width">
            <div className="section-header">
              <h2 className="section-title">Settings</h2>
              <p className="section-subtitle">Application settings</p>
            </div>
            <div className="settings-container">
              <Settings />
            </div>
          </div>
        ) : activeTab === "help" ? (
          <div className="settings-full-width">
            <div className="section-header">
              <h2 className="section-title">Help</h2>
              <p className="section-subtitle">
                Keyboard shortcuts and keybindings
              </p>
            </div>
            <div className="settings-container">
              <Help />
            </div>
          </div>
        ) : activeTab === "movies" ? (
          <>
            <div className="main-content-section">
              <div className="section-header">
                <h2 className="section-title">Movies</h2>
                <p className="section-subtitle">Browse and watch movies</p>
              </div>
              <div className="movies-container">
                <VirtualMovieGrid
                  onMovieSelect={handleMovieSelect}
                  onMoviePlay={handleMoviePlay}
                  onContentSelect={() => setSelectedChannel(null)}
                />
              </div>
            </div>

            <div
              className={`video-section ${!enablePreview ? "preview-disabled" : ""}`}
            >
              {enablePreview && (
                <VideoPlayerWrapper
                  ref={videoRef}
                  selectedXtreamContent={selectedXtreamContent}
                />
              )}

              <ContentDetails selectedXtreamContent={selectedXtreamContent} />
            </div>
          </>
        ) : activeTab === "series" ? (
          <>
            <div className="main-content-section">
              <div className="section-header">
                <h2 className="section-title">Series</h2>
                <p className="section-subtitle">Browse and watch TV series</p>
              </div>
              <div className="series-container">
                <VirtualSeriesBrowser
                  onEpisodePlay={handleEpisodePlay}
                  onContentSelect={() => setSelectedChannel(null)}
                />
              </div>
            </div>

            <div
              className={`video-section ${!enablePreview ? "preview-disabled" : ""}`}
            >
              {enablePreview && (
                <VideoPlayerWrapper
                  ref={videoRef}
                  selectedXtreamContent={selectedXtreamContent}
                />
              )}

              <ContentDetails selectedXtreamContent={selectedXtreamContent} />
            </div>
          </>
        ) : (
          <>
            <MainContent
              filteredChannels={filteredChannels}
              onChannelSelect={() => handleContentSelect(null)}
            />

            <div
              className={`video-section ${!enablePreview ? "preview-disabled" : ""}`}
            >
              {enablePreview && (
                <VideoPlayerWrapper
                  ref={videoRef}
                  selectedXtreamContent={selectedXtreamContent}
                />
              )}

              <ContentDetails selectedXtreamContent={selectedXtreamContent} />
            </div>
          </>
        )}
      </div>
    </div>
  );
}

export default App;
