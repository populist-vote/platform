//! Texas county election results from Clarity.
//!
//! Reads county_clarity_results_urls.csv (columns: url, county_name, party). Downloads each zip
//! into `data/tx/counties/clarity` with county name (and optional party) appended to the filename,
//! uncompresses and renames extracted CSVs with the same suffix, then processes each CSV
//! via tx_results into ingest_staging.stg_tx_results_clarity.

use std::fs;
use std::io::{Cursor, Write};
use std::path::{Path, PathBuf};

use sqlx::PgPool;

use crate::processors::tx::tx_results;

use csv::{ReaderBuilder, WriterBuilder};
use zip::ZipArchive;

use crate::extractors::politician::title_case;
/// Recreate staging table once per Clarity scrape run.
pub async fn ensure_clarity_staging_table(
    pool: &PgPool,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    sqlx::query("CREATE SCHEMA IF NOT EXISTS ingest_staging")
        .execute(pool)
        .await?;

    // Recreate staging table each run to keep schema in sync with code.
    sqlx::query("DROP TABLE IF EXISTS ingest_staging.stg_tx_results_clarity")
        .execute(pool)
        .await?;

    sqlx::query(
        r#"
        CREATE TABLE ingest_staging.stg_tx_results_clarity (
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
    .execute(pool)
    .await?;

    Ok(())
}
use crate::util::decode_csv_bytes_to_utf8;

/// Default directory for TX Clarity data (relative to scrapers crate root).
pub const TX_CLARITY_DATA_DIR: &str = "data/tx/counties/clarity";

/// Directory containing the URL list CSV (parent of TX_CLARITY_DATA_DIR).
pub const TX_COUNTIES_DIR: &str = "data/tx/counties";

/// Default CSV file: columns url, county_name, party (in TX_COUNTIES_DIR).
pub const COUNTY_CLARITY_RESULTS_URLS_CSV: &str = "county_clarity_results_urls.csv";

/// One row from county_clarity_results_urls.csv.
#[derive(Debug)]
pub struct CountyClarityResultsUrl {
    pub url: String,
    pub county_name: String,
    /// If present (e.g. "dem", "rep"), appended to zip and CSV filenames so both party CSVs can be processed per county.
    pub party: Option<String>,
}

/// Return the path to the TX Clarity data directory.
pub fn clarity_data_path() -> PathBuf {
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap_or_else(|_| ".".into());
    Path::new(&manifest_dir).join(TX_CLARITY_DATA_DIR)
}

/// Return the path to the default county Clarity URL list CSV (data/tx/counties/county_clarity_results_urls.csv).
pub fn default_county_clarity_urls_csv_path() -> PathBuf {
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap_or_else(|_| ".".into());
    Path::new(&manifest_dir)
        .join(TX_COUNTIES_DIR)
        .join(COUNTY_CLARITY_RESULTS_URLS_CSV)
}

/// Sanitize county name for use in filenames (spaces -> underscore, drop path-like chars).
fn sanitize_for_filename(s: &str) -> String {
    s.trim()
        .chars()
        .map(|c| match c {
            ' ' | '\t' => '_',
            c if c.is_ascii_alphanumeric() || c == '-' || c == '_' => c,
            _ => '_',
        })
        .collect::<String>()
        .trim_matches('_')
        .to_string()
}

/// Read a CSV file, add a "county" column with the title-cased county name in each row, and write back.
/// Converts from Windows-1252 to UTF-8 if the file isn't valid UTF-8, so the written file is always valid UTF-8.
fn add_county_column_to_csv(
    path: &Path,
    county_name: &str,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let county_value = title_case(county_name);
    let bytes = fs::read(path)?;
    let content = decode_csv_bytes_to_utf8(&bytes);
    let mut rdr = ReaderBuilder::new().from_reader(Cursor::new(content.as_bytes()));
    let headers = rdr.headers()?.clone();
    let mut new_headers: Vec<String> = headers.iter().map(|h| h.to_string()).collect();
    new_headers.push("county".to_string());
    let records: Vec<csv::StringRecord> = rdr.records().collect::<Result<Vec<_>, _>>()?;
    let mut wtr = WriterBuilder::new().from_path(path)?;
    wtr.write_record(&new_headers)?;
    for record in &records {
        let mut row: Vec<String> = record.iter().map(|s| s.to_string()).collect();
        row.push(county_value.clone());
        wtr.write_record(&row)?;
    }
    wtr.flush()?;
    Ok(())
}

/// Load URL + county_name + optional party from county_clarity_results_urls.csv (columns: url, county_name, party).
pub fn load_county_clarity_results_urls(path: &Path) -> Result<Vec<CountyClarityResultsUrl>, Box<dyn std::error::Error + Send + Sync>> {
    let mut rdr = ReaderBuilder::new().from_path(path)?;
    let mut rows = Vec::new();
    for result in rdr.records() {
        let record = result?;
        if record.len() >= 2 {
            let url = record.get(0).unwrap().trim().to_string();
            let county_name = record.get(1).unwrap().trim().to_string();
            let party = record
                .get(2)
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty());
            if !url.is_empty() && !url.starts_with('#') {
                rows.push(CountyClarityResultsUrl { url, county_name, party });
            }
        }
    }
    Ok(rows)
}

/// Build the local zip filename: base name from URL with county (and optional party) appended before the extension.
/// E.g. url "https://x.com/summary.zip", county "dallas", party Some("dem") -> "summary_dallas_dem.zip".
fn zip_filename_for_url_county_party(url: &str, county_name: &str, party: Option<&str>) -> String {
    let base = url.rsplit('/').next().unwrap_or("download.zip");
    let county_sanitized = sanitize_for_filename(county_name);
    let suffix = if county_sanitized.is_empty() {
        String::new()
    } else if let Some(p) = party {
        let party_sanitized = sanitize_for_filename(p);
        if party_sanitized.is_empty() {
            county_sanitized
        } else {
            format!("{}_{}", county_sanitized, party_sanitized)
        }
    } else {
        county_sanitized
    };
    if suffix.is_empty() {
        return base.to_string();
    }
    let (stem, ext) = if let Some(dot) = base.rfind('.') {
        (&base[..dot], &base[dot..])
    } else {
        (base, "")
    };
    format!("{}_{}{}", stem, suffix, ext)
}

/// Download a URL to a destination path.
pub async fn download_to_path(
    client: &reqwest::Client,
    url: &str,
    dest: &Path,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let resp = client.get(url).send().await?.error_for_status()?;
    let bytes = resp.bytes().await?;
    if let Some(parent) = dest.parent() {
        fs::create_dir_all(parent)?;
    }
    let mut f = fs::File::create(dest)?;
    f.write_all(&bytes)?;
    Ok(())
}

/// Extract a zip file into the given directory. All entries are written as files under `out_dir`.
/// Uses only the base name of each entry to avoid path traversal.
/// If `filename_suffix` is Some, extracted CSV files are renamed to append the suffix before .csv
/// (e.g. "results.csv" -> "results_dallas_dem.csv"). If `county_name_for_column` is Some, a "county"
/// column is added to each CSV with that value (title-cased).
pub fn unzip_into_dir(
    zip_path: &Path,
    out_dir: &Path,
    filename_suffix: Option<&str>,
    county_name_for_column: Option<&str>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let file = fs::File::open(zip_path)?;
    let mut archive = ZipArchive::new(file)?;
    let suffix = filename_suffix.map(sanitize_for_filename);
    for i in 0..archive.len() {
        let mut entry = archive.by_index(i)?;
        let name = entry.name();
        // Skip directories and __MACOSX etc.
        if name.ends_with('/') || name.contains("__MACOSX") {
            continue;
        }
        let base = Path::new(name).file_name().unwrap_or_else(|| std::ffi::OsStr::new(name));
        let mut out_path = out_dir.join(base);
        if entry.is_dir() {
            fs::create_dir_all(&out_path)?;
        } else {
            if let Some(p) = out_path.parent() {
                fs::create_dir_all(p)?;
            }
            let mut out_file = fs::File::create(&out_path)?;
            std::io::copy(&mut entry, &mut out_file)?;
            // If this is a CSV and we have a filename suffix, rename to append it before .csv and add county column
            if let (Some(s), Some(ext)) = (suffix.as_ref(), out_path.extension()) {
                if !s.is_empty() && ext.eq_ignore_ascii_case("csv") {
                    if let Some(stem) = out_path.file_stem() {
                        let new_name = format!("{}_{}.csv", stem.to_string_lossy(), s);
                        let new_path = out_path.with_file_name(new_name);
                        if new_path.exists() {
                            fs::remove_file(&new_path)?;
                        }
                        fs::rename(&out_path, &new_path)?;
                        if let Some(county_name) = county_name_for_column {
                            add_county_column_to_csv(&new_path, county_name)?;
                        }
                    }
                }
            }
        }
    }
    Ok(())
}

/// Process a single Clarity CSV file: parse and insert rows into ingest_staging.stg_tx_results_clarity.
pub async fn process_csv(
    pool: &PgPool,
    csv_path: &Path,
) -> Result<u64, Box<dyn std::error::Error + Send + Sync>> {
    let source_file = csv_path
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("unknown");
    tx_results::process_clarity_csv(pool, csv_path, source_file, None).await
}

/// Run the Clarity pipeline: read county_results_urls.csv from `csv_path`, download each zip
/// into the Clarity data dir (filename has county name appended), unzip and rename CSVs with
/// county name, then process each resulting CSV into ingest_staging.stg_tx_results_clarity.
pub async fn run(
    pool: &PgPool,
    csv_path: Option<PathBuf>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // Create/recreate staging table once for the whole run (so we don't drop it per CSV).
    ensure_clarity_staging_table(pool).await?;

    let data_dir = clarity_data_path();
    let urls_csv_path = csv_path.unwrap_or_else(default_county_clarity_urls_csv_path);

    let rows = load_county_clarity_results_urls(&urls_csv_path)?;
    if rows.is_empty() {
        eprintln!("No rows found in {}", urls_csv_path.display());
        return Ok(());
    }

    fs::create_dir_all(&data_dir)?;
    let client = reqwest::Client::new();

    for row in &rows {
        let zip_filename = zip_filename_for_url_county_party(
            &row.url,
            &row.county_name,
            row.party.as_deref(),
        );
        let file_suffix = if let Some(ref p) = row.party {
            format!("{}_{}", sanitize_for_filename(&row.county_name), sanitize_for_filename(p))
        } else {
            sanitize_for_filename(&row.county_name)
        };
        let zip_path = data_dir.join(&zip_filename);
        println!("Downloading {} -> {} ({}{})", row.url, zip_path.display(), row.county_name, row.party.as_deref().map(|p| format!(", {}", p)).unwrap_or_default());
        download_to_path(&client, &row.url, &zip_path).await?;
        println!("Unzipping {} -> {} (CSVs renamed with suffix: {})", zip_path.display(), data_dir.display(), file_suffix);
        unzip_into_dir(
            &zip_path,
            &data_dir,
            if file_suffix.is_empty() { None } else { Some(file_suffix.as_str()) },
            Some(&row.county_name),
        )?;
        fs::remove_file(&zip_path)?;
        println!("Removed {}", zip_path.display());
    }

    let csv_files: Vec<PathBuf> = fs::read_dir(&data_dir)?
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|p| p.extension().map_or(false, |e| e.eq_ignore_ascii_case("csv")))
        .collect();

    for csv_path in csv_files {
        let n = process_csv(pool, &csv_path).await?;
        println!("  {}: {} rows", csv_path.display(), n);
    }

    Ok(())
}
