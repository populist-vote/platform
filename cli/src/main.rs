#[rustfmt::skip]
mod utils;
use slugify::slugify;
use sqlx::Executor;
use std::str::FromStr;

use db::{
    models::enums::{LegislationStatus, PoliticalParty, State},
    CreateBillInput, CreatePoliticianInput, UpdateBillInput, UpdatePoliticianInput,
};
use structopt::StructOpt;

use proxy::{Error, LegiscanProxy, VotesmartProxy};
#[derive(Clone, Debug, StructOpt)]
#[structopt(
    name = "Populist Command Line",
    about = "Query our proxy services to fetch and persist JSON data"
)]
struct Args {
    #[structopt(subcommand)]
    command: Command,
}

#[derive(Debug, Clone, StructOpt)]
enum Command {
    Proxy {
        #[structopt(subcommand)]
        service: Service,
    },
}

#[derive(StructOpt, Debug, Clone)]
#[structopt(about = "Proxy one of our external APIs for data")]
enum Service {
    /// Interact with Legiscan API data
    Legiscan(LegiscanAction),
    /// Interact with Votesmart API data
    Votesmart(VoteSmartAction),
}

#[derive(Clone, Debug, StructOpt)]
enum LegiscanAction {
    /// Get bill data from Legiscan API
    GetBill(GetBillArgs),
    /// Get text of bill as base64 encoded document with metadata
    GetBillText(GetBillTextArgs),
}

#[derive(Clone, Debug, StructOpt)]
struct GetBillArgs {
    #[structopt(about = "Legiscan bill ID")]
    bill_id: i32,
    #[structopt(short, long, about = "Create populist record")]
    create_record: bool,
    #[structopt(short, long, about = "Update populist record")]
    update_record: bool,
    #[structopt(short, long, about = "Print fetched JSON data to console")]
    pretty_print: bool,
}

#[derive(Clone, Debug, StructOpt)]
struct GetBillTextArgs {
    #[structopt(about = "Legiscan bill ID")]
    bill_id: i32,
    #[structopt(short, long, about = "Create populist record")]
    create_record: bool,
    #[structopt(short, long, about = "Update populist record")]
    update_record: bool,
    #[structopt(short, long, about = "Print fetched JSON data to console")]
    pretty_print: bool,
}

#[derive(Clone, Debug, StructOpt)]
enum VoteSmartAction {
    /// Get candidate bio from Votesmart
    GetCandidateBio(GetCandidateBioArgs),
    /// Get candidate voting record from Votesmart
    GetCandidateVotingRecord(GetCandidateVotingRecordArgs),
}

#[derive(Clone, Debug, StructOpt)]
struct GetCandidateBioArgs {
    #[structopt(about = "Votesmart candidate ID")]
    candidate_id: i32,
    #[structopt(short, long, about = "Create populist record")]
    create_record: bool,
    #[structopt(short, long, about = "Update populist record")]
    update_record: bool,
    #[structopt(short, long, about = "Print fetched JSON data to console")]
    pretty_print: bool,
}

#[derive(Clone, Debug, StructOpt)]
struct GetCandidateVotingRecordArgs {
    #[structopt(about = "Votesmart candidate ID")]
    candidate_id: i32,
    #[structopt(short, long, about = "Print fetched JSON data to console")]
    pretty_print: bool,
    #[structopt(short, long, about = "Create bill records and joins")]
    create_record: bool,
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    utils::headline();

    db::init_pool().await.unwrap();
    let args = Args::from_args();

    match args.command {
        Command::Proxy { service } => handle_proxy_services(service).await?,
    }

    async fn handle_proxy_services(service: Service) -> Result<(), Error> {
        match service {
            Service::Legiscan(action) => handle_legiscan_action(action).await,
            Service::Votesmart(action) => handle_votesmart_action(action).await,
        }
    }

    async fn handle_legiscan_action(action: LegiscanAction) -> Result<(), Error> {
        match action {
            LegiscanAction::GetBill(args) => get_bill(args).await,
            LegiscanAction::GetBillText(args) => get_bill_text(args).await,
        }
    }

    async fn handle_votesmart_action(action: VoteSmartAction) -> Result<(), Error> {
        match action {
            VoteSmartAction::GetCandidateBio(args) => get_candidate_bio(args).await,
            VoteSmartAction::GetCandidateVotingRecord(args) => {
                get_candidate_voting_record(args).await
            }
        }
    }

    async fn get_candidate_bio(args: GetCandidateBioArgs) -> Result<(), Error> {
        println!(
            "\n▶️ FETCHING CANDIDATE BIO FROM LEGISCAN\n  📖 candidate_id: {}",
            args.candidate_id
        );

        let data = VotesmartProxy::new()
            .unwrap()
            .get_candidate_bio(args.candidate_id)
            .await;

        let data = data.unwrap().clone();

        if args.pretty_print {
            println!("{}", serde_json::to_string_pretty(&data).unwrap());
        }

        if args.create_record {
            let pool = db::pool().await;
            let vs_id = data
                .candidate
                .candidate_id
                .to_owned()
                .parse::<i32>()
                .unwrap();
            let first_name = data.candidate.first_name.to_owned();
            let middle_name = Some(data.candidate.middle_name.to_owned());
            let last_name = data.candidate.last_name.to_owned();
            let full_name = format!(
                "{:?} {:?}",
                data.candidate.first_name, data.candidate.last_name
            );
            let slug = slugify!(&full_name);
            let home_state = State::from_str(&data.candidate.home_state).unwrap();
            let office_party = Some(
                PoliticalParty::from_str(&data.office.to_owned().unwrap().parties)
                    .unwrap_or_default(),
            );

            let input = CreatePoliticianInput {
                first_name,
                middle_name,
                last_name,
                slug,
                home_state,
                office_party,
                votesmart_candidate_id: Some(vs_id),
                votesmart_candidate_bio: Some(serde_json::to_value(data.clone()).unwrap()),
                ..Default::default()
            };

            let new_record = db::Politician::create(&pool.connection, &input).await?;
            println!(
                "\n✅ Populist politician {} {} has been created and seeded with Votesmart data",
                new_record.first_name, new_record.last_name
            );
        }

        if args.update_record {
            let pool = db::pool().await;
            let vs_id = data.candidate.candidate_id.parse::<i32>().unwrap();

            let input = UpdatePoliticianInput {
                votesmart_candidate_bio: Some(serde_json::to_value(data.clone()).unwrap()),
                ..Default::default()
            };
            let updated_record =
                db::Politician::update(&pool.connection, None, Some(vs_id), &input).await?;
            println!(
                "\n✅ Populist politician with id {} has been updated with Votesmart data",
                updated_record.id
            );
        }

        Ok(())
    }

