use std::error::Error;
use std::io;
use std::process;

use db::CreatePoliticianInput;
use db::Politician;
use proxy::VotesmartProxy;

async fn example() -> Result<(), Box<dyn Error>> {
    // Init database connection singleton
    db::init_pool().await.unwrap();
    let pool = db::pool().await;

    // Build the CSV reader and iterate over each record.
    let mut rdr = csv::Reader::from_reader(io::stdin());
    for result in rdr.deserialize() {
        let mut new_record_input: CreatePoliticianInput = result?;
        let vs_candidate_bio_data = VotesmartProxy::new()
            .unwrap()
            .get_candidate_bio(new_record_input.votesmart_candidate_id.unwrap())
            .await;
        new_record_input.votesmart_candidate_bio =
            Some(serde_json::to_value(vs_candidate_bio_data.unwrap()).unwrap());
        let new_politician_record = Politician::create(&pool.connection, &new_record_input).await;

        match new_politician_record {
            Err(_) => {
                println!("Politician already exists");
            },
            Ok(_) => println!("Politician seeded successfully")
        }
        // Figure out how to implememnt recoverable errors here so that we can continue if we run into foreign key
        // constraint errors OR implement an upsert for the politician
    }
    Ok(())
}

#[tokio::main]
async fn main() {
    if let Err(err) = example().await {
        println!("error running example: {}", err);
        process::exit(1);
    }
}
