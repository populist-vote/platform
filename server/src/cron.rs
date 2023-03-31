use std::collections::HashMap;
use tokio_cron_scheduler::{Job, JobScheduler};
use tracing::{info, warn};

// Creates a new job scheduler and adds an async job to update legiscan bill data
pub async fn init_job_schedule() {
    let environment = config::Config::default().environment;
    if environment != config::Environment::Production {
        warn!("Not running cron jobs in non-production environment");
        return;
    }

    let sched = JobScheduler::new().await.unwrap();

    // Update legiscan bills every four hours
    let update_legiscan_bills_job = Job::new_async("0 0 4,8,12,16,20 * * *", |uuid, mut l| {
        Box::pin(async move {
            update_legiscan_bill_data()
                .await
                .map_err(|e| warn!("Failed to update bill data: {}", e))
                .ok();

            let next_tick = l.next_tick_for_job(uuid).await;
            match next_tick {
                Ok(Some(ts)) => info!("Next time for update_legsican_bill_data is {:?}", ts),
                _ => warn!("Could not get next tick for update_legsican_bill_data job"),
            }
        })
    })
    .unwrap();

    sched.add(update_legiscan_bills_job).await.unwrap();

    sched
        .start()
        .await
        .map_err(|e| warn!("Failed to start scheduler: {}", e))
        .ok();

    // Wait a while so that the jobs actually run
    tokio::time::sleep(core::time::Duration::from_secs(100)).await;
}

pub async fn update_legiscan_bill_data() -> anyhow::Result<()> {
    let legiscan = legiscan::LegiscanProxy::new().unwrap();
    let masterlist = legiscan.get_master_list_raw_by_session(1986).await.unwrap();

    // Check the changehash of each bill with a matching session_id to determine
    // which bill's need to be updated by Legiscan
    let mut bills_hash_map: HashMap<i32, String> = HashMap::new();
    for bill in masterlist.iter() {
        bills_hash_map.insert(bill.bill_id, bill.change_hash.clone());
    }

    let json = serde_json::to_value(bills_hash_map).unwrap();
    let pool = db::pool().await;
    let updated_bills = sqlx::query!(
        r#"
                WITH hash AS (
                    SELECT $1::jsonb h
                )
                UPDATE bill
                SET legiscan_change_hash = value
                FROM hash, jsonb_each_text(h)
                WHERE key::int = legiscan_bill_id
                AND value != legiscan_change_hash
                RETURNING id, legiscan_bill_id, legiscan_change_hash
            "#,
        json
    )
    .fetch_all(&pool.connection)
    .await
    .expect("Failed to update bill change hashes");

    println!("There are {} updated bills", updated_bills.len());

    for bill in updated_bills.iter() {
        let bill_data = legiscan
            .get_bill(bill.legiscan_bill_id.expect("Bill has no Legiscan ID"))
            .await
            .unwrap();
        let bill_data_json = serde_json::to_value(bill_data).unwrap();
        println!("{}", serde_json::to_string_pretty(&bill_data_json).unwrap());
        sqlx::query!(
            r#"
                UPDATE bill
                SET legiscan_data = $1,
                    status = COALESCE(((
                        json_build_object(1, 'introduced', 2, 'in_consideration', 4, 'became_law')::jsonb)
                        ->> ($1::jsonb->>'status'))::bill_status, 'introduced'),
                    legiscan_committee = $1::jsonb->'committee'->>'name',
                    legiscan_committee_id = ($1::jsonb->'committee'->>'committee_id')::int
                WHERE id = $2
            "#,
            bill_data_json,
            bill.id
        )
        .execute(&pool.connection)
        .await
        .expect("Failed to update bill data");
    }

    println!("Updated {} bills", updated_bills.len());

    Ok(())
}

#[tokio::test]

async fn test_update_legiscan_bill_data() {
    let _ = tracing_subscriber::fmt::try_init();
    let _ = db::init_pool().await;
    let _ = update_legiscan_bill_data().await;
}
