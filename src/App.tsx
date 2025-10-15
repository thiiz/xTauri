import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import "./App.css";
import ContentDetails from "./components/ContentDetails";
import Help from "./components/Help";
import MainContent from "./components/MainContent";
import NavigationSidebar from "./components/NavigationSidebar";
import ProfileManager from "./components/ProfileManager";
import Settings from "./components/Settings";
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
    fetchAutoplay,
    fetchMuteOnStart,
    fetchShowControls,
    fetchCacheDuration,
    fetchVolume,
    fetchIsMuted,
  } = useSettingsStore();

  // Refs for video player
  const videoRef = useRef<HTMLVideoElement>(null);

  // State for Xtream content playback
  const [selectedXtreamContent, setSelectedXtreamContent] = useState<ContentItem | null>(null);
  const [nextEpisode, setNextEpisode] = useState<{ episode: XtreamEpisode; series: XtreamShow } | null>(null);

  // Fetch all settings on app load - memoized to run only once
  useEffect(() => {
    const loadSettings = async () => {
      try {
        await Promise.all([
          fetchEnablePreview(),
          fetchAutoplay(),
          fetchMuteOnStart(),
          fetchShowControls(),
          fetchCacheDuration(),
          fetchVolume(),
          fetchIsMuted(),
        ]);
      } catch (error) {
        console.error("Failed to load settings:", error);
      }
    };

    loadSettings();
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []); // Run only once on mount

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
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [activeProfile?.id]); // Only depend on profile ID to avoid unnecessary reloads

  // Sync Xtream channels to channel store - Memoized to prevent unnecessary conversions
  const convertedChannels = useMemo(() => {
    if (xtreamChannels.length === 0) return [];

    return xtreamChannels.map(ch => ({
      name: ch.name,
      url: ch.url || '',
      group_title: ch.category_id,
      logo: ch.stream_icon,
      tvg_id: ch.epg_channel_id || '',
      resolution: 'HD',
      extra_info: '',
    }));
  }, [xtreamChannels]);

  useEffect(() => {
    if (convertedChannels.length > 0) {
      setChannels(convertedChannels);
    }
  }, [convertedChannels, setChannels]);

  const handleSelectGroup = (group: string | null) => {
    setSelectedGroup(group);
    setActiveTab("channels");
  };

  // Memoize filtered channels to prevent unnecessary recalculations
  const filteredChannels = useMemo(() => {
    if (groupDisplayMode === GroupDisplayMode.EnabledGroups) {
      return channels.filter((channel) => enabledGroups.has(channel.group_title));
    }

    if (groupDisplayMode === GroupDisplayMode.AllGroups && selectedGroup) {
      return channels.filter((channel) => channel.group_title === selectedGroup);
    }

    return channels;
  }, [channels, groupDisplayMode, enabledGroups, selectedGroup]);

  // Memoize filtered groups to prevent unnecessary recalculations
  const displayedGroups = useMemo(() => {
    const filteredGroups = groups.filter((group: string) =>
      group.toLowerCase().includes(groupSearchTerm.toLowerCase())
    );

    if (groupDisplayMode === GroupDisplayMode.AllGroups) {
      return [null, ...filteredGroups];
    }

    return filteredGroups;
  }, [groups, groupSearchTerm, groupDisplayMode]);

  // Memoize list items based on active tab
  const listItems = useMemo(() => {
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
  }, [activeTab, filteredChannels, favorites, displayedGroups, history]);

  // Memoize handlers to prevent unnecessary re-renders
  const handleClearGroupSearch = useCallback(() => {
    setGroupSearchTerm("");
    setFocusedIndex(0);
  }, [setGroupSearchTerm, setFocusedIndex]);

  const handleClearAllFilters = useCallback(() => {
    clearSearch();
    setSelectedGroup(null);
    setGroupDisplayMode(GroupDisplayMode.EnabledGroups);
    setActiveTab("channels");
    setFocusedIndex(0);
  }, [clearSearch, setSelectedGroup, setGroupDisplayMode, setActiveTab, setFocusedIndex]);

  const handleSelectAllGroups = useCallback(() => {
    if (activeProfile) {
      selectAllGroups(groups, parseInt(activeProfile.id));
    }
  }, [activeProfile, groups, selectAllGroups]);

  const handleUnselectAllGroups = useCallback(() => {
    if (activeProfile) {
      unselectAllGroups(groups, parseInt(activeProfile.id));
    }
  }, [activeProfile, groups, unselectAllGroups]);

  const handleToggleGroupDisplayMode = useCallback(() => {
    const newMode =
      groupDisplayMode === GroupDisplayMode.EnabledGroups
        ? GroupDisplayMode.AllGroups
        : GroupDisplayMode.EnabledGroups;
    setGroupDisplayMode(newMode);
    setFocusedIndex(0);
  }, [groupDisplayMode, setGroupDisplayMode, setFocusedIndex]);

  const handleToggleCurrentGroupSelection = useCallback(() => {
    if (activeTab === "groups" && activeProfile) {
      const currentGroup = listItems[focusedIndex] as string | null;
      if (currentGroup) {
        toggleGroupEnabled(currentGroup, parseInt(activeProfile.id));
      }
    }
  }, [activeTab, activeProfile, listItems, focusedIndex, toggleGroupEnabled]);



  // Memoized content selection handlers
  const handleContentSelect = useCallback((content: ContentItem | null) => {
    setSelectedXtreamContent(content);
    if (content) {
      setSelectedChannel(null);
    }
  }, [setSelectedChannel]);

  const handleMovieSelect = useCallback((movie: XtreamMoviesListing) => {
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
  }, [handleContentSelect]);

  const handleMoviePlay = handleMovieSelect;

  const getNextEpisode = useCallback((currentEpisode: XtreamEpisode, series: XtreamShow): { episode: XtreamEpisode; series: XtreamShow } | null => {
    const currentSeasonNum = currentEpisode.season;
    const currentEpisodeNum = parseInt(currentEpisode.episode_num);

    const currentSeasonEpisodes = series.episodes[currentSeasonNum.toString()] || [];
    const nextInSeason = currentSeasonEpisodes.find(ep => parseInt(ep.episode_num) === currentEpisodeNum + 1);

    if (nextInSeason) {
      return { episode: nextInSeason, series };
    }

    const nextSeasonNum = currentSeasonNum + 1;
    const nextSeasonEpisodes = series.episodes[nextSeasonNum.toString()] || [];

    if (nextSeasonEpisodes.length > 0) {
      return { episode: nextSeasonEpisodes[0], series };
    }

    return null;
  }, []);

  const handleEpisodePlay = useCallback((episode: XtreamEpisode, series: XtreamShow) => {
    const next = getNextEpisode(episode, series);
    setNextEpisode(next);

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
  }, [getNextEpisode, handleContentSelect]);

  const handlePlayNextEpisode = useCallback(() => {
    if (nextEpisode) {
      handleEpisodePlay(nextEpisode.episode, nextEpisode.series);
    }
  }, [nextEpisode, handleEpisodePlay]);

  useKeyboardNavigation({
    activeTab,
    focusedIndex,
    listItems,
    searchQuery,
    setFocusedIndex,
    setSelectedChannel,
    setActiveTab,
    handleSelectGroup,
    handleToggleFavorite: toggleFavorite,
    clearSearch,
    clearGroupSearch: handleClearGroupSearch,
    clearAllFilters: handleClearAllFilters,
    selectAllGroups: handleSelectAllGroups,
    unselectAllGroups: handleUnselectAllGroups,
    toggleGroupDisplayMode: handleToggleGroupDisplayMode,
    toggleCurrentGroupSelection: handleToggleCurrentGroupSelection,
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
                  nextEpisode={nextEpisode}
                  onPlayNextEpisode={handlePlayNextEpisode}
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
