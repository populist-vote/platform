//! Texas **local** candidate filings processor.
//!
//! Reads `p6t_state_tx.tx_2026_local_filings_nov3` into `ingest_staging.stg_tx_local_*` tables.
//! Merge to production is **not** wired to `tx_merge_filings` yet (that binary reads `stg_tx_*`).

use crate::extractors;
use crate::generators;
use db::{Office, Politician, Race, RaceType, State, VoteType};
use serde_json::Value as JSON;
use sqlx::FromRow;
use sqlx::PgPool;
use std::error::Error;
use uuid::Uuid;

/// Staging address built from a filing; inserted into `stg_tx_local_addresses` only when the politician is inserted.
#[derive(Debug, Clone)]
pub struct TxStagingAddress {
    pub line_1: String,
    pub city: String,
    pub state: String,
    pub country: String,
}

/// One row from `p6t_state_tx.tx_2026_local_filings_nov3`.
#[derive(Debug, FromRow)]
pub struct TxLocalFiling {
    pub county: Option<String>,
    pub municipality: Option<String>,
    pub office_name: Option<String>,
    pub district_type: Option<String>,
    pub school_district: Option<String>,
    pub candidate_name: Option<String>,
    pub incumbent: Option<String>,
    pub is_special_election: Option<bool>,
    pub status: Option<String>,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub campaign_website: Option<String>,
    pub place_fips: Option<String>,
    pub state: Option<String>,
    pub num_elect: Option<i32>,
}

const TX_LOCAL_SOURCE_TABLE: &str = "p6t_state_tx.tx_2026_local_filings_nov3";
const ELECTION_YEAR: i32 = 2026;

/// 2026 TX local election (replace if your ingest targets a different election).
pub const TX_LOCAL_ELECTION_ID: &str = "6138cc76-f273-43cf-a017-a98d1119b0c3";

/// Process local filings into `stg_tx_local_*` staging tables.
pub async fn process_tx_local_filings(pool: &PgPool) -> Result<(), Box<dyn Error>> {
    println!("Starting TX local candidate filings processor...");
    println!("Source table: {}", TX_LOCAL_SOURCE_TABLE);

    create_local_staging_tables(pool).await?;

    let election_slug = get_local_election_slug(pool).await?;
    println!("Election slug: {}", election_slug);

    let present_columns = source_table_columns(pool).await?;
    let missing_columns: Vec<&str> = SOURCE_FILING_COLUMNS
        .iter()
        .filter(|column| !present_columns.contains(column.name))
        .map(|column| column.name)
        .collect();
    if !missing_columns.is_empty() {
        println!(
            "Source table is missing columns; selecting NULL for: {}",
            missing_columns.join(", ")
        );
    }

    let query = format!(
        "SELECT\n            {}\n        FROM {}",
        source_filing_select_list(&present_columns),
        TX_LOCAL_SOURCE_TABLE
    );

    println!("Fetching local filings...");
    let filings: Vec<TxLocalFiling> = sqlx::query_as::<_, TxLocalFiling>(&query)
        .fetch_all(pool)
        .await?;

    println!("Found {} local filings to process", filings.len());

    let mut processed_count = 0usize;
    let mut skipped_status_count = 0usize;
    let mut error_count = 0usize;

    for (index, filing) in filings.iter().enumerate() {
        if index % 100 == 0 {
            println!("Processing filing {}/{}...", index + 1, filings.len());
        }
        match process_and_insert_tx_local_filing(pool, filing, &election_slug).await {
            Ok(true) => processed_count += 1,
            Ok(false) => skipped_status_count += 1,
            Err(e) => {
                error_count += 1;
                eprintln!(
                    "Error processing filing: {} — {}",
                    filing.candidate_name.as_deref().unwrap_or("Unknown"),
                    e
                );
            }
        }
    }

    println!("\n=== TX local processing complete ===");
    println!("Successfully processed: {}", processed_count);
    println!(
        "Skipped (status not Active/Unopposed): {}",
        skipped_status_count
    );
    println!("Errors: {}", error_count);
    println!("\nStaging tables (merge not in tx_merge_filings yet):");
    println!("  - ingest_staging.stg_tx_local_offices");
    println!("  - ingest_staging.stg_tx_local_politicians");
    println!("  - ingest_staging.stg_tx_local_politician_process_dupes");
    println!("  - ingest_staging.stg_tx_local_addresses");
    println!("  - ingest_staging.stg_tx_local_races");
    println!("  - ingest_staging.stg_tx_local_race_candidates");
    println!("  - ingest_staging.stg_tx_local_race_candidates_process_dupes");

    Ok(())
}

