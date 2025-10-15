import { useCallback, useEffect, useMemo, useState } from "react";
import { Virtuoso } from "react-virtuoso";
import { useChannelStore, useProfileStore, useUIStore, useXtreamContentStore } from "../stores";
import type { Channel } from "../types/channel";
import type { EnhancedEPGListing, XtreamChannel } from "../types/types";
import CachedImage from "./CachedImage";

interface VirtualChannelListProps {
  channels?: Channel[];
  useXtreamData?: boolean;
  onChannelSelect?: () => void;
}

export default function VirtualChannelList({ channels, useXtreamData = false, onChannelSelect }: VirtualChannelListProps) {
  const [selectedCategoryId, setSelectedCategoryId] = useState<string | null>(null);
  const [showEPG, setShowEPG] = useState(false);

  const { selectedChannel, setSelectedChannel } = useChannelStore();
  const { focusedIndex, selectedGroup, clearGroupFilter, setFocusedIndex } = useUIStore();

  const {
    channels: xtreamChannels,
    channelCategories,
    filteredChannels,
    isLoadingChannels,
    isLoadingChannelCategories,
    channelsError,
    isSyncing,
    currentAndNextEPG,
    isLoadingEPG,
    favorites: xtreamFavorites,
    fetchChannelCategories,
    fetchChannels,
    fetchCurrentAndNextEPG,
    setSelectedCategory,
    addToFavorites,
    removeFromFavoritesByContent,
    fetchFavorites: fetchXtreamFavorites,
    isFavorite: isXtreamFavorite,
  } = useXtreamContentStore();

  const { activeProfile } = useProfileStore();

  const displayChannels = useMemo(() =>
    useXtreamData
      ? (filteredChannels.length > 0 ? filteredChannels : xtreamChannels)
      : (channels || []),
    [useXtreamData, filteredChannels, xtreamChannels, channels]
  );

  useEffect(() => {
    if (useXtreamData && activeProfile) {
      fetchChannelCategories(activeProfile.id);
      fetchChannels(activeProfile.id);
    }
  }, [useXtreamData, activeProfile, fetchChannelCategories, fetchChannels]);

  // Always fetch favorites if there's an active profile (needed for converted channels too)
  useEffect(() => {
    if (activeProfile) {
      fetchXtreamFavorites(activeProfile.id);
    }
  }, [activeProfile, fetchXtreamFavorites]);

  const isFavorite = useCallback((channel: Channel | XtreamChannel) => {
    if (!activeProfile) return false;

    // Check if it's an Xtream channel (either native or converted)
    const isNativeXtreamChannel = 'stream_id' in channel;
    const isConvertedXtreamChannel = !isNativeXtreamChannel &&
      'extra_info' in channel &&
      channel.extra_info?.startsWith('stream_id:');

    if (!isNativeXtreamChannel && !isConvertedXtreamChannel) return false;

    // Extract stream_id
    let contentId: string;
    if (isNativeXtreamChannel) {
      contentId = (channel as XtreamChannel).stream_id.toString();
    } else {
      contentId = (channel as Channel).extra_info.replace('stream_id:', '');
    }

    const result = isXtreamFavorite(activeProfile.id, 'channel', contentId);
    console.log('isFavorite check:', {
      channelName: channel.name,
      contentId,
      profileId: activeProfile.id,
      result,
      totalFavorites: xtreamFavorites.length
    });
    return result;
  }, [activeProfile, isXtreamFavorite, xtreamFavorites]);

  const handleCategoryFilter = async (categoryId: string | null) => {
    if (!useXtreamData || !activeProfile) return;
    setSelectedCategoryId(categoryId);
    setSelectedCategory(categoryId);
    await fetchChannels(activeProfile.id, categoryId || undefined);
  };

  const handleChannelEPG = async (channel: XtreamChannel) => {
    if (!useXtreamData || !activeProfile) return;
    try {
      await fetchCurrentAndNextEPG(activeProfile.id, channel.stream_id.toString());
    } catch (error) {
      console.error('Failed to fetch EPG for channel:', error);
    }
  };

  const getChannelEPG = (channel: XtreamChannel): { current: EnhancedEPGListing | null, next: EnhancedEPGListing | null } => {
    const channelEPG = currentAndNextEPG[channel.stream_id.toString()];
    return {
      current: channelEPG?.current || null,
      next: channelEPG?.next || null
    };
  };

  const handleToggleFavorite = useCallback(async (channel: Channel | XtreamChannel) => {
    console.log('handleToggleFavorite called', { useXtreamData, activeProfile: activeProfile?.id, channel });

    if (!activeProfile) {
      console.warn('Cannot toggle favorite: no active profile');
      return;
    }

    // Check if it's an Xtream channel (either native XtreamChannel or converted Channel with stream_id)
    const isNativeXtreamChannel = 'stream_id' in channel;
    const isConvertedXtreamChannel = !isNativeXtreamChannel &&
      'extra_info' in channel &&
      channel.extra_info?.startsWith('stream_id:');

    if (!isNativeXtreamChannel && !isConvertedXtreamChannel) {
      console.warn('Cannot toggle favorite: not an Xtream channel');
      return;
    }

    // Extract stream_id
    let contentId: string;
    let channelData: any;

    if (isNativeXtreamChannel) {
      const xtreamChannel = channel as XtreamChannel;
      contentId = xtreamChannel.stream_id.toString();
      channelData = {
        id: contentId,
        name: xtreamChannel.name,
        stream_icon: xtreamChannel.stream_icon,
        category_id: xtreamChannel.category_id,
        epg_channel_id: xtreamChannel.epg_channel_id,
      };
    } else {
      // Converted channel - extract stream_id from extra_info
      const regularChannel = channel as Channel;
      contentId = regularChannel.extra_info.replace('stream_id:', '');
      channelData = {
        id: contentId,
        name: regularChannel.name,
        stream_icon: regularChannel.logo,
        category_id: regularChannel.group_title,
        epg_channel_id: regularChannel.tvg_id,
      };
    }

    const currentlyFavorite = isFavorite(channel);
    console.log('Toggling favorite:', { contentId, currentlyFavorite, isNativeXtreamChannel, isConvertedXtreamChannel });

    try {
      if (currentlyFavorite) {
        console.log('Removing from favorites...');
        await removeFromFavoritesByContent(activeProfile.id, 'channel', contentId);
        console.log('Removed from favorites successfully');
      } else {
        console.log('Adding to favorites...');
        await addToFavorites(
          activeProfile.id,
          'channel',
          contentId,
          channelData
        );
        console.log('Added to favorites successfully');
      }
    } catch (error) {
      console.error('Error toggling favorite:', error);
    }
  }, [activeProfile, isFavorite, addToFavorites, removeFromFavoritesByContent]);

  const handleStartSync = useCallback(async (fullSync: boolean) => {
    if (!activeProfile) return;
    const { startContentSync } = useXtreamContentStore.getState();
    try {
      await startContentSync(activeProfile.id, fullSync);
    } catch (error) {
      console.error('Failed to start sync:', error);
    }
  }, [activeProfile]);

  const rowRenderer = useCallback((index: number) => {
    const channel = displayChannels[index];
    const isXtreamChannel = useXtreamData && 'stream_id' in channel;
    const xtreamChannel = isXtreamChannel ? channel as XtreamChannel : null;
    const epgInfo = xtreamChannel ? getChannelEPG(xtreamChannel) : null;

    return (
      <div
        className={`virtual-channel-item ${selectedChannel?.name === channel.name ? "selected" : ""} ${focusedIndex === index ? "focused" : ""}`}
        onClick={() => {
          // For Xtream channels, convert to Channel format with proper metadata
          // If channel is already converted (has stream_id in extra_info), use as-is
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
          setFocusedIndex(index);
          onChannelSelect?.();
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
            {isXtreamChannel && xtreamChannel!.tv_archive === 1 && (
              <div className="channel-status">
                <span className="archive-indicator" title="Archive Available">📺</span>
              </div>
            )}
          </div>
          <div className="channel-info">
            <div className="channel-header">
              <span className="channel-number">{index + 1}</span>
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
      </div>
    );
  }, [displayChannels, selectedChannel, focusedIndex, showEPG, useXtreamData, channelCategories, isFavorite, handleToggleFavorite]);

  return (
    <div className="virtual-channel-list-container">
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

      {!useXtreamData && selectedGroup && (
        <div className="group-filter-indicator">
          <div className="filter-info">
            <svg className="folder-icon" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
              <path d="M22 19a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h5l2 3h9a2 2 0 0 1 2 2z" />
            </svg>
            <span className="filter-label">Group:</span>
            <span className="filter-value">{selectedGroup}</span>
          </div>
          <button className="clear-filter-btn" onClick={clearGroupFilter} title="Clear group filter">
            <svg className="close-icon" width="12" height="12" viewBox="0 0 24 24" fill="currentColor">
              <path d="M18.3 5.71a.996.996 0 0 0-1.41 0L12 10.59 7.11 5.7A.996.996 0 1 0 5.7 7.11L10.59 12 5.7 16.89a.996.996 0 1 0 1.41 1.41L12 13.41l4.89 4.89a.996.996 0 1 0 1.41-1.41L13.41 12l4.89-4.89c.38-.38.38-1.02 0-1.4z" />
            </svg>
          </button>
        </div>
      )}

      {useXtreamData && selectedCategoryId && (
        <div className="group-filter-indicator">
          <div className="filter-info">
            <svg className="folder-icon" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
              <path d="M22 19a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h5l2 3h9a2 2 0 0 1 2 2z" />
            </svg>
            <span className="filter-label">Category:</span>
            <span className="filter-value">
              {channelCategories.find(c => c.category_id === selectedCategoryId)?.category_name || selectedCategoryId}
            </span>
          </div>
          <button className="clear-filter-btn" onClick={() => handleCategoryFilter(null)} title="Clear category filter">
            <svg className="close-icon" width="12" height="12" viewBox="0 0 24 24" fill="currentColor">
              <path d="M18.3 5.71a.996.996 0 0 0-1.41 0L12 10.59 7.11 5.7A.996.996 0 1 0 5.7 7.11L10.59 12 5.7 16.89a.996.996 0 1 0 1.41 1.41L12 13.41l4.89 4.89a.996.996 0 1 0 1.41-1.41L13.41 12l4.89-4.89c.38-.38.38-1.02 0-1.4z" />
            </svg>
          </button>
        </div>
      )}

      {useXtreamData && isLoadingChannels && (
        <div className="loading-indicator">
          <span>Loading channels...</span>
        </div>
      )}

      {channelsError && !isSyncing && (
        <div className="error-indicator">
          <span>Error loading channels: {channelsError}</span>
          {channelsError.toLowerCase().includes('cache_empty') && activeProfile && (
            <button
              className="btn btn-primary"
              onClick={() => handleStartSync(false)}
              style={{ marginLeft: '1rem' }}
              title="Download channels to cache"
            >
              Download Channels
            </button>
          )}
        </div>
      )}

      {isSyncing && (
        <div className="sync-indicator" role="status" aria-live="polite">
          <span>Syncing content...</span>
        </div>
      )}

      <div className="pagination-info">
        <span className="channel-count">
          {displayChannels.length} channels available
        </span>
      </div>

      <Virtuoso
        style={{ height: '100%' }}
        totalCount={displayChannels.length}
        itemContent={rowRenderer}
        overscan={10}
        className="virtual-channel-list"
      />
    </div>
  );
}
