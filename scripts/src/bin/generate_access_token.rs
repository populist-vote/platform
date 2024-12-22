use clap::Parser;
use colored::*;
use db::SystemRoleType;
use std::error::Error;
use std::process;
use std::str::FromStr;

#[derive(Parser)]
struct Cli {
    #[arg(long, value_enum)]
    role: Option<SystemRoleType>,
}

async fn generate_access_token(role: Option<SystemRoleType>) -> Result<(), Box<dyn Error>> {
    dotenv::dotenv().ok();
    let token = auth::create_token(role.unwrap_or(SystemRoleType::Superuser))?;
    println!("\n🔑 {}", token.bold().green());
    Ok(())
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    if let Err(err) = generate_access_token(cli.role).await {
        println!("Error occurred: {}", err);
        process::exit(1);
    }
}