async fn create_local_staging_tables(pool: &PgPool) -> Result<(), Box<dyn Error>> {
    println!("Creating ingest_staging.stg_tx_local_* tables...");

    sqlx::query("CREATE SCHEMA IF NOT EXISTS ingest_staging")
        .execute(pool)
        .await?;

    sqlx::query(
        "DROP TABLE IF EXISTS ingest_staging.stg_tx_local_race_candidates_process_dupes CASCADE",
    )
    .execute(pool)
    .await?;
    sqlx::query("DROP TABLE IF EXISTS ingest_staging.stg_tx_local_race_candidates CASCADE")
        .execute(pool)
        .await?;
    sqlx::query("DROP TABLE IF EXISTS ingest_staging.stg_tx_local_races CASCADE")
        .execute(pool)
        .await?;
    sqlx::query("DROP TABLE IF EXISTS ingest_staging.stg_tx_local_addresses CASCADE")
        .execute(pool)
        .await?;
    sqlx::query(
        "DROP TABLE IF EXISTS ingest_staging.stg_tx_local_politician_process_dupes CASCADE",
    )
    .execute(pool)
    .await?;
    sqlx::query("DROP TABLE IF EXISTS ingest_staging.stg_tx_local_politicians CASCADE")
        .execute(pool)
        .await?;
    sqlx::query("DROP TABLE IF EXISTS ingest_staging.stg_tx_local_offices CASCADE")
        .execute(pool)
        .await?;

    sqlx::query(
        r#"
        CREATE TABLE ingest_staging.stg_tx_local_offices (
            id UUID PRIMARY KEY,
            slug TEXT NOT NULL UNIQUE,
            name TEXT,
            title TEXT,
            subtitle TEXT,
            subtitle_short TEXT,
            office_type TEXT,
            chamber TEXT,
            district_type TEXT,
            political_scope TEXT,
            election_scope TEXT NOT NULL,
            state TEXT,
            state_id TEXT,
            county TEXT,
            municipality TEXT,
            term_length INTEGER,
            district TEXT,
            subdistrict TEXT,
            seat TEXT,
            school_district TEXT,
            hospital_district TEXT,
            priority INTEGER,
            created_at TIMESTAMPTZ NOT NULL,
            updated_at TIMESTAMPTZ NOT NULL
        )
    "#,
    )
    .execute(pool)
    .await?;

    sqlx::query(
        r#"
        CREATE TABLE ingest_staging.stg_tx_local_politicians (
            id UUID PRIMARY KEY,
            slug TEXT NOT NULL UNIQUE,
            ref_key TEXT,
            first_name TEXT NOT NULL,
            middle_name TEXT,
            last_name TEXT NOT NULL,
            suffix TEXT,
            preferred_name TEXT,
            full_name TEXT,
            biography TEXT,
            biography_source TEXT,
            home_state TEXT,
            party_id UUID,
            date_of_birth DATE,
            office_id UUID,
            upcoming_race_id UUID,
            thumbnail_image_url TEXT,
            assets JSONB,
            official_website_url TEXT,
            campaign_website_url TEXT,
            facebook_url TEXT,
            twitter_url TEXT,
            instagram_url TEXT,
            youtube_url TEXT,
            linkedin_url TEXT,
            tiktok_url TEXT,
            email TEXT,
            phone TEXT,
            votesmart_candidate_id TEXT,
            votesmart_candidate_bio JSONB,
            votesmart_candidate_ratings JSONB,
            legiscan_people_id INTEGER,
            crp_candidate_id TEXT,
            fec_candidate_id TEXT,
            race_wins INTEGER,
            race_losses INTEGER,
            residence_address_id UUID,
            treat_exact_slug_as_same_person BOOLEAN NOT NULL DEFAULT false,
            created_at TIMESTAMPTZ NOT NULL,
            updated_at TIMESTAMPTZ NOT NULL
        )
    "#,
    )
    .execute(pool)
    .await?;

    sqlx::query(
        r#"
        CREATE TABLE ingest_staging.stg_tx_local_addresses (
            id UUID PRIMARY KEY,
            line_1 TEXT NOT NULL,
            city TEXT NOT NULL,
            state TEXT NOT NULL,
            country TEXT NOT NULL,
            politician_id UUID NOT NULL
        )
    "#,
    )
    .execute(pool)
    .await?;

    sqlx::query(
        r#"
        CREATE TABLE ingest_staging.stg_tx_local_politician_process_dupes (
            existing_id UUID NOT NULL,
            existing_slug TEXT NOT NULL,
            existing_email TEXT,
            existing_ref_key TEXT,
            incoming_id UUID NOT NULL,
            incoming_slug TEXT NOT NULL,
            incoming_email TEXT,
            incoming_ref_key TEXT,
            incoming_inserted BOOLEAN NOT NULL,
            created_at TIMESTAMPTZ NOT NULL DEFAULT (now() AT TIME ZONE 'utc')
        )
    "#,
    )
    .execute(pool)
    .await?;

    sqlx::query(
        r#"
        CREATE TABLE ingest_staging.stg_tx_local_races (
            id UUID PRIMARY KEY,
            title TEXT NOT NULL,
            slug TEXT NOT NULL UNIQUE,
            office_id UUID NOT NULL,
            state TEXT,
            race_type TEXT NOT NULL,
            vote_type TEXT NOT NULL,
            party_id UUID,
            description TEXT,
            ballotpedia_link TEXT,
            early_voting_begins_date DATE,
            official_website TEXT,
            election_id UUID,
            winner_ids UUID[],
            total_votes INTEGER,
            num_precincts_reporting INTEGER,
            total_precincts INTEGER,
            is_special_election BOOLEAN NOT NULL,
            num_elect INTEGER,
            created_at TIMESTAMPTZ NOT NULL,
            updated_at TIMESTAMPTZ NOT NULL
        )
    "#,
    )
    .execute(pool)
    .await?;

    sqlx::query(
        r#"
        CREATE TABLE ingest_staging.stg_tx_local_race_candidates (
            race_id UUID NOT NULL,
            candidate_id UUID NOT NULL,
            ref_key TEXT,
            PRIMARY KEY (race_id, candidate_id)
        )
    "#,
    )
    .execute(pool)
    .await?;

    sqlx::query(
        r#"
        CREATE TABLE ingest_staging.stg_tx_local_race_candidates_process_dupes (
            ref_key TEXT NOT NULL,
            existing_race_id UUID NOT NULL,
            existing_candidate_id UUID NOT NULL,
            incoming_race_id UUID NOT NULL,
            incoming_candidate_id UUID NOT NULL,
            created_at TIMESTAMPTZ NOT NULL DEFAULT (now() AT TIME ZONE 'utc')
        )
    "#,
    )
    .execute(pool)
    .await?;

    println!("Local staging tables created.");
    Ok(())
}

/// Ingest only rows whose `status` is **Active** or **Unopposed** (trimmed; case-insensitive).
fn should_ingest_local_filing_status(status: Option<&str>) -> bool {
    status
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .is_some_and(|s| s.eq_ignore_ascii_case("Active") || s.eq_ignore_ascii_case("Unopposed"))
}

struct SourceFilingColumn {
    name: &'static str,
    sql_type: &'static str,
}

