import { useCallback, useEffect, useMemo, useState } from "react";
import { Virtuoso } from "react-virtuoso";
import { useProfileStore } from "../stores/profileStore";
import { useXtreamContentStore } from "../stores/xtreamContentStore";
import { XtreamMovie, XtreamMoviesListing } from "../types/types";
import CachedImage from "./CachedImage";
import SearchBar from "./SearchBar";

interface VirtualMovieGridProps {
  onMovieSelect?: (movie: XtreamMoviesListing) => void;
  onMoviePlay?: (movie: XtreamMoviesListing) => void;
}

const ITEMS_PER_ROW = 6;

export default function VirtualMovieGrid({ onMovieSelect, onMoviePlay }: VirtualMovieGridProps) {
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
    if (activeProfile) {
      fetchMovieCategories(activeProfile.id);
      fetchMovies(activeProfile.id);
    }
  }, [activeProfile]);

  const handleCategoryFilter = async (categoryId: string | null) => {
    if (!activeProfile) return;
    setSelectedCategoryId(categoryId);
    setSelectedCategory(categoryId);
    clearSearch();
    await fetchMovies(activeProfile.id, categoryId || undefined);
  };

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

  const handleMovieClick = async (movie: XtreamMoviesListing) => {
    setSelectedMovie(movie);
    onMovieSelect?.(movie);

    if (activeProfile) {
      try {
        const details = await fetchMovieDetails(activeProfile.id, movie.stream_id.toString());
        setMovieDetails(details);
      } catch (error) {
        console.error('Failed to fetch movie details:', error);
      }
    }
  };

  const handleMoviePlay = (movie: XtreamMoviesListing) => {
    onMoviePlay?.(movie);
  };

  const handleShowDetails = (movie: XtreamMoviesListing) => {
    setSelectedMovie(movie);
    setShowDetails(true);
    handleMovieClick(movie);
  };

  const formatRating = (rating: number): string => rating === 0 ? 'N/A' : rating.toString();
  const formatYear = (year: string | null): string => year || 'Unknown';
  const formatRuntime = (runtime: number | null): string => {
    if (!runtime) return 'Unknown';
    const hours = Math.floor(runtime / 60);
    const minutes = runtime % 60;
    return hours > 0 ? `${hours}h ${minutes}m` : `${minutes}m`;
  };

  const movieRows = useMemo(() => {
    const rows = [];
    for (let i = 0; i < displayMovies.length; i += ITEMS_PER_ROW) {
      rows.push(displayMovies.slice(i, i + ITEMS_PER_ROW));
    }
    return rows;
  }, [displayMovies]);

  const rowRenderer = useCallback((index: number) => {
    const row = movieRows[index];

    return (
      <div className="virtual-movie-row" role="list">
        {row.map((movie) => (
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
              <div className="movie-meta" aria-label="Movie metadata">
                <span className="movie-year" aria-label={`Year ${formatYear(movie.year)}`}>
                  {formatYear(movie.year)}
                </span>
                <span className="movie-rating" aria-label={`Rating ${formatRating(movie.rating)} out of 10`}>
                  ★ {formatRating(movie.rating)}
                </span>
                {movie.episode_run_time && (
                  <span className="movie-runtime" aria-label={`Runtime ${formatRuntime(movie.episode_run_time)}`}>
                    {formatRuntime(movie.episode_run_time)}
                  </span>
                )}
              </div>
              {movie.genre && (
                <div className="movie-genre" aria-label={`Genre ${movie.genre}`}>
                  {movie.genre}
                </div>
              )}
            </div>
          </article>
        ))}
      </div>
    );
  }, [movieRows, selectedMovie]);

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

      {isLoadingMovies && (
        <div className="loading-indicator" role="status" aria-live="polite" aria-busy="true">
          <span>Loading movies...</span>
        </div>
      )}

      {moviesError && (
        <div className="error-indicator" role="alert" aria-live="assertive">
          <span>Error loading movies: {moviesError}</span>
        </div>
      )}

      <div className="pagination-info" role="status" aria-live="polite">
        <span className="item-count">{displayMovies.length} movies available</span>
      </div>

      <Virtuoso
        style={{ height: '100%' }}
        totalCount={movieRows.length}
        itemContent={rowRenderer}
        overscan={3}
        className="virtual-movie-grid"
      />

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
                      {selectedMovie.year && (
                        <span className="meta-badge">{formatYear(selectedMovie.year)}</span>
                      )}
                      {selectedMovie.rating && selectedMovie.rating !== 0 && (
                        <span className="meta-badge rating">
                          <svg width="14" height="14" viewBox="0 0 24 24" fill="currentColor">
                            <path d="M12 2l3.09 6.26L22 9.27l-5 4.87 1.18 6.88L12 17.77l-6.18 3.25L7 14.14 2 9.27l6.91-1.01L12 2z" />
                          </svg>
                          {formatRating(selectedMovie.rating)}
                        </span>
                      )}
                      {selectedMovie.genre && (
                        <span className="meta-badge genre">{selectedMovie.genre}</span>
                      )}
                      {selectedMovie.episode_run_time && (
                        <span className="meta-badge">{formatRuntime(selectedMovie.episode_run_time)}</span>
                      )}
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
