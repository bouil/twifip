use crate::config::Config;
use crate::fip_reader::read_fip;
use crate::track::Track;
use crate::track_store::TrackStore;
use anyhow::Result;
use log::{error, info};
use rustfm_scrobble_proxy::responses::ScrobbleResponse;
use rustfm_scrobble_proxy::{Scrobble, Scrobbler};

pub struct Twifip {
    pub(crate) scrobbler: Scrobbler,
    pub(crate) track_store: TrackStore,
    pub(crate) dry_run: bool,
}

impl Twifip {
    pub fn new(config: Config) -> Result<Twifip> {
        info!("Initializing Lastfm. Using user {}", config.username);
        let mut scrobbler = Scrobbler::new(&config.api_key, &config.api_secret);
        scrobbler.authenticate_with_password(&config.username, &config.password)?;
        let track_store = TrackStore::new(config.twifip_file);
        Ok(Twifip { scrobbler, track_store, dry_run: config.dry_run })
    }

    pub fn check_and_scrobble(&self) {
        let track_result = read_fip();

        match track_result {
            Ok(track) => {
                match self.track_store.store_if_new(&track) {
                    Ok(true) if self.dry_run => info!("[dry-run] would scrobble: {:?}", track),
                    Ok(true) => { let _ = self.scrobble_track(&track); }
                    Ok(false) => {}
                    Err(err) => error!("Failed to access track cache: {}", err),
                }
            }
            Err(err) => {
                error!("{}", err)
            }
        }
    }

    fn scrobble_track(&self, track: &Track) -> Result<ScrobbleResponse> {
        info!("{:?}", track);

        let artist = track.artist.as_str();
        let song = track.title.as_str();
        let song = Scrobble::new(artist, song, track.album.as_deref());
        let result = self.scrobbler.scrobble(&song);

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
                Err(anyhow::anyhow!("{}", error))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Compile-time assertion: Twifip must be Send + 'static for spawn_blocking
    fn _assert_send_static<T: Send + 'static>() {}
    fn _assert_twifip_send_static() { _assert_send_static::<Twifip>(); }
}
