use std::env;
use std::error::Error;

pub struct Config {
    pub username: String,
    pub password: String,
    pub api_key: String,
    pub api_secret: String,
    pub twifip_file: String,
    pub schedule_cron: String,
}

impl Config {
    pub fn from_env() -> Result<Config, Box<dyn Error>> {
        Ok(Config {
            username: env::var("LASTFM_USERNAME").map_err(|_| "Missing ENV variable LASTFM_USERNAME")?,
            password: env::var("LASTFM_PASSWORD").map_err(|_| "Missing ENV variable LASTFM_PASSWORD")?,
            api_key: env::var("LASTFM_API_KEY").map_err(|_| "Missing ENV variable LASTFM_API_KEY")?,
            api_secret: env::var("LASTFM_API_SECRET").map_err(|_| "Missing ENV variable LASTFM_API_SECRET")?,
            twifip_file: env::var("TWIFIP_FILE").map_err(|_| "Missing ENV variable TWIFIP_FILE")?,
            schedule_cron: env::var("TWIFIP_SCHEDULE_CRON")
                .unwrap_or_else(|_| "0/20 * * ? * *".to_string()),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_env_returns_err_when_vars_missing() {
        env::remove_var("LASTFM_USERNAME");
        env::remove_var("LASTFM_PASSWORD");
        env::remove_var("LASTFM_API_KEY");
        env::remove_var("LASTFM_API_SECRET");
        env::remove_var("TWIFIP_FILE");

        let result = Config::from_env();
        assert!(result.is_err());
        assert!(result.err().unwrap().to_string().contains("LASTFM_USERNAME"));
    }
}
