use crate::track::Track;
use log::{error, info};
use serde::Deserialize;
use serde_json::from_str;
use std::{error::Error, fmt};

#[derive(Deserialize, Debug)]
struct FipLine {
    title: String,
}

type TitleLine = FipLine;
type ArtistLine = FipLine;

#[derive(Deserialize, Debug)]
struct Release {
    title: Option<String>,
}

#[derive(Deserialize, Debug)]
struct Song {
    release: Option<Release>,
}

#[derive(Deserialize, Debug)]
struct Now {
    #[serde(rename = "firstLine")]
    first_line: TitleLine,
    #[serde(rename = "secondLine")]
    second_line: ArtistLine,
    song: Option<Song>,
    #[serde(rename = "startTime")]
    start_time: i64,
}

#[derive(Deserialize, Debug)]
struct Fip {
    now: Now,
}

#[derive(Debug)]
struct FipError;

impl Error for FipError {

}

impl fmt::Display for FipError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Error reading or parsing")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fip_json_deserialization() {
        let json = r#"{
            "now": {
                "firstLine": { "title": "Some Artist" },
                "secondLine": { "title": "Some Title" },
                "song": { "release": { "title": "Some Album" } },
                "startTime": 1740168451
            }
        }"#;

        let fip: Fip = serde_json::from_str(json).unwrap();
        assert_eq!(fip.now.first_line.title, "Some Artist");
        assert_eq!(fip.now.second_line.title, "Some Title");
        assert_eq!(fip.now.song.unwrap().release.unwrap().title.unwrap(), "Some Album");
        assert_eq!(fip.now.start_time, 1740168451);
    }
}

pub(crate) fn read_fip() -> Result<Track, Box<dyn Error>> {
    // check if https://api.radiofrance.fr/livemeta/live/7/transistor_musical_player contains interesting data too during the nights (Club Jazzafip, etc))
    let url = "https://www.radiofrance.fr/fip/api/live";
    let response = attohttpc::get(url).send()?;
    if response.is_success() {
        let text = response.text_utf8()?;
        let fip_result: Result<Fip, serde_json::Error> = from_str(&*text);
        match fip_result {
            Ok(fip) => {
                info!("fip: {:?}", fip);
                let track: Option<Track> = Track::create(
                    fip.now.second_line.title,
                    fip.now.first_line.title,
                    fip.now.song.and_then(|s| s.release).and_then(|r| r.title),
                    fip.now.start_time,
                );
                track.ok_or(FipError {}.into())
            }
            Err(error) => {
                Err(error.into())
            }
        }
    } else {
        error!("Invalid response from get {} : {}", url, response.status());
        Err(FipError {}.into())
    }
}
