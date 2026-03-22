use crate::config::Config;
use crate::twifip::Twifip;
use crate::schedule::schedule_jobs;
use anyhow::Result;
use dotenvy::dotenv;
use log::{error, info};
use std::sync::Arc;
use tokio::signal;

mod config;
mod fip_reader;
mod twifip;
mod logging_setup;
mod schedule;
mod track;
mod track_store;

#[tokio::main]
async fn main() -> Result<()> {
    logging_setup::init_logging();
    info!("Loading env vars");
    dotenv().ok();

    let config = Config::from_env().expect("Failed to load config");
    let cron_expression = config.schedule_cron.clone();
    let twifip = Arc::new(Twifip::new(config).expect("Failed to initialize lastfm"));

    let _ = schedule_jobs(twifip, cron_expression).await;

    match signal::ctrl_c().await {
        Ok(()) => {
            info!("Shutting down...");
        }
        Err(err) => {
            error!("Unable to listen for shutdown signal: {}", err);
            // we also shut down in case of error
        }
    }

    Ok(())
}
