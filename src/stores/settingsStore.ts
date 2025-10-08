import { invoke } from "@tauri-apps/api/core";
import { create } from "zustand";
import type { ChannelList } from "../types/settings";

interface SettingsState {
  // Channel lists
  channelLists: ChannelList[];
  channelListName: string | null;

  // Player settings
  enablePreview: boolean;
  muteOnStart: boolean;
  showControls: boolean;
  autoplay: boolean;
  volume: number; // 0-1
  isMuted: boolean;

  // Cache settings
  cacheDuration: number; // in hours

  // Actions
  setChannelLists: (lists: ChannelList[]) => void;
  setChannelListName: (name: string | null) => void;

  // Channel list operations
  fetchChannelLists: () => Promise<void>;
  getChannelListName: (selectedChannelListId: number | null) => Promise<string>;

  // Player settings actions
  setEnablePreview: (enabled: boolean) => void;
  saveEnablePreview: () => Promise<void>;
  fetchEnablePreview: () => Promise<void>;

  // Cache settings actions
  setCacheDuration: (duration: number) => void;
  saveCacheDuration: () => Promise<void>;
  fetchCacheDuration: () => Promise<void>;

  // New actions
  setMuteOnStart: (enabled: boolean) => void;
  saveMuteOnStart: () => Promise<void>;
  fetchMuteOnStart: () => Promise<void>;
  setShowControls: (enabled: boolean) => void;
  saveShowControls: () => Promise<void>;
  fetchShowControls: () => Promise<void>;
  setAutoplay: (enabled: boolean) => void;
  saveAutoplay: () => Promise<void>;
  fetchAutoplay: () => Promise<void>;
  setVolume: (volume: number) => void;
  saveVolume: () => Promise<void>;
  fetchVolume: () => Promise<void>;
  setIsMuted: (muted: boolean) => void;
  saveIsMuted: () => Promise<void>;
  fetchIsMuted: () => Promise<void>;
}

export const useSettingsStore = create<SettingsState>((set, get) => ({
  // Initial state
  channelLists: [],
  channelListName: null,
  enablePreview: true,
  muteOnStart: false,
  showControls: true,
  autoplay: false,
  volume: 1,
  isMuted: false,
  cacheDuration: 24,

  // Basic setters
  setChannelLists: (channelLists) => set({ channelLists }),
  setChannelListName: (channelListName) => set({ channelListName }),
  setEnablePreview: (enablePreview) => set({ enablePreview }),
  setMuteOnStart: (muteOnStart) => set({ muteOnStart }),
  setShowControls: (showControls) => set({ showControls }),
  setAutoplay: (autoplay) => set({ autoplay }),
  setVolume: (volume) => set({ volume }),
  setIsMuted: (isMuted) => set({ isMuted }),
  setCacheDuration: (cacheDuration) => set({ cacheDuration }),

  // Channel list operations
  fetchChannelLists: async () => {
    try {
      const lists = await invoke<ChannelList[]>("get_channel_lists");
      set({ channelLists: lists });
    } catch (error) {
      console.error("Failed to fetch channel lists:", error);
      set({ channelLists: [] });
    }
  },

  getChannelListName: async (selectedChannelListId) => {
    if (selectedChannelListId === null) {
      set({ channelListName: null });
      return "";
    }

    const { channelLists } = get();

    // First check if we already have the lists loaded
    if (channelLists.length > 0) {
      const found = channelLists.find((l) => l.id === selectedChannelListId);
      const name = found ? found.name : "";
      set({ channelListName: name });
      return name;
    }

    // If not loaded, fetch them
    try {
      const lists = await invoke<ChannelList[]>("get_channel_lists");
      set({ channelLists: lists });
      const found = lists.find((l) => l.id === selectedChannelListId);
      const name = found ? found.name : "";
      set({ channelListName: name });
      return name;
    } catch (error) {
      console.error("Failed to fetch channel lists:", error);
      set({ channelListName: null });
      return "";
    }
  },

  // Player settings actions
  saveEnablePreview: async () => {
    const { enablePreview } = get();
    await invoke("set_enable_preview", { enabled: enablePreview });
  },

  fetchEnablePreview: async () => {
    const enabled = await invoke<boolean>("get_enable_preview");
    set({ enablePreview: enabled });
  },

  saveMuteOnStart: async () => {
    const { muteOnStart } = get();
    await invoke("set_mute_on_start", { enabled: muteOnStart });
  },

  fetchMuteOnStart: async () => {
    const enabled = await invoke<boolean>("get_mute_on_start");
    set({ muteOnStart: enabled });
  },

  saveShowControls: async () => {
    const { showControls } = get();
    await invoke("set_show_controls", { enabled: showControls });
  },

  fetchShowControls: async () => {
    const enabled = await invoke<boolean>("get_show_controls");
    set({ showControls: enabled });
  },

  saveAutoplay: async () => {
    const { autoplay } = get();
    await invoke("set_autoplay", { enabled: autoplay });
  },

  fetchAutoplay: async () => {
    const enabled = await invoke<boolean>("get_autoplay");
    set({ autoplay: enabled });
  },

  saveVolume: async () => {
    const { volume } = get();
    await invoke("set_volume", { volume });
  },

  fetchVolume: async () => {
    const volume = await invoke<number>("get_volume");
    set({ volume });
  },

  saveIsMuted: async () => {
    const { isMuted } = get();
    await invoke("set_is_muted", { muted: isMuted });
  },

  fetchIsMuted: async () => {
    const muted = await invoke<boolean>("get_is_muted");
    set({ isMuted: muted });
  },

  // Cache settings actions
  saveCacheDuration: async () => {
    const { cacheDuration } = get();
    await invoke("set_cache_duration", { hours: cacheDuration });
  },

  fetchCacheDuration: async () => {
    const duration = await invoke<number>("get_cache_duration");
    set({ cacheDuration: duration });
  },
}));
