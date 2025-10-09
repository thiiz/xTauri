use serde::{Deserialize, Serialize};
use crate::content_cache::{XtreamChannel, XtreamMovie, XtreamSeries};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    pub channels: Vec<XtreamChannel>,
    pub movies: Vec<XtreamMovie>,
    pub series: Vec<XtreamSeries>,
    pub total_results: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchOptions {
    pub query: String,
    pub search_channels: bool,
    pub search_movies: bool,
    pub search_series: bool,
    pub case_sensitive: bool,
    pub max_results_per_type: Option<usize>,
}

impl Default for SearchOptions {
    fn default() -> Self {
        Self {
            query: String::new(),
            search_channels: true,
            search_movies: true,
            search_series: true,
            case_sensitive: false,
            max_results_per_type: None,
        }
    }
}

/// Search across all content types
pub fn search_all_content(
    channels: &[XtreamChannel],
    movies: &[XtreamMovie],
    series: &[XtreamSeries],
    options: &SearchOptions,
) -> SearchResult {
    let query = if options.case_sensitive {
        options.query.clone()
    } else {
        options.query.to_lowercase()
    };

    let mut result = SearchResult {
        channels: Vec::new(),
        movies: Vec::new(),
        series: Vec::new(),
        total_results: 0,
    };

    if query.is_empty() {
        return result;
    }

    // Search channels
    if options.search_channels {
        result.channels = search_channels(channels, &query, options.case_sensitive);
        if let Some(max) = options.max_results_per_type {
            result.channels.truncate(max);
        }
    }

    // Search movies
    if options.search_movies {
        result.movies = search_movies(movies, &query, options.case_sensitive);
        if let Some(max) = options.max_results_per_type {
            result.movies.truncate(max);
        }
    }

    // Search series
    if options.search_series {
        result.series = search_series(series, &query, options.case_sensitive);
        if let Some(max) = options.max_results_per_type {
            result.series.truncate(max);
        }
    }

    result.total_results = result.channels.len() + result.movies.len() + result.series.len();
    result
}

/// Search channels by name
pub fn search_channels(
    channels: &[XtreamChannel],
    query: &str,
    case_sensitive: bool,
) -> Vec<XtreamChannel> {
    channels
        .iter()
        .filter(|channel| {
            let name = if case_sensitive {
                channel.name.clone()
            } else {
                channel.name.to_lowercase()
            };
            name.contains(query)
        })
        .cloned()
        .collect()
}

/// Search movies by name
pub fn search_movies(
    movies: &[XtreamMovie],
    query: &str,
    case_sensitive: bool,
) -> Vec<XtreamMovie> {
    movies
        .iter()
        .filter(|movie| {
            let name = if case_sensitive {
                movie.name.clone()
            } else {
                movie.name.to_lowercase()
            };
            name.contains(query)
        })
        .cloned()
        .collect()
}

/// Search series by name
pub fn search_series(
    series: &[XtreamSeries],
    query: &str,
    case_sensitive: bool,
) -> Vec<XtreamSeries> {
    series
        .iter()
        .filter(|show| {
            let name = if case_sensitive {
                show.name.clone()
            } else {
                show.name.to_lowercase()
            };
            name.contains(query)
        })
        .cloned()
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_channel(id: &str, name: &str) -> XtreamChannel {
        XtreamChannel {
            num: Some(1),
            name: name.to_string(),
            stream_type: Some("live".to_string()),
            stream_id: id.parse().unwrap_or(0),
            stream_icon: None,
            thumbnail: None,
            epg_channel_id: None,
            added: None,
            category_id: None,
            custom_sid: None,
            tv_archive: None,
            direct_source: None,
            tv_archive_duration: None,
        }
    }

    fn create_test_movie(id: &str, name: &str) -> XtreamMovie {
        XtreamMovie {
            num: Some(1),
            name: name.to_string(),
            title: None,
            year: None,
            stream_type: Some("movie".to_string()),
            stream_id: id.parse().unwrap_or(0),
            stream_icon: None,
            rating: None,
            rating_5based: None,
            genre: None,
            added: None,
            episode_run_time: None,
            category_id: None,
            container_extension: None,
            custom_sid: None,
            direct_source: None,
            release_date: None,
            cast: None,
            director: None,
            plot: None,
            youtube_trailer: None,
        }
    }

    fn create_test_series(id: &str, name: &str) -> XtreamSeries {
        XtreamSeries {
            num: Some(1),
            name: name.to_string(),
            title: None,
            series_id: id.parse().unwrap_or(0),
            year: None,
            cover: None,
            plot: None,
            cast: None,
            director: None,
            genre: None,
            release_date: None,
            last_modified: None,
            rating: None,
            rating_5based: None,
            episode_run_time: None,
            category_id: None,
        }
    }

    #[test]
    fn test_search_channels() {
        let channels = vec![
            create_test_channel("1", "HBO"),
            create_test_channel("2", "ESPN Sports"),
            create_test_channel("3", "Discovery Channel"),
        ];

        let results = search_channels(&channels, "hbo", false);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].name, "HBO");

        let results = search_channels(&channels, "sport", false);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].name, "ESPN Sports");
    }

    #[test]
    fn test_search_movies() {
        let movies = vec![
            create_test_movie("1", "The Matrix"),
            create_test_movie("2", "Matrix Reloaded"),
            create_test_movie("3", "Inception"),
        ];

        let results = search_movies(&movies, "matrix", false);
        assert_eq!(results.len(), 2);

        let results = search_movies(&movies, "inception", false);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].name, "Inception");
    }

    #[test]
    fn test_search_series() {
        let series = vec![
            create_test_series("1", "Breaking Bad"),
            create_test_series("2", "Better Call Saul"),
            create_test_series("3", "Game of Thrones"),
        ];

        let results = search_series(&series, "break", false);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].name, "Breaking Bad");

        let results = search_series(&series, "game", false);
        assert_eq!(results.len(), 1);
    }

    #[test]
    fn test_search_all_content() {
        let channels = vec![create_test_channel("1", "HBO")];
        let movies = vec![create_test_movie("1", "The Matrix")];
        let series = vec![create_test_series("1", "Breaking Bad")];

        let options = SearchOptions {
            query: "h".to_string(),
            ..Default::default()
        };

        let results = search_all_content(&channels, &movies, &series, &options);
        assert_eq!(results.total_results, 2); // HBO and The Matrix
    }

    #[test]
    fn test_case_sensitive_search() {
        let channels = vec![create_test_channel("1", "HBO")];

        let results = search_channels(&channels, "hbo", false);
        assert_eq!(results.len(), 1);

        let results = search_channels(&channels, "hbo", true);
        assert_eq!(results.len(), 0);

        let results = search_channels(&channels, "HBO", true);
        assert_eq!(results.len(), 1);
    }

    #[test]
    fn test_max_results_per_type() {
        let channels = vec![
            create_test_channel("1", "HBO 1"),
            create_test_channel("2", "HBO 2"),
            create_test_channel("3", "HBO 3"),
        ];
        let movies = vec![];
        let series = vec![];

        let options = SearchOptions {
            query: "hbo".to_string(),
            max_results_per_type: Some(2),
            ..Default::default()
        };

        let results = search_all_content(&channels, &movies, &series, &options);
        assert_eq!(results.channels.len(), 2);
    }
}
