import { useCallback, useEffect, useMemo, useState } from "react";
import { Virtuoso } from "react-virtuoso";
import { useProfileStore } from "../stores/profileStore";
import { useXtreamContentStore } from "../stores/xtreamContentStore";
import { XtreamMovie, XtreamMoviesListing } from "../types/types";
import { formatRating, formatRuntime, formatYear } from "../utils/formatters";
import CachedImage from "./CachedImage";
import EmptyState from "./EmptyState";
import SearchBar from "./SearchBar";
import { SkeletonMovieGrid } from "./SkeletonLoader";

interface VirtualMovieGridProps {
  onMovieSelect?: (movie: XtreamMoviesListing) => void;
  onMoviePlay?: (movie: XtreamMoviesListing) => void;
  onContentSelect?: () => void;
}

export default function VirtualMovieGrid({ onMovieSelect, onMoviePlay, onContentSelect }: VirtualMovieGridProps) {
  const [selectedCategoryId, setSelectedCategoryId] = useState<string | null>(null);
  const [selectedMovie, setSelectedMovie] = useState<XtreamMoviesListing | null>(null);
  const [movieDetails, setMovieDetails] = useState<XtreamMovie | null>(null);
  const [showDetails, setShowDetails] = useState(false);
  const [searchQuery, setSearchQuery] = useState("");

  const {
    movies,
    movieCategories,
    filteredMovies,
    isLoadingMovies,
    isLoadingMovieCategories,
    moviesError,
    fetchMovieCategories,
    fetchMovies,
    fetchMovieDetails,
    searchMovies,
    setSelectedCategory,
    clearSearch
  } = useXtreamContentStore();

  const { activeProfile } = useProfileStore();

  const displayMovies = useMemo(() =>
    filteredMovies.length > 0 ? filteredMovies : movies,
    [filteredMovies, movies]
  );

  useEffect(() => {
    if (!activeProfile) return;

    fetchMovieCategories(activeProfile.id);
    fetchMovies(activeProfile.id);
  }, [activeProfile, fetchMovieCategories, fetchMovies]);

  const handleCategoryFilter = useCallback(async (categoryId: string | null) => {
    if (!activeProfile) return;

    setSelectedCategoryId(categoryId);
    setSelectedCategory(categoryId);
    clearSearch();
    await fetchMovies(activeProfile.id, categoryId || undefined);
  }, [activeProfile, setSelectedCategory, clearSearch, fetchMovies]);

  const handleSearchChange = useCallback(async (query: string) => {
    if (!activeProfile) return;

    setSearchQuery(query);

    if (query.trim()) {
      await searchMovies(activeProfile.id, query);
    } else {
      clearSearch();
      await fetchMovies(activeProfile.id, selectedCategoryId || undefined);
    }
  }, [activeProfile, selectedCategoryId, searchMovies, clearSearch, fetchMovies]);

  const handleMovieClick = useCallback(async (movie: XtreamMoviesListing) => {
    setSelectedMovie(movie);
    onContentSelect?.();
    onMovieSelect?.(movie);

    if (!activeProfile) return;

    try {
      const details = await fetchMovieDetails(activeProfile.id, movie.stream_id.toString());
      setMovieDetails(details);
    } catch (error) {
      console.error('Failed to fetch movie details:', error);
    }
  }, [activeProfile, fetchMovieDetails, onContentSelect, onMovieSelect]);

  const handleMoviePlay = useCallback((movie: XtreamMoviesListing) => {
    onContentSelect?.();
    onMoviePlay?.(movie);
  }, [onContentSelect, onMoviePlay]);

  const handleShowDetails = useCallback((movie: XtreamMoviesListing) => {
    setSelectedMovie(movie);
    setShowDetails(true);
    handleMovieClick(movie);
  }, [handleMovieClick]);

  // Formatting functions are now imported from utils/formatters

  const rowRenderer = useCallback((index: number) => {
    const startIdx = index * 6;
    const endIdx = Math.min(startIdx + 6, displayMovies.length);
    const rowMovies = displayMovies.slice(startIdx, endIdx);

    return (
      <div className="virtual-movie-row" role="list">
        {rowMovies.map((movie) => (
          <article
            key={movie.stream_id}
            className={`virtual-movie-card ${selectedMovie?.stream_id === movie.stream_id ? 'selected' : ''}`}
            onClick={() => handleMovieClick(movie)}
            role="listitem"
            tabIndex={0}
            onKeyDown={(e) => {
              if (e.key === 'Enter' || e.key === ' ') {
                e.preventDefault();
                handleMovieClick(movie);
              }
            }}
            aria-label={`${movie.name}, ${formatYear(movie.year)}, Rating ${formatRating(movie.rating)}`}
          >
            <div className="movie-poster-container">
              <CachedImage
                src={movie.stream_icon}
                alt={`${movie.name} poster`}
                className="movie-poster"
                lazy={true}
                rootMargin="200px"
              />
              <div className="movie-overlay" aria-hidden="true">
                <button
                  className="play-button"
                  onClick={(e) => {
                    e.stopPropagation();
                    handleMoviePlay(movie);
                  }}
                  aria-label={`Play ${movie.name}`}
                  title={`Play ${movie.name}`}
                >
                  <span aria-hidden="true">▶</span>
                </button>
                <button
                  className="details-button"
                  onClick={(e) => {
                    e.stopPropagation();
                    handleShowDetails(movie);
                  }}
                  aria-label={`Show details for ${movie.name}`}
                  title={`Show details for ${movie.name}`}
                >
                  <span aria-hidden="true">ℹ</span>
                </button>
              </div>
            </div>

            <div className="movie-info">
              <h3 className="movie-title">{movie.name}</h3>
              <div className="movie-meta">
                {movie.year && <span className="movie-year">{formatYear(movie.year)}</span>}
                {movie.rating > 0 && <span className="movie-rating">★ {formatRating(movie.rating)}</span>}
                {movie.episode_run_time && <span className="movie-runtime">{formatRuntime(movie.episode_run_time)}</span>}
              </div>
            </div>
          </article>
        ))}
      </div>
    );
  }, [displayMovies, selectedMovie, handleMovieClick, handleMoviePlay, handleShowDetails]);

  const totalRows = Math.ceil(displayMovies.length / 6);

  return (
    <div className="virtual-movie-grid-container" role="region" aria-label="Movies browser">
      <div className="movie-controls" role="toolbar" aria-label="Movie filters and search">
        <div className="category-filter">
          <label htmlFor="movie-category-select">Category:</label>
          <select
            id="movie-category-select"
            value={selectedCategoryId || ''}
            onChange={(e) => handleCategoryFilter(e.target.value || null)}
            disabled={isLoadingMovieCategories}
            aria-label="Filter movies by category"
          >
            <option value="">All Categories</option>
            {movieCategories.map((category) => (
              <option key={category.category_id} value={category.category_id}>
                {category.category_name}
              </option>
            ))}
          </select>
        </div>

        <SearchBar
          value={searchQuery}
          onChange={handleSearchChange}
          placeholder="Search movies..."
          debounceDelay={300}
        />
      </div>

      {selectedCategoryId && (
        <div className="filter-indicator" role="status" aria-live="polite">
          <div className="filter-info">
            <span className="filter-label">Category:</span>
            <span className="filter-value">
              {movieCategories.find(c => c.category_id === selectedCategoryId)?.category_name || selectedCategoryId}
            </span>
          </div>
          <button
            className="clear-filter-btn"
            onClick={() => handleCategoryFilter(null)}
            aria-label="Clear category filter"
            title="Clear category filter"
          >
            <span aria-hidden="true">×</span>
          </button>
        </div>
      )}

      {moviesError && (
        <div className="error-indicator" role="alert" aria-live="assertive">
          <span>Error loading movies: {moviesError}</span>
        </div>
      )}

      {isLoadingMovies ? (
        <SkeletonMovieGrid count={18} />
      ) : displayMovies.length === 0 ? (
        <EmptyState
          icon="🎬"
          title={searchQuery ? "No movies found" : "No movies available"}
          description={
            searchQuery
              ? `No results for "${searchQuery}". Try a different search term.`
              : selectedCategoryId
                ? "This category doesn't have any movies yet."
                : "No movies available in your library."
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
          <div className="pagination-info" role="status" aria-live="polite">
            <span className="item-count">{displayMovies.length} movies available</span>
          </div>

          <Virtuoso
            style={{ height: '100%' }}
            totalCount={totalRows}
            itemContent={rowRenderer}
            overscan={2}
            className="virtual-movie-grid"
          />
        </>
      )}

      {showDetails && selectedMovie && (
        <div className="movie-details-modal" onClick={() => setShowDetails(false)}>
          <div className="movie-details-content" onClick={(e) => e.stopPropagation()}>
            {/* Hero Section with Background */}
            <div className="movie-hero-section">
              <div className="movie-hero-backdrop">
                <CachedImage
                  src={selectedMovie.stream_icon}
                  alt=""
                  className="movie-backdrop-image"
                />
                <div className="movie-hero-overlay"></div>
              </div>

              <div className="movie-hero-content">
                <button className="close-button-hero" onClick={() => setShowDetails(false)}>
                  <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
                    <path d="M18 6L6 18M6 6l12 12" />
                  </svg>
                </button>

                <div className="movie-hero-info">
                  <div className="movie-poster-compact">
                    <CachedImage src={selectedMovie.stream_icon} alt={selectedMovie.name} className="movie-poster-image" />
                  </div>

                  <div className="movie-hero-details">
                    <h1 className="movie-hero-title">{selectedMovie.name}</h1>

                    <div className="movie-hero-meta">
                      {selectedMovie.year && <span className="meta-badge">{formatYear(selectedMovie.year)}</span>}
                      {selectedMovie.rating > 0 && (
                        <span className="meta-badge rating">★ {formatRating(selectedMovie.rating)}</span>
                      )}
                      {selectedMovie.genre && <span className="meta-badge genre">{selectedMovie.genre}</span>}
                      {selectedMovie.episode_run_time && <span className="meta-badge">{formatRuntime(selectedMovie.episode_run_time)}</span>}
                    </div>

                    {(selectedMovie.plot || movieDetails?.info?.plot) && (
                      <p className="movie-hero-plot">{selectedMovie.plot || movieDetails?.info?.plot}</p>
                    )}

                    {(movieDetails?.info?.director || movieDetails?.info?.cast) && (
                      <div className="movie-hero-credits">
                        {movieDetails?.info?.director && (
                          <div className="credit-item">
                            <span className="credit-label">Director:</span>
                            <span className="credit-value">{movieDetails.info.director}</span>
                          </div>
                        )}
                        {movieDetails?.info?.cast && (
                          <div className="credit-item">
                            <span className="credit-label">Cast:</span>
                            <span className="credit-value">{movieDetails.info.cast}</span>
                          </div>
                        )}
                      </div>
                    )}

                    <div className="movie-hero-actions">
                      <button
                        className="play-button-hero"
                        onClick={() => {
                          handleMoviePlay(selectedMovie);
                          setShowDetails(false);
                        }}
                      >
                        <svg width="20" height="20" viewBox="0 0 24 24" fill="currentColor">
                          <path d="M8 5v14l11-7z" />
                        </svg>
                        Play Movie
                      </button>
                    </div>
                  </div>
                </div>
              </div>
            </div>
          </div>
        </div>
      )}
    </div>
  );
}
