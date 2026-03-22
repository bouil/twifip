use serde::Deserialize;
use std::fmt::Display;

#[derive(Deserialize, Debug, Clone, PartialEq)]
pub struct Track {
    pub artist: String,
    pub title: String,
    pub album: Option<String>,
    pub start_time: i64,
}

impl Display for Track {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "TrackInfo{{title='{}', artist='{}', album='{}', startTime={}}}",
            self.title,
            self.artist,
            self.album.clone().unwrap_or(String::from("null")),
            self.start_time
        )
    }
}

impl Track {
    pub(crate) fn create(
        artist: String,
        title: String,
        album: Option<String>,
        start_time: i64,
    ) -> Option<Track> {
        if title.contains("BONNE NUIT SUR FIP")
            || title.contains("FIP ACTUALITE")
            || (artist.eq("La radio la plus éclectique du monde")
                && title.eq("Le direct")
                && album.is_none())
        {
            // ignore track
            None
        } else {
            let fixed_album = album.map(|a| Self::fix_album(a));
            Some(Track {
                artist,
                title,
                album: fixed_album,
                start_time,
            })
        }
    }

    /// Remove the trailing year from the album name
    fn fix_album(album: String) -> String {
        let space_position = album.rfind(' ');
        space_position
            .and_then(|position| {
                let end_part = &album[position + 1..];
                if end_part.len() == 4 {
                    end_part.parse::<u16>().ok().map(|_| String::from(&album[0..position]))
                } else {
                    None
                }
            })
            .unwrap_or(album)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ignored_track() {
        let track = Track::create(
            String::from("Artist"),
            String::from("FIP ACTUALITE"),
            Some(String::from("Album")),
            1740168451,
        );

        assert_eq!(track, None);
    }

    #[test]
    fn test_ignored_track_2() {
        let track = Track::create(
            String::from("La radio la plus éclectique du monde"),
            String::from("Le direct"),
            None,
            1740168451,
        );

        assert_eq!(track, None);
    }

    #[test]
    fn test_not_ignored_track() {
        let track = Track::create(
            String::from("La radio la plus éclectique du monde"),
            String::from("Le direct"),
            Some(String::from("Some album")),
            1740168451,
        );

        assert_ne!(track, None);
    }
    #[test]
    fn test_not_ignored_track_2() {
        let track = Track::create(
            String::from("La radio la plus éclectique du monde"),
            String::from("Title"),
            None,
            1740168451,
        );

        assert_ne!(track, None);
    }

    #[test]
    fn test_create_track() {
        let track = Track::create(
            String::from("Artist"),
            String::from("Title"),
            Some(String::from("Album")),
            1740168451,
        )
        .unwrap();
        assert_eq!(track.album.clone().unwrap().to_string(), "Album");
        assert_eq!(
            track.to_string(),
            "TrackInfo{title='Title', artist='Artist', album='Album', startTime=1740168451}"
        );
    }

    #[test]
    fn test_create_track_with_year() {
        let track = Track::create(
            String::from("Artist"),
            String::from("Title"),
            Some(String::from("Album 2020")),
            1740168451,
        )
        .unwrap();
        assert_eq!(track.album.clone().unwrap().to_string(), "Album");
    }

    #[test]
    fn test_create_track_with_numbers() {
        let track = Track::create(
            String::from("Artist"),
            String::from("Title"),
            Some(String::from("Album 202")),
            1740168451,
        )
        .unwrap();
        assert_eq!(track.album.clone().unwrap().to_string(), "Album 202");
    }

    #[test]
    fn test_create_track_with_five_digit_number_not_stripped() {
        let track = Track::create(
            String::from("Artist"),
            String::from("Title"),
            Some(String::from("Album 20201")),
            1740168451,
        )
        .unwrap();
        assert_eq!(track.album.clone().unwrap().to_string(), "Album 20201");
    }
}
