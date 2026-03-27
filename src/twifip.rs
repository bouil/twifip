use crate::config::Config;
use crate::error::TwifipError;
use crate::fip_reader::read_fip;
use crate::track::Track;
use crate::track_store::TrackStore;
use log::{error, info};
use rustfm_scrobble_proxy::responses::ScrobbleResponse;
use rustfm_scrobble_proxy::{Scrobble, Scrobbler};

pub type Result<T> = std::result::Result<T, TwifipError>;

pub struct Twifip {
    pub(crate) scrobbler: Scrobbler,
    pub(crate) track_store: TrackStore,
    pub(crate) dry_run: bool,
}

impl Twifip {
    pub fn new(config: Config) -> Result<Twifip> {
        info!("Initializing Lastfm. Using user {}", config.username);
        let mut scrobbler = Scrobbler::new(&config.api_key, &config.api_secret);
        scrobbler.authenticate_with_password(&config.username, &config.password)
            .map_err(|e| TwifipError::LastFmAuthError(e.to_string()))?;
        let track_store = TrackStore::new(config.twifip_file);
        Ok(Twifip { scrobbler, track_store, dry_run: config.dry_run })
    }

    pub fn check_and_scrobble(&self) -> Result<()> {
        let track_result = read_fip()
            .map_err(|e| TwifipError::FipApiError(e.to_string()))?;

        match self.track_store.store_if_new(&track_result) {
            Ok(true) if self.dry_run => {
                info!("[dry-run] would scrobble: {:?}", track_result);
                Ok(())
            }
            Ok(true) => {
                self.scrobble_track(&track_result).map(|_| ())
            }
            Ok(false) => Ok(()),
            Err(err) => {
                error!("Failed to access track cache: {}", err);
                Err(TwifipError::TrackCacheError(err.to_string()))
            }
        }
    }

    fn scrobble_track(&self, track: &Track) -> Result<ScrobbleResponse> {
        info!("{:?}", track);

        let artist = track.artist.as_str();
        let song = track.title.as_str();
        let song = Scrobble::new(artist, song, track.album.as_deref());
        let result = self.scrobbler.scrobble(&song)
            .map_err(|e| TwifipError::LastFmScrobbleError(e.to_string()))?;

        info!(
            "{} (corrected={}), {} (corrected={}), {} (corrected={})",
            result.artist.text,
            result.artist.corrected,
            result.album.text,
            result.album.corrected,
            result.track.text,
            result.track.corrected
        );
        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Compile-time assertion: Twifip must be Send + 'static for spawn_blocking
    fn _assert_send_static<T: Send + 'static>() {}
    fn _assert_twifip_send_static() { _assert_send_static::<Twifip>(); }
}
