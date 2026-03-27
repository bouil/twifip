use std::sync::Arc;
use log::{error, info};
use tokio::task;
use tokio_cron_scheduler::{Job, JobScheduler};
use crate::twifip::{Twifip, Result};
use crate::TwifipError;

pub async fn schedule_jobs(twifip: Arc<Twifip>, cron_expression: String) -> Result<()> {
    let sched = build_scheduler(twifip, cron_expression).await
        .map_err(|e| TwifipError::SchedulerError(e.to_string()))?;

    // Feature 'signal' must be enabled
    sched.shutdown_on_ctrl_c();

    sched.start().await
        .map_err(|e| TwifipError::SchedulerError(e.to_string()))?;
    Ok(())
}

pub(crate) async fn build_scheduler(twifip: Arc<Twifip>, cron_expression: String) -> Result<JobScheduler> {
    info!("Scheduling jobs");

    let mut sched = JobScheduler::new().await?;

    // Add code to be run during/after shutdown
    sched.set_shutdown_handler(Box::new(|| {
        Box::pin(async move {
            info!("Shut down done");
        })
    }));

    info!("Starting job with cron expression: {}", cron_expression);
    sched
        .add(Job::new_async(cron_expression, move |_uuid, _l| {
            let twifip = Arc::clone(&twifip);
            Box::pin(async move {
                if let Err(e) = task::spawn_blocking(move || twifip.check_and_scrobble())
                    .await
                {
                    error!("Error in scheduled task: {}", e);
                }
            })
        })?)
        .await
        .map_err(|e| TwifipError::SchedulerError(e.to_string()))?;

    info!("Done scheduling jobs");
    Ok(sched)
}

#[cfg(test)]
mod tests {
    use super::*;
    use rustfm_scrobble_proxy::Scrobbler;
    use crate::track_store::TrackStore;
    use std::time::Duration;

    #[tokio::test]
    async fn test_scheduler_starts_and_shuts_down_cleanly() {
        let twifip = Arc::new(Twifip {
            scrobbler: Scrobbler::new("test_key", "test_secret"),
            track_store: TrackStore::new("/tmp/twifip_test.txt".to_string()),
            dry_run: true,
        });

        let mut sched = build_scheduler(twifip, "0/20 * * ? * *".to_string()).await.unwrap();
        sched.start().await.unwrap();

        tokio::time::sleep(Duration::from_millis(100)).await;

        let result = sched.shutdown().await;
        assert!(result.is_ok());
    }
}
