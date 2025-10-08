/**
 * Utility functions for formatting data across the application
 * Centralized to ensure consistency and reusability
 */

/**
 * Format a rating value to a fixed decimal string
 * @param rating - Rating value (number or string)
 * @returns Formatted rating string or 'N/A'
 */
export function formatRating(rating: number | string | null | undefined): string {
  if (!rating || rating === 0 || rating === '0') return 'N/A';

  const numRating = typeof rating === 'string' ? parseFloat(rating) : rating;
  return isNaN(numRating) ? 'N/A' : numRating.toFixed(1);
}

/**
 * Format a year value
 * @param year - Year string
 * @returns Year string or 'N/A'
 */
export function formatYear(year: string | null | undefined): string {
  return year || 'N/A';
}

/**
 * Format runtime in minutes to human-readable format
 * @param runtime - Runtime in minutes
 * @returns Formatted runtime string (e.g., "1h 30m" or "45m")
 */
export function formatRuntime(runtime: number | null | undefined): string {
  if (!runtime) return '';

  const hours = Math.floor(runtime / 60);
  const minutes = runtime % 60;

  return hours > 0 ? `${hours}h ${minutes}m` : `${minutes}m`;
}

/**
 * Format episode runtime from string to human-readable format
 * @param runtime - Runtime string in minutes
 * @returns Formatted runtime string
 */
export function formatEpisodeRuntime(runtime: string | null | undefined): string {
  if (!runtime) return '';

  const minutes = parseInt(runtime);
  if (isNaN(minutes)) return runtime;

  const hours = Math.floor(minutes / 60);
  const mins = minutes % 60;

  return hours > 0 ? `${hours}h ${mins}m` : `${mins}m`;
}

/**
 * Format duration in seconds to human-readable format
 * @param seconds - Duration in seconds
 * @returns Formatted duration string (e.g., "1:30:45" or "45:30")
 */
export function formatDuration(seconds: number | null | undefined): string {
  if (!seconds || seconds === 0) return '0:00';

  const hours = Math.floor(seconds / 3600);
  const minutes = Math.floor((seconds % 3600) / 60);
  const secs = Math.floor(seconds % 60);

  if (hours > 0) {
    return `${hours}:${minutes.toString().padStart(2, '0')}:${secs.toString().padStart(2, '0')}`;
  }

  return `${minutes}:${secs.toString().padStart(2, '0')}`;
}

/**
 * Format file size in bytes to human-readable format
 * @param bytes - Size in bytes
 * @returns Formatted size string (e.g., "1.5 MB")
 */
export function formatFileSize(bytes: number | null | undefined): string {
  if (!bytes || bytes === 0) return '0 B';

  const units = ['B', 'KB', 'MB', 'GB', 'TB'];
  const k = 1024;
  const i = Math.floor(Math.log(bytes) / Math.log(k));

  return `${(bytes / Math.pow(k, i)).toFixed(2)} ${units[i]}`;
}

/**
 * Format a date to a localized string
 * @param date - Date string or Date object
 * @returns Formatted date string
 */
export function formatDate(date: string | Date | null | undefined): string {
  if (!date) return 'N/A';

  try {
    const dateObj = typeof date === 'string' ? new Date(date) : date;
    return dateObj.toLocaleDateString();
  } catch {
    return 'Invalid Date';
  }
}

/**
 * Format a timestamp to a localized time string
 * @param timestamp - Unix timestamp in seconds
 * @returns Formatted time string
 */
export function formatTimestamp(timestamp: number | null | undefined): string {
  if (!timestamp) return 'N/A';

  try {
    const date = new Date(timestamp * 1000);
    return date.toLocaleTimeString();
  } catch {
    return 'Invalid Time';
  }
}

/**
 * Truncate text to a maximum length with ellipsis
 * @param text - Text to truncate
 * @param maxLength - Maximum length
 * @returns Truncated text
 */
export function truncateText(text: string | null | undefined, maxLength: number): string {
  if (!text) return '';
  if (text.length <= maxLength) return text;

  return `${text.substring(0, maxLength)}...`;
}

/**
 * Format a percentage value
 * @param value - Value between 0 and 1
 * @returns Formatted percentage string
 */
export function formatPercentage(value: number | null | undefined): string {
  if (value === null || value === undefined) return '0%';

  return `${Math.round(value * 100)}%`;
}
