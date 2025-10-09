import { invoke } from '@tauri-apps/api/core';
import { create } from 'zustand';
import type {
  CurrentAndNextEPG,
  EnhancedEPGListing,
  EPGSearchOptions,
  EPGTimeFilter,
  XtreamCategory,
  XtreamChannel,
  XtreamFavorite,
  XtreamHistory,
  XtreamMovie,
  XtreamMoviesListing,
  XtreamShow,
  XtreamShowListing
} from '../types/types';

interface XtreamContentState {
  // Content data
  channels: XtreamChannel[];
  channelCategories: XtreamCategory[];
  movies: XtreamMoviesListing[];
  movieCategories: XtreamCategory[];
  series: XtreamShowListing[];
  seriesCategories: XtreamCategory[];

  // Filtered content data
  filteredChannels: XtreamChannel[];
  filteredMovies: XtreamMoviesListing[];
  filteredSeries: XtreamShowListing[];

  // EPG data
  epgData: Record<string, EnhancedEPGListing[]>;
  currentAndNextEPG: Record<string, CurrentAndNextEPG>;

  // History and favorites
  history: XtreamHistory[];
  favorites: XtreamFavorite[];
  isLoadingHistory: boolean;
  isLoadingFavorites: boolean;
  historyError: string | null;
  favoritesError: string | null;

  // Content type and navigation state
  activeContentType: 'channels' | 'movies' | 'series';
  selectedCategoryId: string | null;
  searchQuery: string;

  // Pagination state
  currentPage: number;
  itemsPerPage: number;
  totalItems: number;
  hasNextPage: boolean;

  // Loading states
  isLoadingChannels: boolean;
  isLoadingChannelCategories: boolean;
  isLoadingMovies: boolean;
  isLoadingMovieCategories: boolean;
  isLoadingSeries: boolean;
  isLoadingSeriesCategories: boolean;
  isLoadingEPG: boolean;
  isFiltering: boolean;
  isSearching: boolean;

  // Error states
  channelsError: string | null;
  moviesError: string | null;
  seriesError: string | null;
  epgError: string | null;
  filterError: string | null;
  searchError: string | null;

  // Actions
  fetchChannelCategories: (profileId: string) => Promise<void>;
  fetchChannels: (profileId: string, categoryId?: string) => Promise<void>;
  fetchMovieCategories: (profileId: string) => Promise<void>;
  fetchMovies: (profileId: string, categoryId?: string) => Promise<void>;
  fetchSeriesCategories: (profileId: string) => Promise<void>;
  fetchSeries: (profileId: string, categoryId?: string) => Promise<void>;
  fetchMovieDetails: (profileId: string, movieId: string) => Promise<XtreamMovie>;
  fetchSeriesDetails: (profileId: string, seriesId: string) => Promise<XtreamShow>;

  // EPG Actions
  fetchShortEPG: (profileId: string, channelId: string) => Promise<void>;
  fetchFullEPG: (profileId: string, channelId: string, startDate?: string, endDate?: string) => Promise<void>;
  fetchCurrentAndNextEPG: (profileId: string, channelId: string) => Promise<void>;
  fetchEPGForChannels: (profileId: string, channelIds: string[]) => Promise<void>;
  fetchEPGByDateRange: (profileId: string, channelId: string, startTimestamp: number, endTimestamp: number) => Promise<void>;

  // EPG Utility Actions
  parseAndEnhanceEPGData: (epgData: any, timezone?: string) => EnhancedEPGListing[];
  filterEPGByTimeRange: (channelId: string, filter: EPGTimeFilter) => EnhancedEPGListing[];
  searchEPGPrograms: (channelId: string, options: EPGSearchOptions) => EnhancedEPGListing[];

  // Content filtering and search actions
  filterChannels: (profileId: string, nameFilter?: string, categoryFilter?: string, groupFilter?: string) => Promise<void>;
  searchChannels: (profileId: string, searchQuery: string) => Promise<void>;
  filterMovies: (profileId: string, nameFilter?: string, categoryFilter?: string, genreFilter?: string, yearFilter?: string, ratingFilter?: number) => Promise<void>;
  searchMovies: (profileId: string, searchQuery: string) => Promise<void>;
  filterSeries: (profileId: string, nameFilter?: string, categoryFilter?: string, genreFilter?: string, yearFilter?: string, ratingFilter?: number) => Promise<void>;
  searchSeries: (profileId: string, searchQuery: string) => Promise<void>;