const SOURCE_FILING_COLUMNS: &[SourceFilingColumn] = &[
    SourceFilingColumn {
        name: "county",
        sql_type: "text",
    },
    SourceFilingColumn {
        name: "municipality",
        sql_type: "text",
    },
    SourceFilingColumn {
        name: "office_name",
        sql_type: "text",
    },
    SourceFilingColumn {
        name: "district_type",
        sql_type: "text",
    },
    SourceFilingColumn {
        name: "school_district",
        sql_type: "text",
    },
    SourceFilingColumn {
        name: "candidate_name",
        sql_type: "text",
    },
    SourceFilingColumn {
        name: "incumbent",
        sql_type: "text",
    },
    SourceFilingColumn {
        name: "is_special_election",
        sql_type: "boolean",
    },
    SourceFilingColumn {
        name: "status",
        sql_type: "text",
    },
    SourceFilingColumn {
        name: "email",
        sql_type: "text",
    },
    SourceFilingColumn {
        name: "phone",
        sql_type: "text",
    },
    SourceFilingColumn {
        name: "campaign_website",
        sql_type: "text",
    },
    SourceFilingColumn {
        name: "place_fips",
        sql_type: "text",
    },
    SourceFilingColumn {
        name: "state",
        sql_type: "text",
    },
    SourceFilingColumn {
        name: "num_elect",
        sql_type: "integer",
    },
];

async fn source_table_columns(
    pool: &PgPool,
) -> Result<std::collections::HashSet<String>, Box<dyn Error>> {
    let (schema, table) = TX_LOCAL_SOURCE_TABLE
        .split_once('.')
        .ok_or("TX local source table must be schema-qualified")?;
    let columns = sqlx::query_scalar::<_, String>(
        r#"
        SELECT column_name
        FROM information_schema.columns
        WHERE table_schema = $1 AND table_name = $2
        "#,
    )
    .bind(schema)
    .bind(table)
    .fetch_all(pool)
    .await?;
    Ok(columns.into_iter().collect())
}

/// Selects each filing column that exists. A missing column is `NULL` of the expected type.
fn source_filing_select_list(present: &std::collections::HashSet<String>) -> String {
    SOURCE_FILING_COLUMNS
        .iter()
        .map(|column| {
            if present.contains(column.name) {
                column.name.to_string()
            } else {
                format!("NULL::{} AS {}", column.sql_type, column.name)
            }
        })
        .collect::<Vec<_>>()
        .join(",\n            ")
}

/// Returns `Ok(false)` if the row is skipped because `status` is not Active or Unopposed.
async fn process_and_insert_tx_local_filing(
    pool: &PgPool,
    filing: &TxLocalFiling,
    election_slug: &str,
) -> Result<bool, Box<dyn Error>> {
    if !should_ingest_local_filing_status(filing.status.as_deref()) {
        return Ok(false);
    }

    let office = process_tx_local_office(filing)?;
    let office_id = get_staging_local_office_id_by_slug(pool, &office.slug).await?;
    if office_id.is_none() {
        // TODO(custom): derive `state_id` for local offices if needed (primary uses SOS office title).
        let state_id: Option<String> = None;
        insert_staging_local_office(pool, &office, state_id.as_ref()).await?;
    }
    let resolved_office_id = office_id.unwrap_or(office.id);
    let race = process_tx_local_race(filing, &office, office_id, None)?;

    let mut politician = process_tx_local_politician(
        pool,
        filing,
        resolved_office_id,
        election_slug,
        &office,
        &race,
    )
    .await?;
    let address: Option<TxStagingAddress> = None;
    // TODO(custom): derive address from `place_fips` or other columns if desired.

    let politician_inserted =
        insert_staging_local_politician(pool, &mut politician, address).await?;

    if !politician_inserted {
        let name = politician.full_name.as_deref().unwrap_or("(no name)");
        eprintln!(
            "Politician not inserted (slug conflict, emails equal); skipping race/race_candidate: {}",
            name
        );
        return Ok(true);
    }

    insert_staging_local_race(pool, &race).await?;

    let race_id = get_staging_local_race_id_by_slug(pool, &race.slug)
        .await?
        .unwrap_or(race.id);
    let office_identity = local_office_identity_for_ref_key(&office, &race);
    let candidate_name = filing.candidate_name.as_deref().unwrap_or("");
    let race_candidate_ref_key = generators::politician::PoliticianRefKeyGenerator::new(
        "tx-local",
        election_slug,
        &office_identity,
        Some(candidate_name),
    )
    .generate();
    insert_staging_local_race_candidate(pool, race_id, &politician, &race_candidate_ref_key)
        .await?;

    Ok(true)
}

/// Strip a leading `"City of"` from a filing municipality (`"City of Austin"` → `"Austin"`).
fn normalize_municipality(municipality: &str) -> String {
    let trimmed = municipality.trim();
    let rest = trimmed
        .get(.."city of".len())
        .filter(|prefix| prefix.eq_ignore_ascii_case("city of"))
        .map(|_| trimmed["city of".len()..].trim_start())
        .unwrap_or(trimmed);
    rest.trim().to_string()
}

/// Filing counties are separated by `;`. Stored `office.county` uses `,`.
fn normalize_filing_counties(county: &str) -> Option<String> {
    let counties: Vec<&str> = county
        .split(';')
        .map(str::trim)
        .filter(|county| !county.is_empty())
        .collect();
    if counties.is_empty() {
        None
    } else {
        Some(counties.join(", "))
    }
}

/// Title-case a filing school district and shorten the district-type phrases.
/// `"Austin Independent School District"` → `"Austin ISD"`.
/// `"Example Common School District"` → `"Example CSD"`.
fn normalize_school_district(school_district: &str) -> String {
    extractors::politician::title_case(school_district.trim())
        .replace("Independent School District", "ISD")
        .replace("Common School District", "CSD")
}

