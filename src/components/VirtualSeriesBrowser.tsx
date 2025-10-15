import { useCallback, useEffect, useMemo, useState } from "react";
import { Virtuoso } from "react-virtuoso";
import { useNormalizedSeriesDetails } from "../hooks/useNormalizedSeriesDetails";
import { useProfileStore } from "../stores/profileStore";
import { useXtreamContentStore } from "../stores/xtreamContentStore";
import { XtreamEpisode, XtreamSeason, XtreamShow, XtreamShowListing } from "../types/types";
import { formatEpisodeRuntime, formatRating, formatYear } from "../utils/formatters";
import CachedImage from "./CachedImage";
import EmptyState from "./EmptyState";
import SearchBar from "./SearchBar";
import { SkeletonEpisodeList, SkeletonMovieGrid } from "./SkeletonLoader";

// Heart icon component
const HeartIcon = ({ filled }: { filled: boolean }) => (
  <svg className="w-5 h-5" fill={filled ? "currentColor" : "none"} stroke="currentColor" strokeWidth="2" viewBox="0 0 24 24">
    <path d="M20.84 4.61a5.5 5.5 0 0 0-7.78 0L12 5.67l-1.06-1.06a5.5 5.5 0 0 0-7.78 7.78l1.06 1.06L12 21.23l7.78-7.78 1.06-1.06a5.5 5.5 0 0 0 0-7.78z" />
  </svg>
);

interface VirtualSeriesBrowserProps {
  onEpisodePlay?: (episode: XtreamEpisode, series: XtreamShow) => void;
  onContentSelect?: () => void;
  currentEpisode?: XtreamEpisode | null;
  onGetNextEpisode?: (currentEpisode: XtreamEpisode, series: XtreamShow) => { episode: XtreamEpisode; series: XtreamShow } | null;
}

