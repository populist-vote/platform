//! Texas election results scraper: SOS, Clarity, Hart, and Other county formats.
//!
//! Run one or more format scrapers via flags. With no format flag, runs all.
//!
//! Flags:
//!   --sos       Scrape TX Secretary of State results (SFTP download + process to stg_tx_results_sos).
//!   --clarity   Scrape Clarity county results (URL list CSV → stg_tx_results_clarity).
//!   --hart      Scrape Hart/CIRA PDF county results → stg_tx_results_hart.
//!   --other     Scrape Other county CSVs from data dir → stg_tx_results_other.
//!
//! SOS-only flags (when running SOS):
//!   --download-only   Only download from SFTP; do not run the processor.
//!   --no-download     Only run the processor; do not download.
//!
//! Optional positional arg: path to Clarity URL list CSV (used when running clarity; default from data/tx/counties).
//!
//! Env for SOS: TX_SOS_SFTP_HOST, TX_SOS_SFTP_USER, TX_SOS_SFTP_PASSWORD;
//! optional TX_SOS_SFTP_PORT (default 22), TX_SOS_SFTP_REMOTE_DIR (default ".").

use std::fs;
use std::path::{Path, PathBuf};
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

fn tx_sos_data_path() -> PathBuf {
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap_or_else(|_| ".".into());
    Path::new(&manifest_dir).join(TX_SOS_DATA_DIR)
}

fn get_env(key: &str) -> Result<String, String> {
    std::env::var(key).map_err(|_| format!("Missing env: {}", key))
}

fn filename_to_string(path: &rusftp::message::Path) -> String {
    path.0.clone()
}

async fn download_sos_file(
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

async fn run_sos(
    download_only: bool,
    no_download: bool,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
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
            let remote_path = if remote_dir == "." || remote_dir == "/" || remote_dir.is_empty() {
                name_str.clone()
            } else {
                let base = remote_dir.trim_end_matches('/');
                format!("{}/{}", base, name_str)
            };
            let local_path = local_dir.join(&name_str);
            match download_sos_file(&sftp, &remote_path, &local_path).await {
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
        let pool = db::pool().await;
        match scrapers::processors::tx::tx_results::process_tx_sos_results(
            &pool.connection,
            true,
        )
        .await
        {
            Ok((files, rows)) => {
                println!(
                    "\n✓ Processed {} file(s), {} rows loaded into ingest_staging.stg_tx_results_sos",
                    files, rows
                );
            }
            Err(e) => {
                eprintln!("\n✗ Processor error: {}", e);
                return Err(e.into());
            }
        }
    }
    Ok(())
}

async fn run_clarity(
    csv_path: Option<PathBuf>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    println!("=== TX Clarity County Results ===\n");
    println!(
        "Data dir: {}\n",
        scrapers::tx::counties::tx_clarity_results::clarity_data_path().display()
    );
    scrapers::tx::counties::tx_clarity_results::run(&db::pool().await.connection, csv_path)
        .await
        .map_err(|e| e.into())
}

async fn run_hart() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    println!("=== TX Hart County Results ===\n");
    println!(
        "Input:  {}",
        scrapers::tx::counties::tx_hart_results::hart_input_path().display()
    );
    println!(
        "Output: {}\n",
        scrapers::tx::counties::tx_hart_results::hart_output_path().display()
    );
    scrapers::tx::counties::tx_hart_results::run(&db::pool().await.connection)
        .await
        .map_err(|e| e.into())
}

