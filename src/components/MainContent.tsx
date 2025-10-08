import { useCallback, useEffect } from "react";
import {
  useChannelStore,
  useProfileStore,
  useSearchStore,
  useSettingsStore,
  useUIStore,
  useXtreamContentStore,
} from "../stores";
import { type Channel } from "./ChannelList";
import GroupList from "./GroupList";
import MovieGrid from "./MovieGrid";
import ProfileManager from "./ProfileManager";
import SearchBar from "./SearchBar";
import SeriesBrowser from "./SeriesBrowser";
import VirtualChannelList from "./VirtualChannelList";

interface MainContentProps {
  filteredChannels: Channel[];
}

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

  const handleSearchChange = useCallback((value: string) => {
    setSearchQuery(value);
  }, [setSearchQuery]);

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
            <SearchBar
              value={searchQuery}
              onChange={handleSearchChange}
              placeholder="Search channels (min 3 characters)..."
              debounceDelay={300}
            />
            {searchQuery.length > 0 && searchQuery.length < 3 && (
              <div className="search-status">
                Type at least 3 characters to search...
              </div>
            )}
            {isSearching && <div className="search-status">Searching...</div>}
            <div className="content-list">
              <VirtualChannelList channels={combinedChannels} />
            </div>
          </>
        );
      case "favorites":
        return (
          <div className="content-list">
            <VirtualChannelList channels={favorites} />
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
            <VirtualChannelList channels={history} />
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
