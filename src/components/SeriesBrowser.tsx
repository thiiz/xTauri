import { useEffect, useRef, useState } from "react";
import { useProfileStore } from "../stores/profileStore";
import { useXtreamContentStore } from "../stores/xtreamContentStore";
import { XtreamEpisode, XtreamSeason, XtreamShow, XtreamShowListing } from "../types/types";
import CachedImage from "./CachedImage";

interface SeriesBrowserProps {
  onSeriesSelect?: (series: XtreamShowListing) => void;
  onEpisodePlay?: (episode: XtreamEpisode, series: XtreamShow) => void;
}

const SERIES_PER_PAGE = 24; // Grid layout works well with multiples of 6

export default function SeriesBrowser({ onSeriesSelect, onEpisodePlay }: SeriesBrowserProps) {
  const [currentPage, setCurrentPage] = useState(1);
  const [selectedCategoryId, setSelectedCategoryId] = useState<string | null>(null);
  const [selectedSeries, setSelectedSeries] = useState<XtreamShowListing | null>(null);
  const [seriesDetails, setSeriesDetails] = useState<XtreamShow | null>(null);
  const [selectedSeason, setSelectedSeason] = useState<XtreamSeason | null>(null);

  const [searchQuery, setSearchQuery] = useState("");
  const [viewMode, setViewMode] = useState<'grid' | 'details'>('grid');
  const gridRef = useRef<HTMLDivElement>(null);

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

  // Determine which series to display
  const displaySeries = filteredSeries.length > 0 ? filteredSeries : series;

  // Load series data when component mounts or profile changes
  useEffect(() => {
    if (activeProfile) {
      fetchSeriesCategories(activeProfile.id);
      fetchSeries(activeProfile.id);
    }
  }, [activeProfile, fetchSeriesCategories, fetchSeries]);

  // Reset to first page when series change
  useEffect(() => {
    setCurrentPage(1);
  }, [displaySeries.length, selectedCategoryId, searchQuery]);

  // Reset selected season when series details change
  useEffect(() => {
    if (seriesDetails && seriesDetails.seasons.length > 0) {
      setSelectedSeason(seriesDetails.seasons[0]);
    }
  }, [seriesDetails]);

  // Pagination calculations
  const totalPages = Math.ceil(displaySeries.length / SERIES_PER_PAGE);
  const startIndex = (currentPage - 1) * SERIES_PER_PAGE;
  const endIndex = startIndex + SERIES_PER_PAGE;
  const currentSeries = displaySeries.slice(startIndex, endIndex);

  // Handle category filtering
  const handleCategoryFilter = async (categoryId: string | null) => {
    if (!activeProfile) return;

    setSelectedCategoryId(categoryId);
    setSelectedCategory(categoryId);
    setSearchQuery("");

    if (categoryId) {
      await fetchSeries(activeProfile.id, categoryId);
    } else {
      await fetchSeries(activeProfile.id);
    }
  };

  // Handle search
  const handleSearch = async (query: string) => {
    if (!activeProfile) return;

    setSearchQuery(query);

    if (query.trim()) {
      await searchSeries(activeProfile.id, query);
    } else {
      setSearchQuery("");
      if (selectedCategoryId) {
        await fetchSeries(activeProfile.id, selectedCategoryId);
      } else {
        await fetchSeries(activeProfile.id);
      }
    }
  };

  // Handle series selection
  const handleSeriesClick = async (seriesItem: XtreamShowListing) => {
    setSelectedSeries(seriesItem);
    onSeriesSelect?.(seriesItem);

    // Fetch detailed information
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

  // Handle episode play
  const handleEpisodePlay = (episode: XtreamEpisode) => {
    if (seriesDetails) {
      onEpisodePlay?.(episode, seriesDetails);
    }
  };

  // Handle back to grid
  const handleBackToGrid = () => {
    setViewMode('grid');
    setSelectedSeries(null);
    setSeriesDetails(null);
    setSelectedSeason(null);
  };

  // Format rating for display
  const formatRating = (rating: string | number): string => {
    if (!rating || rating === '0') return 'N/A';
    const numRating = typeof rating === 'string' ? parseFloat(rating) : rating;
    return numRating.toString();
  };

  // Format year for display
  const formatYear = (year: string | null): string => {
    if (!year) return 'Unknown';
    return year;
  };

  // Format episode runtime
  const formatEpisodeRuntime = (runtime: string | null): string => {
    if (!runtime) return 'Unknown';
    const minutes = parseInt(runtime);
    if (isNaN(minutes)) return runtime;

    const hours = Math.floor(minutes / 60);
    const mins = minutes % 60;
    if (hours > 0) {
      return `${hours}h ${mins}m`;
    }
    return `${mins}m`;
  };

  // Get episodes for selected season
  const getSeasonEpisodes = (): XtreamEpisode[] => {
    if (!seriesDetails || !selectedSeason) return [];
    return seriesDetails.episodes[selectedSeason.season_number.toString()] || [];
  };

  if (viewMode === 'details' && seriesDetails && selectedSeries) {
    return (
      <div className="series-details-container">
        {/* Header */}
        <div className="series-details-header">
          <button className="back-button" onClick={handleBackToGrid}>
            ← Back to Series
          </button>
          <h1 className="series-title">{selectedSeries.name}</h1>
        </div>

        {/* Series Info */}
        <div className="series-info-section">
          <div className="series-poster">
            <CachedImage
              src={selectedSeries.cover}
              alt={selectedSeries.name}
              className="series-poster-image"
            />
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

        {/* Season Selection */}
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

        {/* Episodes List */}
        {selectedSeason && (
          <div className="episodes-section">
            <h3>Season {selectedSeason.season_number} Episodes</h3>
            <div className="episodes-list">
              {getSeasonEpisodes().map((episode) => (
                <div key={episode.id} className="episode-item">
                  <div className="episode-thumbnail">
                    <CachedImage
                      src={episode.info.movie_image || selectedSeries.cover}
                      alt={episode.title}
                      className="episode-image"
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
              ))}
            </div>
          </div>
        )}
      </div>
    );
  }

  return (
    <div className="series-browser-container">
      {/* Controls */}
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

        <div className="search-container">
          <div className="search-input-wrapper">
            <input
              type="text"
              className="search-input"
              placeholder="Search series..."
              value={searchQuery}
              onChange={(e) => handleSearch(e.target.value)}
            />
            {searchQuery && (
              <button
                className="clear-search-btn"
                onClick={() => handleSearch('')}
                title="Clear search"
              >
                ×
              </button>
            )}
          </div>
        </div>
      </div>

      {/* Filter indicators */}
      {selectedCategoryId && (
        <div className="filter-indicator">
          <div className="filter-info">
            <span className="filter-label">Category:</span>
            <span className="filter-value">
              {seriesCategories.find(c => c.category_id === selectedCategoryId)?.category_name || selectedCategoryId}
            </span>
          </div>
          <button
            className="clear-filter-btn"
            onClick={() => handleCategoryFilter(null)}
            title="Clear category filter"
          >
            ×
          </button>
        </div>
      )}

      {searchQuery && (
        <div className="filter-indicator">
          <div className="filter-info">
            <span className="filter-label">Search:</span>
            <span className="filter-value">{searchQuery}</span>
          </div>
          <button
            className="clear-filter-btn"
            onClick={() => handleSearch('')}
            title="Clear search"
          >
            ×
          </button>
        </div>
      )}

      {/* Loading State */}
      {isLoadingSeries && (
        <div className="loading-indicator">
          <span>Loading series...</span>
        </div>
      )}

      {/* Error State */}
      {seriesError && (
        <div className="error-indicator">
          <span>Error loading series: {seriesError}</span>
        </div>
      )}

      {/* Pagination Info */}
      <div className="pagination-info">
        <span className="item-count">
          Showing {startIndex + 1}-{Math.min(endIndex, displaySeries.length)} of{" "}
          {displaySeries.length} series
          {totalPages > 1 && ` (Page ${currentPage} of ${totalPages})`}
        </span>
      </div>

      {/* Series Grid */}
      <div className="series-grid" ref={gridRef}>
        {currentSeries.map((seriesItem) => (
          <div
            key={seriesItem.series_id}
            className={`series-card ${selectedSeries?.series_id === seriesItem.series_id ? 'selected' : ''}`}
            onClick={() => handleSeriesClick(seriesItem)}
          >
            <div className="series-poster-container">
              <CachedImage
                src={seriesItem.cover}
                alt={seriesItem.name}
                className="series-poster"
              />
              <div className="series-overlay">
                <button
                  className="view-button"
                  onClick={(e) => {
                    e.stopPropagation();
                    handleSeriesClick(seriesItem);
                  }}
                  title="View series"
                >
                  👁
                </button>
              </div>
            </div>

            <div className="series-info">
              <h3 className="series-title">{seriesItem.name}</h3>
              <div className="series-meta">
                <span className="series-year">{formatYear(seriesItem.year)}</span>
                <span className="series-rating">★ {formatRating(seriesItem.rating)}</span>
              </div>
              {seriesItem.genre && (
                <div className="series-genre">{seriesItem.genre}</div>
              )}
            </div>
          </div>
        ))}
      </div>

      {/* Pagination Controls */}
      {totalPages > 1 && (
        <div className="pagination-controls">
          <button
            className="pagination-btn"
            onClick={() => setCurrentPage(1)}
            disabled={currentPage === 1}
            title="First page"
          >
            ««
          </button>
          <button
            className="pagination-btn"
            onClick={() => setCurrentPage(currentPage - 1)}
            disabled={currentPage === 1}
            title="Previous page"
          >
            ‹
          </button>

          {Array.from({ length: Math.min(5, totalPages) }, (_, i) => {
            const page = Math.max(1, Math.min(totalPages - 4, currentPage - 2)) + i;
            if (page > totalPages) return null;
            return (
              <button
                key={page}
                className={`pagination-btn ${page === currentPage ? 'active' : ''}`}
                onClick={() => setCurrentPage(page)}
              >
                {page}
              </button>
            );
          })}

          <button
            className="pagination-btn"
            onClick={() => setCurrentPage(currentPage + 1)}
            disabled={currentPage === totalPages}
            title="Next page"
          >
            ›
          </button>
          <button
            className="pagination-btn"
            onClick={() => setCurrentPage(totalPages)}
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