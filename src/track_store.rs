use crate::track::Track;
use log::info;
use std::fs::{read_to_string, write};
use std::path::Path;

pub(crate) struct TrackStore {
    file: String,
}

impl TrackStore {

    pub fn new(file: String) -> TrackStore {
        TrackStore { file }
    }

    /// Store the track in the cache file if it differs from the last seen track.
    /// Returns true if the track was new and stored, false if it was already cached.
    pub fn store_if_new(&self, track: &Track) -> bool {
        let file = Path::new(self.file.as_str());
        let track_to_string = track.to_string();
        if file.exists() {
            info!("File already exists: {}", file.display());
            let cached: String = read_to_string(&file).expect("Failed to read track file");
            if cached.eq(&track_to_string) {
                info!("Cached Track is the same");
                false
            } else {
                info!("Cached Track is different");
                write(file, track_to_string).expect("Failed to write track file");
                true
            }
        } else {
            info!("File does not exist: {}, creating cache.", file.display());
            write(file, track_to_string).expect("Failed to write track file");
            true
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

        assert!(store.store_if_new(&make_track("Title A")));
    }

    #[test]
    fn test_store_if_new_returns_false_for_same_track() {
        let path = "/tmp/twifip_test_same.txt";
        let _ = std::fs::remove_file(path);
        let store = TrackStore::new(path.to_string());
        let track = make_track("Title B");

        store.store_if_new(&track);
        assert!(!store.store_if_new(&track));
    }

    #[test]
    fn test_store_if_new_returns_true_for_changed_track() {
        let path = "/tmp/twifip_test_changed.txt";
        let _ = std::fs::remove_file(path);
        let store = TrackStore::new(path.to_string());

        store.store_if_new(&make_track("Title C"));
        assert!(store.store_if_new(&make_track("Title D")));
    }
}
