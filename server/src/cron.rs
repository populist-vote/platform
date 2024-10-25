use tokio_cron_scheduler::{Job, JobScheduler};
use tracing::{error, info, warn};

use crate::update_legiscan_bill_data;

// Creates a new job scheduler and adds an async job to update legiscan bill data
pub async fn init_job_schedule() {
    let environment = config::Config::default().environment;
    if environment != config::Environment::Production && environment != config::Environment::Staging
    {
        warn!(
            "{}",
            format!(
                "Not running cron jobs in non-production environment: {}",
                environment
            )
        );
        return;
    } else {
        info!("Initializing job scheduler");
    }

    let sched = JobScheduler::new().await.unwrap();

    // Update legiscan bills every four hours
    let update_legiscan_bills_job = Job::new_async("0 0 1/4 * * *", |uuid, mut l| {
        Box::pin(async move {
            tracing::warn!("Running update_legiscan_bill_data job");
            update_legiscan_bill_data::run()
                .await
                .map_err(|e| error!("Failed to update bill data: {}", e))
                .ok();

            let next_tick = l.next_tick_for_job(uuid).await;
            match next_tick {
                Ok(Some(ts)) => info!("Next time for update_legsican_bill_data is {:?}", ts),
                _ => warn!("Could not get next tick for update_legsican_bill_data job"),
            }
        })
    })
    .unwrap();

    // Run job every 10 minutes on the load test Fridays for MN Sos results
    let update_mn_results_job = Job::new_async("0 1/10 * * * * 2024", |uuid, mut l| {
        Box::pin(async move {
            tracing::warn!("Running update_mn_results job");
            scrapers::mn_sos_results::fetch_results()
                .await
                .map_err(|e| warn!("Failed to update Minnesota SoS results: {}", e))
                .ok();

            let next_tick = l.next_tick_for_job(uuid).await;
            match next_tick {
                Ok(Some(ts)) => info!("Next time for update_mn_results is {:?}", ts),
                _ => warn!("Could not get next tick for update_mn_results job"),
            }
        })
    })
    .unwrap();

    match environment {
        config::Environment::Production => {
            info!("Running cron jobs in production environment");
            sched.add(update_legiscan_bills_job).await.unwrap();
            // sched.add(update_mn_results_job).await.unwrap(); // Uncomment after load tests are completed Nov 1, 2024
        }
        config::Environment::Staging => {
            info!("Running cron jobs in staging environment");
            sched.add(update_legiscan_bills_job).await.unwrap();
            sched.add(update_mn_results_job).await.unwrap();
        }
        _ => {
            warn!("Not running cron jobs in non-production environment");
            return;
        }
    }

    sched
        .start()
        .await
        .map_err(|e| error!("Failed to start job scheduler: {}", e))
        .ok();

    // Wait a while so that the jobs actually run
    tokio::time::sleep(core::time::Duration::from_secs(5)).await;
}
