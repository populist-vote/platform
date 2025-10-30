use clap::Parser;
use colored::*;
use db::SystemRoleType;
use std::error::Error;
use std::process;
use std::str::FromStr;

#[derive(Parser)]
#[command(
    author,
    version,
    about = "Generate an access token. If no options are provided sensible defaults are used."
)]
struct Cli {
    /// Role for the token (e.g. Superuser). If omitted, defaults to Superuser.
    #[arg(long)]
    role: Option<String>,

    /// Optional email to embed in the token.
    #[arg(long)]
    email: Option<String>,

    /// Optional username to embed in the token.
    #[arg(long)]
    username: Option<String>,

    /// Optional comma-separated list of organization ids (e.g. --organizations 1,2,3).
    #[arg(long, value_delimiter = ',', value_parser = clap::value_parser!(i64))]
    organizations: Option<Vec<i64>>,
}

async fn generate_access_token(cli: Cli) -> Result<(), Box<dyn Error>> {
    dotenv::dotenv().ok();

    // sensible default
    let default_role = SystemRoleType::Superuser;

    // parse role string if provided, otherwise use default
    let role = match cli.role {
        Some(s) => match SystemRoleType::from_str(&s) {
            Ok(r) => r,
            Err(_) => {
                eprintln!(
                    "Warning: unrecognized role '{}', falling back to default: {:?}",
                    s, default_role
                );
                default_role
            }
        },
        None => default_role,
    };

    // Pass optional email, username and organizations through to token creation.
    // Adjust `auth::create_token` signature accordingly to accept these extra arguments.
    // convert organization ids (i64) into OrganizationRole instances if provided
    let organizations: Option<Vec<db::OrganizationRole>> = cli.organizations.map(|ids| {
        ids.into_iter()
            .map(|id| db::OrganizationRole {
                organization_id: uuid::Uuid::from_u128(id as u128),
                role: db::OrganizationRoleType::Member,
            })
            .collect()
    });

    let token = auth::create_token(Some(role), cli.username, cli.email, organizations)?;
    println!("\n🔑 {}", token.bold().green());
    Ok(())
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    if let Err(err) = generate_access_token(cli).await {
        eprintln!("Error occurred: {}", err);
        process::exit(1);
    }
}
