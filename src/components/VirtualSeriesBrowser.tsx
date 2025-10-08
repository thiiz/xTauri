import { useCallback, useEffect, useMemo, useState } from "react";
import { Virtuoso } from "react-virtuoso";
import { useProfileStore } from "../stores/profileStore";
import { useXtreamContentStore } from "../stores/xtreamContentStore";
import { XtreamEpisode, XtreamSeason, XtreamShow, XtreamShowListing } from "../types/types";
import CachedImage from "./CachedImage";
import SearchBar from "./SearchBar";

interface VirtualSeriesBrowserProps {
  onSeriesSelect?: (series: XtreamShowListing) => void;
  onEpisodePlay?: (episode: XtreamEpisode, series: XtreamShow) => void;
}

const ITEMS_PER_ROW = 6;

export default function VirtualSeriesBrowser({ onSeriesSelect, onEpisodePlay }: VirtualSeriesBrowserProps) {
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
    if (activeProfile) {
      fetchSeriesCategories(activeProfile.id);
      fetchSeries(activeProfile.id);
    }
  }, [activeProfile]);

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
      setSearchQuery("");
      await fetchSeries(activeProfile.id, selectedCategoryId || undefined);
    }
  }, [activeProfile, selectedCategoryId, searchSeries, fetchSeries]);

  const handleSeriesClick = async (seriesItem: XtreamShowListing) => {
    setSelectedSeries(seriesItem);
    onSeriesSelect?.(seriesItem);

    if (activeProfile) {
      try {
        const details = await fetchSeriesDetails(activeProfile.id, seriesItem.series_id.toString());
        setSeriesDetails(details);
        setViewMode('details');
      } catch (error) {
        console.error('Failed to fetch series details:', error);
      }
    }
  };

  const handleEpisodePlay = (episode: XtreamEpisode) => {
    if (seriesDetails) {
      onEpisodePlay?.(episode, seriesDetails);
    }
  };

  const handleBackToGrid = () => {
    setViewMode('grid');
    setSelectedSeries(null);
    setSeriesDetails(null);
    setSelectedSeason(null);
  };

  const formatRating = (rating: string | number): string => {
    if (!rating || rating === '0') return 'N/A';
    const numRating = typeof rating === 'string' ? parseFloat(rating) : rating;
    return numRating.toString();
  };

  const formatYear = (year: string | null): string => year || 'Unknown';

  const formatEpisodeRuntime = (runtime: string | null): string => {
    if (!runtime) return 'Unknown';
    const minutes = parseInt(runtime);
    if (isNaN(minutes)) return runtime;
    const hours = Math.floor(minutes / 60);
    const mins = minutes % 60;
    return hours > 0 ? `${hours}h ${mins}m` : `${mins}m`;
  };

  const getSeasonEpisodes = (): XtreamEpisode[] => {
    if (!seriesDetails || !selectedSeason) return [];
    return seriesDetails.episodes[selectedSeason.season_number.toString()] || [];
  };

  const seriesRows = useMemo(() => {
    const rows = [];
    for (let i = 0; i < displaySeries.length; i += ITEMS_PER_ROW) {
      rows.push(displaySeries.slice(i, i + ITEMS_PER_ROW));
    }
    return rows;
  }, [displaySeries]);

  const rowRenderer = useCallback((index: number) => {
    const row = seriesRows[index];

    return (
      <div className="virtual-series-row" role="list">
        {row.map((seriesItem) => (
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
              <div className="series-meta" aria-label="Series metadata">
                <span className="series-year" aria-label={`Year ${formatYear(seriesItem.year)}`}>
                  {formatYear(seriesItem.year)}
                </span>
                <span className="series-rating" aria-label={`Rating ${formatRating(seriesItem.rating)} out of 10`}>
                  ★ {formatRating(seriesItem.rating)}
                </span>
              </div>
              {seriesItem.genre && (
                <div className="series-genre" aria-label={`Genre ${seriesItem.genre}`}>
                  {seriesItem.genre}
                </div>
              )}
            </div>
          </article>
        ))}
      </div>
    );
  }, [seriesRows, selectedSeries]);

  if (viewMode === 'details' && seriesDetails && selectedSeries) {
    const seasonEpisodes = getSeasonEpisodes();

    return (
      <div className="virtual-series-details-container">
        <div className="series-details-header">
          <button className="back-button" onClick={handleBackToGrid}>
            ← Back to Series
          </button>
          <h1 className="series-title">{selectedSeries.name}</h1>
        </div>

        <div className="series-info-section">
          <div className="series-poster">
            <CachedImage src={selectedSeries.cover} alt={selectedSeries.name} className="series-poster-image" />
          </div>

          <div className="series-meta">
            <div className="series-meta-grid">
              <div className="meta-item">
                <span className="meta-label">Year:</span>
                <span className="meta-value">{formatYear(selectedSeries.year)}</span>
              </div>
              <div className="meta-item">
                <span className="meta-label">Rating:</span>
                <span className="meta-value">★ {formatRating(selectedSeries.rating)}</span>
              </div>
              {selectedSeries.genre && (
                <div className="meta-item">
                  <span className="meta-label">Genre:</span>
                  <span className="meta-value">{selectedSeries.genre}</span>
                </div>
              )}
              {selectedSeries.episode_run_time && (
                <div className="meta-item">
                  <span className="meta-label">Episode Runtime:</span>
                  <span className="meta-value">{formatEpisodeRuntime(selectedSeries.episode_run_time)}</span>
                </div>
              )}
              {selectedSeries.director && (
                <div className="meta-item">
                  <span className="meta-label">Director:</span>
                  <span className="meta-value">{selectedSeries.director}</span>
                </div>
              )}
              {selectedSeries.cast && (
                <div className="meta-item">
                  <span className="meta-label">Cast:</span>
                  <span className="meta-value">{selectedSeries.cast}</span>
                </div>
              )}
            </div>

            {selectedSeries.plot && (
              <div className="series-plot">
                <h3>Plot</h3>
                <p>{selectedSeries.plot}</p>
              </div>
            )}
          </div>
        </div>

        <div className="season-selection">
          <h3>Seasons</h3>
          <div className="season-tabs">
            {seriesDetails.seasons.map((season) => (
              <button
                key={season.season_number}
                className={`season-tab ${selectedSeason?.season_number === season.season_number ? 'active' : ''}`}
                onClick={() => setSelectedSeason(season)}
              >
                Season {season.season_number}
                <span className="episode-count">({season.episode_count} episodes)</span>
              </button>
            ))}
          </div>
        </div>

        {selectedSeason && (
          <div className="episodes-section">
            <h3>Season {selectedSeason.season_number} Episodes</h3>
            <Virtuoso
              style={{ height: '600px' }}
              totalCount={seasonEpisodes.length}
              itemContent={(index) => {
                const episode = seasonEpisodes[index];
                return (
                  <div className="virtual-episode-item">
                    <div className="episode-thumbnail">
                      <CachedImage
                        src={episode.info.movie_image || selectedSeries.cover}
                        alt={episode.title}
                        className="episode-image"
                        lazy={true}
                        rootMargin="100px"
                      />
                      <button
                        className="episode-play-button"
                        onClick={() => handleEpisodePlay(episode)}
                        title="Play episode"
                      >
                        ▶
                      </button>
                    </div>

                    <div className="episode-info">
                      <div className="episode-header">
                        <span className="episode-number">E{episode.episode_num}</span>
                        <h4 className="episode-title">{episode.title}</h4>
                      </div>

                      <div className="episode-meta">
                        {episode.info.air_date && (
                          <span className="episode-date">{episode.info.air_date}</span>
                        )}
                        {episode.info.duration && (
                          <span className="episode-duration">{episode.info.duration}</span>
                        )}
                        {episode.info.rating && (
                          <span className="episode-rating">★ {episode.info.rating}</span>
                        )}
                      </div>

                      {episode.info.plot && (
                        <p className="episode-plot">{episode.info.plot}</p>
                      )}
                    </div>
                  </div>
                );
              }}
              overscan={5}
              className="virtual-episodes-list"
            />
          </div>
        )}
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

      {isLoadingSeries && (
        <div className="loading-indicator">
          <span>Loading series...</span>
        </div>
      )}

      {seriesError && (
        <div className="error-indicator">
          <span>Error loading series: {seriesError}</span>
        </div>
      )}

      <div className="pagination-info">
        <span className="item-count">{displaySeries.length} series available</span>
      </div>

      <Virtuoso
        style={{ height: '100%' }}
        totalCount={seriesRows.length}
        itemContent={rowRenderer}
        overscan={3}
        className="virtual-series-grid"
      />
    </div>
  );
}
