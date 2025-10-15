import { useMemo } from 'react';
import type { ContentItem } from '../components/VideoPlayerWrapper';
import type { Channel } from '../types/channel';
import type { XtreamChannel, XtreamMoviesListing, XtreamShow } from '../types/types';

interface ContentMetadata {
  title: string;
  statusText: string;
  qualityBadge: string;
  metadata: {
    genre?: string | null;
    year?: string | null;
    rating?: number;
    duration?: number | null;
    cast?: string | null;
    director?: string | null;
    description?: string | null;
  } | null;
}

/**
 * Custom hook to extract and format content metadata
 * Memoizes expensive computations for better performance
 */
export const useContentMetadata = (activeContent: ContentItem | null): ContentMetadata => {
  const title = useMemo(() => {
    if (!activeContent) return '';

    switch (activeContent.type) {
      case 'channel':
        return (activeContent.data as Channel).name;
      case 'xtream-channel':
        return (activeContent.data as XtreamChannel).name;
      case 'xtream-movie':
        return (activeContent.data as XtreamMoviesListing).title || (activeContent.data as XtreamMoviesListing).name;
      case 'xtream-series':
        return (activeContent.data as XtreamShow).info?.title || (activeContent.data as XtreamShow).info?.name || '';
      default:
        return '';
    }
  }, [activeContent]);

  const statusText = useMemo(() => {
    if (!activeContent) return '';

    switch (activeContent.type) {
      case 'channel':
      case 'xtream-channel':
        return 'Live';
      case 'xtream-movie':
        return 'Movie';
      case 'xtream-series':
        return 'Series';
      default:
        return '';
    }
  }, [activeContent]);

  const qualityBadge = useMemo(() => {
    if (!activeContent) return 'HD';

    switch (activeContent.type) {
      case 'channel':
        return (activeContent.data as Channel).resolution || 'HD';
      case 'xtream-channel':
      case 'xtream-movie':
      case 'xtream-series':
        return 'HD';
      default:
        return 'HD';
    }
  }, [activeContent]);

  const metadata = useMemo(() => {
    if (!activeContent) return null;

    switch (activeContent.type) {
      case 'xtream-movie': {
        const movie = activeContent.data as XtreamMoviesListing;
        return {
          genre: movie.genre,
          year: movie.year,
          rating: movie.rating,
          duration: movie.episode_run_time,
          cast: movie.cast,
          director: movie.director,
          description: movie.plot
        };
      }
      case 'xtream-series': {
        const series = activeContent.data as XtreamShow;
        return {
          genre: series.info?.genre,
          year: series.info?.year,
          rating: parseFloat(series.info?.rating || '0'),
          cast: series.info?.cast,
          director: series.info?.director,
          description: series.info?.plot
        };
      }
      default:
        return null;
    }
  }, [activeContent]);

  return {
    title,
    statusText,
    qualityBadge,
    metadata
  };
};
