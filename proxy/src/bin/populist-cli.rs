use db::UpdateBillInput;
use structopt::StructOpt;

use proxy::{Error, LegiscanProxy};

static POPULIST: &'static str = r#"
8888888b.   .d88888b.  8888888b.  888     888 888      8888888 .d8888b. 88888888888 
888   Y88b d88P" "Y88b 888   Y88b 888     888 888        888  d88P  Y88b    888     
888    888 888     888 888    888 888     888 888        888  Y88b.         888     
888   d88P 888     888 888   d88P 888     888 888        888   "Y888b.      888     
8888888P"  888     888 8888888P"  888     888 888        888      "Y88b.    888     
888        888     888 888        888     888 888        888        "888    888     
888        Y88b. .d88P 888        Y88b. .d88P 888        888  Y88b  d88P    888     
888         "Y88888P"  888         "Y88888P"  88888888 8888888 "Y8888P"     WMC      
"#;

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
    #[structopt(about = "Legiscan bill id")]
    bill_id: i32,
    #[structopt(short, long, about = "Update populist record")]
    update_populist_record: bool,
    #[structopt(short, long, about = "Print fetched JSON data to console")]
    pretty_print: bool,
}

#[derive(Clone, Debug, StructOpt)]
struct GetBillTextArgs {
    #[structopt(about = "Legiscan bill id")]
    bill_id: String,
    #[structopt(short, long, about = "Update populist record")]
    update_populist_record: bool,
    #[structopt(short, long, about = "Print fetched JSON data to console")]
    pretty_print: bool,
}

#[derive(Clone, Debug, StructOpt)]
enum VoteSmartAction {
    GetCandidate { candidate_id: String },
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    eprintln!("{}", POPULIST);

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

    async fn get_bill(args: GetBillArgs) -> Result<(), Error> {
        println!(
            "\n▶️ FETCHING BILL DATA FROM LEGISCAN\n  📖 bill_id: {}",
            args.bill_id
        );

        let data = LegiscanProxy::new()
            .unwrap()
            .get_bill("234444".to_string())
            .await;

        let data = data.unwrap().clone();

        if args.pretty_print {
            println!("{}", serde_json::to_string_pretty(&data).unwrap());
        }

        if args.update_populist_record {
            let pool = db::pool().await;
            let input = UpdateBillInput {
                legiscan_data: Some(data.clone()),
                ..Default::default()
            };
            let updated_record =
                db::Bill::update(&pool.connection, None, Some(args.bill_id), &input).await?;
            println!("\n Populist bill record has been updated with legiscan data \n Populist bill: {}", updated_record.id);
        }

        Ok(())
    }

    async fn get_bill_text(args: GetBillTextArgs) -> Result<(), Error> {
        println!("{:?}", args.bill_id);
        Ok(())
    }

    async fn handle_votesmart_action(action: VoteSmartAction) -> Result<(), Error> {
        todo!()
    }

    Ok(())
}
