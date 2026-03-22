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

    /// check if the track is new or already seen, by checking in a cache file
    pub fn is_new_track(&self, track: &Track) -> bool {
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