/// Maps local filing columns to [`Office`] using the same TX office extractors as primary
/// filings, with raw title from [`TxLocalFiling::office_name`]. There is no party on local
/// rows, so [`extract_office_name`](crate::extractors::tx::tx_office::extract_office_name) is
/// called with `party: None`. County is taken only from [`TxLocalFiling::county`], not parsed
/// from the office title. Counties separated by `;` are stored separated by `,`. Seat is parsed
/// from `office_name`. When `save_name_as_district` is true, the office name is stored as
/// the district (with a trailing `" Board"` removed). A school board or special district
/// also stores the extracted subdivision on `subdistrict`. Otherwise the district is parsed
/// from `office_name`. A filing `school_district` replaces that district. Chamber is left
/// unset (not applicable to local offices).
fn process_tx_local_office(filing: &TxLocalFiling) -> Result<Office, Box<dyn Error>> {
    use crate::extractors::tx::tx_office as office;

    let raw_filing_title = filing.office_name.as_ref().ok_or("Missing office name")?;

    let (name, save_name_as_district) = office::extract_office_name(raw_filing_title, None)
        .ok_or("Failed to extract office name")?;

    let county = filing.county.as_deref().and_then(normalize_filing_counties);

    let municipality = filing
        .municipality
        .as_deref()
        .map(normalize_municipality)
        .filter(|m| !m.is_empty())
        .map(|m| extractors::politician::title_case(&m));

    let (seat, office_title_no_seat) = office::extract_office_seat(
        raw_filing_title,
        filing.num_elect,
        municipality.as_deref(),
        Some(&name),
    );
    
    //This code sets the district equal to the office name - right now this only applies
    //to school boards and special districts. We then extract the subdistrict, if it exists.
    let mut district = None;
    let mut subdistrict = None;
    let is_school_board = name.contains("School Board");
    if save_name_as_district {
        district = Some(
            name.strip_suffix(" Board")
                .unwrap_or(&name)
                .to_string(),
        );
        if office::is_special_district_with_subdivision(&name)
        {
            subdistrict = office::extract_office_district(
                &office_title_no_seat,
                seat.as_deref(),
                municipality.as_deref(),
                Some(&name),
            );
        }
    } else {
        if is_school_board {
            subdistrict = office::extract_office_district(
                &office_title_no_seat,
                seat.as_deref(),
                municipality.as_deref(),
                Some(&name),
            );
        } else {
        district = office::extract_office_district(
            &office_title_no_seat,
            seat.as_deref(),
            municipality.as_deref(),
            Some(&name),
        );
        }
    }

    let school_district = filing
        .school_district
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(normalize_school_district);
    if let Some(school_district) = &school_district {
        district = Some(school_district.clone());
    }

    let title = office::extract_office_title(&name).unwrap_or_default();

    let (political_scope, election_scope, district_type) = office::extract_office_scope(
        &name,
        county.as_deref(),
        seat.as_deref(),
        district.as_deref(),
    )
    .ok_or("Failed to extract office scope")?;

    let slug = generators::tx::tx_office::OfficeSlugGenerator {
        state: &State::TX,
        name: &name,
        county: county.as_deref(),
        district: district.as_deref(),
        subdistrict: subdistrict.as_deref(),
        seat: seat.as_deref(),
        school_district: school_district.as_deref(),
        hospital_district: None,
        municipality: municipality.as_deref(),
        election_scope: Some(&election_scope),
        district_type: district_type.as_ref(),
    }
    .generate();

    let (subtitle, subtitle_short) = generators::tx::tx_office::OfficeSubtitleGenerator {
        state: &State::TX,
        office_name: Some(&name),
        election_scope: &election_scope,
        district_type: district_type.as_ref(),
        county: county.as_deref(),
        district: district.as_deref(),
        subdistrict: subdistrict.as_deref(),
        seat: seat.as_deref(),
        school_district: school_district.as_deref(),
        hospital_district: None,
        municipality: municipality.as_deref(),
    }
    .generate();

    let priority = generators::tx::tx_office::office_priority(
        &title,
        county.as_deref(),
        district.as_deref(),
        district_type.as_ref(),
    );

    Ok(Office {
        id: Uuid::new_v4(),
        slug,
        name: Some(name),
        title,
        subtitle: Some(subtitle),
        subtitle_short: Some(subtitle_short),
        office_type: None,
        chamber: None,
        district_type,
        political_scope,
        election_scope,
        state: Some(State::TX),
        county,
        municipality,
        term_length: None,
        district,
        subdistrict,
        seat,
        school_district,
        hospital_district: None,
        priority,
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    })
}

fn process_tx_local_race(
    filing: &TxLocalFiling,
    office: &Office,
    office_id: Option<Uuid>,
    party_id: Option<Uuid>,
) -> Result<Race, Box<dyn Error>> {
    let election_id = Uuid::parse_str(TX_LOCAL_ELECTION_ID).map_err(|e| e.to_string())?;

    let is_special_election = filing.is_special_election.unwrap_or(false);
    let num_elect = filing.num_elect;

    let race_type = RaceType::General;
    let party_fec: Option<&str> = None;

    let (title, slug) = generators::tx::tx_race::RaceTitleGenerator::from_source(
        &race_type,
        office,
        is_special_election,
        party_fec,
        ELECTION_YEAR,
    )
    .generate();

    let resolved_office_id = office_id.unwrap_or(office.id);
    Ok(Race {
        id: Uuid::new_v4(),
        title,
        slug,
        office_id: resolved_office_id,
        state: Some(State::TX),
        race_type,
        vote_type: VoteType::Plurality,
        party_id,
        description: None,
        ballotpedia_link: None,
        early_voting_begins_date: None,
        official_website: None,
        election_id: Some(election_id),
        winner_ids: None,
        total_votes: None,
        num_precincts_reporting: None,
        total_precincts: None,
        is_special_election,
        num_elect,
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    })
}

fn local_office_identity_for_ref_key(office: &Office, race: &Race) -> String {
    let mut identity = format!(
        "{}|{}",
        office.name.as_deref().unwrap_or(""),
        office.subtitle.as_deref().unwrap_or(""),
    );
    if race.is_special_election {
        identity.push_str("|Special Election");
    }
    identity
}

