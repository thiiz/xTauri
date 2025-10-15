import { invoke } from "@tauri-apps/api/core";
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
  groups: string[];
  history: Channel[];
  selectedChannel: Channel | null;

  // Actions
  setChannels: (channels: Channel[]) => void;
  setGroups: (groups: string[]) => void;
  setHistory: (history: Channel[]) => void;
  setSelectedChannel: (channel: Channel | null) => void;

  // API actions
  fetchHistory: () => Promise<void>;
}

export const useChannelStore = create<ChannelState>((set) => ({
  // Initial state
  channels: [],
  groups: [],
  history: [],
  selectedChannel: null,

  // Basic setters
  setChannels: (channels) => set({ channels }),
  setGroups: (groups) => set({ groups }),
  setHistory: (history) => set({ history }),
  setSelectedChannel: (selectedChannel) => set({ selectedChannel }),

  // API actions
  fetchHistory: async () => {
    const fetchedHistory = await invoke<Channel[]>("get_history");
    set({ history: fetchedHistory });
  },
}));
