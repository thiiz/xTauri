import { useMemo } from 'react';
import { XtreamSeason, XtreamShow } from '../types/types';

/**
 * Hook que normaliza os detalhes de uma série para garantir que sempre tenha
 * a propriedade seasons, mesmo quando a API retorna apenas episodes.
 */
export function useNormalizedSeriesDetails(seriesDetails: XtreamShow | null) {
  return useMemo(() => {
    if (!seriesDetails) return null;

    // Se já tem seasons, retorna como está
    if (seriesDetails.seasons && seriesDetails.seasons.length > 0) {
      return seriesDetails;
    }

    // Se não tem seasons mas tem episodes, cria seasons virtuais
    if (seriesDetails.episodes && Object.keys(seriesDetails.episodes).length > 0) {
      const virtualSeasons: XtreamSeason[] = Object.keys(seriesDetails.episodes)
        .sort((a, b) => parseInt(a) - parseInt(b))
        .map((seasonKey) => {
          const episodeCount = seriesDetails.episodes[seasonKey]?.length || 0;
          const seasonNumber = parseInt(seasonKey) || 1;

          return {
            air_date: seriesDetails.info.releaseDate || seriesDetails.info.release_date || null,
            cover: seriesDetails.info.cover || '',
            cover_big: seriesDetails.info.cover || '',
            episode_count: episodeCount,
            id: seasonNumber,
            name: `Season ${seasonNumber}`,
            overview: '',
            season_number: seasonNumber,
            vote_average: 0,
          };
        });

      return {
        ...seriesDetails,
        seasons: virtualSeasons,
      };
    }

    // Se não tem nem seasons nem episodes, retorna com array vazio
    return {
      ...seriesDetails,
      seasons: [],
    };
  }, [seriesDetails]);
}