async fn process_tx_local_politician(
    pool: &PgPool,
    filing: &TxLocalFiling,
    current_office_id: Uuid,
    election_slug: &str,
    office: &Office,
    race: &Race,
) -> Result<Politician, Box<dyn Error>> {
    let candidate_name_raw = filing
        .candidate_name
        .as_ref()
        .ok_or("Missing candidate name")?;
    let candidate_name = extractors::politician::normalize_name(candidate_name_raw);

    let office_identity = local_office_identity_for_ref_key(office, race);
    let ref_key = generators::politician::PoliticianRefKeyGenerator::new(
        "TX-LOCAL",
        election_slug,
        &office_identity,
        Some(&candidate_name),
    )
    .generate();

    let fec_code = "N";
    let party_id = sqlx::query_scalar!(r#"SELECT id FROM party WHERE fec_code = $1"#, fec_code)
        .fetch_optional(pool)
        .await?;

    let name_parts = match extractors::politician::extract_politician_name(&candidate_name) {
        Some(parts) => parts,
        None => {
            eprintln!(
                "extract_politician_name returned None; using simple split for: {:?}",
                candidate_name
            );
            let parts: Vec<&str> = candidate_name.split_whitespace().collect();
            if let (Some(first), Some(last)) = (parts.first(), parts.last()) {
                extractors::politician::PoliticianName {
                    first: (*first).to_string(),
                    middle: if parts.len() > 2 {
                        Some(parts[1..parts.len() - 1].join(" "))
                    } else {
                        None
                    },
                    last: Some((*last).to_string()),
                    suffix: None,
                    preferred: None,
                }
            } else {
                return Err("Failed to parse candidate name".into());
            }
        }
    };

    let title = extractors::politician::title_case;
    let first_name = title(&name_parts.first);
    let middle_name = name_parts.middle.as_deref().map(&title);
    let last_name = name_parts.last.as_deref().map(&title).unwrap_or_default();
    let suffix = name_parts.suffix.as_deref().map(&title);
    let preferred_name = name_parts.preferred.as_deref().map(&title);
    let full_name_display = title(&candidate_name);
    let slug = generators::politician::PoliticianSlugGenerator::new(&full_name_display)
        .with_state("TX")
        .generate();

    let is_incumbent = filing
        .incumbent
        .as_deref()
        .map(|s| s.trim().eq_ignore_ascii_case("YES"))
        .unwrap_or(false);
    let office_id = if is_incumbent {
        Some(current_office_id)
    } else {
        None
    };

    let assets = generators::politician::politician_thumbnail_assets(&slug);

    let campaign_website_trimmed = filing
        .campaign_website
        .as_deref()
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .map(|s| s.to_string());
    let phone_trimmed = filing
        .phone
        .as_deref()
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .map(|s| s.to_string());

    Ok(Politician {
        id: Uuid::new_v4(),
        slug,
        ref_key: Some(ref_key),
        first_name,
        middle_name,
        last_name,
        suffix,
        preferred_name,
        full_name: Some(full_name_display),
        biography: None,
        biography_source: None,
        home_state: Some(State::TX),
        party_id,
        date_of_birth: None,
        office_id,
        upcoming_race_id: None,
        thumbnail_image_url: None,
        assets,
        official_website_url: None,
        ballotpedia_url: None,
        campaign_website_url: campaign_website_trimmed,
        facebook_url: None,
        twitter_url: None,
        instagram_url: None,
        youtube_url: None,
        linkedin_url: None,
        tiktok_url: None,
        email: filing.email.as_ref().map(|e| e.to_lowercase()),
        phone: phone_trimmed,
        votesmart_candidate_id: None,
        votesmart_candidate_bio: JSON::Object(serde_json::Map::new()),
        votesmart_candidate_ratings: JSON::Object(serde_json::Map::new()),
        legiscan_people_id: None,
        crp_candidate_id: None,
        fec_candidate_id: None,
        race_wins: None,
        race_losses: None,
        residence_address_id: None,
        campaign_address_id: None,
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    })
}

async fn get_local_election_slug(pool: &PgPool) -> Result<String, Box<dyn Error>> {
    let election_id = Uuid::parse_str(TX_LOCAL_ELECTION_ID)?;
    let slug = sqlx::query_scalar!(r#"SELECT slug FROM election WHERE id = $1"#, election_id)
        .fetch_optional(pool)
        .await?
        .ok_or_else(|| format!("No election found for id {}", TX_LOCAL_ELECTION_ID))?;
    Ok(slug)
}

async fn get_staging_local_office_id_by_slug(
    pool: &PgPool,
    slug: &str,
) -> Result<Option<Uuid>, Box<dyn Error>> {
    let row: Option<(Uuid,)> =
        sqlx::query_as("SELECT id FROM ingest_staging.stg_tx_local_offices WHERE slug = $1")
            .bind(slug)
            .fetch_optional(pool)
            .await?;
    Ok(row.map(|(id,)| id))
}

async fn get_staging_local_race_id_by_slug(
    pool: &PgPool,
    slug: &str,
) -> Result<Option<Uuid>, Box<dyn Error>> {
    let row: Option<(Uuid,)> =
        sqlx::query_as("SELECT id FROM ingest_staging.stg_tx_local_races WHERE slug = $1")
            .bind(slug)
            .fetch_optional(pool)
            .await?;
    Ok(row.map(|(id,)| id))
}

async fn insert_staging_local_office(
    pool: &PgPool,
    office: &Office,
    state_id: Option<&String>,
) -> Result<(), Box<dyn Error>> {
    sqlx::query(
        r#"
        INSERT INTO ingest_staging.stg_tx_local_offices (
            id, slug, name, title, subtitle, subtitle_short, office_type, chamber,
            district_type, political_scope, election_scope, state, state_id, county, municipality,
            term_length, district, subdistrict, seat, school_district, hospital_district, priority,
            created_at, updated_at
        ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18, $19, $20, $21, $22, $23, $24)
        ON CONFLICT (slug) DO NOTHING
        "#,
    )
    .bind(office.id)
    .bind(&office.slug)
    .bind(&office.name)
    .bind(&office.title)
    .bind(&office.subtitle)
    .bind(&office.subtitle_short)
    .bind(office.office_type.as_deref())
    .bind(office.chamber.as_ref().map(|c| format!("{:?}", c)))
    .bind(office.district_type.as_ref().map(|d| format!("{:?}", d)))
    .bind(format!("{:?}", office.political_scope))
    .bind(format!("{:?}", office.election_scope))
    .bind(office.state.as_ref().map(|s| s.as_ref().to_string()))
    .bind(state_id)
    .bind(&office.county)
    .bind(&office.municipality)
    .bind(office.term_length)
    .bind(&office.district)
    .bind(&office.subdistrict)
    .bind(&office.seat)
    .bind(&office.school_district)
    .bind(&office.hospital_district)
    .bind(office.priority)
    .bind(office.created_at)
    .bind(office.updated_at)
    .execute(pool)
    .await?;
    Ok(())
}

async fn execute_staging_local_politician_insert(
    pool: &PgPool,
    politician: &Politician,
) -> Result<u64, sqlx::Error> {
    let result = sqlx::query(
        r#"
        INSERT INTO ingest_staging.stg_tx_local_politicians (
            id, slug, ref_key, first_name, middle_name, last_name, suffix, preferred_name,
            full_name, biography, biography_source, home_state, party_id, date_of_birth,
            office_id, upcoming_race_id, thumbnail_image_url, assets, official_website_url,
            campaign_website_url, facebook_url, twitter_url, instagram_url, youtube_url,
            linkedin_url, tiktok_url, email, phone, votesmart_candidate_id,
            votesmart_candidate_bio, votesmart_candidate_ratings, legiscan_people_id,
            crp_candidate_id, fec_candidate_id, race_wins, race_losses,
            residence_address_id, treat_exact_slug_as_same_person, created_at, updated_at
        ) VALUES (
            $1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18,
            $19, $20, $21, $22, $23, $24, $25, $26, $27, $28, $29, $30, $31, $32, $33, $34,
            $35, $36, $37, $38, $39, $40
        )
        ON CONFLICT (slug) DO NOTHING
        "#,
    )
    .bind(politician.id)
    .bind(&politician.slug)
    .bind(&politician.ref_key)
    .bind(&politician.first_name)
    .bind(&politician.middle_name)
    .bind(&politician.last_name)
    .bind(&politician.suffix)
    .bind(&politician.preferred_name)
    .bind(&politician.full_name)
    .bind(&politician.biography)
    .bind(&politician.biography_source)
    .bind(
        politician
            .home_state
            .as_ref()
            .map(|s| s.as_ref().to_string()),
    )
    .bind(politician.party_id)
    .bind(politician.date_of_birth)
    .bind(politician.office_id)
    .bind(politician.upcoming_race_id)
    .bind(&politician.thumbnail_image_url)
    .bind(&politician.assets)
    .bind(&politician.official_website_url)
    .bind(&politician.campaign_website_url)
    .bind(&politician.facebook_url)
    .bind(&politician.twitter_url)
    .bind(&politician.instagram_url)
    .bind(&politician.youtube_url)
    .bind(&politician.linkedin_url)
    .bind(&politician.tiktok_url)
    .bind(&politician.email)
    .bind(&politician.phone)
    .bind(politician.votesmart_candidate_id)
    .bind(&politician.votesmart_candidate_bio)
    .bind(&politician.votesmart_candidate_ratings)
    .bind(politician.legiscan_people_id)
    .bind(&politician.crp_candidate_id)
    .bind(&politician.fec_candidate_id)
    .bind(politician.race_wins)
    .bind(politician.race_losses)
    .bind(politician.residence_address_id)
    .bind(false)
    .bind(politician.created_at)
    .bind(politician.updated_at)
    .execute(pool)
    .await?;
    Ok(result.rows_affected())
}

fn is_same_person_from_emails_and_addresses(
    existing_email_trimmed: Option<&str>,
    existing_address: Option<&TxStagingAddress>,
    incoming_email_trimmed: Option<&str>,
    incoming_address: Option<&TxStagingAddress>,
) -> (bool, bool) {
    let both_emails_empty = existing_email_trimmed.is_none() && incoming_email_trimmed.is_none();
    let emails_equal_not_both_empty = match (existing_email_trimmed, incoming_email_trimmed) {
        (None, None) => false,
        (Some(a), Some(b)) => a.eq_ignore_ascii_case(b),
        _ => false,
    };
    let both_addresses_empty = existing_address.is_none() && incoming_address.is_none();
    let addresses_equal = match (existing_address, incoming_address) {
        (Some(ex), Some(in_)) => {
            ex.line_1.trim().eq_ignore_ascii_case(in_.line_1.trim())
                && ex.city.trim().eq_ignore_ascii_case(in_.city.trim())
                && ex.state.trim().eq_ignore_ascii_case(in_.state.trim())
                && ex.country.trim().eq_ignore_ascii_case(in_.country.trim())
        }
        _ => false,
    };
    let same_person = if both_emails_empty {
        !both_addresses_empty && addresses_equal
    } else if emails_equal_not_both_empty {
        true
    } else {
        !both_addresses_empty && addresses_equal
    };
    (same_person, emails_equal_not_both_empty)
}

async fn record_politician_local_dupe(
    pool: &PgPool,
    existing_id: Uuid,
    existing_slug: &str,
    existing_email: Option<&str>,
    existing_ref_key: Option<&str>,
    incoming_id: Uuid,
    incoming_slug: &str,
    incoming_email: Option<&str>,
    incoming_ref_key: Option<&str>,
    incoming_inserted: bool,
) -> Result<(), Box<dyn Error>> {
    sqlx::query(
        r#"
        INSERT INTO ingest_staging.stg_tx_local_politician_process_dupes
            (existing_id, existing_slug, existing_email, existing_ref_key, incoming_id, incoming_slug, incoming_email, incoming_ref_key, incoming_inserted)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
        "#,
    )
    .bind(existing_id)
    .bind(existing_slug)
    .bind(existing_email)
    .bind(existing_ref_key)
    .bind(incoming_id)
    .bind(incoming_slug)
    .bind(incoming_email)
    .bind(incoming_ref_key)
    .bind(incoming_inserted)
    .execute(pool)
    .await?;
    Ok(())
}

async fn insert_staging_local_address_for_politician(
    pool: &PgPool,
    addr: &TxStagingAddress,
    politician_id: Uuid,
    update_politician_row: bool,
) -> Result<Uuid, Box<dyn Error>> {
    let stg_address_id = Uuid::new_v4();
    sqlx::query(
        r#"
        INSERT INTO ingest_staging.stg_tx_local_addresses (id, line_1, city, state, country, politician_id)
        VALUES ($1, $2, $3, $4, $5, $6)
        "#,
    )
    .bind(stg_address_id)
    .bind(&addr.line_1)
    .bind(&addr.city)
    .bind(&addr.state)
    .bind(&addr.country)
    .bind(politician_id)
    .execute(pool)
    .await?;
    if update_politician_row {
        sqlx::query(
            r#"UPDATE ingest_staging.stg_tx_local_politicians SET residence_address_id = $1 WHERE id = $2"#,
        )
        .bind(stg_address_id)
        .bind(politician_id)
        .execute(pool)
        .await?;
    }
    Ok(stg_address_id)
}

async fn apply_same_person_updates_and_dupes_local(
    pool: &PgPool,
    existing_id: uuid::Uuid,
    existing_slug: &str,
    existing_email: &Option<String>,
    existing_ref_key: &Option<String>,
    existing_residence_address_id: Option<uuid::Uuid>,
    emails_equal_not_both_empty: bool,
    politician: &Politician,
    incoming_address: Option<&TxStagingAddress>,
    incoming_slug_used: &str,
) -> Result<(), Box<dyn Error>> {
    if emails_equal_not_both_empty && existing_residence_address_id.is_none() {
        if let Some(addr) = incoming_address {
            insert_staging_local_address_for_politician(pool, addr, existing_id, true).await?;
        }
    }
    if let Some(incoming_trimmed) = politician
        .email
        .as_deref()
        .map(|e| e.trim())
        .filter(|e| !e.is_empty())
    {
        sqlx::query(
            r#"UPDATE ingest_staging.stg_tx_local_politicians SET email = $1 WHERE id = $2"#,
        )
        .bind(incoming_trimmed)
        .bind(existing_id)
        .execute(pool)
        .await?;
    }
    record_politician_local_dupe(
        pool,
        existing_id,
        existing_slug,
        existing_email.as_deref(),
        existing_ref_key.as_deref(),
        politician.id,
        incoming_slug_used,
        politician.email.as_deref(),
        politician.ref_key.as_deref(),
        false,
    )
    .await?;
    Ok(())
}

async fn insert_staging_local_politician(
    pool: &PgPool,
    politician: &mut Politician,
    address: Option<TxStagingAddress>,
) -> Result<bool, Box<dyn Error>> {
    let mut rows_affected = execute_staging_local_politician_insert(pool, politician).await?;

    if rows_affected > 0 {
        if let Some(addr) = &address {
            let stg_address_id =
                insert_staging_local_address_for_politician(pool, addr, politician.id, true)
                    .await?;
            politician.residence_address_id = Some(stg_address_id);
        }
        return Ok(true);
    }

    let existing: (
        uuid::Uuid,
        String,
        Option<String>,
        Option<String>,
        Option<uuid::Uuid>,
    ) = sqlx::query_as(
        r#"
        SELECT id, slug, email, ref_key, residence_address_id
        FROM ingest_staging.stg_tx_local_politicians
        WHERE slug = $1
        "#,
    )
    .bind(&politician.slug)
    .fetch_optional(pool)
    .await?
    .ok_or_else(|| "Existing row not found for slug conflict".to_string())?;

    let (
        existing_id,
        existing_slug,
        existing_email,
        existing_ref_key,
        existing_residence_address_id,
    ) = existing;
    let existing_email_trimmed = existing_email
        .as_deref()
        .map(|e| e.trim())
        .filter(|e| !e.is_empty());
    let incoming_email_trimmed = politician
        .email
        .as_deref()
        .map(|e| e.trim())
        .filter(|e| !e.is_empty());

    let existing_address = fetch_staging_local_address(pool, existing_residence_address_id).await?;
    let incoming_address = address.as_ref();

    let (same_person, emails_equal_not_both_empty) = is_same_person_from_emails_and_addresses(
        existing_email_trimmed,
        existing_address.as_ref(),
        incoming_email_trimmed,
        incoming_address,
    );

    if same_person {
        apply_same_person_updates_and_dupes_local(
            pool,
            existing_id,
            &existing_slug,
            &existing_email,
            &existing_ref_key,
            existing_residence_address_id,
            emails_equal_not_both_empty,
            politician,
            incoming_address,
            &politician.slug,
        )
        .await?;
        return Ok(false);
    }

    let base_slug = &politician.slug;
    let final_incoming_slug;
    let mut n = 1u32;
    loop {
        let candidate = format!("{}-{}", base_slug, n);
        let existing: Option<(
            uuid::Uuid,
            String,
            Option<String>,
            Option<String>,
            Option<uuid::Uuid>,
        )> = sqlx::query_as(
            r#"
            SELECT id, slug, email, ref_key, residence_address_id
            FROM ingest_staging.stg_tx_local_politicians
            WHERE slug = $1
            "#,
        )
        .bind(&candidate)
        .fetch_optional(pool)
        .await?;

        match existing {
            None => {
                final_incoming_slug = candidate;
                break;
            }
            Some((eid, eslug, eemail, eref_key, eaddr_id)) => {
                let e_email_trimmed = eemail
                    .as_deref()
                    .map(|e| e.trim())
                    .filter(|e| !e.is_empty());
                let e_address = fetch_staging_local_address(pool, eaddr_id).await?;
                let (same, emails_eq) = is_same_person_from_emails_and_addresses(
                    e_email_trimmed,
                    e_address.as_ref(),
                    incoming_email_trimmed,
                    incoming_address,
                );
                if same {
                    apply_same_person_updates_and_dupes_local(
                        pool,
                        eid,
                        &eslug,
                        &eemail,
                        &eref_key,
                        eaddr_id,
                        emails_eq,
                        politician,
                        incoming_address,
                        &candidate,
                    )
                    .await?;
                    return Ok(false);
                }
            }
        }
        n += 1;
    }

    let mut incoming_with_new_slug = politician.clone();

    if let Some(addr) = &address {
        let stg_address_id =
            insert_staging_local_address_for_politician(pool, addr, politician.id, false).await?;
        incoming_with_new_slug.residence_address_id = Some(stg_address_id);
        politician.residence_address_id = Some(stg_address_id);
    }
    incoming_with_new_slug.slug = final_incoming_slug.clone();
    rows_affected = execute_staging_local_politician_insert(pool, &incoming_with_new_slug).await?;

    let incoming_inserted = rows_affected > 0;
    record_politician_local_dupe(
        pool,
        existing_id,
        &existing_slug,
        existing_email.as_deref(),
        existing_ref_key.as_deref(),
        politician.id,
        &final_incoming_slug,
        politician.email.as_deref(),
        politician.ref_key.as_deref(),
        incoming_inserted,
    )
    .await?;

    Ok(rows_affected > 0)
}

async fn fetch_staging_local_address(
    pool: &PgPool,
    residence_address_id: Option<uuid::Uuid>,
) -> Result<Option<TxStagingAddress>, Box<dyn Error>> {
    let addr_id = match residence_address_id {
        Some(id) => id,
        None => return Ok(None),
    };
    let row: Option<(String, String, String, String)> = sqlx::query_as(
        r#"SELECT line_1, city, state, country FROM ingest_staging.stg_tx_local_addresses WHERE id = $1"#,
    )
    .bind(addr_id)
    .fetch_optional(pool)
    .await?;
    Ok(row.map(|(line_1, city, state, country)| TxStagingAddress {
        line_1,
        city,
        state,
        country,
    }))
}

async fn insert_staging_local_race(pool: &PgPool, race: &Race) -> Result<(), Box<dyn Error>> {
    sqlx::query(
        r#"
        INSERT INTO ingest_staging.stg_tx_local_races (
            id, title, slug, office_id, state, race_type, vote_type, party_id,
            description, ballotpedia_link, early_voting_begins_date, official_website,
            election_id, winner_ids, total_votes, num_precincts_reporting, total_precincts,
            is_special_election, num_elect, created_at, updated_at
        ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18, $19, $20, $21)
        ON CONFLICT (slug) DO NOTHING
        "#,
    )
    .bind(race.id)
    .bind(&race.title)
    .bind(&race.slug)
    .bind(race.office_id)
    .bind(race.state.as_ref().map(|s| s.as_ref().to_string()))
    .bind(format!("{:?}", race.race_type))
    .bind(format!("{:?}", race.vote_type))
    .bind(race.party_id)
    .bind(&race.description)
    .bind(&race.ballotpedia_link)
    .bind(race.early_voting_begins_date)
    .bind(&race.official_website)
    .bind(race.election_id)
    .bind(&race.winner_ids)
    .bind(race.total_votes)
    .bind(race.num_precincts_reporting)
    .bind(race.total_precincts)
    .bind(race.is_special_election)
    .bind(race.num_elect)
    .bind(race.created_at)
    .bind(race.updated_at)
    .execute(pool)
    .await?;
    Ok(())
}

async fn insert_staging_local_race_candidate(
    pool: &PgPool,
    race_id: Uuid,
    politician: &Politician,
    ref_key: &str,
) -> Result<(), Box<dyn Error>> {
    #[derive(sqlx::FromRow)]
    struct ExistingRow {
        race_id: Uuid,
        candidate_id: Uuid,
    }

    let existing: Option<ExistingRow> = sqlx::query_as(
        r#"
        SELECT race_id, candidate_id
        FROM ingest_staging.stg_tx_local_race_candidates
        WHERE ref_key = $1
        "#,
    )
    .bind(ref_key)
    .fetch_optional(pool)
    .await?;

    if let Some(row) = existing {
        sqlx::query(
            r#"
            INSERT INTO ingest_staging.stg_tx_local_race_candidates_process_dupes
                (ref_key, existing_race_id, existing_candidate_id, incoming_race_id, incoming_candidate_id)
            VALUES ($1, $2, $3, $4, $5)
            "#,
        )
        .bind(ref_key)
        .bind(row.race_id)
        .bind(row.candidate_id)
        .bind(race_id)
        .bind(politician.id)
        .execute(pool)
        .await?;
        return Ok(());
    }

    sqlx::query(
        r#"
        INSERT INTO ingest_staging.stg_tx_local_race_candidates (race_id, candidate_id, ref_key)
        VALUES ($1, $2, $3)
        ON CONFLICT (race_id, candidate_id) DO UPDATE SET ref_key = EXCLUDED.ref_key
        "#,
    )
    .bind(race_id)
    .bind(politician.id)
    .bind(ref_key)
    .execute(pool)
    .await?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{process_tx_local_office, normalize_filing_counties, TxLocalFiling};

    #[test]
    fn school_district_replaces_office_district() {
        let filing = TxLocalFiling {
            county: Some("Bexar".to_string()),
            municipality: None,
            office_name: Some(
                "Somerset Independent School District, District 3".to_string(),
            ),
            district_type: None,
            school_district: Some("Austin Independent School District".to_string()),
            candidate_name: None,
            incumbent: None,
            is_special_election: None,
            status: Some("Active".to_string()),
            email: None,
            phone: None,
            campaign_website: None,
            place_fips: None,
            state: None,
            num_elect: Some(1),
        };
        let office = process_tx_local_office(&filing).unwrap();
        assert_eq!(office.district.as_deref(), Some("Austin ISD"));
        assert_eq!(office.school_district.as_deref(), Some("Austin ISD"));
    }

    #[test]
    fn missing_source_columns_are_selected_as_null() {
        let present = ["county", "office_name", "num_elect"]
            .into_iter()
            .map(str::to_string)
            .collect();
        let select_list = super::source_filing_select_list(&present);
        assert!(select_list.contains("county"));
        assert!(select_list.contains("NULL::text AS district_type"));
        assert!(select_list.contains("NULL::text AS incumbent"));
        assert!(select_list.contains("NULL::boolean AS is_special_election"));
        assert!(select_list.contains("\n            num_elect"));
    }

    #[test]
    fn filing_counties_separated_by_semicolons_are_stored_with_commas() {
        assert_eq!(
            normalize_filing_counties("Harris; Fort Bend").as_deref(),
            Some("Harris, Fort Bend")
        );
        assert_eq!(
            normalize_filing_counties("  Collin  ").as_deref(),
            Some("Collin")
        );
        assert_eq!(normalize_filing_counties(" ; "), None);
    }
}