  // Content type and navigation actions
  setActiveContentType: (contentType: 'channels' | 'movies' | 'series') => void;
  setSelectedCategory: (categoryId: string | null) => void;
  setSearchQuery: (query: string) => void;

  // Pagination actions
  setCurrentPage: (page: number) => void;
  setItemsPerPage: (itemsPerPage: number) => void;
  fetchNextPage: (profileId: string) => Promise<void>;
  fetchPreviousPage: (profileId: string) => Promise<void>;

  // History actions
  fetchHistory: (profileId: string, limit?: number) => Promise<void>;
  fetchHistoryByType: (profileId: string, contentType: string, limit?: number) => Promise<void>;
  addToHistory: (profileId: string, contentType: string, contentId: string, contentData: any, position?: number, duration?: number) => Promise<void>;
  updatePlaybackPosition: (profileId: string, contentType: string, contentId: string, position: number, duration?: number) => Promise<void>;
  removeFromHistory: (historyId: string) => Promise<void>;
  clearHistory: (profileId: string) => Promise<void>;
  getHistoryItem: (profileId: string, contentType: string, contentId: string) => XtreamHistory | null;

  // Favorites actions
  fetchFavorites: (profileId: string) => Promise<void>;
  fetchFavoritesByType: (profileId: string, contentType: string) => Promise<void>;
  addToFavorites: (profileId: string, contentType: string, contentId: string, contentData: any) => Promise<void>;
  removeFromFavorites: (favoriteId: string) => Promise<void>;
  removeFromFavoritesByContent: (profileId: string, contentType: string, contentId: string) => Promise<void>;
  clearFavorites: (profileId: string) => Promise<void>;
  isFavorite: (profileId: string, contentType: string, contentId: string) => boolean;

  // Clear actions
  clearChannels: () => void;
  clearMovies: () => void;
  clearSeries: () => void;
  clearEPG: (channelId?: string) => void;
  clearFilters: () => void;
  clearSearch: () => void;
  clearAll: () => void;
}

