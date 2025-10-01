/**
 * Example usage of the XtreamContentStore
 * 
 * This file demonstrates how to use the enhanced XtreamContentStore
 * with filtering, search, and pagination functionality.
 */

import { useXtreamContentStore } from '../xtreamContentStore';

// Example: Basic content fetching
export const fetchChannelsExample = async (profileId: string) => {
  const store = useXtreamContentStore.getState();

  // Fetch channel categories first
  await store.fetchChannelCategories(profileId);

  // Fetch all channels
  await store.fetchChannels(profileId);

  // Or fetch channels for a specific category
  await store.fetchChannels(profileId, 'category-123');
};

// Example: Content type switching
export const switchContentTypeExample = () => {
  const store = useXtreamContentStore.getState();

  // Switch to movies
  store.setActiveContentType('movies');

  // Switch to series
  store.setActiveContentType('series');

  // Switch back to channels
  store.setActiveContentType('channels');
};

// Example: Category filtering
export const categoryFilteringExample = async (profileId: string) => {
  const store = useXtreamContentStore.getState();

  // Set active content type to movies
  store.setActiveContentType('movies');

  // Select a specific category
  store.setSelectedCategory('movie-category-123');

  // Fetch movies for that category
  await store.fetchMovies(profileId, 'movie-category-123');
};

// Example: Search functionality
export const searchExample = async (profileId: string) => {
  const store = useXtreamContentStore.getState();

  // Search channels
  store.setActiveContentType('channels');
  await store.searchChannels(profileId, 'CNN');

  // Search movies
  store.setActiveContentType('movies');
  await store.searchMovies(profileId, 'Matrix');

  // Search series
  store.setActiveContentType('series');
  await store.searchSeries(profileId, 'Game of Thrones');
};

// Example: Advanced filtering
export const advancedFilteringExample = async (profileId: string) => {
  const store = useXtreamContentStore.getState();

  // Filter channels by name and category
  await store.filterChannels(profileId, 'News', 'news-category', 'News Group');

  // Filter movies by multiple criteria
  await store.filterMovies(
    profileId,
    'Action',           // name filter
    'action-category',  // category filter
    'Action',           // genre filter
    '2023',             // year filter
    8.0                 // minimum rating
  );

  // Filter series by genre and year
  await store.filterSeries(
    profileId,
    undefined,          // no name filter
    undefined,          // no category filter
    'Drama',            // genre filter
    '2020'              // year filter
  );
};

// Example: Pagination
export const paginationExample = async (profileId: string) => {
  const store = useXtreamContentStore.getState();

  // Set items per page
  store.setItemsPerPage(25);

  // Navigate to next page
  await store.fetchNextPage(profileId);

  // Navigate to previous page
  await store.fetchPreviousPage(profileId);

  // Jump to specific page
  store.setCurrentPage(5);
};

// Example: Clearing data
export const clearDataExample = () => {
  const store = useXtreamContentStore.getState();

  // Clear search results
  store.clearSearch();

  // Clear filters
  store.clearFilters();

  // Clear specific content type
  store.clearChannels();
  store.clearMovies();
  store.clearSeries();

  // Clear everything
  store.clearAll();
};

// Example: Using the store in a React component
export const useXtreamContentExample = () => {
  // Get current state
  const {
    channels,
    filteredChannels,
    movies,
    filteredMovies,
    series,
    filteredSeries,
    activeContentType,
    selectedCategoryId,
    searchQuery,
    currentPage,
    totalItems,
    hasNextPage,
    isLoadingChannels,
    isLoadingMovies,
    isLoadingSeries,
    isFiltering,
    isSearching,
    channelsError,
    moviesError,
    seriesError,
    filterError,
    searchError
  } = useXtreamContentStore();

  // Get actions
  const {
    setActiveContentType,
    setSelectedCategory,
    setSearchQuery,
    fetchChannels,
    fetchMovies,
    fetchSeries,
    filterChannels,
    searchChannels,
    filterMovies,
    searchMovies,
    filterSeries,
    searchSeries,
    fetchNextPage,
    fetchPreviousPage,
    clearFilters,
    clearSearch
  } = useXtreamContentStore();

  return {
    // State
    channels: searchQuery || selectedCategoryId ? filteredChannels : channels,
    movies: searchQuery || selectedCategoryId ? filteredMovies : movies,
    series: searchQuery || selectedCategoryId ? filteredSeries : series,
    activeContentType,
    selectedCategoryId,
    searchQuery,
    currentPage,
    totalItems,
    hasNextPage,

    // Loading states
    isLoading: isLoadingChannels || isLoadingMovies || isLoadingSeries || isFiltering || isSearching,

    // Error states
    error: channelsError || moviesError || seriesError || filterError || searchError,

    // Actions
    setActiveContentType,
    setSelectedCategory,
    setSearchQuery,
    fetchChannels,
    fetchMovies,
    fetchSeries,
    filterChannels,
    searchChannels,
    filterMovies,
    searchMovies,
    filterSeries,
    searchSeries,
    fetchNextPage,
    fetchPreviousPage,
    clearFilters,
    clearSearch
  };
};