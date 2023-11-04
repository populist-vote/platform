use std::collections::HashMap;

use tracing::info;

pub async fn run() -> anyhow::Result<()> {
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
                AND is_locked = false
                RETURNING id, legiscan_bill_id, legiscan_change_hash
            "#,
        json
    )
    .fetch_all(&pool.connection)
    .await
    .expect("Failed to update bill change hashes");

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

    info!("Updated {} bills", updated_bills.len());

    Ok(())
}

#[tokio::test]

async fn test_update_legiscan_bill_data() {
    let _ = tracing_subscriber::fmt::try_init();
    let _ = db::init_pool().await;
    let _ = run().await;
}