export const useXtreamContentStore = create<XtreamContentState>((set, get) => ({
  // Initial state
  channels: [],
  channelCategories: [],
  movies: [],
  movieCategories: [],
  series: [],
  seriesCategories: [],
  filteredChannels: [],
  filteredMovies: [],
  filteredSeries: [],
  epgData: {},
  currentAndNextEPG: {},
  history: [],
  favorites: [],
  isLoadingHistory: false,
  isLoadingFavorites: false,
  historyError: null,
  favoritesError: null,

  // Content type and navigation state
  activeContentType: 'channels',
  selectedCategoryId: null,
  searchQuery: '',

  // Pagination state
  currentPage: 1,
  itemsPerPage: 50,
  totalItems: 0,
  hasNextPage: false,

  // Loading states
  isLoadingChannels: false,
  isLoadingChannelCategories: false,
  isLoadingMovies: false,
  isLoadingMovieCategories: false,
  isLoadingSeries: false,
  isLoadingSeriesCategories: false,
  isLoadingEPG: false,
  isFiltering: false,
  isSearching: false,

  // Error states
  channelsError: null,
  moviesError: null,
  seriesError: null,
  epgError: null,
  filterError: null,
  searchError: null,

  // Channel actions
  fetchChannelCategories: async (profileId: string) => {
    set({ isLoadingChannelCategories: true, channelsError: null });
    try {
      const categories = await invoke<XtreamCategory[]>('get_xtream_channel_categories', { profileId });
      set({ channelCategories: categories, isLoadingChannelCategories: false });
    } catch (error) {
      set({
        channelsError: error as string,
        isLoadingChannelCategories: false,
        channelCategories: []
      });
    }
  },

  fetchChannels: async (profileId: string, categoryId?: string) => {
    set({ isLoadingChannels: true, channelsError: null });
    try {
      const channels = await invoke<XtreamChannel[]>('get_xtream_channels', {
        profileId,
        categoryId: categoryId || null
      });
      set({
        channels,
        isLoadingChannels: false,
        totalItems: channels.length,
        hasNextPage: false,
        currentPage: 1
      });
    } catch (error) {
      set({
        channelsError: error as string,
        isLoadingChannels: false,
        channels: [],
        totalItems: 0,
        hasNextPage: false
      });
    }
  },

  // Movie actions
  fetchMovieCategories: async (profileId: string) => {
    set({ isLoadingMovieCategories: true, moviesError: null });
    try {
      const categories = await invoke<XtreamCategory[]>('get_xtream_movie_categories', { profileId });
      set({ movieCategories: categories, isLoadingMovieCategories: false });
    } catch (error) {
      set({
        moviesError: error as string,
        isLoadingMovieCategories: false,
        movieCategories: []
      });
    }
  },

  fetchMovies: async (profileId: string, categoryId?: string) => {
    set({ isLoadingMovies: true, moviesError: null });
    try {
      const movies = await invoke<XtreamMoviesListing[]>('get_xtream_movies', {
        profileId,
        categoryId: categoryId || null
      });
      set({
        movies,
        isLoadingMovies: false,
        totalItems: movies.length,
        hasNextPage: false,
        currentPage: 1
      });
    } catch (error) {
      set({
        moviesError: error as string,
        isLoadingMovies: false,
        movies: [],
        totalItems: 0,
        hasNextPage: false
      });
    }
  },

  fetchMovieDetails: async (profileId: string, movieId: string) => {
    try {
      const movieDetails = await invoke<XtreamMovie>('get_xtream_movie_info', {
        profileId,
        movieId
      });
      return movieDetails;
    } catch (error) {
      throw new Error(error as string);
    }
  },

  // Series actions
  fetchSeriesCategories: async (profileId: string) => {
    set({ isLoadingSeriesCategories: true, seriesError: null });
    try {
      const categories = await invoke<XtreamCategory[]>('get_xtream_series_categories', { profileId });
      set({ seriesCategories: categories, isLoadingSeriesCategories: false });
    } catch (error) {
      set({
        seriesError: error as string,
        isLoadingSeriesCategories: false,
        seriesCategories: []
      });
    }
  },

  fetchSeries: async (profileId: string, categoryId?: string) => {
    set({ isLoadingSeries: true, seriesError: null });
    try {
      const series = await invoke<XtreamShowListing[]>('get_xtream_series', {
        profileId,
        categoryId: categoryId || null
      });
      set({
        series,
        isLoadingSeries: false,
        totalItems: series.length,
        hasNextPage: false,
        currentPage: 1
      });
    } catch (error) {
      set({
        seriesError: error as string,
        isLoadingSeries: false,
        series: [],
        totalItems: 0,
        hasNextPage: false
      });
    }
  },

  fetchSeriesDetails: async (profileId: string, seriesId: string) => {
    try {
      const seriesDetails = await invoke<XtreamShow>('get_xtream_series_info', {
        profileId,
        seriesId
      });
      return seriesDetails;
    } catch (error) {
      throw new Error(error as string);
    }
  },

  // EPG actions
  fetchShortEPG: async (profileId: string, channelId: string) => {
    set({ isLoadingEPG: true, epgError: null });
    try {
      const epgData = await invoke<any>('get_xtream_short_epg', {
        profileId,
        channelId
      });

      // Parse and enhance the EPG data
      const enhancedEPG = await invoke<EnhancedEPGListing[]>('parse_and_enhance_epg_data', {
        epgData,
        timezone: null
      });

      set(state => ({
        epgData: {
          ...state.epgData,
          [channelId]: enhancedEPG
        },
        isLoadingEPG: false
      }));
    } catch (error) {
      set({
        epgError: error as string,
        isLoadingEPG: false
      });
    }
  },

  fetchFullEPG: async (profileId: string, channelId: string, startDate?: string, endDate?: string) => {
    set({ isLoadingEPG: true, epgError: null });
    try {
      const epgData = await invoke<any>('get_xtream_full_epg', {
        profileId,
        channelId,
        startDate: startDate || null,
        endDate: endDate || null
      });

      // Parse and enhance the EPG data
      const enhancedEPG = await invoke<EnhancedEPGListing[]>('parse_and_enhance_epg_data', {
        epgData,
        timezone: null
      });

      set(state => ({
        epgData: {
          ...state.epgData,
          [channelId]: enhancedEPG
        },
        isLoadingEPG: false
      }));
    } catch (error) {
      set({
        epgError: error as string,
        isLoadingEPG: false
      });
    }
  },

  fetchCurrentAndNextEPG: async (profileId: string, channelId: string) => {
    set({ isLoadingEPG: true, epgError: null });
    try {
      const currentAndNext = await invoke<CurrentAndNextEPG>('get_xtream_current_and_next_epg', {
        profileId,
        channelId
      });

      set(state => ({
        currentAndNextEPG: {
          ...state.currentAndNextEPG,
          [channelId]: currentAndNext
        },
        isLoadingEPG: false
      }));
    } catch (error) {
      set({
        epgError: error as string,
        isLoadingEPG: false
      });
    }
  },

  fetchEPGForChannels: async (profileId: string, channelIds: string[]) => {
    set({ isLoadingEPG: true, epgError: null });
    try {
      const epgData = await invoke<any>('get_xtream_epg_for_channels', {
        profileId,
        channelIds
      });

      // Parse and enhance the EPG data
      const enhancedEPG = await invoke<EnhancedEPGListing[]>('parse_and_enhance_epg_data', {
        epgData,
        timezone: null
      });

      // Group EPG data by channel if possible
      // For now, we'll store it under a combined key
      const combinedKey = channelIds.join(',');
      set(state => ({
        epgData: {
          ...state.epgData,
          [combinedKey]: enhancedEPG
        },
        isLoadingEPG: false
      }));
    } catch (error) {
      set({
        epgError: error as string,
        isLoadingEPG: false
      });
    }
  },

  fetchEPGByDateRange: async (profileId: string, channelId: string, startTimestamp: number, endTimestamp: number) => {
    set({ isLoadingEPG: true, epgError: null });
    try {
      const epgData = await invoke<any>('get_xtream_epg_by_date_range', {
        profileId,
        channelId,
        startTimestamp,
        endTimestamp
      });

      // Parse and enhance the EPG data
      const enhancedEPG = await invoke<EnhancedEPGListing[]>('parse_and_enhance_epg_data', {
        epgData,
        timezone: null
      });

      set(state => ({
        epgData: {
          ...state.epgData,
          [channelId]: enhancedEPG
        },
        isLoadingEPG: false
      }));
    } catch (error) {
      set({
        epgError: error as string,
        isLoadingEPG: false
      });
    }
  },

  // Content filtering and search actions
  filterChannels: async (profileId: string, nameFilter?: string, categoryFilter?: string, groupFilter?: string) => {
    set({ isFiltering: true, filterError: null });
    try {
      const state = get();
      const channelsToFilter = state.channels.length > 0 ? state.channels :
        await invoke<XtreamChannel[]>('get_xtream_channels', { profileId, categoryId: categoryFilter || null });

      const filteredChannels = await invoke<XtreamChannel[]>('filter_xtream_channels', {
        channels: channelsToFilter,
        nameFilter: nameFilter || null,
        categoryFilter: categoryFilter || null,
        groupFilter: groupFilter || null
      });

      set({
        filteredChannels,
        isFiltering: false,
        totalItems: filteredChannels.length,
        hasNextPage: false
      });
    } catch (error) {
      set({
        filterError: error as string,
        isFiltering: false,
        filteredChannels: []
      });
    }
  },

  searchChannels: async (profileId: string, searchQuery: string) => {
    set({ isSearching: true, searchError: null, searchQuery });
    try {
      const state = get();
      const channelsToSearch = state.channels.length > 0 ? state.channels :
        await invoke<XtreamChannel[]>('get_xtream_channels', { profileId, categoryId: null });

      const searchResults = await invoke<XtreamChannel[]>('search_xtream_channels', {
        channels: channelsToSearch,
        searchQuery
      });

      set({
        filteredChannels: searchResults,
        isSearching: false,
        totalItems: searchResults.length,
        hasNextPage: false
      });
    } catch (error) {
      set({
        searchError: error as string,
        isSearching: false,
        filteredChannels: []
      });
    }
  },

  filterMovies: async (profileId: string, nameFilter?: string, categoryFilter?: string, genreFilter?: string, yearFilter?: string, ratingFilter?: number) => {
    set({ isFiltering: true, filterError: null });
    try {
      const state = get();
      const moviesToFilter = state.movies.length > 0 ? state.movies :
        await invoke<XtreamMoviesListing[]>('get_xtream_movies', { profileId, categoryId: categoryFilter || null });

      const filteredMovies = await invoke<XtreamMoviesListing[]>('filter_xtream_movies', {
        movies: moviesToFilter,
        nameFilter: nameFilter || null,
        categoryFilter: categoryFilter || null,
        genreFilter: genreFilter || null,
        yearFilter: yearFilter || null,
        ratingFilter: ratingFilter || null
      });

      set({
        filteredMovies,
        isFiltering: false,
        totalItems: filteredMovies.length,
        hasNextPage: false
      });
    } catch (error) {
      set({
        filterError: error as string,
        isFiltering: false,
        filteredMovies: []
      });
    }
  },

  searchMovies: async (profileId: string, searchQuery: string) => {
    set({ isSearching: true, searchError: null, searchQuery });
    try {
      const state = get();
      const moviesToSearch = state.movies.length > 0 ? state.movies :
        await invoke<XtreamMoviesListing[]>('get_xtream_movies', { profileId, categoryId: null });

      const searchResults = await invoke<XtreamMoviesListing[]>('search_xtream_movies', {
        movies: moviesToSearch,
        searchQuery
      });

      set({
        filteredMovies: searchResults,
        isSearching: false,
        totalItems: searchResults.length,
        hasNextPage: false
      });
    } catch (error) {
      set({
        searchError: error as string,
        isSearching: false,
        filteredMovies: []
      });
    }
  },

  filterSeries: async (profileId: string, nameFilter?: string, categoryFilter?: string, genreFilter?: string, yearFilter?: string, ratingFilter?: number) => {
    set({ isFiltering: true, filterError: null });
    try {
      const state = get();
      const seriesToFilter = state.series.length > 0 ? state.series :
        await invoke<XtreamShowListing[]>('get_xtream_series', { profileId, categoryId: categoryFilter || null });

      const filteredSeries = await invoke<XtreamShowListing[]>('filter_xtream_series', {
        series: seriesToFilter,
        nameFilter: nameFilter || null,
        categoryFilter: categoryFilter || null,
        genreFilter: genreFilter || null,
        yearFilter: yearFilter || null,
        ratingFilter: ratingFilter || null
      });

      set({
        filteredSeries,
        isFiltering: false,
        totalItems: filteredSeries.length,
        hasNextPage: false
      });
    } catch (error) {
      set({
        filterError: error as string,
        isFiltering: false,
        filteredSeries: []
      });
    }
  },

  searchSeries: async (profileId: string, searchQuery: string) => {
    set({ isSearching: true, searchError: null, searchQuery });
    try {
      const state = get();
      const seriesToSearch = state.series.length > 0 ? state.series :
        await invoke<XtreamShowListing[]>('get_xtream_series', { profileId, categoryId: null });

      const searchResults = await invoke<XtreamShowListing[]>('search_xtream_series', {
        series: seriesToSearch,
        searchQuery
      });

      set({
        filteredSeries: searchResults,
        isSearching: false,
        totalItems: searchResults.length,
        hasNextPage: false
      });
    } catch (error) {
      set({
        searchError: error as string,
        isSearching: false,
        filteredSeries: []
      });
    }
  },

  // Content type and navigation actions
  setActiveContentType: (contentType: 'channels' | 'movies' | 'series') => {
    set({
      activeContentType: contentType,
      currentPage: 1,
      selectedCategoryId: null,
      searchQuery: '',
      filteredChannels: [],
      filteredMovies: [],
      filteredSeries: []
    });
  },

  setSelectedCategory: (categoryId: string | null) => {
    set({
      selectedCategoryId: categoryId,
      currentPage: 1,
      searchQuery: '',
      filteredChannels: [],
      filteredMovies: [],
      filteredSeries: []
    });
  },

  setSearchQuery: (query: string) => {
    set({ searchQuery: query });
  },

  // Pagination actions
  setCurrentPage: (page: number) => {
    set({ currentPage: page });
  },

  setItemsPerPage: (itemsPerPage: number) => {
    set({ itemsPerPage, currentPage: 1 });
  },

  fetchNextPage: async (profileId: string) => {
    const state = get();
    if (!state.hasNextPage) return;

    const nextPage = state.currentPage + 1;
    set({ currentPage: nextPage });

    try {
      // Fetch paginated content based on active content type
      switch (state.activeContentType) {
        case 'channels':
          await invoke<XtreamChannel[]>('get_xtream_channels_paginated', {
            profileId,
            categoryId: state.selectedCategoryId,
            page: nextPage,
            limit: state.itemsPerPage
          });
          break;
        case 'movies':
          await invoke<XtreamMoviesListing[]>('get_xtream_movies_paginated', {
            profileId,
            categoryId: state.selectedCategoryId,
            page: nextPage,
            limit: state.itemsPerPage
          });
          break;
        case 'series':
          await invoke<XtreamShowListing[]>('get_xtream_series_paginated', {
            profileId,
            categoryId: state.selectedCategoryId,
            page: nextPage,
            limit: state.itemsPerPage
          });
          break;
      }
    } catch (error) {
      // Revert page on error
      set({ currentPage: state.currentPage });
      throw error;
    }
  },

  fetchPreviousPage: async (profileId: string) => {
    const state = get();
    if (state.currentPage <= 1) return;

    const prevPage = state.currentPage - 1;
    set({ currentPage: prevPage });

    try {
      // Fetch paginated content based on active content type
      switch (state.activeContentType) {
        case 'channels':
          await invoke<XtreamChannel[]>('get_xtream_channels_paginated', {
            profileId,
            categoryId: state.selectedCategoryId,
            page: prevPage,
            limit: state.itemsPerPage
          });
          break;
        case 'movies':
          await invoke<XtreamMoviesListing[]>('get_xtream_movies_paginated', {
            profileId,
            categoryId: state.selectedCategoryId,
            page: prevPage,
            limit: state.itemsPerPage
          });
          break;
        case 'series':
          await invoke<XtreamShowListing[]>('get_xtream_series_paginated', {
            profileId,
            categoryId: state.selectedCategoryId,
            page: prevPage,
            limit: state.itemsPerPage
          });
          break;
      }
    } catch (error) {
      // Revert page on error
      set({ currentPage: state.currentPage });
      throw error;
    }
  },

  // EPG utility actions
  parseAndEnhanceEPGData: (epgData: any, _timezone?: string) => {
    try {
      // This would typically be called from the backend, but we can also do client-side parsing
      return epgData as EnhancedEPGListing[];
    } catch (error) {
      console.error('Failed to parse EPG data:', error);
      return [];
    }
  },

  filterEPGByTimeRange: (channelId: string, filter: EPGTimeFilter) => {
    const state = get();
    const epgData = state.epgData[channelId] || [];

    try {
      // Use the backend filtering function
      invoke<EnhancedEPGListing[]>('filter_epg_by_time_range', {
        epgData,
        startTimestamp: filter.start_timestamp || null,
        endTimestamp: filter.end_timestamp || null
      }).then((filteredData: EnhancedEPGListing[]) => {
        set(state => ({
          epgData: {
            ...state.epgData,
            [`${channelId}_filtered`]: filteredData
          }
        }));
      });

      return epgData; // Return current data immediately, filtered data will be set async
    } catch (error) {
      console.error('Failed to filter EPG data:', error);
      return epgData;
    }
  },

  searchEPGPrograms: (channelId: string, options: EPGSearchOptions) => {
    const state = get();
    const epgData = state.epgData[channelId] || [];

    try {
      // Use the backend search function
      invoke<EnhancedEPGListing[]>('search_epg_programs', {
        epgData,
        searchQuery: options.query
      }).then((searchResults: EnhancedEPGListing[]) => {
        set(state => ({
          epgData: {
            ...state.epgData,
            [`${channelId}_search`]: searchResults
          }
        }));
      });

      return epgData; // Return current data immediately, search results will be set async
    } catch (error) {
      console.error('Failed to search EPG data:', error);
      return epgData;
    }
  },

  // History actions
  fetchHistory: async (profileId: string, limit?: number) => {
    set({ isLoadingHistory: true, historyError: null });
    try {
      const history = await invoke<XtreamHistory[]>('get_xtream_history', {
        profileId,
        limit: limit || null
      });
      set({ history, isLoadingHistory: false });
    } catch (error) {
      set({
        historyError: error as string,
        isLoadingHistory: false,
        history: []
      });
    }
  },

  fetchHistoryByType: async (profileId: string, contentType: string, limit?: number) => {
    set({ isLoadingHistory: true, historyError: null });
    try {
      const history = await invoke<XtreamHistory[]>('get_xtream_history_by_type', {
        profileId,
        contentType,
        limit: limit || null
      });
      set({ history, isLoadingHistory: false });
    } catch (error) {
      set({
        historyError: error as string,
        isLoadingHistory: false,
        history: []
      });
    }
  },

  addToHistory: async (profileId: string, contentType: string, contentId: string, contentData: any, position?: number, duration?: number) => {
    try {
      await invoke('add_xtream_history', {
        request: {
          profile_id: profileId,
          content_type: contentType,
          content_id: contentId,
          content_data: contentData,
          position: position || null,
          duration: duration || null
        }
      });
      // Refresh history after adding
      await get().fetchHistory(profileId);
    } catch (error) {
      set({ historyError: error as string });
      throw error;
    }
  },

  updatePlaybackPosition: async (profileId: string, contentType: string, contentId: string, position: number, duration?: number) => {
    try {
      await invoke('update_xtream_history_position', {
        request: {
          profile_id: profileId,
          content_type: contentType,
          content_id: contentId,
          position,
          duration: duration || null
        }
      });
      // Update local state
      set(state => ({
        history: state.history.map(item =>
          item.profile_id === profileId &&
            item.content_type === contentType &&
            item.content_id === contentId
            ? { ...item, position, duration, watched_at: new Date().toISOString() }
            : item
        )
      }));
    } catch (error) {
      set({ historyError: error as string });
      throw error;
    }
  },

  removeFromHistory: async (historyId: string) => {
    try {
      await invoke('remove_xtream_history', { historyId });
      // Remove from local state
      set(state => ({
        history: state.history.filter(item => item.id !== historyId)
      }));
    } catch (error) {
      set({ historyError: error as string });
      throw error;
    }
  },

  clearHistory: async (profileId: string) => {
    try {
      await invoke('clear_xtream_history', { profileId });
      set({ history: [] });
    } catch (error) {
      set({ historyError: error as string });
      throw error;
    }
  },

  getHistoryItem: (profileId: string, contentType: string, contentId: string) => {
    const state = get();
    return state.history.find(
      item =>
        item.profile_id === profileId &&
        item.content_type === contentType &&
        item.content_id === contentId
    ) || null;
  },

  // Favorites actions
  fetchFavorites: async (profileId: string) => {
    set({ isLoadingFavorites: true, favoritesError: null });
    try {
      const favorites = await invoke<XtreamFavorite[]>('get_xtream_favorites', { profileId });
      set({ favorites, isLoadingFavorites: false });
    } catch (error) {
      set({
        favoritesError: error as string,
        isLoadingFavorites: false,
        favorites: []
      });
    }
  },

  fetchFavoritesByType: async (profileId: string, contentType: string) => {
    set({ isLoadingFavorites: true, favoritesError: null });
    try {
      const favorites = await invoke<XtreamFavorite[]>('get_xtream_favorites_by_type', {
        profileId,
        contentType
      });
      set({ favorites, isLoadingFavorites: false });
    } catch (error) {
      set({
        favoritesError: error as string,
        isLoadingFavorites: false,
        favorites: []
      });
    }
  },

  addToFavorites: async (profileId: string, contentType: string, contentId: string, contentData: any) => {
    try {
      await invoke('add_xtream_favorite', {
        request: {
          profile_id: profileId,
          content_type: contentType,
          content_id: contentId,
          content_data: contentData
        }
      });
      // Refresh favorites after adding
      await get().fetchFavorites(profileId);
    } catch (error) {
      set({ favoritesError: error as string });
      throw error;
    }
  },

  removeFromFavorites: async (favoriteId: string) => {
    try {
      await invoke('remove_xtream_favorite', { favoriteId });
      // Remove from local state
      set(state => ({
        favorites: state.favorites.filter(item => item.id !== favoriteId)
      }));
    } catch (error) {
      set({ favoritesError: error as string });
      throw error;
    }
  },

  removeFromFavoritesByContent: async (profileId: string, contentType: string, contentId: string) => {
    try {
      await invoke('remove_xtream_favorite_by_content', {
        profileId,
        contentType,
        contentId
      });
      // Remove from local state
      set(state => ({
        favorites: state.favorites.filter(
          item =>
            !(item.profile_id === profileId &&
              item.content_type === contentType &&
              item.content_id === contentId)
        )
      }));
    } catch (error) {
      set({ favoritesError: error as string });
      throw error;
    }
  },

  clearFavorites: async (profileId: string) => {
    try {
      await invoke('clear_xtream_favorites', { profileId });
      set({ favorites: [] });
    } catch (error) {
      set({ favoritesError: error as string });
      throw error;
    }
  },

  isFavorite: (profileId: string, contentType: string, contentId: string) => {
    const state = get();
    return state.favorites.some(
      item =>
        item.profile_id === profileId &&
        item.content_type === contentType &&
        item.content_id === contentId
    );
  },

  // Clear actions
  clearChannels: () => {
    set({
      channels: [],
      channelCategories: [],
      filteredChannels: [],
      channelsError: null
    });
  },

  clearMovies: () => {
    set({
      movies: [],
      movieCategories: [],
      filteredMovies: [],
      moviesError: null
    });
  },

  clearSeries: () => {
    set({
      series: [],
      seriesCategories: [],
      filteredSeries: [],
      seriesError: null
    });
  },

  clearFilters: () => {
    set({
      filteredChannels: [],
      filteredMovies: [],
      filteredSeries: [],
      selectedCategoryId: null,
      filterError: null,
      currentPage: 1
    });
  },

  clearSearch: () => {
    set({
      searchQuery: '',
      filteredChannels: [],
      filteredMovies: [],
      filteredSeries: [],
      searchError: null,
      currentPage: 1
    });
  },

  clearEPG: (channelId?: string) => {
    if (channelId) {
      set(state => {
        const newEPGData = { ...state.epgData };
        const newCurrentAndNext = { ...state.currentAndNextEPG };
        delete newEPGData[channelId];
        delete newCurrentAndNext[channelId];
        return {
          epgData: newEPGData,
          currentAndNextEPG: newCurrentAndNext,
          epgError: null
        };
      });
    } else {
      set({
        epgData: {},
        currentAndNextEPG: {},
        epgError: null
      });
    }
  },

  clearAll: () => {
    set({
      channels: [],
      channelCategories: [],
      movies: [],
      movieCategories: [],
      series: [],
      seriesCategories: [],
      filteredChannels: [],
      filteredMovies: [],
      filteredSeries: [],
      epgData: {},
      currentAndNextEPG: {},
      history: [],
      favorites: [],
      activeContentType: 'channels',
      selectedCategoryId: null,
      searchQuery: '',
      currentPage: 1,
      itemsPerPage: 50,
      totalItems: 0,
      hasNextPage: false,
      isLoadingChannels: false,
      isLoadingChannelCategories: false,
      isLoadingMovies: false,
      isLoadingMovieCategories: false,
      isLoadingSeries: false,
      isLoadingSeriesCategories: false,
      isLoadingEPG: false,
      isLoadingHistory: false,
      isLoadingFavorites: false,
      isFiltering: false,
      isSearching: false,
      channelsError: null,
      moviesError: null,
      seriesError: null,
      epgError: null,
      historyError: null,
      favoritesError: null,
      filterError: null,
      searchError: null,
    });
  },
}));