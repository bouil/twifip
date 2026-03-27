use thiserror::Error;

#[derive(Error, Debug)]
pub enum TwifipError {
    #[error("FIP API error: {0}")]
    FipApiError(String),
    
    #[error("Last.fm authentication error: {0}")]
    LastFmAuthError(String),
    
    #[error("Last.fm scrobble error: {0}")]
    LastFmScrobbleError(String),
    
    #[error("Track cache error: {0}")]
    TrackCacheError(String),
    
    #[error("Configuration error: {0}")]
    ConfigError(String),
    
    #[error("Track filtering error: {0}")]
    TrackFilterError(String),
    
    #[error("HTTP request failed: {0}")]
    HttpError(String),
    
    #[error("JSON parsing error: {0}")]
    JsonError(String),
    
    #[error("Scheduler error: {0}")]
    SchedulerError(String),
}

impl From<attohttpc::Error> for TwifipError {
    fn from(err: attohttpc::Error) -> Self {
        TwifipError::HttpError(err.to_string())
    }
}

impl From<serde_json::Error> for TwifipError {
    fn from(err: serde_json::Error) -> Self {
        TwifipError::JsonError(err.to_string())
    }
}

impl From<std::io::Error> for TwifipError {
    fn from(err: std::io::Error) -> Self {
        TwifipError::TrackCacheError(err.to_string())
    }
}

impl From<anyhow::Error> for TwifipError {
    fn from(err: anyhow::Error) -> Self {
        TwifipError::FipApiError(err.to_string())
    }
}

impl From<tokio_cron_scheduler::JobSchedulerError> for TwifipError {
    fn from(err: tokio_cron_scheduler::JobSchedulerError) -> Self {
        TwifipError::SchedulerError(err.to_string())
    }
}

impl From<rustfm_scrobble_proxy::ScrobblerError> for TwifipError {
    fn from(err: rustfm_scrobble_proxy::ScrobblerError) -> Self {
        TwifipError::LastFmScrobbleError(err.to_string())
    }
}