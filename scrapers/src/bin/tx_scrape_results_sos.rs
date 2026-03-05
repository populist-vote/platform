//! Download TX SOS results XML files from the Texas Secretary of State SFTP server
//! to data/tx/sos, then run the SOS results processor (same as process_tx_sos_results).
//!
//! Env: TX_SOS_SFTP_HOST, TX_SOS_SFTP_USER, TX_SOS_SFTP_PASSWORD; optional TX_SOS_SFTP_PORT (default 22),
//! TX_SOS_SFTP_REMOTE_DIR (default ".").
//!
//! Flags: --download-only (skip running the processor), --no-download (only run the processor).

use std::path::Path;
use std::sync::Arc;

use async_trait::async_trait;
use rusftp::client::SftpClient;
use russh::client::Handler;
use russh_keys::key::PublicKey;
use tokio::io::AsyncReadExt;

const TX_SOS_DATA_DIR: &str = "data/tx/sos";

struct SshHandler;

#[async_trait]
impl Handler for SshHandler {
    type Error = russh::Error;

    async fn check_server_key(
        &mut self,
        _server_public_key: &PublicKey,
    ) -> Result<bool, Self::Error> {
        Ok(true)
    }
}

fn tx_sos_data_path() -> std::path::PathBuf {
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap_or_else(|_| ".".into());
    Path::new(&manifest_dir).join(TX_SOS_DATA_DIR)
}

fn get_env(key: &str) -> Result<String, String> {
    std::env::var(key).map_err(|_| format!("Missing env: {}", key))
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // Load .env from cwd or parent (e.g. platform/.env) so TX_SOS_SFTP_* and DATABASE_URL are set
    dotenvy::dotenv().ok();

    let args: Vec<String> = std::env::args().collect();
    let download_only = args.iter().any(|a| a == "--download-only");
    let no_download = args.iter().any(|a| a == "--no-download");

    let do_download = !no_download;
    let do_process = !download_only;

    if do_download {
        let host = get_env("TX_SOS_SFTP_HOST").map_err(|e| {
            eprintln!("{}", e);
            e
        })?;
        let port: u16 = std::env::var("TX_SOS_SFTP_PORT")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(22);
        let user = get_env("TX_SOS_SFTP_USER").map_err(|e| {
            eprintln!("{}", e);
            e
        })?;
        let password = get_env("TX_SOS_SFTP_PASSWORD").map_err(|e| {
            eprintln!("{}", e);
            e
        })?;
        let remote_dir = std::env::var("TX_SOS_SFTP_REMOTE_DIR").unwrap_or_else(|_| ".".into());

        let local_dir = tx_sos_data_path();
        std::fs::create_dir_all(&local_dir)?;
        println!("=== TX SOS Results SFTP Download ===\n");
        println!("Host: {}:{}", host, port);
        println!("Remote dir: {}", remote_dir);
        println!("Local dir: {}\n", local_dir.display());

        let config = Arc::new(russh::client::Config::default());
        let mut ssh = russh::client::connect(config, (host.as_str(), port), SshHandler).await?;
        ssh.authenticate_password(&user, &password).await?;

        let mut sftp = SftpClient::new(ssh).await?;

        let name = sftp.readdir(&remote_dir).await?;
        let entries = name.0;
        let xml_entries: Vec<_> = entries
            .iter()
            .filter(|e| {
                let s = filename_to_string(&e.filename);
                !s.is_empty() && s != "." && s != ".." && s.ends_with(".xml")
            })
            .collect();

        println!("Found {} XML file(s)", xml_entries.len());
        let mut downloaded = 0;
        for entry in xml_entries {
            let name_str = filename_to_string(&entry.filename);
            // Use path relative to the directory we listed; many servers (e.g. chrooted) reject
            // absolute paths like "/ResultsData_53813.xml" with NoSuchFile.
            let remote_path = if remote_dir == "." || remote_dir == "/" || remote_dir.is_empty() {
                name_str.clone()
            } else {
                let base = remote_dir.trim_end_matches('/');
                format!("{}/{}", base, name_str)
            };
            let local_path = local_dir.join(&name_str);
            match download_file(&sftp, &remote_path, &local_path).await {
                Ok(()) => {
                    println!("  {} -> {}", name_str, local_path.display());
                    downloaded += 1;
                }
                Err(e) => {
                    eprintln!("  {}: download error: {}", name_str, e);
                }
            }
        }
        sftp.stop().await;
        println!("\nDownloaded {} file(s) to {}", downloaded, local_dir.display());
    }

    if do_process {
        println!("\n=== Running TX SOS Results Processor ===\n");
        db::init_pool().await.unwrap();
        let pool = db::pool().await;

        match scrapers::processors::tx::tx_sos_results::process_tx_sos_results(&pool.connection, true).await {
            Ok((files, rows)) => {
                println!(
                    "\n✓ Processed {} file(s), {} rows loaded into ingest_staging.stg_tx_results_sos",
                    files, rows
                );
            }
            Err(e) => {
                eprintln!("\n✗ Processor error: {}", e);
                std::process::exit(1);
            }
        }
    }

    println!("\n✓ Done.");
    Ok(())
}

fn filename_to_string(path: &rusftp::message::Path) -> String {
    path.0.clone()
}

async fn download_file(
    sftp: &SftpClient,
    remote_path: &str,
    local_path: &std::path::Path,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let mut file = sftp.open(remote_path).await?;
    let mut contents = Vec::new();
    file.read_to_end(&mut contents).await?;
    file.close().await?;
    tokio::fs::write(local_path, &contents).await?;
    Ok(())
}
