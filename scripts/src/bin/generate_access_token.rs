use colored::*;
use std::error::Error;
use std::process;

async fn generate_access_token() -> Result<(), Box<dyn Error>> {
    dotenv::dotenv().ok();
    let token = auth::create_power_token()?;
    println!("\n🔑 {}", token.bold().green());
    Ok(())
}

#[tokio::main]
async fn main() {
    if let Err(err) = generate_access_token().await {
        println!("Error occurred: {}", err);
        process::exit(1);
    }
}