    async fn get_candidate_voting_record(args: GetCandidateVotingRecordArgs) -> Result<(), Error> {
        println!(
            "\n▶️ FETCHING CANDIDATE VOTING RECORD FROM VOTESMART\n  📖 candidate_id: {}",
            args.candidate_id
        );

        let data = VotesmartProxy::new()
            .unwrap()
            .get_candidate_voting_record(args.candidate_id)
            .await;

        let data = data.unwrap().clone();

        if args.pretty_print {
            println!("{}", serde_json::to_string_pretty(&data).unwrap());
        }

        if args.create_record {
            let pool = db::pool().await;

            // Find the populist politician
            let politician_id = sqlx::query!(
                r#"SELECT id FROM politician WHERE votesmart_candidate_id = $1"#,
                args.candidate_id
            )
            .fetch_one(&pool.connection)
            .await?
            .id;

            // Create all of the populist bill objects
            for bill in data.bills.bill.into_iter() {
                let vote_status = match bill.stage.as_ref() {
                    "Introduced" => LegislationStatus::INTRODUCED,
                    "Passage" => LegislationStatus::PASSED,
                    "Amendment Vote" => LegislationStatus::SIGNED,
                    "Concurrence Vote" => LegislationStatus::SIGNED,
                    "Conference Report Vote" => LegislationStatus::UNKNOWN,
                    "Motion Vote" => LegislationStatus::UNKNOWN,
                    _ => LegislationStatus::UNKNOWN,
                };

                let bill_input = CreateBillInput {
                    slug: Some(slugify!(&bill.bill_number)),
                    title: bill.title,
                    bill_number: bill.bill_number,
                    description: None,
                    vote_status,
                    official_summary: None,
                    populist_summary: None,
                    full_text_url: None,
                    legiscan_bill_id: None,
                    votesmart_bill_id: Some(bill.bill_id.parse::<i32>().unwrap()),
                    legiscan_data: None,
                    arguments: None,
                };

                let new_bill_record = db::Bill::create(&pool.connection, &bill_input).await;

                match new_bill_record {
                    Err(e) => {
                        let bill_row = sqlx::query!(
                            r#"
                                SELECT id FROM bill WHERE votesmart_bill_id = $1
                            "#,
                            bill.bill_id.parse::<i32>().unwrap()
                        )
                        .fetch_one(&pool.connection)
                        .await?;

                        sqlx::query!(
                            r#"
                            INSERT INTO politician_bills (politician_id, bill_id, vote)
                            VALUES ($1, $2, $3);
                        "#,
                            politician_id,
                            bill_row.id,
                            bill.vote
                        )
                        .fetch_optional(&pool.connection)
                        .await?;
                    }
                    Ok(_) => {
                        // Create the joins between our found politician and the bills they voted on
                        sqlx::query!(
                            r#"
                                INSERT INTO politician_bills (politician_id, bill_id, vote)
                                VALUES ($1, $2, $3);
                            "#,
                            politician_id,
                            new_bill_record.unwrap().id,
                            bill.vote
                        )
                        .fetch_optional(&pool.connection)
                        .await?;
                    }
                }
            }
        }
        Ok(())
    }

    async fn get_bill(args: GetBillArgs) -> Result<(), Error> {
        println!(
            "\n▶️ FETCHING BILL DATA FROM LEGISCAN\n  📖 bill_id: {}",
            args.bill_id
        );

        let data = LegiscanProxy::new()
            .unwrap()
            .get_bill(args.bill_id.to_string())
            .await;

        let data = data.unwrap().clone();

        if data == serde_json::Value::Null {
            println!(
                "Bill with bill_id: {} does not exist in the Legiscan API",
                args.bill_id
            );
            std::process::exit(0);
        }

        if args.pretty_print {
            println!("{}", serde_json::to_string_pretty(&data).unwrap());
        }

        if args.update_record {
            let pool = db::pool().await;
            let input = UpdateBillInput {
                legiscan_data: Some(data.clone()),
                ..Default::default()
            };
            let updated_record =
                db::Bill::update(&pool.connection, None, Some(args.bill_id), &input).await?;

            println!(
                "\n✅ Populist bill with id {} has been updated with legiscan data",
                updated_record.id
            );
        }

        Ok(())
    }

    async fn get_bill_text(args: GetBillTextArgs) -> Result<(), Error> {
        println!("{:?}", args.bill_id);
        Ok(())
    }

    Ok(())
}
