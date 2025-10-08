import { GroupDisplayMode, useChannelStore, useUIStore } from "../stores";
import type { XtreamMoviesListing, XtreamShow } from "../types/types";
import CachedImage from "./CachedImage";
import { SignalIcon, StarIcon } from "./Icons";
import type { ContentItem } from "./VideoPlayerWrapper";

interface ContentDetailsProps {
  selectedXtreamContent?: ContentItem | null;
}

export default function ContentDetails({ selectedXtreamContent }: ContentDetailsProps) {
  const {
    selectedChannel,
    channels,
    favorites,
    toggleFavorite,
  } = useChannelStore();
  const { setSelectedGroup, setActiveTab, setGroupDisplayMode } = useUIStore();

  // Determine what content to show details for
  const hasContent = selectedChannel || selectedXtreamContent;

  if (!hasContent) {
    return (
      <aside className="channel-details" role="complementary" aria-label="Content details">
        <div className="channel-details-content">
          <p>No content selected</p>
        </div>
      </aside>
    );
  }

  // Handle channel details (existing functionality)
  if (selectedChannel && !selectedXtreamContent) {
    const isFavorite = favorites.some((fav) => fav.name === selectedChannel.name);

    const handleFilterByGroup = () => {
      if (selectedChannel?.group_title) {
        setGroupDisplayMode(GroupDisplayMode.AllGroups);
        setSelectedGroup(selectedChannel.group_title);
        setActiveTab("channels");
      }
    };

    return (
      <aside className="channel-details" role="complementary" aria-label="Channel details">
        <div className="channel-details-content">
          <div className="channel-main-info">
            <CachedImage
              src={selectedChannel.logo}
              alt={`${selectedChannel.name} logo`}
              className="channel-details-logo"
            />
            <div className="channel-meta">
              <div className="channel-title-row">
                <h1 className="channel-details-title">{selectedChannel.name}</h1>
                <span className="channel-number-badge" aria-label={`Channel ${channels.indexOf(selectedChannel) + 1}`}>
                  CH {channels.indexOf(selectedChannel) + 1}
                </span>
              </div>
              <div className="channel-meta-row" aria-label="Channel metadata">
                <div className="meta-item" aria-label={`Resolution ${selectedChannel.resolution || "HD"}`}>
                  <SignalIcon aria-hidden="true" />
                  {selectedChannel.resolution || "HD"}
                </div>
                <div className="meta-item" aria-label="Rating 4.5 out of 5">
                  <StarIcon aria-hidden="true" />
                  4.5
                </div>
                <span className="badge badge-category" aria-label={`Category ${selectedChannel.group_title}`}>
                  {selectedChannel.group_title}
                </span>
              </div>
            </div>
          </div>

          <div className="separator" role="separator" aria-hidden="true"></div>

          <div className="actions-section">
            <button
              className="secondary-button"
              onClick={() => toggleFavorite(selectedChannel)}
              aria-label={isFavorite ? `Remove ${selectedChannel.name} from favorites` : `Add ${selectedChannel.name} to favorites`}
            >
              {isFavorite ? "Remove from Favorites" : "Add to Favorites"}
            </button>
          </div>

          <div className="details-grid">
            <div className="detail-item">
              <div className="detail-label">Group</div>
              <div className="detail-value-with-action">
                <span className="detail-text">{selectedChannel.group_title}</span>
                <button
                  className="detail-action-button"
                  onClick={handleFilterByGroup}
                  title="Filter channels by this group"
                >
                  <svg
                    width="12"
                    height="12"
                    viewBox="0 0 24 24"
                    fill="currentColor"
                  >
                    <path d="M10 18h4v-2h-4v2zM3 6v2h18V6H3zm3 7h12v-2H6v2z" />
                  </svg>
                  Filter
                </button>
              </div>
            </div>
            <div className="detail-item">
              <div className="detail-label">TVG ID</div>
              <div className="detail-value">
                {selectedChannel.tvg_id || "N/A"}
              </div>
            </div>
            <div className="detail-item">
              <div className="detail-label">Resolution</div>
              <div className="detail-value">
                {selectedChannel.resolution || "HD"}
              </div>
            </div>
            <div className="detail-item">
              <div className="detail-label">Extra Info</div>
              <div className="detail-value">
                {selectedChannel.extra_info || "No additional information"}
              </div>
            </div>
          </div>
        </div>
      </aside>
    );
  }

  // Handle Xtream content details (movies and series)
  if (selectedXtreamContent) {
    const getContentTitle = (): string => {
      switch (selectedXtreamContent.type) {
        case 'xtream-movie':
          const movie = selectedXtreamContent.data as XtreamMoviesListing;
          return movie.title || movie.name;
        case 'xtream-series':
          const series = selectedXtreamContent.data as XtreamShow;
          return selectedXtreamContent.metadata?.title || series.info?.title || series.info?.name || '';
        default:
          return '';
      }
    };

    const getContentImage = (): string => {
      switch (selectedXtreamContent.type) {
        case 'xtream-movie':
          const movie = selectedXtreamContent.data as XtreamMoviesListing;
          return movie.stream_icon;
        case 'xtream-series':
          const series = selectedXtreamContent.data as XtreamShow;
          return series.info?.cover || '';
        default:
          return '';
      }
    };

    const getContentType = (): string => {
      switch (selectedXtreamContent.type) {
        case 'xtream-movie':
          return 'Movie';
        case 'xtream-series':
          return `Episode ${selectedXtreamContent.metadata?.episodeNumber || ''} - Season ${selectedXtreamContent.metadata?.seasonNumber || ''}`;
        default:
          return '';
      }
    };

    const formatRating = (rating?: number): string => {
      if (!rating || rating === 0) return 'N/A';
      return rating.toString();
    };

    const formatDuration = (duration?: number): string => {
      if (!duration) return 'N/A';
      const hours = Math.floor(duration / 60);
      const minutes = duration % 60;
      if (hours > 0) {
        return `${hours}h ${minutes}m`;
      }
      return `${minutes}m`;
    };

    return (
      <div className="channel-details">
        <div className="channel-details-content">
          <div className="channel-main-info">
            <CachedImage
              src={getContentImage()}
              alt={getContentTitle()}
              className="channel-details-logo"
            />
            <div className="channel-meta">
              <div className="channel-title-row">
                <h1 className="channel-details-title">{getContentTitle()}</h1>
                <span className="channel-number-badge">
                  {getContentType()}
                </span>
              </div>
              <div className="channel-meta-row">
                {selectedXtreamContent.metadata?.rating && (
                  <div className="meta-item">
                    <StarIcon />
                    {formatRating(selectedXtreamContent.metadata.rating)}
                  </div>
                )}
                {selectedXtreamContent.metadata?.duration && (
                  <div className="meta-item">
                    <SignalIcon />
                    {formatDuration(selectedXtreamContent.metadata.duration)}
                  </div>
                )}
                {selectedXtreamContent.metadata?.genre && (
                  <span className="badge badge-category">
                    {selectedXtreamContent.metadata.genre}
                  </span>
                )}
              </div>
            </div>
          </div>

          {selectedXtreamContent.metadata?.description && (
            <>
              <div className="separator"></div>
              <div className="content-description">
                <h3>Description</h3>
                <p>{selectedXtreamContent.metadata.description}</p>
              </div>
            </>
          )}

          <div className="separator"></div>

          <div className="details-grid">
            {selectedXtreamContent.metadata?.year && (
              <div className="detail-item">
                <div className="detail-label">Year</div>
                <div className="detail-value">
                  {selectedXtreamContent.metadata.year}
                </div>
              </div>
            )}
            {selectedXtreamContent.metadata?.genre && (
              <div className="detail-item">
                <div className="detail-label">Genre</div>
                <div className="detail-value">
                  {selectedXtreamContent.metadata.genre}
                </div>
              </div>
            )}
            {selectedXtreamContent.metadata?.director && (
              <div className="detail-item">
                <div className="detail-label">Director</div>
                <div className="detail-value">
                  {selectedXtreamContent.metadata.director}
                </div>
              </div>
            )}
            {selectedXtreamContent.metadata?.cast && (
              <div className="detail-item">
                <div className="detail-label">Cast</div>
                <div className="detail-value">
                  {selectedXtreamContent.metadata.cast}
                </div>
              </div>
            )}
            {selectedXtreamContent.type === 'xtream-series' && selectedXtreamContent.metadata?.seasonNumber && (
              <div className="detail-item">
                <div className="detail-label">Season</div>
                <div className="detail-value">
                  {selectedXtreamContent.metadata.seasonNumber}
                </div>
              </div>
            )}
            {selectedXtreamContent.type === 'xtream-series' && selectedXtreamContent.metadata?.episodeNumber && (
              <div className="detail-item">
                <div className="detail-label">Episode</div>
                <div className="detail-value">
                  {selectedXtreamContent.metadata.episodeNumber}
                </div>
              </div>
            )}
          </div>
        </div>
      </div>
    );
  }

  return null;
}