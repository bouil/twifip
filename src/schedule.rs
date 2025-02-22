use std::env;
use log::info;
use tokio_cron_scheduler::{Job, JobScheduler, JobSchedulerError};
use crate::fip_reader::read_fip;
use crate::lastfm::{TrackScrobbler, Twifip};

pub async fn schedule_jobs(twifip: Twifip) -> Result<(), JobSchedulerError> {
    info!("Scheduling jobs");

    let mut sched = JobScheduler::new().await?;

    // Feature 'signal' must be enabled
    sched.shutdown_on_ctrl_c();

    // Add code to be run during/after shutdown
    sched.set_shutdown_handler(Box::new(|| {
        Box::pin(async move {
            println!("Shut down done");
        })
    }));
    let cron_expression =
        env::var("TWIFIP_SCHEDULE_CRON").unwrap_or(String::from("0/20 * * ? * *"));

    info!("Starting job with cron expression: {}", cron_expression);
    sched
        .add(Job::new(cron_expression, move |_uuid, _l| {
            let track_result = read_fip();

            match track_result {
                Ok(track) => {
                    if twifip.track_store.is_new_track(&track) {
                        let _ = twifip.scrobbler.scrobble_track(&track);
                    }
                }
                Err(err) => {
                    log::error!("{}", err)
                }
            }

            ();
        })?)
        .await?;

    info!("Done scheduling jobs");
    sched.start().await
}
