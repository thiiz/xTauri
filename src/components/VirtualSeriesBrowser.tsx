import { useCallback, useEffect, useMemo, useState } from "react";
import { Virtuoso } from "react-virtuoso";
import { useProfileStore } from "../stores/profileStore";
import { useXtreamContentStore } from "../stores/xtreamContentStore";
import { XtreamEpisode, XtreamSeason, XtreamShow, XtreamShowListing } from "../types/types";
import CachedImage from "./CachedImage";
import EmptyState from "./EmptyState";
import SearchBar from "./SearchBar";
import { SkeletonEpisodeList, SkeletonMovieGrid } from "./SkeletonLoader";

interface VirtualSeriesBrowserProps {
  onEpisodePlay?: (episode: XtreamEpisode, series: XtreamShow) => void;
  onContentSelect?: () => void;
}

export default function VirtualSeriesBrowser({ onEpisodePlay, onContentSelect }: VirtualSeriesBrowserProps) {
  const [selectedCategoryId, setSelectedCategoryId] = useState<string | null>(null);
  const [selectedSeries, setSelectedSeries] = useState<XtreamShowListing | null>(null);
  const [seriesDetails, setSeriesDetails] = useState<XtreamShow | null>(null);
  const [selectedSeason, setSelectedSeason] = useState<XtreamSeason | null>(null);
  const [searchQuery, setSearchQuery] = useState("");
  const [viewMode, setViewMode] = useState<'grid' | 'details'>('grid');

  const {
    series,
    seriesCategories,
    filteredSeries,
    isLoadingSeries,
    isLoadingSeriesCategories,
    seriesError,
    fetchSeriesCategories,
    fetchSeries,
    fetchSeriesDetails,
    searchSeries,
    setSelectedCategory
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
  }, [activeProfile, fetchSeriesCategories, fetchSeries]);

  useEffect(() => {
    if (seriesDetails && seriesDetails.seasons.length > 0) {
      setSelectedSeason(seriesDetails.seasons[0]);
    }
  }, [seriesDetails]);

  const handleCategoryFilter = async (categoryId: string | null) => {
    if (!activeProfile) return;

    setSelectedCategoryId(categoryId);
    setSelectedCategory(categoryId);
    setSearchQuery("");
    await fetchSeries(activeProfile.id, categoryId || undefined);
  };

  const handleSearchChange = useCallback(async (query: string) => {
    if (!activeProfile) return;

    setSearchQuery(query);

    if (query.trim()) {
      await searchSeries(activeProfile.id, query);
    } else {
      await fetchSeries(activeProfile.id, selectedCategoryId || undefined);
    }
  }, [activeProfile, selectedCategoryId, searchSeries, fetchSeries]);

  const handleSeriesClick = async (seriesItem: XtreamShowListing) => {
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
  };

  const handleEpisodePlay = (episode: XtreamEpisode) => {
    if (seriesDetails) {
      onContentSelect?.();
      onEpisodePlay?.(episode, seriesDetails);
    }
  };

  const handleBackToGrid = () => {
    setViewMode('grid');
    setSelectedSeries(null);
    setSeriesDetails(null);
    setSelectedSeason(null);
  };

  const formatRating = (rating: string | number) => {
    if (!rating || rating === '0') return 'N/A';
    const numRating = typeof rating === 'string' ? parseFloat(rating) : rating;
    return numRating.toFixed(1);
  };

  const formatYear = (year: string | null) => year || 'N/A';

  const formatEpisodeRuntime = (runtime: string | null) => {
    if (!runtime) return '';
    const minutes = parseInt(runtime);
    if (isNaN(minutes)) return runtime;
    const hours = Math.floor(minutes / 60);
    const mins = minutes % 60;
    return hours > 0 ? `${hours}h ${mins}m` : `${mins}m`;
  };

  const getSeasonEpisodes = () => {
    if (!seriesDetails || !selectedSeason) return [];
    return seriesDetails.episodes[selectedSeason.season_number.toString()] || [];
  };

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

  if (viewMode === 'details' && seriesDetails && selectedSeries) {
    const seasonEpisodes = getSeasonEpisodes();

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
              </div>
            </div>
          </div>
        </div>

        {/* Seasons & Episodes Section */}
        <div className="series-content-section">
          <div className="season-selector">
            <h2 className="section-title">Episodes</h2>
            <div className="season-dropdown-wrapper">
              <select
                className="season-dropdown"
                value={selectedSeason?.season_number || ''}
                onChange={(e) => {
                  const season = seriesDetails.seasons.find(s => s.season_number === parseInt(e.target.value));
                  if (season) setSelectedSeason(season);
                }}
              >
                {seriesDetails.seasons.map((season) => (
                  <option key={season.season_number} value={season.season_number}>
                    Season {season.season_number} ({season.episode_count} episodes)
                  </option>
                ))}
              </select>
            </div>
          </div>

          {selectedSeason && seasonEpisodes.length === 0 ? (
            <SkeletonEpisodeList count={8} />
          ) : selectedSeason && (
            <div className="episodes-grid">
              <Virtuoso
                style={{ height: '600px' }}
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

      {seriesError && (
        <div className="error-indicator">
          <span>Error loading series: {seriesError}</span>
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
