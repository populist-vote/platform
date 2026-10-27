#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    match args
        .iter()
        .map(String::as_str)
        .collect::<Vec<_>>()
        .as_slice()
    {
        [] => {
            server::run().await;
            Ok(())
        }
        ["migrate"] => server::migrations::release(false).await,
        ["migrate", "--local"] => server::migrations::release(true).await,
        _ => anyhow::bail!("Usage: server [migrate [--local]]"),
    }
}
