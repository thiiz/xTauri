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
      <div className="virtual-movie-row">
        {row.map((movie) => (
          <div
            key={movie.stream_id}
            className={`virtual-movie-card ${selectedMovie?.stream_id === movie.stream_id ? 'selected' : ''}`}
            onClick={() => handleMovieClick(movie)}
          >
            <div className="movie-poster-container">
              <CachedImage
                src={movie.stream_icon}
                alt={movie.name}
                className="movie-poster"
                lazy={true}
                rootMargin="200px"
              />
              <div className="movie-overlay">
                <button
                  className="play-button"
                  onClick={(e) => {
                    e.stopPropagation();
                    handleMoviePlay(movie);
                  }}
                  title="Play movie"
                >
                  ▶
                </button>
                <button
                  className="details-button"
                  onClick={(e) => {
                    e.stopPropagation();
                    handleShowDetails(movie);
                  }}
                  title="Show details"
                >
                  ℹ
                </button>
              </div>
            </div>

            <div className="movie-info">
              <h3 className="movie-title">{movie.name}</h3>
              <div className="movie-meta">
                <span className="movie-year">{formatYear(movie.year)}</span>
                <span className="movie-rating">★ {formatRating(movie.rating)}</span>
                {movie.episode_run_time && (
                  <span className="movie-runtime">{formatRuntime(movie.episode_run_time)}</span>
                )}
              </div>
              {movie.genre && (
                <div className="movie-genre">{movie.genre}</div>
              )}
            </div>
          </div>
        ))}
      </div>
    );
  }, [movieRows, selectedMovie]);

  return (
    <div className="virtual-movie-grid-container">
      <div className="movie-controls">
        <div className="category-filter">
          <label htmlFor="movie-category-select">Category:</label>
          <select
            id="movie-category-select"
            value={selectedCategoryId || ''}
            onChange={(e) => handleCategoryFilter(e.target.value || null)}
            disabled={isLoadingMovieCategories}
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
        <div className="filter-indicator">
          <div className="filter-info">
            <span className="filter-label">Category:</span>
            <span className="filter-value">
              {movieCategories.find(c => c.category_id === selectedCategoryId)?.category_name || selectedCategoryId}
            </span>
          </div>
          <button className="clear-filter-btn" onClick={() => handleCategoryFilter(null)} title="Clear category filter">
            ×
          </button>
        </div>
      )}

      {isLoadingMovies && (
        <div className="loading-indicator">
          <span>Loading movies...</span>
        </div>
      )}

      {moviesError && (
        <div className="error-indicator">
          <span>Error loading movies: {moviesError}</span>
        </div>
      )}

      <div className="pagination-info">
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
            <div className="movie-details-header">
              <h2>{selectedMovie.name}</h2>
              <button className="close-button" onClick={() => setShowDetails(false)} title="Close">×</button>
            </div>

            <div className="movie-details-body">
              <div className="movie-details-poster">
                <CachedImage src={selectedMovie.stream_icon} alt={selectedMovie.name} className="details-poster" />
              </div>

              <div className="movie-details-info">
                <div className="movie-details-meta">
                  <div className="meta-item">
                    <span className="meta-label">Year:</span>
                    <span className="meta-value">{formatYear(selectedMovie.year)}</span>
                  </div>
                  <div className="meta-item">
                    <span className="meta-label">Rating:</span>
                    <span className="meta-value">★ {formatRating(selectedMovie.rating)}</span>
                  </div>
                  {selectedMovie.episode_run_time && (
                    <div className="meta-item">
                      <span className="meta-label">Runtime:</span>
                      <span className="meta-value">{formatRuntime(selectedMovie.episode_run_time)}</span>
                    </div>
                  )}
                  {selectedMovie.genre && (
                    <div className="meta-item">
                      <span className="meta-label">Genre:</span>
                      <span className="meta-value">{selectedMovie.genre}</span>
                    </div>
                  )}
                  {movieDetails?.info?.director && (
                    <div className="meta-item">
                      <span className="meta-label">Director:</span>
                      <span className="meta-value">{movieDetails.info.director}</span>
                    </div>
                  )}
                  {movieDetails?.info?.cast && (
                    <div className="meta-item">
                      <span className="meta-label">Cast:</span>
                      <span className="meta-value">{movieDetails.info.cast}</span>
                    </div>
                  )}
                </div>

                {(selectedMovie.plot || movieDetails?.info?.plot) && (
                  <div className="movie-plot">
                    <h4>Plot</h4>
                    <p>{selectedMovie.plot || movieDetails?.info?.plot}</p>
                  </div>
                )}

                <div className="movie-actions">
                  <button
                    className="play-button-large"
                    onClick={() => {
                      handleMoviePlay(selectedMovie);
                      setShowDetails(false);
                    }}
                  >
                    ▶ Play Movie
                  </button>
                </div>
              </div>
            </div>
          </div>
        </div>
      )}
    </div>
  );
}