export default function VirtualSeriesBrowser({ onEpisodePlay, onContentSelect }: VirtualSeriesBrowserProps) {
  const [selectedCategoryId, setSelectedCategoryId] = useState<string | null>(null);
  const [selectedSeries, setSelectedSeries] = useState<XtreamShowListing | null>(null);
  const [seriesDetails, setSeriesDetails] = useState<XtreamShow | null>(null);
  const [selectedSeason, setSelectedSeason] = useState<XtreamSeason | null>(null);
  const [searchQuery, setSearchQuery] = useState("");
  const [viewMode, setViewMode] = useState<'grid' | 'details'>('grid');

  // Normaliza os detalhes da série para garantir estrutura consistente
  const normalizedSeriesDetails = useNormalizedSeriesDetails(seriesDetails);

  const {
    series,
    seriesCategories,
    filteredSeries,
    isLoadingSeries,
    isLoadingSeriesCategories,
    seriesError,
    isSyncing,
    fetchSeriesCategories,
    fetchSeries,
    fetchSeriesDetails,
    searchSeries,
    setSelectedCategory,
    addToFavorites,
    removeFromFavoritesByContent,
    isFavorite,
    fetchFavorites
  } = useXtreamContentStore();

  const { activeProfile } = useProfileStore();

  const displaySeries = useMemo(() =>
    filteredSeries.length > 0 ? filteredSeries : series,
    [filteredSeries, series]
  );

  useEffect(() => {
    if (!activeProfile) return;

    fetchSeriesCategories(activeProfile.id);
    fetchSeries(activeProfile.id);
    fetchFavorites(activeProfile.id);
  }, [activeProfile, fetchSeriesCategories, fetchSeries, fetchFavorites]);

  useEffect(() => {
    if (normalizedSeriesDetails?.seasons && normalizedSeriesDetails.seasons.length > 0) {
      setSelectedSeason(normalizedSeriesDetails.seasons[0]);
    }
  }, [normalizedSeriesDetails]);

  const handleCategoryFilter = useCallback(async (categoryId: string | null) => {
    if (!activeProfile) return;

    setSelectedCategoryId(categoryId);
    setSelectedCategory(categoryId);
    setSearchQuery("");
    await fetchSeries(activeProfile.id, categoryId || undefined);
  }, [activeProfile, setSelectedCategory, fetchSeries]);

  const handleSearchChange = useCallback(async (query: string) => {
    if (!activeProfile) return;

    setSearchQuery(query);

    if (query.trim()) {
      await searchSeries(activeProfile.id, query);
    } else {
      await fetchSeries(activeProfile.id, selectedCategoryId || undefined);
    }
  }, [activeProfile, selectedCategoryId, searchSeries, fetchSeries]);

  const handleSeriesClick = useCallback(async (seriesItem: XtreamShowListing) => {
    setSelectedSeries(seriesItem);
    onContentSelect?.();

    if (!activeProfile) return;

    try {
      const details = await fetchSeriesDetails(activeProfile.id, seriesItem.series_id.toString());
      setSeriesDetails(details);
      setViewMode('details');
    } catch (error) {
      console.error('Failed to fetch series details:', error);
    }
  }, [activeProfile, fetchSeriesDetails, onContentSelect]);

  const handleEpisodePlay = useCallback((episode: XtreamEpisode) => {
    if (seriesDetails) {
      onContentSelect?.();
      onEpisodePlay?.(episode, seriesDetails);
    }
  }, [seriesDetails, onContentSelect, onEpisodePlay]);

  const handleBackToGrid = useCallback(() => {
    setViewMode('grid');
    setSelectedSeries(null);
    setSeriesDetails(null);
    setSelectedSeason(null);
  }, []);

  const handleToggleFavorite = useCallback(async (seriesItem: XtreamShowListing, e: React.MouseEvent) => {
    e.stopPropagation();
    if (!activeProfile) return;

    const seriesId = seriesItem.series_id.toString();
    const isCurrentlyFavorite = isFavorite(activeProfile.id, 'series', seriesId);

    try {
      if (isCurrentlyFavorite) {
        await removeFromFavoritesByContent(activeProfile.id, 'series', seriesId);
      } else {
        await addToFavorites(activeProfile.id, 'series', seriesId, seriesItem);
      }
    } catch (error) {
      const errorMessage = error as string;
      console.error('Failed to toggle favorite:', error);

      // If the error is "already in favorites", try to remove it instead
      if (errorMessage.includes('already in favorites')) {
        try {
          await removeFromFavoritesByContent(activeProfile.id, 'series', seriesId);
        } catch (removeError) {
          console.error('Failed to remove favorite:', removeError);
        }
      }
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

  // Formatting functions are now imported from utils/formatters

  const seasonEpisodes = useMemo(() => {
    if (!normalizedSeriesDetails || !selectedSeason) return [];
    return normalizedSeriesDetails.episodes[selectedSeason.season_number.toString()] || [];
  }, [normalizedSeriesDetails, selectedSeason]);

  const rowRenderer = useCallback((index: number) => {
    const startIdx = index * 6;
    const endIdx = Math.min(startIdx + 6, displaySeries.length);
    const rowSeries = displaySeries.slice(startIdx, endIdx);

    return (
      <div className="virtual-series-row" role="list">
        {rowSeries.map((seriesItem) => (
          <article
            key={seriesItem.series_id}
            className={`virtual-series-card ${selectedSeries?.series_id === seriesItem.series_id ? 'selected' : ''}`}
            onClick={() => handleSeriesClick(seriesItem)}
            role="listitem"
            tabIndex={0}
            onKeyDown={(e) => {
              if (e.key === 'Enter' || e.key === ' ') {
                e.preventDefault();
                handleSeriesClick(seriesItem);
              }
            }}
            aria-label={`${seriesItem.name}, ${formatYear(seriesItem.year)}, Rating ${formatRating(seriesItem.rating)}`}
          >
            <div className="series-poster-container">
              <CachedImage
                src={seriesItem.cover}
                alt={`${seriesItem.name} poster`}
                className="series-poster"
                lazy={true}
                rootMargin="200px"
              />
              <div className="series-overlay" aria-hidden="true">
                <button
                  className="view-button"
                  onClick={(e) => {
                    e.stopPropagation();
                    handleSeriesClick(seriesItem);
                  }}
                  aria-label={`View ${seriesItem.name} details`}
                  title={`View ${seriesItem.name} details`}
                >
                  <span aria-hidden="true">👁</span>
                </button>
                <button
                  className={`favorite-button ${activeProfile && isFavorite(activeProfile.id, 'series', seriesItem.series_id.toString()) ? 'active' : ''}`}
                  onClick={(e) => handleToggleFavorite(seriesItem, e)}
                  aria-label={activeProfile && isFavorite(activeProfile.id, 'series', seriesItem.series_id.toString()) ? `Remove ${seriesItem.name} from favorites` : `Add ${seriesItem.name} to favorites`}
                  title={activeProfile && isFavorite(activeProfile.id, 'series', seriesItem.series_id.toString()) ? "Remove from favorites" : "Add to favorites"}
                >
                  <HeartIcon filled={activeProfile ? isFavorite(activeProfile.id, 'series', seriesItem.series_id.toString()) : false} />
                </button>
              </div>
            </div>

            <div className="series-info">
              <h3 className="series-title">{seriesItem.name}</h3>
              <div className="series-meta">
                {seriesItem.year && <span className="series-year">{formatYear(seriesItem.year)}</span>}
                {seriesItem.rating && seriesItem.rating !== '0' && (
                  <span className="series-rating">★ {formatRating(seriesItem.rating)}</span>
                )}
              </div>
            </div>
          </article>
        ))}
      </div>
    );
  }, [displaySeries, selectedSeries, handleSeriesClick]);

  const totalRows = Math.ceil(displaySeries.length / 6);

  if (viewMode === 'details' && normalizedSeriesDetails && selectedSeries) {
    return (
      <div className="virtual-series-details-container">
        {/* Hero Section with Background */}
        <div className="series-hero-section">
          <div className="series-hero-backdrop">
            <CachedImage
              src={selectedSeries.cover}
              alt=""
              className="series-backdrop-image"
            />
            <div className="series-hero-overlay"></div>
          </div>

          <div className="series-hero-content">
            <button className="back-button-hero" onClick={handleBackToGrid}>
              <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
                <path d="M19 12H5M12 19l-7-7 7-7" />
              </svg>
              Back
            </button>

            <div className="series-hero-info">
              <div className="series-poster-compact">
                <CachedImage src={selectedSeries.cover} alt={selectedSeries.name} className="series-poster-image" />
              </div>

              <div className="series-hero-details">
                <h1 className="series-hero-title">{selectedSeries.name}</h1>

                <div className="series-hero-meta">
                  {selectedSeries.year && <span className="meta-badge">{formatYear(selectedSeries.year)}</span>}
                  {selectedSeries.rating && selectedSeries.rating !== '0' && (
                    <span className="meta-badge rating">★ {formatRating(selectedSeries.rating)}</span>
                  )}
                  {selectedSeries.genre && <span className="meta-badge genre">{selectedSeries.genre}</span>}
                  {selectedSeries.episode_run_time && <span className="meta-badge">{formatEpisodeRuntime(selectedSeries.episode_run_time)}</span>}
                </div>

                {selectedSeries.plot && (
                  <p className="series-hero-plot">{selectedSeries.plot}</p>
                )}

                {(selectedSeries.director || selectedSeries.cast) && (
                  <div className="series-hero-credits">
                    {selectedSeries.director && (
                      <div className="credit-item">
                        <span className="credit-label">Director:</span>
                        <span className="credit-value">{selectedSeries.director}</span>
                      </div>
                    )}
                    {selectedSeries.cast && (
                      <div className="credit-item">
                        <span className="credit-label">Cast:</span>
                        <span className="credit-value">{selectedSeries.cast}</span>
                      </div>
                    )}
                  </div>
                )}

                <div className="series-hero-actions">
                  <button
                    className={`favorite-button-hero ${activeProfile && isFavorite(activeProfile.id, 'series', selectedSeries.series_id.toString()) ? 'active' : ''}`}
                    onClick={(e) => handleToggleFavorite(selectedSeries, e)}
                  >
                    <HeartIcon filled={activeProfile ? isFavorite(activeProfile.id, 'series', selectedSeries.series_id.toString()) : false} />
                    {activeProfile && isFavorite(activeProfile.id, 'series', selectedSeries.series_id.toString()) ? 'Remove from Favorites' : 'Add to Favorites'}
                  </button>
                </div>
              </div>
            </div>
          </div>
        </div>

        {/* Seasons & Episodes Section */}
        <div className="series-content-section">
          <div className="season-selector">
            <h2 className="section-title">Episodes</h2>
            {normalizedSeriesDetails.seasons.length > 0 && (
              <div className="season-dropdown-wrapper">
                <select
                  className="season-dropdown"
                  value={selectedSeason?.season_number?.toString() || ''}
                  onChange={(e) => {
                    const selectedValue = e.target.value;
                    const season = normalizedSeriesDetails.seasons.find(s => s.season_number.toString() === selectedValue);
                    if (season) {
                      setSelectedSeason(season);
                    }
                  }}
                >
                  {normalizedSeriesDetails.seasons.map((season) => (
                    <option key={season.season_number} value={season.season_number}>
                      Season {season.season_number} ({season.episode_count} episodes)
                    </option>
                  ))}
                </select>
              </div>
            )}
          </div>

          {selectedSeason && seasonEpisodes.length === 0 ? (
            <SkeletonEpisodeList count={8} />
          ) : selectedSeason && (
            <div className="episodes-grid">
              <Virtuoso
                style={{ height: '400px' }}
                totalCount={seasonEpisodes.length}
                itemContent={(index) => {
                  const episode = seasonEpisodes[index];
                  return (
                    <div className="episode-card" onClick={() => handleEpisodePlay(episode)}>
                      <div className="episode-card-thumbnail">
                        <CachedImage
                          src={episode.info.movie_image || selectedSeries.cover}
                          alt={episode.title}
                          className="episode-card-image"
                          lazy={true}
                          rootMargin="100px"
                        />
                        <div className="episode-card-overlay">
                          <button
                            className="episode-card-play"
                            onClick={(e) => {
                              e.stopPropagation();
                              handleEpisodePlay(episode);
                            }}
                            aria-label="Play episode"
                          >
                            ▶
                          </button>
                        </div>
                        <div className="episode-card-number">{episode.episode_num}</div>
                      </div>

                      <div className="episode-card-content">
                        <h3 className="episode-card-title">{episode.title}</h3>
                        <div className="episode-card-meta">
                          {episode.info.duration && <span className="episode-card-duration">{episode.info.duration}</span>}
                          {episode.info.rating && <span className="episode-card-rating">★ {episode.info.rating}</span>}
                        </div>
                        {episode.info.plot && <p className="episode-card-plot">{episode.info.plot}</p>}
                      </div>
                    </div>
                  );
                }}
                overscan={2}
                className="episodes-list"
              />
            </div>
          )}
        </div>
      </div>
    );
  }

  return (
    <div className="virtual-series-browser-container">
      <div className="series-controls">
        <div className="category-filter">
          <label htmlFor="series-category-select">Category:</label>
          <select
            id="series-category-select"
            value={selectedCategoryId || ''}
            onChange={(e) => handleCategoryFilter(e.target.value || null)}
            disabled={isLoadingSeriesCategories}
          >
            <option value="">All Categories</option>
            {seriesCategories.map((category) => (
              <option key={category.category_id} value={category.category_id}>
                {category.category_name}
              </option>
            ))}
          </select>
        </div>

        <SearchBar
          value={searchQuery}
          onChange={handleSearchChange}
          placeholder="Search series..."
          debounceDelay={300}
        />
      </div>

      {selectedCategoryId && (
        <div className="filter-indicator">
          <div className="filter-info">
            <span className="filter-label">Category:</span>
            <span className="filter-value">
              {seriesCategories.find(c => c.category_id === selectedCategoryId)?.category_name || selectedCategoryId}
            </span>
          </div>
          <button className="clear-filter-btn" onClick={() => handleCategoryFilter(null)} title="Clear category filter">
            ×
          </button>
        </div>
      )}

      {seriesError && !isSyncing && (
        <div className="error-indicator">
          <span>Error loading series: {seriesError}</span>
          {seriesError.toLowerCase().includes('cache_empty') && activeProfile && (
            <button
              className="btn btn-primary"
              onClick={() => handleStartSync(false)}
              style={{ marginLeft: '1rem' }}
              title="Download series to cache"
            >
              Download Series
            </button>
          )}
        </div>
      )}

      {isSyncing && (
        <div className="sync-indicator" role="status" aria-live="polite">
          <span>Syncing content...</span>
        </div>
      )}

      {isLoadingSeries ? (
        <SkeletonMovieGrid count={18} />
      ) : displaySeries.length === 0 ? (
        <EmptyState
          icon="📺"
          title={searchQuery ? "No series found" : "No series available"}
          description={
            searchQuery
              ? `No results for "${searchQuery}". Try a different search term.`
              : selectedCategoryId
                ? "This category doesn't have any series yet."
                : "No series available in your library."
          }
          action={
            searchQuery || selectedCategoryId
              ? {
                label: "Clear filters",
                onClick: () => {
                  setSearchQuery("");
                  handleCategoryFilter(null);
                },
              }
              : undefined
          }
        />
      ) : (
        <>
          <div className="pagination-info">
            <span className="item-count">{displaySeries.length} series available</span>
          </div>

          <Virtuoso
            style={{ height: '100%' }}
            totalCount={totalRows}
            itemContent={rowRenderer}
            overscan={2}
            className="virtual-series-grid"
          />
        </>
      )}
    </div>
  );
}
