import { useEffect, useRef, useState } from "react";
import { useChannelStore, useProfileStore, useUIStore, useXtreamContentStore } from "../stores";
import type { EnhancedEPGListing, XtreamChannel } from "../types/types";
import CachedImage from "./CachedImage";

export interface Channel {
  name: string;
  logo: string;
  url: string;
  group_title: string;
  tvg_id: string;
  resolution: string;
  extra_info: string;
}

interface ChannelListProps {
  channels?: Channel[];
  useXtreamData?: boolean;
}

const CHANNELS_PER_PAGE = 200; // Reasonable number for performance

export default function ChannelList({ channels, useXtreamData = false }: ChannelListProps) {
  const [currentPage, setCurrentPage] = useState(1);
  const [selectedCategoryId, setSelectedCategoryId] = useState<string | null>(null);
  const [showEPG, setShowEPG] = useState(false);
  const channelListRef = useRef<HTMLUListElement>(null);

  // Get state and actions from stores
  const { selectedChannel, favorites, setSelectedChannel, toggleFavorite } =
    useChannelStore();

  const { focusedIndex, selectedGroup, clearGroupFilter, setFocusedIndex } =
    useUIStore();

  // Xtream-specific stores
  const {
    channels: xtreamChannels,
    channelCategories,
    filteredChannels,
    isLoadingChannels,
    isLoadingChannelCategories,
    channelsError,
    currentAndNextEPG,
    isLoadingEPG,
    fetchChannelCategories,
    fetchChannels,
    fetchCurrentAndNextEPG,
    setSelectedCategory
  } = useXtreamContentStore();

  const { activeProfile } = useProfileStore();

  // Determine which channels to use
  const displayChannels = useXtreamData
    ? (filteredChannels.length > 0 ? filteredChannels : xtreamChannels)
    : (channels || []);

  // Load Xtream data when component mounts or profile changes
  useEffect(() => {
    if (useXtreamData && activeProfile) {
      fetchChannelCategories(activeProfile.id);
      fetchChannels(activeProfile.id);
    }
  }, [useXtreamData, activeProfile, fetchChannelCategories, fetchChannels]);

  // Reset to first page when channels change
  useEffect(() => {
    setCurrentPage(1);
  }, [displayChannels.length, selectedGroup, selectedCategoryId]);

  // Handle pagination and scrolling when focusedIndex changes
  useEffect(() => {
    if (displayChannels.length === 0) return;

    const requiredPage = Math.ceil((focusedIndex + 1) / CHANNELS_PER_PAGE);

    // Change page if focused item is on a different page
    if (requiredPage !== currentPage) {
      setCurrentPage(requiredPage);
    }

    // Scroll focused item into view
    const focusedElement = channelListRef.current?.querySelector(
      ".channel-item.focused",
    );
    if (focusedElement) {
      focusedElement.scrollIntoView({
        behavior: "smooth",
        block: "center",
        inline: "nearest",
      });
    }
  }, [focusedIndex, displayChannels.length, currentPage]);

  const totalPages = Math.ceil(displayChannels.length / CHANNELS_PER_PAGE);
  const startIndex = (currentPage - 1) * CHANNELS_PER_PAGE;
  const endIndex = startIndex + CHANNELS_PER_PAGE;
  const currentChannels = displayChannels.slice(startIndex, endIndex);

  const isFavorite = (channel: Channel | XtreamChannel) => {
    return favorites.some((fav) => fav.name === channel.name);
  };

  // Handle category filtering for Xtream channels
  const handleCategoryFilter = async (categoryId: string | null) => {
    if (!useXtreamData || !activeProfile) return;

    setSelectedCategoryId(categoryId);
    setSelectedCategory(categoryId);

    if (categoryId) {
      await fetchChannels(activeProfile.id, categoryId);
    } else {
      await fetchChannels(activeProfile.id);
    }
  };

  // Handle EPG data fetching for a channel
  const handleChannelEPG = async (channel: XtreamChannel) => {
    if (!useXtreamData || !activeProfile) return;

    try {
      await fetchCurrentAndNextEPG(activeProfile.id, channel.stream_id.toString());
    } catch (error) {
      console.error('Failed to fetch EPG for channel:', error);
    }
  };

  // Get EPG data for a channel
  const getChannelEPG = (channel: XtreamChannel): { current: EnhancedEPGListing | null, next: EnhancedEPGListing | null } => {
    const channelEPG = currentAndNextEPG[channel.stream_id.toString()];
    return {
      current: channelEPG?.current || null,
      next: channelEPG?.next || null
    };
  };

  const handlePageChange = (page: number) => {
    setCurrentPage(page);
  };

  const handleToggleFavorite = async (channel: Channel | XtreamChannel) => {
    // Convert XtreamChannel to Channel format for compatibility
    const channelToToggle: Channel = useXtreamData && 'stream_id' in channel ? {
      name: channel.name,
      logo: channel.stream_icon,
      url: channel.url || '',
      group_title: '', // Will be populated from category
      tvg_id: channel.epg_channel_id,
      resolution: 'HD', // Default for Xtream channels
      extra_info: ''
    } : channel as Channel;

    await toggleFavorite(channelToToggle);
  };

  const getPageNumbers = () => {
    const pages = [];
    const maxVisiblePages = 5;

    let startPage = Math.max(1, currentPage - Math.floor(maxVisiblePages / 2));
    let endPage = Math.min(totalPages, startPage + maxVisiblePages - 1);

    // Adjust startPage if we're near the end
    if (endPage - startPage < maxVisiblePages - 1) {
      startPage = Math.max(1, endPage - maxVisiblePages + 1);
    }

    for (let i = startPage; i <= endPage; i++) {
      pages.push(i);
    }

    return pages;
  };

  return (
    <div className="channel-list-container">
      {/* Xtream Category Filter */}
      {useXtreamData && (
        <div className="xtream-controls">
          <div className="category-filter">
            <label htmlFor="category-select">Category:</label>
            <select
              id="category-select"
              value={selectedCategoryId || ''}
              onChange={(e) => handleCategoryFilter(e.target.value || null)}
              disabled={isLoadingChannelCategories}
            >
              <option value="">All Categories</option>
              {channelCategories.map((category) => (
                <option key={category.category_id} value={category.category_id}>
                  {category.category_name}
                </option>
              ))}
            </select>
          </div>

          <div className="epg-controls">
            <label>
              <input
                type="checkbox"
                checked={showEPG}
                onChange={(e) => setShowEPG(e.target.checked)}
              />
              Show EPG Info
            </label>
          </div>
        </div>
      )}

      {/* Group Filter Indicator (for legacy channels) */}
      {!useXtreamData && selectedGroup && (
        <div className="group-filter-indicator">
          <div className="filter-info">
            <svg
              className="folder-icon"
              width="16"
              height="16"
              viewBox="0 0 24 24"
              fill="none"
              stroke="currentColor"
              strokeWidth="2"
              strokeLinecap="round"
              strokeLinejoin="round"
            >
              <path d="M22 19a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h5l2 3h9a2 2 0 0 1 2 2z" />
            </svg>
            <span className="filter-label">Group:</span>
            <span className="filter-value">{selectedGroup}</span>
          </div>
          <button
            className="clear-filter-btn"
            onClick={clearGroupFilter}
            title="Clear group filter"
          >
            <svg
              className="close-icon"
              width="12"
              height="12"
              viewBox="0 0 24 24"
              fill="currentColor"
              stroke="none"
            >
              <path d="M18.3 5.71a.996.996 0 0 0-1.41 0L12 10.59 7.11 5.7A.996.996 0 1 0 5.7 7.11L10.59 12 5.7 16.89a.996.996 0 1 0 1.41 1.41L12 13.41l4.89 4.89a.996.996 0 1 0 1.41-1.41L13.41 12l4.89-4.89c.38-.38.38-1.02 0-1.4z" />
            </svg>
          </button>
        </div>
      )}

      {/* Category Filter Indicator (for Xtream channels) */}
      {useXtreamData && selectedCategoryId && (
        <div className="group-filter-indicator">
          <div className="filter-info">
            <svg
              className="folder-icon"
              width="16"
              height="16"
              viewBox="0 0 24 24"
              fill="none"
              stroke="currentColor"
              strokeWidth="2"
              strokeLinecap="round"
              strokeLinejoin="round"
            >
              <path d="M22 19a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h5l2 3h9a2 2 0 0 1 2 2z" />
            </svg>
            <span className="filter-label">Category:</span>
            <span className="filter-value">
              {channelCategories.find(c => c.category_id === selectedCategoryId)?.category_name || selectedCategoryId}
            </span>
          </div>
          <button
            className="clear-filter-btn"
            onClick={() => handleCategoryFilter(null)}
            title="Clear category filter"
          >
            <svg
              className="close-icon"
              width="12"
              height="12"
              viewBox="0 0 24 24"
              fill="currentColor"
              stroke="none"
            >
              <path d="M18.3 5.71a.996.996 0 0 0-1.41 0L12 10.59 7.11 5.7A.996.996 0 1 0 5.7 7.11L10.59 12 5.7 16.89a.996.996 0 1 0 1.41 1.41L12 13.41l4.89 4.89a.996.996 0 1 0 1.41-1.41L13.41 12l4.89-4.89c.38-.38.38-1.02 0-1.4z" />
            </svg>
          </button>
        </div>
      )}

      {/* Loading State */}
      {useXtreamData && isLoadingChannels && (
        <div className="loading-indicator">
          <span>Loading channels...</span>
        </div>
      )}

      {/* Error State */}
      {useXtreamData && channelsError && (
        <div className="error-indicator">
          <span>Error loading channels: {channelsError}</span>
        </div>
      )}

      {/* Pagination Info */}
      <div className="pagination-info">
        <span className="channel-count">
          Showing {startIndex + 1}-{Math.min(endIndex, displayChannels.length)} of{" "}
          {displayChannels.length} channels
          {totalPages > 1 && ` (Page ${currentPage} of ${totalPages})`}
        </span>
      </div>

      <ul className="channel-list" ref={channelListRef}>
        {currentChannels.map((channel, index) => {
          const globalIndex = startIndex + index;
          const isXtreamChannel = useXtreamData && 'stream_id' in channel;
          const xtreamChannel = isXtreamChannel ? channel as XtreamChannel : null;
          const epgInfo = xtreamChannel ? getChannelEPG(xtreamChannel) : null;

          return (
            <li
              key={`${channel.name}-${globalIndex}`}
              className={`channel-item ${selectedChannel?.name === channel.name ? "selected" : ""} ${focusedIndex === globalIndex ? "focused" : ""}`}
              onClick={() => {
                // For Xtream channels, convert to Channel format with proper metadata
                const channelToSelect: Channel = isXtreamChannel ? {
                  name: xtreamChannel!.name,
                  logo: xtreamChannel!.stream_icon,
                  url: '', // URL will be generated by ModernVideoPlayer
                  group_title: xtreamChannel!.category_id,
                  tvg_id: xtreamChannel!.epg_channel_id,
                  resolution: 'HD',
                  extra_info: `stream_id:${xtreamChannel!.stream_id}` // Store stream_id for URL generation
                } : channel as Channel;

                setSelectedChannel(channelToSelect);
                setFocusedIndex(globalIndex);

                // Fetch EPG data for Xtream channels if EPG is enabled
                if (showEPG && xtreamChannel) {
                  handleChannelEPG(xtreamChannel);
                }
              }}
            >
              <div className="channel-content">
                <div className="channel-logo-container">
                  <CachedImage
                    src={isXtreamChannel ? xtreamChannel!.stream_icon : (channel as Channel).logo}
                    alt={channel.name}
                    className="channel-logo"
                  />
                  <div className="channel-status">
                    {isXtreamChannel && xtreamChannel!.tv_archive === 1 && (
                      <span className="archive-indicator" title="Archive Available">📺</span>
                    )}
                  </div>
                </div>
                <div className="channel-info">
                  <div className="channel-header">
                    <span className="channel-number">{globalIndex + 1}</span>
                    <span className="channel-name">{channel.name}</span>
                    {isXtreamChannel && (
                      <span className="stream-id">ID: {xtreamChannel!.stream_id}</span>
                    )}
                  </div>
                  <div className="channel-badges">
                    <span className="badge badge-category">
                      {isXtreamChannel
                        ? channelCategories.find(c => c.category_id === xtreamChannel!.category_id)?.category_name || 'Unknown'
                        : (channel as Channel).group_title
                      }
                    </span>
                    <span className="badge badge-quality">
                      {isXtreamChannel ? "HD" : (channel as Channel).resolution || "HD"}
                    </span>
                    {isXtreamChannel && xtreamChannel!.tv_archive === 1 && (
                      <span className="badge badge-archive">Archive</span>
                    )}
                  </div>

                  {/* EPG Information for Xtream channels */}
                  {showEPG && isXtreamChannel && epgInfo && (
                    <div className="channel-epg">
                      {epgInfo.current && (
                        <div className="epg-current">
                          <span className="epg-label">Now:</span>
                          <span className="epg-title">{epgInfo.current.title}</span>
                          {epgInfo.current.formatted_start && epgInfo.current.formatted_stop && (
                            <span className="epg-time">
                              {epgInfo.current.formatted_start} - {epgInfo.current.formatted_stop}
                            </span>
                          )}
                        </div>
                      )}
                      {epgInfo.next && (
                        <div className="epg-next">
                          <span className="epg-label">Next:</span>
                          <span className="epg-title">{epgInfo.next.title}</span>
                          {epgInfo.next.formatted_start && (
                            <span className="epg-time">at {epgInfo.next.formatted_start}</span>
                          )}
                        </div>
                      )}
                      {!epgInfo.current && !epgInfo.next && isLoadingEPG && (
                        <div className="epg-loading">Loading EPG...</div>
                      )}
                    </div>
                  )}

                  <div className="channel-group">
                    {isXtreamChannel ? xtreamChannel!.epg_channel_id : (channel as Channel).extra_info}
                  </div>
                </div>
                <div className="channel-actions">
                  <button
                    className={`action-button ${isFavorite(channel) ? "favorite" : ""}`}
                    onClick={(e) => {
                      e.stopPropagation();
                      handleToggleFavorite(channel);
                    }}
                  >
                    {isFavorite(channel) ? "★" : "☆"}
                  </button>
                </div>
              </div>
            </li>
          );
        })}
      </ul>

      {/* Pagination Controls */}
      {totalPages > 1 && (
        <div className="pagination-controls">
          <button
            className="pagination-btn"
            onClick={() => handlePageChange(1)}
            disabled={currentPage === 1}
            title="First page"
          >
            ««
          </button>
          <button
            className="pagination-btn"
            onClick={() => handlePageChange(currentPage - 1)}
            disabled={currentPage === 1}
            title="Previous page"
          >
            ‹
          </button>

          {getPageNumbers().map((page) => (
            <button
              key={page}
              className={`pagination-btn ${page === currentPage ? "active" : ""}`}
              onClick={() => handlePageChange(page)}
            >
              {page}
            </button>
          ))}

          <button
            className="pagination-btn"
            onClick={() => handlePageChange(currentPage + 1)}
            disabled={currentPage === totalPages}
            title="Next page"
          >
            ›
          </button>
          <button
            className="pagination-btn"
            onClick={() => handlePageChange(totalPages)}
            disabled={currentPage === totalPages}
            title="Last page"
          >
            »»
          </button>
        </div>
      )}
    </div>
  );
}
