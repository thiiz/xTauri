import { invoke } from "@tauri-apps/api/core";
import { create } from "zustand";
import type { Channel } from "../types/channel";

interface SearchState {
  // Search state
  searchQuery: string;
  debouncedSearchQuery: string;
  isSearching: boolean;

  // Actions
  setSearchQuery: (query: string) => void;
  setDebouncedSearchQuery: (query: string) => void;
  setIsSearching: (searching: boolean) => void;
  clearSearch: () => void;

  // Search operations
  searchChannels: (
    query: string,
    channelListId: number | null,
  ) => Promise<Channel[]>;

  // Debounce timer management
  debounceTimer: ReturnType<typeof setTimeout> | null;
  setDebounceTimer: (timer: ReturnType<typeof setTimeout> | null) => void;
}

export const useSearchStore = create<SearchState>((set, get) => ({
  // Initial state
  searchQuery: "",
  debouncedSearchQuery: "",
  isSearching: false,
  debounceTimer: null,

  // Basic setters
  setSearchQuery: (searchQuery) => {
    set({ searchQuery });

    // Clear existing timer
    const { debounceTimer } = get();
    if (debounceTimer) {
      clearTimeout(debounceTimer);
    }

    // Set new timer for debouncing (reduced from 400ms to 300ms for better UX)
    const newTimer = setTimeout(() => {
      set({ debouncedSearchQuery: searchQuery });
    }, 300);

    set({ debounceTimer: newTimer });
  },

  setDebouncedSearchQuery: (debouncedSearchQuery) =>
    set({ debouncedSearchQuery }),
  setIsSearching: (isSearching) => set({ isSearching }),
  setDebounceTimer: (debounceTimer) => set({ debounceTimer }),

  clearSearch: () => {
    const { debounceTimer } = get();
    if (debounceTimer) {
      clearTimeout(debounceTimer);
    }

    set({
      searchQuery: "",
      debouncedSearchQuery: "",
      isSearching: false,
      debounceTimer: null,
    });
  },

  // Search operations with improved error handling and caching
  searchChannels: async (query, channelListId) => {
    // Early return for empty or too short queries
    if (!query || query.length < 2) {
      set({ isSearching: false });
      try {
        const fetchedChannels = await invoke<Channel[]>("get_channels", {
          id: channelListId,
        });
        return fetchedChannels;
      } catch (error) {
        console.error("Failed to fetch channels:", error);
        return [];
      }
    }

    set({ isSearching: true });
    try {
      const searchedChannels = await invoke<Channel[]>("search_channels", {
        query,
        id: channelListId,
      });
      return searchedChannels;
    } catch (error) {
      // Handle search cancellation gracefully
      if (
        error &&
        typeof error === "string" &&
        error.includes("Search cancelled")
      ) {
        console.log("Search was cancelled - ignoring result");
        return [];
      }

      console.error("Search failed:", error);

      // For other errors, fall back to returning all channels
      try {
        const fetchedChannels = await invoke<Channel[]>("get_channels", {
          id: channelListId,
        });
        return fetchedChannels;
      } catch (fallbackError) {
        console.error("Fallback fetch also failed:", fallbackError);
        return [];
      }
    } finally {
      set({ isSearching: false });
    }
  },
}));
