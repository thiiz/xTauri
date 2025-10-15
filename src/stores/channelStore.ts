import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { create } from "zustand";
import type { Channel } from "../types/channel";

export interface ChannelLoadingStatus {
  progress: number; // 0.0 to 1.0
  message: string;
  channel_count?: number;
  is_complete: boolean;
}

interface ChannelState {
  // Data
  channels: Channel[];
  favorites: Channel[];
  groups: string[];
  history: Channel[];
  selectedChannel: Channel | null;

  // Actions
  setChannels: (channels: Channel[]) => void;
  setFavorites: (favorites: Channel[]) => void;
  setGroups: (groups: string[]) => void;
  setHistory: (history: Channel[]) => void;
  setSelectedChannel: (channel: Channel | null) => void;

  // API actions
  fetchFavorites: () => Promise<void>;
  fetchHistory: () => Promise<void>;
  toggleFavorite: (channel: Channel) => Promise<void>;
}

export const useChannelStore = create<ChannelState>((set, get) => ({
  // Initial state
  channels: [],
  favorites: [],
  groups: [],
  history: [],
  selectedChannel: null,

  // Basic setters
  setChannels: (channels) => set({ channels }),
  setFavorites: (favorites) => set({ favorites }),
  setGroups: (groups) => set({ groups }),
  setHistory: (history) => set({ history }),
  setSelectedChannel: (selectedChannel) => set({ selectedChannel }),

  // API actions
  fetchFavorites: async () => {
    const fetchedFavorites = await invoke<Channel[]>("get_favorites");
    set({ favorites: fetchedFavorites });
  },

  fetchHistory: async () => {
    const fetchedHistory = await invoke<Channel[]>("get_history");
    set({ history: fetchedHistory });
  },

  toggleFavorite: async (channel) => {
    const { favorites } = get();
    const isFavorite = favorites.some((fav) => fav.name === channel.name);

    if (isFavorite) {
      await invoke("remove_favorite", { name: channel.name });
    } else {
      await invoke("add_favorite", { channel });
    }

    // Refresh favorites
    get().fetchFavorites();
  },
}));

// Setup event listeners for operations
listen<string>("favorite_operation", (event) => {
  console.log("Favorite operation:", event.payload);
});

listen<string>("favorites_loading", (event) => {
  console.log("Favorites loading:", event.payload);
});

listen<string>("history_loading", (event) => {
  console.log("History loading:", event.payload);
});
