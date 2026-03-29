use crate::track::Track;
use crate::track_store::TrackStore;
use tokio::sync::Mutex;

// Mock FIP reader for testing
struct MockFipReader {
    responses: Mutex<Vec<Result<Track, String>>>,
}

impl MockFipReader {
    fn new(responses: Vec<Result<Track, String>>) -> Self {
        Self {
            responses: Mutex::new(responses),
        }
    }

    async fn read(&self) -> Result<Track, String> {
        let mut responses = self.responses.lock().await;
        if responses.is_empty() {
            return Err("No more mock responses".to_string());
        }
        // Use remove(0) to get FIFO behavior instead of pop() which is LIFO
        responses.remove(0)
    }
}

// Mock Last.fm scrobbler for testing
struct MockScrobbler {
    scrobbles: Mutex<Vec<Track>>,
}

impl MockScrobbler {
    fn new() -> Self {
        Self {
            scrobbles: Mutex::new(Vec::new()),
        }
    }

    async fn scrobble(&self, track: &Track) -> Result<(), String> {
        let mut scrobbles = self.scrobbles.lock().await;
        scrobbles.push(track.clone());
        Ok(())
    }

    async fn get_scrobbles(&self) -> Vec<Track> {
        let scrobbles = self.scrobbles.lock().await;
        scrobbles.clone()
    }
}

#[tokio::test]
async fn test_full_flow_integration() {
    // Setup mock responses
    let track1 = Track::create(
        "Artist1".to_string(),
        "Title1".to_string(),
        Some("Album1".to_string()),
        1234567890,
    ).unwrap();

    let track2 = Track::create(
        "Artist2".to_string(),
        "Title2".to_string(),
        Some("Album2".to_string()),
        1234567891,
    ).unwrap();

    // Create identical track for duplicate test
    let track2_duplicate = Track::create(
        "Artist2".to_string(),
        "Title2".to_string(),
        Some("Album2".to_string()),
        1234567891,
    ).unwrap();

    let mock_fip = MockFipReader::new(vec![
        Ok(track1.clone()),
        Ok(track2.clone()),
        Ok(track2_duplicate.clone()), // Same track should not be scrobbled again
    ]);

    let mock_scrobbler = MockScrobbler::new();
    let temp_file = "/tmp/twifip_integration_test.txt";
    let _ = std::fs::remove_file(temp_file);

    let track_store = TrackStore::new(temp_file.to_string());

    // Simulate the main flow
    // First track - should be scrobbled
    let result1 = mock_fip.read().await;
    assert!(result1.is_ok());
    let track_result1 = result1.unwrap();
    let is_new = track_store.store_if_new(&track_result1).unwrap();
    assert!(is_new);
    mock_scrobbler.scrobble(&track_result1).await.unwrap();

    // Second track - should be scrobbled
    let result2 = mock_fip.read().await;
    assert!(result2.is_ok());
    let track_result2 = result2.unwrap();
    let is_new = track_store.store_if_new(&track_result2).unwrap();
    assert!(is_new);
    mock_scrobbler.scrobble(&track_result2).await.unwrap();

    // Third track (same as second) - should NOT be scrobbled
    let result3 = mock_fip.read().await;
    assert!(result3.is_ok());
    let track_result3 = result3.unwrap();
    let is_new = track_store.store_if_new(&track_result3).unwrap();
    assert!(!is_new);

    // Verify scrobble count (only 2 tracks were scrobbled)
    let scrobbles = mock_scrobbler.get_scrobbles().await;
    assert_eq!(scrobbles.len(), 2);
    assert_eq!(scrobbles[0].title, "Title1");
    assert_eq!(scrobbles[1].title, "Title2");

    // Cleanup
    let _ = std::fs::remove_file(temp_file);
}

#[tokio::test]
async fn test_error_handling_integration() {
    // Setup mock responses with errors
    let mock_fip = MockFipReader::new(vec![
        Err("FIP API error".to_string()),
        Ok(Track::create(
            "Artist1".to_string(),
            "Title1".to_string(),
            Some("Album1".to_string()),
            1234567890,
        ).unwrap()),
        Err("FIP API error 2".to_string()),
    ]);

    let mock_scrobbler = MockScrobbler::new();
    let temp_file = "/tmp/twifip_error_test.txt";
    let _ = std::fs::remove_file(temp_file);

    let track_store = TrackStore::new(temp_file.to_string());

    // Test error handling
    let result1 = mock_fip.read().await;
    assert!(result1.is_err());

    let result2 = mock_fip.read().await;
    assert!(result2.is_ok());
    let track = result2.unwrap();
    track_store.store_if_new(&track).unwrap();
    mock_scrobbler.scrobble(&track).await.unwrap();

    let result3 = mock_fip.read().await;
    assert!(result3.is_err());

    // Verify only one successful scrobble
    let scrobbles = mock_scrobbler.get_scrobbles().await;
    assert_eq!(scrobbles.len(), 1);

    // Cleanup
    let _ = std::fs::remove_file(temp_file);
}
