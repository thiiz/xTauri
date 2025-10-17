/**
 * Application-wide constants
 * Centralized to avoid magic numbers and improve maintainability
 */

// Performance & Timing
export const DEBOUNCE_DELAY = {
  SEARCH: 300,
  INPUT: 300,
  RESIZE: 150,
  SCROLL: 100,
} as const;

export const THROTTLE_DELAY = {
  SCROLL: 100,
  RESIZE: 200,
  MOUSE_MOVE: 50,
} as const;



// Pagination
export const PAGINATION = {
  ITEMS_PER_PAGE: 50,
  MOVIES_PER_ROW: 6,
  SERIES_PER_ROW: 6,
  CHANNELS_PER_PAGE: 200,
  OVERSCAN_COUNT: 2,
} as const;

// Search
export const SEARCH = {
  MIN_QUERY_LENGTH: 2,
  MAX_QUERY_LENGTH: 100,
  DEBOUNCE_MS: 300,
} as const;

// Video Player
export const VIDEO_PLAYER = {
  DEFAULT_VOLUME: 1.0,
  VOLUME_STEP: 0.1,
  SEEK_STEP: 10, // seconds
  SEEK_STEP_LARGE: 30, // seconds
  CONTROLS_HIDE_DELAY: 3000, // ms
  BUFFER_AHEAD: 30, // seconds
} as const;



// API
export const API = {
  TIMEOUT: 30000, // 30 seconds
  MAX_RETRIES: 3,
  RETRY_DELAY: 1000, // ms
  RETRY_BACKOFF_MULTIPLIER: 2,
} as const;

// Local Storage Keys
export const STORAGE_KEYS = {
  ACTIVE_PROFILE: 'active_profile',
  THEME: 'theme',
  VOLUME: 'volume',
  MUTED: 'muted',
  PLAYBACK_RATE: 'playback_rate',
  SUBTITLE_SETTINGS: 'subtitle_settings',
  RECENT_SEARCHES: 'recent_searches',
} as const;

// UI
export const UI = {
  SIDEBAR_WIDTH: 240,
  HEADER_HEIGHT: 60,
  FOOTER_HEIGHT: 40,
  MOBILE_BREAKPOINT: 768,
  TABLET_BREAKPOINT: 1024,
  DESKTOP_BREAKPOINT: 1280,
} as const;

// Keyboard Shortcuts
export const KEYBOARD = {
  SPACE: ' ',
  ENTER: 'Enter',
  ESCAPE: 'Escape',
  ARROW_UP: 'ArrowUp',
  ARROW_DOWN: 'ArrowDown',
  ARROW_LEFT: 'ArrowLeft',
  ARROW_RIGHT: 'ArrowRight',
  TAB: 'Tab',
} as const;

// Content Types
export const CONTENT_TYPE = {
  CHANNEL: 'channel',
  XTREAM_CHANNEL: 'xtream-channel',
  XTREAM_MOVIE: 'xtream-movie',
  XTREAM_SERIES: 'xtream-series',
} as const;

// Error Messages
export const ERROR_MESSAGES = {
  NETWORK_ERROR: 'Network error. Please check your connection.',
  AUTH_ERROR: 'Authentication failed. Please check your credentials.',
  NOT_FOUND: 'Content not found.',
  TIMEOUT: 'Request timed out. Please try again.',
  UNKNOWN: 'An unknown error occurred.',
} as const;

// Success Messages
export const SUCCESS_MESSAGES = {
  PROFILE_CREATED: 'Profile created successfully.',
  PROFILE_UPDATED: 'Profile updated successfully.',
  PROFILE_DELETED: 'Profile deleted successfully.',
  FAVORITE_ADDED: 'Added to favorites.',
  FAVORITE_REMOVED: 'Removed from favorites.',
} as const;

// Validation
export const VALIDATION = {
  MIN_PASSWORD_LENGTH: 6,
  MAX_PASSWORD_LENGTH: 128,
  MIN_USERNAME_LENGTH: 3,
  MAX_USERNAME_LENGTH: 50,
  URL_PATTERN: /^https?:\/\/.+/,
} as const;

// Feature Flags (for gradual rollout of new features)
export const FEATURES = {
  ENABLE_EPG: true,
  ENABLE_FAVORITES: true,
  ENABLE_HISTORY: true,
  ENABLE_SEARCH: true,
  ENABLE_SYNC: true,
  ENABLE_OFFLINE_MODE: false, // Future feature
  ENABLE_DOWNLOADS: false, // Future feature
} as const;



// Export all as a single object for convenience
export const CONSTANTS = {
  DEBOUNCE_DELAY,
  THROTTLE_DELAY,
  PAGINATION,
  SEARCH,
  VIDEO_PLAYER,
  API,
  STORAGE_KEYS,
  UI,
  KEYBOARD,
  CONTENT_TYPE,
  ERROR_MESSAGES,
  SUCCESS_MESSAGES,
  VALIDATION,
  FEATURES,
} as const;

export default CONSTANTS;
