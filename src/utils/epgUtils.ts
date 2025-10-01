import { invoke } from '@tauri-apps/api/core';
import { EnhancedEPGListing, EPGSearchOptions, EPGTimeFilter } from '../types/types';

/**
 * EPG utility functions for formatting, parsing, and manipulating EPG data
 */

/**
 * Format a timestamp for display using the backend formatter
 */
export async function formatEPGTime(timestamp: number, timezone?: string): Promise<string> {
  try {
    return await invoke<string>('format_epg_time', {
      timestamp,
      timezone: timezone || null
    });
  } catch (error) {
    console.error('Failed to format EPG time:', error);
    return new Date(timestamp * 1000).toLocaleString();
  }
}

/**
 * Get the current timestamp for EPG queries
 */
export async function getCurrentTimestamp(): Promise<number> {
  try {
    return await invoke<number>('get_current_timestamp');
  } catch (error) {
    console.error('Failed to get current timestamp:', error);
    return Math.floor(Date.now() / 1000);
  }
}

/**
 * Get timestamp for a specific number of hours from now
 */
export async function getTimestampHoursFromNow(hours: number): Promise<number> {
  try {
    return await invoke<number>('get_timestamp_hours_from_now', { hours });
  } catch (error) {
    console.error('Failed to get future timestamp:', error);
    return Math.floor(Date.now() / 1000) + (hours * 3600);
  }
}

/**
 * Parse EPG data and extract program information
 */
export async function parseEPGPrograms(epgData: any): Promise<EnhancedEPGListing[]> {
  try {
    return await invoke<EnhancedEPGListing[]>('parse_epg_programs', { epgData });
  } catch (error) {
    console.error('Failed to parse EPG programs:', error);
    return [];
  }
}

/**
 * Parse and enhance EPG data with formatted times and additional metadata
 */
export async function parseAndEnhanceEPGData(epgData: any, timezone?: string): Promise<EnhancedEPGListing[]> {
  try {
    return await invoke<EnhancedEPGListing[]>('parse_and_enhance_epg_data', {
      epgData,
      timezone: timezone || null
    });
  } catch (error) {
    console.error('Failed to parse and enhance EPG data:', error);
    return [];
  }
}

/**
 * Filter EPG programs by time range
 */
export async function filterEPGByTimeRange(
  epgData: EnhancedEPGListing[],
  filter: EPGTimeFilter
): Promise<EnhancedEPGListing[]> {
  try {
    return await invoke<EnhancedEPGListing[]>('filter_epg_by_time_range', {
      epgData,
      startTimestamp: filter.start_timestamp || null,
      endTimestamp: filter.end_timestamp || null
    });
  } catch (error) {
    console.error('Failed to filter EPG by time range:', error);
    return epgData;
  }
}

/**
 * Search EPG programs by title or description
 */
export async function searchEPGPrograms(
  epgData: EnhancedEPGListing[],
  options: EPGSearchOptions
): Promise<EnhancedEPGListing[]> {
  try {
    return await invoke<EnhancedEPGListing[]>('search_epg_programs', {
      epgData,
      searchQuery: options.query
    });
  } catch (error) {
    console.error('Failed to search EPG programs:', error);
    return epgData;
  }
}

/**
 * Check if a program is currently airing
 */
export function isProgramCurrent(program: EnhancedEPGListing): boolean {
  if (program.is_current !== undefined) {
    return program.is_current;
  }

  const now = Math.floor(Date.now() / 1000);
  const start = program.start_timestamp || parseInt(program.start);
  const stop = program.stop_timestamp || parseInt(program.stop);

  return now >= start && now <= stop;
}

/**
 * Check if a program has already aired
 */
export function isProgramPast(program: EnhancedEPGListing): boolean {
  if (program.is_past !== undefined) {
    return program.is_past;
  }

  const now = Math.floor(Date.now() / 1000);
  const stop = program.stop_timestamp || parseInt(program.stop);

  return now > stop;
}

/**
 * Check if a program will air in the future
 */
export function isProgramFuture(program: EnhancedEPGListing): boolean {
  if (program.is_future !== undefined) {
    return program.is_future;
  }

  const now = Math.floor(Date.now() / 1000);
  const start = program.start_timestamp || parseInt(program.start);

  return now < start;
}

/**
 * Calculate the progress percentage of a currently airing program
 */