async fn run_other() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    use scrapers::processors::tx::tx_results;

    println!("=== TX Other County Results ===\n");
    let data_dir = tx_results::other_data_path();
    println!("Data dir: {}\n", data_dir.display());

    if !data_dir.is_dir() {
        eprintln!("Directory does not exist: {}", data_dir.display());
        eprintln!("Create it and add CSV files, then run again.");
        return Err("Other data dir missing".into());
    }

    let db = &db::pool().await.connection;

    sqlx::query("CREATE SCHEMA IF NOT EXISTS ingest_staging")
        .execute(db)
        .await
        .expect("create schema");
    sqlx::query("DROP TABLE IF EXISTS ingest_staging.stg_tx_results_other")
        .execute(db)
        .await
        .expect("drop table");
    sqlx::query(
        r#"
        CREATE TABLE ingest_staging.stg_tx_results_other (
            id BIGSERIAL PRIMARY KEY,
            office_name TEXT,
            office_key TEXT,
            candidate_name TEXT,
            candidate_key TEXT,
            precincts_reporting BIGINT,
            precincts_total BIGINT,
            votes_for_candidate BIGINT,
            total_votes BIGINT,
            total_voters BIGINT,
            party TEXT,
            race_type TEXT,
            election_year INTEGER,
            ref_key TEXT NOT NULL,
            source_file TEXT,
            county TEXT,
            ingested_at TIMESTAMPTZ NOT NULL DEFAULT (now() AT TIME ZONE 'utc')
        )
        "#,
    )
    .execute(db)
    .await
    .expect("create table");

    let csv_files: Vec<_> = fs::read_dir(&data_dir)
        .expect("read dir")
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|p| {
            p.extension()
                .map_or(false, |e| e.eq_ignore_ascii_case("csv"))
        })
        .collect();

    if csv_files.is_empty() {
        eprintln!("No CSV files found in {}", data_dir.display());
        return Err("No CSV files".into());
    }

    println!("Found {} CSV(s)\n", csv_files.len());
    let mut total_rows = 0u64;
    for csv_path in &csv_files {
        let source_file = csv_path
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("unknown");
        match tx_results::process_other_csv(db, csv_path, source_file, None).await
        {
            Ok(n) => {
                println!("  {}: {} rows", csv_path.display(), n);
                total_rows += n;
            }
            Err(e) => {
                eprintln!("  {}: error: {}", csv_path.display(), e);
            }
        }
    }
    println!(
        "\n✓ Done. {} total rows -> ingest_staging.stg_tx_results_other",
        total_rows
    );
    Ok(())
}

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();

    let args: Vec<String> = std::env::args().collect();
    let run_sos_flag = args.iter().any(|a| a == "--sos");
    let run_clarity_flag = args.iter().any(|a| a == "--clarity");
    let run_hart_flag = args.iter().any(|a| a == "--hart");
    let run_other_flag = args.iter().any(|a| a == "--other");

    let any_format = run_sos_flag || run_clarity_flag || run_hart_flag || run_other_flag;
    let do_sos = run_sos_flag || !any_format;
    let do_clarity = run_clarity_flag || !any_format;
    let do_hart = run_hart_flag || !any_format;
    let do_other = run_other_flag || !any_format;

    let download_only = args.iter().any(|a| a == "--download-only");
    let no_download = args.iter().any(|a| a == "--no-download");

    let clarity_csv_path = args
        .iter()
        .skip(1)
        .find(|a| !a.starts_with("--"))
        .map(PathBuf::from);

    let need_db = (do_sos && !download_only && !no_download) || do_clarity || do_hart || do_other;
    if need_db {
        db::init_pool().await.unwrap();
    }

    let mut failed = false;

    if do_sos {
        if let Err(e) = run_sos(download_only, no_download).await {
            eprintln!("\n✗ SOS error: {}", e);
            failed = true;
        }
    }

    if do_clarity {
        if let Err(e) = run_clarity(clarity_csv_path.clone()).await {
            eprintln!("\n✗ Clarity error: {}", e);
            failed = true;
        }
    }

    if do_hart {
        if let Err(e) = run_hart().await {
            eprintln!("\n✗ Hart error: {}", e);
            failed = true;
        }
    }

    if do_other {
        if let Err(e) = run_other().await {
            eprintln!("\n✗ Other error: {}", e);
            failed = true;
        }
    }

    if failed {
        std::process::exit(1);
    }
    println!("\n✓ Done.");
}
