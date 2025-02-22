use crate::lastfm::Twifip;
use crate::schedule::schedule_jobs;
use dotenvy::dotenv;
use log::{error, info};
use std::error::Error;
use tokio::signal;

mod fip_reader;
mod lastfm;
mod logging_setup;
mod schedule;
mod track;
mod track_store;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    logging_setup::init_logging();
    info!("Loading env vars");
    dotenv().ok();

    let twifip = Twifip::new().expect("Failed to initialize lastfm");

    let _ = schedule_jobs(twifip).await;

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
