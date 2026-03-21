use crate::twifip::Twifip;
use crate::schedule::schedule_jobs;
use dotenvy::dotenv;
use log::info;
use std::error::Error;
use std::sync::Arc;

mod fip_reader;
mod twifip;
mod logging_setup;
mod schedule;
mod track;
mod track_store;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    logging_setup::init_logging();
    info!("Loading env vars");
    dotenv().ok();

    let twifip = Arc::new(Twifip::new().expect("Failed to initialize lastfm"));

    let _ = schedule_jobs(twifip).await;

    info!("Shutting down...");
    Ok(())
}
