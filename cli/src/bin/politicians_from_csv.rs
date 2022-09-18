use db::UpsertPoliticianInput;
use std::error::Error;
use std::io;
use std::process;

async fn upsert_politicians_from_csv() -> Result<(), Box<dyn Error>> {
    // Init database connection singleton
    db::init_pool().await.unwrap();
    let pool = db::pool().await;

    // Build the CSV reader and iterate over each record.
    let mut rdr = csv::Reader::from_reader(io::stdin());
    for result in rdr.deserialize() {
        let input: UpsertPoliticianInput = result?;

        let upserted_record = db::Politician::upsert(&pool.connection, &input)
            .await
            .expect(
                format!(
                    "Failed to upsert politician: {:?} {:?}",
                    input.first_name, input.last_name
                )
                .as_str(),
            );
        println!(
            "Upserted politician: {:?} {:?}",
            upserted_record.first_name, upserted_record.last_name
        );
    }

    Ok(())
}

#[tokio::main]
async fn main() {
    if let Err(err) = upsert_politicians_from_csv().await {
        println!("error upserting politicians: {}", err);
        process::exit(1);
    }
}
