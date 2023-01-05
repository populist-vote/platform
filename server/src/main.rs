#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    server::run().await
}
