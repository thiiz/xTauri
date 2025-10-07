import { useEffect } from "react";
import {
  useChannelStore,
  useProfileStore,
  useSearchStore,
  useSettingsStore,
  useUIStore,
  useXtreamContentStore,
} from "../stores";
import ChannelList, { type Channel } from "./ChannelList";
import GroupList from "./GroupList";
import MovieGrid from "./MovieGrid";
import ProfileManager from "./ProfileManager";
import SeriesBrowser from "./SeriesBrowser";

interface MainContentProps {
  filteredChannels: Channel[];
}

// Loading indicator component
const LoadingChannelList = () => (
  <div className="loading-channel-list">
    <div className="loading-content">
      <div className="loading-spinner-large">
        <div className="spinner-large"></div>
      </div>
      <h3>Loading Channel List</h3>
      <p>Setting up channels and groups...</p>
    </div>
  </div>
);

export default function MainContent({ filteredChannels }: MainContentProps) {
  // Get state from stores
  const {
    favorites,
    groups,
    history,
  } = useChannelStore();

  const { activeTab } = useUIStore();

  const { searchQuery, isSearching, setSearchQuery } = useSearchStore();

  const { channelListName } = useSettingsStore();

  const { activeProfile } = useProfileStore();

  const {
    movies,
    series,
    channels: xtreamChannels,
    fetchChannels: fetchXtreamChannels
  } = useXtreamContentStore();

  // selectedChannelListId was removed from channelStore
  // Channel list name functionality is disabled for now

  // Load Xtream channels when profile becomes active
  useEffect(() => {
    if (activeProfile && activeTab === "channels") {
      fetchXtreamChannels(activeProfile.id);
    }
  }, [activeProfile, activeTab, fetchXtreamChannels]);

  const handleSearch = (e: React.ChangeEvent<HTMLInputElement>) => {
    setSearchQuery(e.target.value);
  };

  const handleClearSearch = () => {
    setSearchQuery("");
  };

  const handleKeyDown = (e: React.KeyboardEvent<HTMLInputElement>) => {
    if (e.ctrlKey) {
      switch (e.key) {
        case "w":
          e.preventDefault();
          // Remove last word
          const input = e.currentTarget;
          const value = input.value;
          const cursorPos = input.selectionStart || 0;
          const beforeCursor = value.substring(0, cursorPos);
          const afterCursor = value.substring(cursorPos);

          // Find the start of the last word before cursor
          const words = beforeCursor.trimEnd();
          const lastSpaceIndex = words.lastIndexOf(" ");
          const newBeforeCursor =
            lastSpaceIndex >= 0 ? words.substring(0, lastSpaceIndex + 1) : "";

          const newValue = newBeforeCursor + afterCursor;
          setSearchQuery(newValue);

          // Set cursor position after the removed word
          setTimeout(() => {
            input.setSelectionRange(
              newBeforeCursor.length,
              newBeforeCursor.length,
            );
          }, 0);
          break;

        case "u":
          e.preventDefault();
          // Clear entire input
          setSearchQuery("");
          break;

        case "c":
          e.preventDefault();
          // Unfocus the input
          e.currentTarget.blur();
          break;
      }
    }
  };

  const getTabTitle = () => {
    switch (activeTab) {
      case "channels":
        return channelListName ? `Channels (${channelListName})` : "Channels";
      case "favorites":
        return "Favorites";
      case "groups":
        return channelListName ? `Groups (${channelListName})` : "Groups";
      case "history":
        return "History";
      case "movies":
        return "Movies";
      case "series":
        return "TV Series";
      case "profiles":
        return "Xtream Profiles";
      default:
        return "IPTV Player";
    }
  };

  // Combine traditional channels with Xtream channels
  const combinedChannels = (() => {
    const combined = [...filteredChannels];

    if (activeProfile && xtreamChannels.length > 0) {
      // Convert Xtream channels to traditional channel format
      const convertedXtreamChannels = xtreamChannels.map(xtreamChannel => ({
        name: xtreamChannel.name,
        logo: xtreamChannel.stream_icon,
        url: xtreamChannel.url || '',
        group_title: 'Xtream Live TV', // Default group for Xtream channels
        tvg_id: xtreamChannel.epg_channel_id,
        resolution: 'HD',
        extra_info: `Stream ID: ${xtreamChannel.stream_id}`
      }));

      combined.push(...convertedXtreamChannels);
    }

    return combined;
  })();

  const getTabSubtitle = () => {
    switch (activeTab) {
      case "channels":
        const totalChannels = combinedChannels.length;
        const xtreamCount = activeProfile ? xtreamChannels.length : 0;
        const traditionalCount = filteredChannels.length;

        if (xtreamCount > 0 && traditionalCount > 0) {
          return `${totalChannels} channels available (${traditionalCount} playlist + ${xtreamCount} Xtream)`;
        } else if (xtreamCount > 0) {
          return `${totalChannels} Xtream channels available`;
        } else {
          return `${totalChannels} channels available`;
        }
      case "favorites":
        return `${favorites.length} favorite channels`;
      case "groups":
        return `${groups.length} groups available`;
      case "history":
        return `${history.length} recently watched`;
      case "movies":
        return activeProfile
          ? `${movies.length} movies available`
          : "Select an Xtream profile to view movies";
      case "series":
        return activeProfile
          ? `${series.length} series available`
          : "Select an Xtream profile to view series";
      case "profiles":
        return "Manage your Xtream Codes profiles";
      default:
        return "";
    }
  };

  const renderContent = () => {
    switch (activeTab) {
      case "channels":
        // Loading functionality was removed

        return (
          <>
            <div className="search-container">
              <div className="search-input-wrapper">
                <input
                  type="text"
                  className="search-input"
                  placeholder="Search channels (min 3 characters)..."
                  value={searchQuery}
                  onChange={handleSearch}
                  onKeyDown={handleKeyDown}
                />
                {searchQuery && (
                  <button
                    className="clear-search-btn"
                    onClick={handleClearSearch}
                    type="button"
                    title="Clear search"
                  >
                    ×
                  </button>
                )}
              </div>
            </div>
            {searchQuery.length > 0 && searchQuery.length < 3 && (
              <div className="search-status">
                Type at least 3 characters to search...
              </div>
            )}
            {isSearching && <div className="search-status">Searching...</div>}
            <div className="content-list">
              <ChannelList channels={combinedChannels} />
            </div>
          </>
        );
      case "favorites":
        return (
          <div className="content-list">
            <ChannelList channels={favorites} />
          </div>
        );
      case "groups":
        return (
          <div className="content-list">
            <GroupList />
          </div>
        );
      case "history":
        return (
          <div className="content-list">
            <ChannelList channels={history} />
          </div>
        );
      case "movies":
        if (!activeProfile) {
          return (
            <div className="content-placeholder">
              <h3>No Xtream Profile Selected</h3>
              <p>Please select an Xtream profile to browse movies.</p>
            </div>
          );
        }
        return <MovieGrid />;
      case "series":
        if (!activeProfile) {
          return (
            <div className="content-placeholder">
              <h3>No Xtream Profile Selected</h3>
              <p>Please select an Xtream profile to browse TV series.</p>
            </div>
          );
        }
        return <SeriesBrowser />;
      case "profiles":
        return <ProfileManager />;

      default:
        return null;
    }
  };

  return (
    <div className="channels-section">
      <div className="section-header">
        <h2 className="section-title">{getTabTitle()}</h2>
        <p className="section-subtitle">{getTabSubtitle()}</p>
      </div>
      {renderContent()}
    </div>
  );
}
