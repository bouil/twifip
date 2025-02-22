use crate::track::Track;
use log::{error, info};
use serde::Deserialize;
use serde_json::from_str;
use std::{error::Error, fmt};

#[derive(Deserialize, Debug)]
struct FirstLine {
    title: String,
}

#[derive(Deserialize, Debug)]
struct SecondLine {
    title: String,
}

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
    first_line: FirstLine,
    #[serde(rename = "secondLine")]
    second_line: SecondLine,
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

pub(crate) fn read_fip() -> Result<Track, Box<dyn Error>> {
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
