import { useCallback, useEffect, useMemo, useState } from "react";
import {
  useChannelStore,
  useProfileStore,
  useSettingsStore,
  useUIStore,
  useXtreamContentStore
} from "../stores";
import { type Channel } from "./ChannelList";
import { FavoritesView } from "./FavoritesView";
import GroupList from "./GroupList";
import SearchBar from "./SearchBar";
import VirtualChannelList from "./VirtualChannelList";
import VirtualMovieGrid from "./VirtualMovieGrid";
import VirtualSeriesBrowser from "./VirtualSeriesBrowser";

interface MainContentProps {
  filteredChannels: Channel[];
  onChannelSelect?: () => void;
}

export default function MainContent({ filteredChannels, onChannelSelect }: MainContentProps) {
  // Get state from stores
  const {
    favorites,
    groups,
    history,
  } = useChannelStore();

  const { activeTab } = useUIStore();

  const { channelListName } = useSettingsStore();

  const { activeProfile } = useProfileStore();

  const {
    movies,
    series,
    channels: xtreamChannels,
    channelCategories,
    filteredChannels: filteredXtreamChannels,
    isLoadingChannelCategories,
    fetchChannels: fetchXtreamChannels,
    fetchChannelCategories,
    searchChannels,
    clearSearch: clearXtreamSearch,
    favorites: xtreamFavorites
  } = useXtreamContentStore();

  // Local state for channel category filter
  const [selectedChannelCategoryId, setSelectedChannelCategoryId] = useState<string | null>(null);
  const [channelSearchQuery, setChannelSearchQuery] = useState("");

  // selectedChannelListId was removed from channelStore
  // Channel list name functionality is disabled for now

  // Load Xtream channels and categories when profile becomes active
  useEffect(() => {
    if (activeProfile && activeTab === "channels") {
      fetchChannelCategories(activeProfile.id);
      fetchXtreamChannels(activeProfile.id);
    }
  }, [activeProfile, activeTab, fetchXtreamChannels, fetchChannelCategories]);

  // Display channels - use filtered if available, otherwise use all channels
  const displayXtreamChannels = useMemo(() =>
    filteredXtreamChannels.length > 0 ? filteredXtreamChannels : xtreamChannels,
    [filteredXtreamChannels, xtreamChannels]
  );

  // Handle channel category filter
  const handleChannelCategoryFilter = async (categoryId: string | null) => {
    if (!activeProfile) return;
    setSelectedChannelCategoryId(categoryId);
    clearXtreamSearch();
    setChannelSearchQuery("");
    await fetchXtreamChannels(activeProfile.id, categoryId || undefined);
  };

  // Handle channel search
  const handleChannelSearchChange = useCallback(async (query: string) => {
    if (!activeProfile) return;
    setChannelSearchQuery(query);

    if (query.trim()) {
      await searchChannels(activeProfile.id, query);
    } else {
      clearXtreamSearch();
      await fetchXtreamChannels(activeProfile.id, selectedChannelCategoryId || undefined);
    }
  }, [activeProfile, selectedChannelCategoryId, searchChannels, clearXtreamSearch, fetchXtreamChannels]);

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
      default:
        return "IPTV Player";
    }
  };

  // Combine traditional channels with Xtream channels
  const combinedChannels = useMemo(() => {
    const combined = [...filteredChannels];

    if (activeProfile && displayXtreamChannels.length > 0) {
      // Convert Xtream channels to traditional channel format
      const convertedXtreamChannels = displayXtreamChannels.map(xtreamChannel => ({
        name: xtreamChannel.name,
        logo: xtreamChannel.stream_icon,
        url: xtreamChannel.url || '',
        group_title: 'Xtream Live TV', // Default group for Xtream channels
        tvg_id: xtreamChannel.epg_channel_id,
        resolution: 'HD',
        extra_info: `stream_id:${xtreamChannel.stream_id}` // Store stream_id for URL generation
      }));

      combined.push(...convertedXtreamChannels);
    }

    return combined;
  }, [filteredChannels, activeProfile, displayXtreamChannels]);

  const getTabSubtitle = () => {
    switch (activeTab) {
      case "channels":
        const totalChannels = combinedChannels.length;
        const xtreamCount = activeProfile ? displayXtreamChannels.length : 0;
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
      default:
        return "";
    }
  };

  const renderContent = () => {
    switch (activeTab) {
      case "channels":
        return (
          <>
            <div className="channel-controls">
              {activeProfile && channelCategories.length > 0 && (
                <div className="category-filter">
                  <label htmlFor="channel-category-select">Category:</label>
                  <select
                    id="channel-category-select"
                    value={selectedChannelCategoryId || ''}
                    onChange={(e) => handleChannelCategoryFilter(e.target.value || null)}
                    disabled={isLoadingChannelCategories}
                    aria-label="Filter channels by category"
                  >
                    <option value="">All Categories</option>
                    {channelCategories.map((category) => (
                      <option key={category.category_id} value={category.category_id}>
                        {category.category_name}
                      </option>
                    ))}
                  </select>
                </div>
              )}

              <SearchBar
                value={channelSearchQuery}
                onChange={handleChannelSearchChange}
                placeholder="Search channels..."
                debounceDelay={300}
              />
            </div>

            {selectedChannelCategoryId && (
              <div className="filter-indicator" role="status" aria-live="polite">
                <div className="filter-info">
                  <span className="filter-label">Category:</span>
                  <span className="filter-value">
                    {channelCategories.find(c => c.category_id === selectedChannelCategoryId)?.category_name || selectedChannelCategoryId}
                  </span>
                </div>
                <button
                  className="clear-filter-btn"
                  onClick={() => handleChannelCategoryFilter(null)}
                  aria-label="Clear category filter"
                  title="Clear category filter"
                >
                  <span aria-hidden="true">×</span>
                </button>
              </div>
            )}

            <div className="content-list">
              <VirtualChannelList channels={combinedChannels} onChannelSelect={onChannelSelect} />
            </div>
          </>
        );
      case "favorites":
        return <FavoritesView />;
      case "groups":
        return (
          <div className="content-list">
            <GroupList />
          </div>
        );
      case "history":
        return (
          <div className="content-list">
            <VirtualChannelList channels={history} onChannelSelect={onChannelSelect} />
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
        return <VirtualMovieGrid />;
      case "series":
        if (!activeProfile) {
          return (
            <div className="content-placeholder">
              <h3>No Xtream Profile Selected</h3>
              <p>Please select an Xtream profile to browse TV series.</p>
            </div>
          );
        }
        return <VirtualSeriesBrowser />;

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
