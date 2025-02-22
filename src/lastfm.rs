use std::env;
use crate::track::Track;
use log::{error, info};
use rustfm_scrobble_proxy::responses::ScrobbleResponse;
use rustfm_scrobble_proxy::{Scrobble, Scrobbler};
use std::error::Error;
use crate::track_store::TrackStore;

pub trait TrackScrobbler {
    fn scrobble_track(&self, track: &Track) -> Result<ScrobbleResponse, Box<dyn Error>>;
}

pub struct Twifip {
    pub scrobbler: Scrobbler,
    pub track_store: TrackStore
}

impl Twifip {
    pub fn new() -> Result<Twifip, Box<dyn Error>> {
        info!("Initializing Lastfm");
        let username = env::var("LASTFM_USERNAME").expect("Missing ENV variable LASTFM_USERNAME");
        let password = env::var("LASTFM_PASSWORD").expect("Missing ENV variable LASTFM_PASSWORD");
        let api_key = env::var("LASTFM_API_KEY").expect("Missing ENV variable LASTFM_API_KEY");
        let api_secret = env::var("LASTFM_API_SECRET").expect("Missing ENV variable LASTFM_API_SECRET");
        info!("Loaded env vars. Using user {}", username);
        let mut scrobbler = Scrobbler::new(&*api_key, &*api_secret);
        scrobbler.authenticate_with_password(&*username, &*password)?;
        let track_store = TrackStore::new();
        Ok(Twifip {
            scrobbler,
            track_store
        })
    }
}

impl TrackScrobbler for Scrobbler {
    fn scrobble_track(self: &Self, track: &Track) -> Result<ScrobbleResponse, Box<dyn Error>> {
        info!("{:?}", track);

        let artist = track.artist.as_str();
        let song = track.title.as_str();
        let song = Scrobble::new(artist, song, track.album.as_deref());
        let result = self.scrobble(&song);

        match result {
            Ok(scrobble_response) => {
                info!(
                    "{} (corrected={}), {} (corrected={}), {} (corrected={})",
                    scrobble_response.artist.text,
                    scrobble_response.artist.corrected,
                    scrobble_response.album.text,
                    scrobble_response.album.corrected,
                    scrobble_response.track.text,
                    scrobble_response.track.corrected
                );
                Ok(scrobble_response)
            }
            Err(error) => {
                error!("{}", error.to_string());
                Err(Box::try_from(error).unwrap())
            }
        }
    }
}