export function getProgramProgress(program: EnhancedEPGListing): number {
  if (program.progress_percent !== undefined) {
    return program.progress_percent;
  }

  if (!isProgramCurrent(program)) {
    return 0;
  }

  const now = Math.floor(Date.now() / 1000);
  const start = program.start_timestamp || parseInt(program.start);
  const stop = program.stop_timestamp || parseInt(program.stop);

  if (stop <= start) {
    return 0;
  }

  const progress = ((now - start) / (stop - start)) * 100;
  return Math.max(0, Math.min(100, Math.round(progress)));
}

/**
 * Format program duration for display
 */
export function formatProgramDuration(program: EnhancedEPGListing): string {
  if (program.formatted_duration) {
    return program.formatted_duration;
  }

  if (program.duration_minutes) {
    const hours = Math.floor(program.duration_minutes / 60);
    const minutes = program.duration_minutes % 60;

    if (hours > 0) {
      return `${hours}h ${minutes}m`;
    } else {
      return `${minutes}m`;
    }
  }

  // Calculate from start and stop times
  const start = program.start_timestamp || parseInt(program.start);
  const stop = program.stop_timestamp || parseInt(program.stop);

  if (stop > start) {
    const durationMinutes = Math.floor((stop - start) / 60);
    const hours = Math.floor(durationMinutes / 60);
    const minutes = durationMinutes % 60;

    if (hours > 0) {
      return `${hours}h ${minutes}m`;
    } else {
      return `${minutes}m`;
    }
  }

  return 'Unknown';
}

/**
 * Get programs for a specific time range
 */
export function getProgramsInTimeRange(
  programs: EnhancedEPGListing[],
  startTime: number,
  endTime: number
): EnhancedEPGListing[] {
  return programs.filter(program => {
    const programStart = program.start_timestamp || parseInt(program.start);
    const programStop = program.stop_timestamp || parseInt(program.stop);

    // Program overlaps if it starts before range ends and ends after range starts
    return programStart < endTime && programStop > startTime;
  });
}

/**
 * Get the current program from a list of programs
 */
export function getCurrentProgram(programs: EnhancedEPGListing[]): EnhancedEPGListing | null {
  return programs.find(program => isProgramCurrent(program)) || null;
}

/**
 * Get the next program from a list of programs
 */
export function getNextProgram(programs: EnhancedEPGListing[]): EnhancedEPGListing | null {
  const futurePrograms = programs
    .filter(program => isProgramFuture(program))
    .sort((a, b) => {
      const startA = a.start_timestamp || parseInt(a.start);
      const startB = b.start_timestamp || parseInt(b.start);
      return startA - startB;
    });

  return futurePrograms[0] || null;
}

/**
 * Group programs by date
 */
export function groupProgramsByDate(programs: EnhancedEPGListing[]): Record<string, EnhancedEPGListing[]> {
  const grouped: Record<string, EnhancedEPGListing[]> = {};

  programs.forEach(program => {
    const timestamp = program.start_timestamp || parseInt(program.start);
    const date = new Date(timestamp * 1000).toDateString();

    if (!grouped[date]) {
      grouped[date] = [];
    }

    grouped[date].push(program);
  });

  // Sort programs within each date
  Object.keys(grouped).forEach(date => {
    grouped[date].sort((a, b) => {
      const startA = a.start_timestamp || parseInt(a.start);
      const startB = b.start_timestamp || parseInt(b.start);
      return startA - startB;
    });
  });

  return grouped;
}

/**
 * Create time filter for common time ranges
 */
export class EPGTimeFilters {
  static async today(): Promise<EPGTimeFilter> {
    const now = await getCurrentTimestamp();
    const startOfDay = Math.floor(now / 86400) * 86400; // Start of current day
    const endOfDay = startOfDay + 86400; // End of current day

    return {
      start_timestamp: startOfDay,
      end_timestamp: endOfDay
    };
  }

  static async next24Hours(): Promise<EPGTimeFilter> {
    const now = await getCurrentTimestamp();
    const next24Hours = await getTimestampHoursFromNow(24);

    return {
      start_timestamp: now,
      end_timestamp: next24Hours
    };
  }

  static async nextWeek(): Promise<EPGTimeFilter> {
    const now = await getCurrentTimestamp();
    const nextWeek = now + (7 * 24 * 3600); // 7 days from now

    return {
      start_timestamp: now,
      end_timestamp: nextWeek
    };
  }

  static async currentHour(): Promise<EPGTimeFilter> {
    const now = await getCurrentTimestamp();
    const startOfHour = Math.floor(now / 3600) * 3600; // Start of current hour
    const endOfHour = startOfHour + 3600; // End of current hour

    return {
      start_timestamp: startOfHour,
      end_timestamp: endOfHour
    };
  }
}