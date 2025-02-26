use clap::Parser;
use db::State;
use server::jobs::import_legiscan_dataset;
use std::process;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(long)]
    session_id: i32,
    #[arg(long)]
    state: State,
    #[arg(long)]
    year: i32,
}

#[tokio::main]
async fn main() {
    let args = Args::parse();

    let params = import_legiscan_dataset::ImportSessionDataParams {
        session_id: args.session_id,
        state: args.state,
        year: args.year,
    };

    if let Err(err) = import_legiscan_dataset::run(params).await {
        println!("Error occurred: {}", err);
        process::exit(1);
    }
}
