use crate::track::Track;
use log::info;
use std::fs::{read_to_string, write};
use std::io;
use std::path::Path;

pub(crate) struct TrackStore {
    file: String,
}

impl TrackStore {

    pub fn new(file: String) -> TrackStore {
        TrackStore { file }
    }

    /// Store the track in the cache file if it differs from the last seen track.
    /// Returns Ok(true) if the track was new and stored, Ok(false) if already cached.
    pub fn store_if_new(&self, track: &Track) -> Result<bool, io::Error> {
        let file = Path::new(self.file.as_str());
        let track_to_string = track.to_string();
        if file.exists() {
            info!("File already exists: {}", file.display());
            let cached: String = read_to_string(file)?;
            if cached.eq(&track_to_string) {
                info!("Cached Track is the same");
                Ok(false)
            } else {
                info!("Cached Track is different");
                write(file, track_to_string)?;
                Ok(true)
            }
        } else {
            info!("File does not exist: {}, creating cache.", file.display());
            write(file, track_to_string)?;
            Ok(true)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::track::Track;

    fn make_track(title: &str) -> Track {
        Track::create("Artist".to_string(), title.to_string(), None, 0).unwrap()
    }

    #[test]
    fn test_store_if_new_returns_true_for_new_track() {
        let path = "/tmp/twifip_test_new.txt";
        let _ = std::fs::remove_file(path);
        let store = TrackStore::new(path.to_string());

        assert!(store.store_if_new(&make_track("Title A")).unwrap());
    }

    #[test]
    fn test_store_if_new_returns_false_for_same_track() {
        let path = "/tmp/twifip_test_same.txt";
        let _ = std::fs::remove_file(path);
        let store = TrackStore::new(path.to_string());
        let track = make_track("Title B");

        store.store_if_new(&track).unwrap();
        assert!(!store.store_if_new(&track).unwrap());
    }

    #[test]
    fn test_store_if_new_returns_true_for_changed_track() {
        let path = "/tmp/twifip_test_changed.txt";
        let _ = std::fs::remove_file(path);
        let store = TrackStore::new(path.to_string());

        store.store_if_new(&make_track("Title C")).unwrap();
        assert!(store.store_if_new(&make_track("Title D")).unwrap());
    }

    #[test]
    fn test_store_if_new_returns_err_on_bad_path() {
        let store = TrackStore::new("/no/such/dir/twifip_test.txt".to_string());
        assert!(store.store_if_new(&make_track("Title E")).is_err());
    }
}
