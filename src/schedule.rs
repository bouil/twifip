use std::env;
use std::sync::Arc;
use log::info;
use tokio::task;
use tokio_cron_scheduler::{Job, JobScheduler, JobSchedulerError};
use crate::twifip::Twifip;

pub async fn schedule_jobs(twifip: Arc<Twifip>) -> Result<(), JobSchedulerError> {
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
        .add(Job::new_async(cron_expression, move |_uuid, _l| {
            let twifip = Arc::clone(&twifip);
            Box::pin(async move {
                task::spawn_blocking(move || twifip.check_and_scrobble())
                    .await
                    .ok();
            })
        })?)
        .await?;

    info!("Done scheduling jobs");
    sched.start().await
}
