use serde::{Deserialize, Serialize};
use crate::content_cache::{XtreamChannel, XtreamMovie, XtreamSeries};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChannelFilter {
    pub name: Option<String>,
    pub category_id: Option<String>,
    pub group: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MovieFilter {
    pub name: Option<String>,
    pub category_id: Option<String>,
    pub genre: Option<String>,
    pub year: Option<String>,
    pub min_rating: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SeriesFilter {
    pub name: Option<String>,
    pub category_id: Option<String>,
    pub genre: Option<String>,
    pub year: Option<String>,
    pub min_rating: Option<f64>,
}

/// Filter channels based on criteria
pub fn filter_channels(
    channels: &[XtreamChannel],
    filter: &ChannelFilter,
) -> Vec<XtreamChannel> {
    channels
        .iter()
        .filter(|channel| {
            // Name filter
            if let Some(name) = &filter.name {
                if !channel.name.to_lowercase().contains(&name.to_lowercase()) {
                    return false;
                }
            }

            // Category filter
            if let Some(category_id) = &filter.category_id {
                if let Some(channel_category_id) = &channel.category_id {
                    if channel_category_id != category_id {
                        return false;
                    }
                } else {
                    return false;
                }
            }

            // Group filter (using category_id as group for now)
            if let Some(group) = &filter.group {
                if let Some(channel_category_id) = &channel.category_id {
                    if channel_category_id != group {
                        return false;
                    }
                } else {
                    return false;
                }
            }

            true
        })
        .cloned()
        .collect()
}

/// Filter movies based on criteria
pub fn filter_movies(
    movies: &[XtreamMovie],
    filter: &MovieFilter,
) -> Vec<XtreamMovie> {
    movies
        .iter()
        .filter(|movie| {
            // Name filter
            if let Some(name) = &filter.name {
                if !movie.name.to_lowercase().contains(&name.to_lowercase()) {
                    return false;
                }
            }

            // Category filter
            if let Some(category_id) = &filter.category_id {
                if let Some(movie_category_id) = &movie.category_id {
                    if movie_category_id != category_id {
                        return false;
                    }
                } else {
                    return false;
                }
            }

            // Year filter
            if let Some(year) = &filter.year {
                if let Some(movie_year) = &movie.year {
                    if movie_year != year {
                        return false;
                    }
                } else {
                    return false;
                }
            }

            // Rating filter
            if let Some(min_rating) = filter.min_rating {
                if let Some(rating) = movie.rating_5based {
                    if rating < min_rating {
                        return false;
                    }
                } else {
                    return false;
                }
            }

            true
        })
        .cloned()
        .collect()
}

/// Filter series based on criteria
pub fn filter_series(
    series: &[XtreamSeries],
    filter: &SeriesFilter,
) -> Vec<XtreamSeries> {
    series
        .iter()
        .filter(|show| {
            // Name filter
            if let Some(name) = &filter.name {
                if !show.name.to_lowercase().contains(&name.to_lowercase()) {
                    return false;
                }
            }

            // Category filter
            if let Some(category_id) = &filter.category_id {
                if let Some(show_category_id) = &show.category_id {
                    if show_category_id != category_id {
                        return false;
                    }
                } else {
                    return false;
                }
            }

            // Genre filter
            if let Some(genre) = &filter.genre {
                if let Some(show_genre) = &show.genre {
                    if !show_genre.to_lowercase().contains(&genre.to_lowercase()) {
                        return false;
                    }
                } else {
                    return false;
                }
            }

            // Year filter
            if let Some(year) = &filter.year {
                if let Some(release_date) = &show.release_date {
                    if !release_date.starts_with(year) {
                        return false;
                    }
                } else {
                    return false;
                }
            }

            // Rating filter
            if let Some(min_rating) = filter.min_rating {
                if let Some(rating) = show.rating_5based {
                    if rating < min_rating {
                        return false;
                    }
                } else {
                    return false;
                }
            }

            true
        })
        .cloned()
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_channel(id: &str, name: &str, category_id: Option<&str>) -> XtreamChannel {
        XtreamChannel {
            num: Some(1),
            name: name.to_string(),
            stream_type: Some("live".to_string()),
            stream_id: id.parse().unwrap_or(0),
            stream_icon: None,
            thumbnail: None,
            epg_channel_id: None,
            added: None,
            category_id: category_id.map(|s| s.to_string()),
            custom_sid: None,
            tv_archive: None,
            direct_source: None,
            tv_archive_duration: None,
        }
    }

    fn create_test_movie(
        id: &str,
        name: &str,
        category_id: Option<&str>,
        year: Option<&str>,
        rating: Option<f64>,
    ) -> XtreamMovie {
        XtreamMovie {
            num: Some(1),
            name: name.to_string(),
            title: None,
            year: year.map(|s| s.to_string()),
            stream_type: Some("movie".to_string()),
            stream_id: id.parse().unwrap_or(0),
            stream_icon: None,
            rating: None,
            rating_5based: rating,
            genre: None,
            added: None,
            episode_run_time: None,
            category_id: category_id.map(|s| s.to_string()),
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

    fn create_test_series(
        id: &str,
        name: &str,
        category_id: Option<&str>,
        genre: Option<&str>,
        year: Option<&str>,
        rating: Option<f64>,
    ) -> XtreamSeries {
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
            genre: genre.map(|s| s.to_string()),
            release_date: year.map(|s| s.to_string()),
            last_modified: None,
            rating: None,
            rating_5based: rating,
            episode_run_time: None,
            category_id: category_id.map(|s| s.to_string()),
        }
    }

    #[test]
    fn test_filter_channels_by_name() {
        let channels = vec![
            create_test_channel("1", "HBO", None),
            create_test_channel("2", "ESPN Sports", None),
            create_test_channel("3", "Discovery Channel", None),
        ];

        let filter = ChannelFilter {
            name: Some("sport".to_string()),
            category_id: None,
            group: None,
        };

        let results = filter_channels(&channels, &filter);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].name, "ESPN Sports");
    }

    #[test]
    fn test_filter_channels_by_category() {
        let channels = vec![
            create_test_channel("1", "HBO", Some("1")),
            create_test_channel("2", "ESPN Sports", Some("2")),
            create_test_channel("3", "Discovery Channel", Some("1")),
        ];

        let filter = ChannelFilter {
            name: None,
            category_id: Some("1".to_string()),
            group: None,
        };

        let results = filter_channels(&channels, &filter);
        assert_eq!(results.len(), 2);
    }

    #[test]
    fn test_filter_movies_by_year() {
        let movies = vec![
            create_test_movie("1", "The Matrix", None, Some("1999"), None),
            create_test_movie("2", "Inception", None, Some("2010"), None),
            create_test_movie("3", "Interstellar", None, Some("2014"), None),
        ];

        let filter = MovieFilter {
            name: None,
            category_id: None,
            genre: None,
            year: Some("2010".to_string()),
            min_rating: None,
        };

        let results = filter_movies(&movies, &filter);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].name, "Inception");
    }

    #[test]
    fn test_filter_movies_by_rating() {
        let movies = vec![
            create_test_movie("1", "The Matrix", None, None, Some(4.5)),
            create_test_movie("2", "Inception", None, None, Some(4.8)),
            create_test_movie("3", "Bad Movie", None, None, Some(2.0)),
        ];

        let filter = MovieFilter {
            name: None,
            category_id: None,
            genre: None,
            year: None,
            min_rating: Some(4.0),
        };

        let results = filter_movies(&movies, &filter);
        assert_eq!(results.len(), 2);
    }

    #[test]
    fn test_filter_series_by_genre() {
        let series = vec![
            create_test_series("1", "Breaking Bad", None, Some("Drama"), None, None),
            create_test_series("2", "The Office", None, Some("Comedy"), None, None),
            create_test_series("3", "Better Call Saul", None, Some("Drama"), None, None),
        ];

        let filter = SeriesFilter {
            name: None,
            category_id: None,
            genre: Some("drama".to_string()),
            year: None,
            min_rating: None,
        };

        let results = filter_series(&series, &filter);
        assert_eq!(results.len(), 2);
    }

    #[test]
    fn test_combined_filters() {
        let movies = vec![
            create_test_movie("1", "The Matrix", Some("1"), Some("1999"), Some(4.5)),
            create_test_movie("2", "Matrix Reloaded", Some("1"), Some("2003"), Some(4.0)),
            create_test_movie("3", "Inception", Some("2"), Some("2010"), Some(4.8)),
        ];

        let filter = MovieFilter {
            name: Some("matrix".to_string()),
            category_id: Some("1".to_string()),
            genre: None,
            year: None,
            min_rating: Some(4.2),
        };

        let results = filter_movies(&movies, &filter);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].name, "The Matrix");
    }
}
