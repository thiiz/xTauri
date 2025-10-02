import { useEffect, useRef, useState } from "react";
import { useProfileStore } from "../stores/profileStore";
import { useXtreamContentStore } from "../stores/xtreamContentStore";
import { XtreamMovie, XtreamMoviesListing } from "../types/types";
import CachedImage from "./CachedImage";

interface MovieGridProps {
  onMovieSelect?: (movie: XtreamMoviesListing) => void;
  onMoviePlay?: (movie: XtreamMoviesListing) => void;
}

const MOVIES_PER_PAGE = 24; // Grid layout works well with multiples of 6

export default function MovieGrid({ onMovieSelect, onMoviePlay }: MovieGridProps) {
  const [currentPage, setCurrentPage] = useState(1);
  const [selectedCategoryId, setSelectedCategoryId] = useState<string | null>(null);
  const [selectedMovie, setSelectedMovie] = useState<XtreamMoviesListing | null>(null);
  const [movieDetails, setMovieDetails] = useState<XtreamMovie | null>(null);
  const [showDetails, setShowDetails] = useState(false);
  const [searchQuery, setSearchQuery] = useState("");
  const gridRef = useRef<HTMLDivElement>(null);

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

  // Determine which movies to display
  const displayMovies = filteredMovies.length > 0 ? filteredMovies : movies;

  // Load movie data when component mounts or profile changes
  useEffect(() => {
    if (activeProfile) {
      fetchMovieCategories(activeProfile.id);
      fetchMovies(activeProfile.id);
    }
  }, [activeProfile, fetchMovieCategories, fetchMovies]);

  // Reset to first page when movies change
  useEffect(() => {
    setCurrentPage(1);
  }, [displayMovies.length, selectedCategoryId, searchQuery]);

  // Pagination calculations
  const totalPages = Math.ceil(displayMovies.length / MOVIES_PER_PAGE);
  const startIndex = (currentPage - 1) * MOVIES_PER_PAGE;
  const endIndex = startIndex + MOVIES_PER_PAGE;
  const currentMovies = displayMovies.slice(startIndex, endIndex);

  // Handle category filtering
  const handleCategoryFilter = async (categoryId: string | null) => {
    if (!activeProfile) return;

    setSelectedCategoryId(categoryId);
    setSelectedCategory(categoryId);
    clearSearch();

    if (categoryId) {
      await fetchMovies(activeProfile.id, categoryId);
    } else {
      await fetchMovies(activeProfile.id);
    }
  };

  // Handle search
  const handleSearch = async (query: string) => {
    if (!activeProfile) return;

    setSearchQuery(query);

    if (query.trim()) {
      await searchMovies(activeProfile.id, query);
    } else {
      clearSearch();
      if (selectedCategoryId) {
        await fetchMovies(activeProfile.id, selectedCategoryId);
      } else {
        await fetchMovies(activeProfile.id);
      }
    }
  };

  // Handle movie selection
  const handleMovieClick = async (movie: XtreamMoviesListing) => {
    setSelectedMovie(movie);
    onMovieSelect?.(movie);

    // Fetch detailed information
    if (activeProfile) {
      try {
        const details = await fetchMovieDetails(activeProfile.id, movie.stream_id.toString());
        setMovieDetails(details);
      } catch (error) {
        console.error('Failed to fetch movie details:', error);
      }
    }
  };

  // Handle movie play
  const handleMoviePlay = async (movie: XtreamMoviesListing) => {
    if (!activeProfile) return;

    try {
      // Call the onMoviePlay callback with the movie data
      onMoviePlay?.(movie);

      console.log('Playing movie:', movie.name);
    } catch (error) {
      console.error('Failed to play movie:', error);
    }
  };

  // Show movie details modal
  const handleShowDetails = (movie: XtreamMoviesListing) => {
    setSelectedMovie(movie);
    setShowDetails(true);
    handleMovieClick(movie);
  };

  // Format rating for display
  const formatRating = (rating: number): string => {
    if (rating === 0) return 'N/A';
    return rating.toString();
  };

  // Format year for display
  const formatYear = (year: string | null): string => {
    if (!year) return 'Unknown';
    return year;
  };

  // Format runtime for display
  const formatRuntime = (runtime: number | null): string => {
    if (!runtime) return 'Unknown';
    const hours = Math.floor(runtime / 60);
    const minutes = runtime % 60;
    if (hours > 0) {
      return `${hours}h ${minutes}m`;
    }
    return `${minutes}m`;
  };

  return (
    <div className="movie-grid-container">
      {/* Controls */}
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

        <div className="search-container">
          <div className="search-input-wrapper">
            <input
              type="text"
              className="search-input"
              placeholder="Search movies..."
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
              {movieCategories.find(c => c.category_id === selectedCategoryId)?.category_name || selectedCategoryId}
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
      {isLoadingMovies && (
        <div className="loading-indicator">
          <span>Loading movies...</span>
        </div>
      )}

      {/* Error State */}
      {moviesError && (
        <div className="error-indicator">
          <span>Error loading movies: {moviesError}</span>
        </div>
      )}

      {/* Pagination Info */}
      <div className="pagination-info">
        <span className="item-count">
          Showing {startIndex + 1}-{Math.min(endIndex, displayMovies.length)} of{" "}
          {displayMovies.length} movies
          {totalPages > 1 && ` (Page ${currentPage} of ${totalPages})`}
        </span>
      </div>

      {/* Movie Grid */}
      <div className="movie-grid" ref={gridRef}>
        {currentMovies.map((movie) => (
          <div
            key={movie.stream_id}
            className={`movie-card ${selectedMovie?.stream_id === movie.stream_id ? 'selected' : ''}`}
            onClick={() => handleMovieClick(movie)}
          >
            <div className="movie-poster-container">
              <CachedImage
                src={movie.stream_icon}
                alt={movie.name}
                className="movie-poster"
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

      {/* Movie Details Modal */}
      {showDetails && selectedMovie && (
        <div className="movie-details-modal" onClick={() => setShowDetails(false)}>
          <div className="movie-details-content" onClick={(e) => e.stopPropagation()}>
            <div className="movie-details-header">
              <h2>{selectedMovie.name}</h2>
              <button
                className="close-button"
                onClick={() => setShowDetails(false)}
                title="Close"
              >
                ×
              </button>
            </div>

            <div className="movie-details-body">
              <div className="movie-details-poster">
                <CachedImage
                  src={selectedMovie.stream_icon}
                  alt={selectedMovie.name}
                  className="details-poster"
                />
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