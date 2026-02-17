//! Texas candidate filings processor.
//!
//! Reads from existing table `p6t_state_tx.tx_primaries_20260209`.
//! Creates staging tables (ingest_staging.stg_tx_offices, stg_tx_politicians, politician_process_dupes, stg_tx_addresses, stg_tx_races, stg_tx_race_candidates, stg_tx_race_candidates_process_dupes)
//! and processes each row into staging using TX office/race extractors and generators.

use std::error::Error;
use std::str::FromStr;
use sqlx::FromRow;
use sqlx::PgPool;
use db::{Office, Politician, Race, State, RaceType, VoteType};
use uuid::Uuid;
use serde_json::Value as JSON;
use slugify::slugify;
use crate::extractors;
use crate::generators;

/// Staging address built from a filing; inserted into stg_tx_addresses only when the politician is inserted.
#[derive(Debug, Clone)]
pub struct TxStagingAddress {
    pub line_1: String,
    pub city: String,
    pub state: String,
    pub country: String,
}

/// One row from the Texas primaries table. Field mapping from
/// `p6t_state_tx.tx_primaries_20260209` is TBD; update the SELECT
/// in `process_tx_candidate_filings` when column names are known.
#[derive(Debug, FromRow)]
pub struct TxCandidateFiling {
    pub office_title: Option<String>,
    pub office_type: Option<String>,
    pub candidate_name: Option<String>,
    pub county: Option<String>,
    pub email: Option<String>,
    pub party: Option<String>,
    pub address_street: Option<String>,
    pub address_city: Option<String>,
    pub address_state: Option<String>,
    pub occupation: Option<String>,
    pub incumbent: Option<String>,
    pub status: Option<String>,
}

/// Source table for TX primaries. Use quoted name if the table has hyphens:
/// r#"p6t_state_tx."tx-primaries-2026-02-09""#. Otherwise: p6t_state_tx.tx_primaries_20260209
const TX_SOURCE_TABLE: &str = "p6t_state_tx.tx_primaries_20260212_all";
const ELECTION_YEAR: i32 = 2026;

/// Process Texas candidate filings from the existing DB table into staging tables.
/// Run after the TX table is populated. Merge to production via mn_merge_staging_to_production
/// (or a TX-specific merge binary) after this.
pub async fn process_tx_candidate_filings(
    pool: &PgPool,
    race_type: &str,
) -> Result<(), Box<dyn Error>> {
    println!("Starting TX candidate filings processor...");
    println!("Source table: {}", TX_SOURCE_TABLE);
    println!("Race type: {}", race_type);

    create_staging_tables(pool).await?;

    // TODO: Replace NULL::text placeholders with actual column names from
    // p6t_state_tx.tx_primaries_20260209 once column mapping is provided.
    let query = format!(
        r#"
        SELECT
            office_title AS office_title,
            office_type AS office_type,
            candidate_name AS candidate_name,
            county AS county,
            email AS email,
            party AS party,
            street_address AS address_street,
            city AS address_city,
            state AS address_state,
            occupation AS occupation,
            incumbent AS incumbent,
            status AS status
        FROM {}
        "#,
        TX_SOURCE_TABLE
    );

    println!("Fetching candidate filings from TX table...");
    let filings: Vec<TxCandidateFiling> = sqlx::query_as::<_, TxCandidateFiling>(&query)
        .fetch_all(pool)
        .await?;

    println!("Found {} candidate filings to process", filings.len());

    let mut processed_count = 0usize;
    let mut error_count = 0usize;

    for (index, filing) in filings.iter().enumerate() {
        if index % 100 == 0 {
            println!("Processing filing {}/{}...", index + 1, filings.len());
        }
        match process_and_insert_tx_filing(pool, filing, race_type).await {
            Ok(_) => processed_count += 1,
            Err(e) => {
                error_count += 1;
                eprintln!(
                    "Error processing filing: {} - {}",
                    filing
                        .candidate_name
                        .as_deref()
                        .unwrap_or("Unknown"),
                    e
                );
            }
        }
    }

    println!("\n=== Processing Complete ===");
    println!("Successfully processed: {}", processed_count);
    println!("Errors: {}", error_count);
    println!("\nStaging tables:");
    println!("  - ingest_staging.stg_tx_offices");
    println!("  - ingest_staging.stg_tx_politicians");
    println!("  - ingest_staging.politician_process_dupes");
    println!("  - ingest_staging.stg_tx_races");
    println!("  - ingest_staging.stg_tx_race_candidates");
    println!("  - ingest_staging.stg_tx_race_candidates_process_dupes");
    println!("  - ingest_staging.stg_tx_addresses");

    Ok(())
}

async fn create_staging_tables(pool: &PgPool) -> Result<(), Box<dyn Error>> {
    println!("Creating staging tables in ingest_staging schema...");

    sqlx::query("CREATE SCHEMA IF NOT EXISTS ingest_staging")
        .execute(pool)
        .await?;

    sqlx::query("DROP TABLE IF EXISTS ingest_staging.stg_tx_race_candidates_process_dupes CASCADE")
        .execute(pool)
        .await?;
    sqlx::query("DROP TABLE IF EXISTS ingest_staging.stg_tx_race_candidates CASCADE")
        .execute(pool)
        .await?;
    sqlx::query("DROP TABLE IF EXISTS ingest_staging.stg_tx_races CASCADE")
        .execute(pool)
        .await?;
    sqlx::query("DROP TABLE IF EXISTS ingest_staging.stg_tx_addresses CASCADE")
        .execute(pool)
        .await?;
    sqlx::query("DROP TABLE IF EXISTS ingest_staging.politician_process_dupes CASCADE")
        .execute(pool)
        .await?;
    sqlx::query("DROP TABLE IF EXISTS ingest_staging.stg_tx_politicians CASCADE")
        .execute(pool)
        .await?;
    sqlx::query("DROP TABLE IF EXISTS ingest_staging.stg_tx_offices CASCADE")
        .execute(pool)
        .await?;

    sqlx::query(r#"
        CREATE TABLE ingest_staging.stg_tx_offices (
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
            seat TEXT,
            school_district TEXT,
            hospital_district TEXT,
            priority INTEGER,
            created_at TIMESTAMPTZ NOT NULL,
            updated_at TIMESTAMPTZ NOT NULL
        )
    "#)
    .execute(pool)
    .await?;

    sqlx::query(r#"
        CREATE TABLE ingest_staging.stg_tx_politicians (
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
    "#)
    .execute(pool)
    .await?;

    sqlx::query(r#"
        CREATE TABLE ingest_staging.stg_tx_addresses (
            id UUID PRIMARY KEY,
            line_1 TEXT NOT NULL,
            city TEXT NOT NULL,
            state TEXT NOT NULL,
            country TEXT NOT NULL,
            politician_id UUID NOT NULL
        )
    "#)
    .execute(pool)
    .await?;

    sqlx::query(r#"
        CREATE TABLE ingest_staging.politician_process_dupes (
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
    "#)
    .execute(pool)
    .await?;

    sqlx::query(r#"
        CREATE TABLE ingest_staging.stg_tx_races (
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
    "#)
    .execute(pool)
    .await?;

    sqlx::query(r#"
        CREATE TABLE ingest_staging.stg_tx_race_candidates (
            race_id UUID NOT NULL,
            candidate_id UUID NOT NULL,
            ref_key TEXT,
            PRIMARY KEY (race_id, candidate_id)
        )
    "#)
    .execute(pool)
    .await?;

    sqlx::query(r#"
        CREATE TABLE ingest_staging.stg_tx_race_candidates_process_dupes (
            ref_key TEXT NOT NULL,
            existing_race_id UUID NOT NULL,
            existing_candidate_id UUID NOT NULL,
            incoming_race_id UUID NOT NULL,
            incoming_candidate_id UUID NOT NULL,
            created_at TIMESTAMPTZ NOT NULL DEFAULT (now() AT TIME ZONE 'utc')
        )
    "#)
    .execute(pool)
    .await?;

    println!("Staging tables created successfully!");
    Ok(())
}

/// Process one TX filing and insert into staging tables.
/// Only processes when filing.status is "in primary" (case-insensitive); otherwise skips and returns Ok.
async fn process_and_insert_tx_filing(
    pool: &PgPool,
    filing: &TxCandidateFiling,
    race_type: &str,
) -> Result<(), Box<dyn Error>> {
    let status_trimmed = filing.status.as_deref().map(|s| s.trim());
    if !status_trimmed.is_some_and(|s| s.eq_ignore_ascii_case("in primary")) {
        return Ok(());
    }

    let office = process_tx_office(filing)?;
    let office_id = get_staging_office_id_by_slug(pool, &office.slug).await?;
    if office_id.is_none() {
        let state_id = filing
            .office_title
            .as_ref()
            .map(|t| generators::politician::PoliticianRefKeyGenerator::new("tx-sos", t).generate());
        insert_staging_office(pool, &office, state_id.as_ref()).await?;
    }
    let resolved_office_id = office_id.unwrap_or(office.id);

    let mut politician = process_tx_politician(pool, filing, resolved_office_id).await?;
    let race = process_tx_race(filing, &office, office_id, race_type)?;
    let address = process_tx_address(filing);

    let politician_inserted = insert_staging_politician(pool, &mut politician, address).await?;
    
    // If the politician was not inserted, exit and don't insert the race or race_candidate
    if !politician_inserted {
        let name = politician.full_name.as_deref().unwrap_or("(no name)");
        eprintln!("Politician not inserted (slug conflict, emails equal); skipping race/race_candidate: {}", name);
        return Ok(());
    }

    insert_staging_race(pool, &race).await?;

    let race_id = get_staging_race_id_by_slug(pool, &race.slug).await?.unwrap_or(race.id);
    let race_candidate_ref_key = process_tx_race_candidate_ref_key(filing);
    insert_staging_race_candidate(pool, race_id, &politician, &race_candidate_ref_key).await?;

    Ok(())
}

/// Strips the suffix " - unexpired term" (case-insensitive) from a raw office title.
fn strip_unexpired_term(raw: &str) -> String {
    const SUFFIX: &str = " - unexpired term";
    let trimmed = raw.trim_end_matches(|c: char| c.is_ascii_whitespace());
    if trimmed.len() >= SUFFIX.len()
        && trimmed[trimmed.len() - SUFFIX.len()..].eq_ignore_ascii_case(SUFFIX)
    {
        trimmed[..trimmed.len() - SUFFIX.len()].trim().to_string()
    } else {
        trimmed.to_string()
    }
}

fn process_tx_office(filing: &TxCandidateFiling) -> Result<Office, Box<dyn Error>> {
    use crate::extractors::tx::office;

    let raw_filing_title = filing.office_title.as_ref().ok_or("Missing office title")?;
    let raw_filing_title_cleaned = strip_unexpired_term(raw_filing_title);
    
    // Extract the office name, and if it fails record the error and skip this filing
    let name = office::extract_office_name(&raw_filing_title_cleaned, filing.party.as_deref())
        .ok_or("Failed to extract office name")?;
    
    let (extracted_county, office_title_stripped) =
        office::extract_tx_county_from_office_title(&raw_filing_title_cleaned, Some(&name));
    let county = filing
        .county
        .clone()
        .filter(|c| !c.trim().is_empty())
        .map(|c| extractors::politician::title_case(&c))
        .or(extracted_county);
    let office_title = office_title_stripped.as_str();

    let title = office::extract_office_title(&name).unwrap_or_default();
    let chamber = office::extract_office_chamber(&name);

    // Extract and strip the seat from the office title (before scope, so scope can use seat for some offices)
    let (seat, office_title_no_seat) = office::extract_office_seat(office_title);
    // Extract the district from the office title (so district can also be used for some scopes)
    let district = office::extract_office_district(&office_title_no_seat);
    let (political_scope, election_scope, district_type) = office::extract_office_scope(
        &name,
        county.as_deref(),
        seat.as_deref(),
        district.as_deref(),
    )
    .ok_or("Failed to extract office scope")?;

    let slug = generators::tx::office::OfficeSlugGenerator {
        state: &State::TX,
        name: &name,
        county: county.as_deref(),
        district: district.as_deref(),
        seat: seat.as_deref(),
        school_district: None,
        hospital_district: None,
        municipality: None,
        election_scope: Some(&election_scope),
        district_type: district_type.as_ref(),
    }
    .generate();

    let (subtitle, subtitle_short) = generators::tx::office::OfficeSubtitleGenerator {
        state: &State::TX,
        office_name: Some(&name),
        election_scope: &election_scope,
        district_type: district_type.as_ref(),
        county: county.as_deref(),
        district: district.as_deref(),
        seat: seat.as_deref(),
        school_district: None,
        hospital_district: None,
        municipality: None,
    }
    .generate();

    let priority = generators::tx::office::office_priority(&title, county.as_deref());

    Ok(Office {
        id: Uuid::new_v4(),
        slug,
        name: Some(name),
        title,
        subtitle: Some(subtitle),
        subtitle_short: Some(subtitle_short),
        office_type: None,
        chamber,
        district_type,
        political_scope,
        election_scope,
        state: Some(State::TX),
        county,
        municipality: None,
        term_length: None,
        district,
        seat,
        school_district: None,
        hospital_district: None,
        priority,
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    })
}

async fn process_tx_politician(
    pool: &PgPool,
    filing: &TxCandidateFiling,
    current_office_id: Uuid,
) -> Result<Politician, Box<dyn Error>> {
    let candidate_name_raw = filing.candidate_name.as_ref().ok_or("Missing candidate name")?;
    let candidate_name = extractors::politician::normalize_name(candidate_name_raw);
    let ref_key_input = format!(
        "{} {} {} {}",
        ELECTION_YEAR,
        filing.office_title.as_deref().unwrap_or(""),        
        candidate_name,
        filing.status.as_deref().unwrap_or(""),
    )
    .trim()
    .to_string();
    let ref_key = generators::politician::PoliticianRefKeyGenerator::new("TX-SOS", &ref_key_input).generate();

    // Resolve party_id from production party table
    // If no party or empty, use "UN" (unaffiliated)
    let fec_code = filing
        .party
        .as_ref()
        .map(|a| {
            if a.trim().is_empty() {
                "UN".to_string()
            } else {
                extractors::party::extract_party_fec_code(a).unwrap_or_else(|| "UN".to_string())
            }
        })
        .unwrap_or_else(|| "UN".to_string());

    let party_id = sqlx::query_scalar!(r#"SELECT id FROM party WHERE fec_code = $1"#, fec_code)
        .fetch_optional(pool)
        .await?;

    // Parse name using extractor (with fallback to simple split) — use normalized name, no title case yet
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

    // Apply title case after extraction
    let title = extractors::politician::title_case;
    let first_name = title(&name_parts.first);
    let middle_name = name_parts.middle.as_deref().map(|s| title(s));
    let last_name = name_parts.last.as_deref().map(|s| title(s)).unwrap_or_default();
    let suffix = name_parts.suffix.as_deref().map(|s| title(s));
    let preferred_name = name_parts.preferred.as_deref().map(|s| title(s));
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
        campaign_website_url: None,
        facebook_url: None,
        twitter_url: None,
        instagram_url: None,
        youtube_url: None,
        linkedin_url: None,
        tiktok_url: None,
        email: filing.email.as_ref().map(|e| e.to_lowercase()),
        phone: None,
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

fn process_tx_race(
    filing: &TxCandidateFiling,
    office: &Office,
    office_id: Option<Uuid>,
    race_type: &str,
) -> Result<Race, Box<dyn Error>> {
    let election_id = Uuid::parse_str("0d586931-c119-4fe7-814f-f679e91282a8").unwrap_or_else(|_| Uuid::nil());

    let is_special_election = filing
        .office_title
        .as_ref()
        .map(|t| extractors::tx::race::extract_is_special_election(t))
        .unwrap_or(false);
    let num_elect = filing.office_title.as_ref().and_then(|t| extractors::tx::race::extract_num_elect(t));

    let party_fec = filing
        .party
        .as_deref()
        .and_then(extractors::party::extract_party_fec_code);

    let (title, slug) = generators::tx::race::RaceTitleGenerator::from_source(
        &RaceType::from_str(race_type)?,
        office,
        is_special_election,
        party_fec.as_deref(),
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
        race_type: RaceType::from_str(race_type)?,
        vote_type: VoteType::Plurality,
        party_id: party_fec,
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

fn process_tx_race_candidate_ref_key(filing: &TxCandidateFiling) -> String {
    let office_title = filing.office_title.as_deref().unwrap_or("");
    let candidate_name = filing.candidate_name.as_deref().unwrap_or("");
    slugify!(&format!("tx-primaries-{}-{}-{}", ELECTION_YEAR, office_title, candidate_name))
}

/// Build an address object from the filing (street → line_1, city → city, state → state, country = USA).
/// Returns None if the filing has no usable address (both line_1 and city empty).
/// Trim and collapse multiple whitespace to a single space.
fn normalize_address_string(s: &str) -> String {
    s.trim().split_whitespace().collect::<Vec<_>>().join(" ")
}

fn process_tx_address(filing: &TxCandidateFiling) -> Option<TxStagingAddress> {
    let line_1 = filing
        .address_street
        .as_deref()
        .map(|s| normalize_address_string(s))
        .filter(|s| !s.is_empty())
        .unwrap_or_default();
    let city = filing
        .address_city
        .as_deref()
        .map(|s| normalize_address_string(s))
        .filter(|s| !s.is_empty())
        .unwrap_or_default();
    let state = filing
        .address_state
        .as_deref()
        .map(|s| normalize_address_string(s))
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| "TX".to_string());
    let country = "USA".to_string();

    if line_1.is_empty() && city.is_empty() {
        return None;
    }

    Some(TxStagingAddress {
        line_1,
        city,
        state,
        country,
    })
}

async fn get_staging_office_id_by_slug(pool: &PgPool, slug: &str) -> Result<Option<Uuid>, Box<dyn Error>> {
    let row: Option<(Uuid,)> =
        sqlx::query_as("SELECT id FROM ingest_staging.stg_tx_offices WHERE slug = $1")
            .bind(slug)
            .fetch_optional(pool)
            .await?;
    Ok(row.map(|(id,)| id))
}

async fn get_staging_race_id_by_slug(pool: &PgPool, slug: &str) -> Result<Option<Uuid>, Box<dyn Error>> {
    let row: Option<(Uuid,)> =
        sqlx::query_as("SELECT id FROM ingest_staging.stg_tx_races WHERE slug = $1")
            .bind(slug)
            .fetch_optional(pool)
            .await?;
    Ok(row.map(|(id,)| id))
}

async fn insert_staging_office(
    pool: &PgPool,
    office: &Office,
    state_id: Option<&String>,
) -> Result<(), Box<dyn Error>> {
    sqlx::query(
        r#"
        INSERT INTO ingest_staging.stg_tx_offices (
            id, slug, name, title, subtitle, subtitle_short, office_type, chamber,
            district_type, political_scope, election_scope, state, state_id, county, municipality,
            term_length, district, seat, school_district, hospital_district, priority,
            created_at, updated_at
        ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18, $19, $20, $21, $22, $23)
        ON CONFLICT (slug) DO NOTHING
        "#,
    )
    .bind(office.id)
    .bind(&office.slug)
    .bind(&office.name)
    .bind(&office.title)
    .bind(&office.subtitle)
    .bind(&office.subtitle_short)
    .bind(office.office_type.as_ref().map(|o| o.as_str()))
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

/// Execute the staging politician INSERT; returns rows affected (0 on slug conflict).
async fn execute_staging_politician_insert(
    pool: &PgPool,
    politician: &Politician,
) -> Result<u64, sqlx::Error> {
    let result = sqlx::query(
        r#"
        INSERT INTO ingest_staging.stg_tx_politicians (
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
    .bind(politician.home_state.as_ref().map(|s| s.as_ref().to_string()))
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
    .bind(&politician.votesmart_candidate_id)
    .bind(&politician.votesmart_candidate_bio)
    .bind(&politician.votesmart_candidate_ratings)
    .bind(politician.legiscan_people_id)
    .bind(&politician.crp_candidate_id)
    .bind(&politician.fec_candidate_id)
    .bind(politician.race_wins)
    .bind(politician.race_losses)
    .bind(politician.residence_address_id)
    .bind(false) // treat_exact_slug_as_same_person
    .bind(politician.created_at)
    .bind(politician.updated_at)
    .execute(pool)
    .await?;
    Ok(result.rows_affected())
}

/// Tests to see if existing and incoming are the same person by comparing email and address.
/// Returns booleans (same_person, emails_equal_not_both_empty) 
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

/// Record a politician duplicate: incoming staging row matched or collided with an existing one.
async fn record_politician_dupe(
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
        INSERT INTO ingest_staging.politician_process_dupes
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

/// Insert a staging address and optionally set the politician row's residence_address_id.
/// Returns the new stg_tx_addresses row id. Set `update_politician_row` to false when the
/// politician row does not exist yet (e.g. we're about to insert it with residence_address_id already set).
async fn insert_staging_address_for_politician(
    pool: &PgPool,
    addr: &TxStagingAddress,
    politician_id: Uuid,
    update_politician_row: bool,
) -> Result<Uuid, Box<dyn Error>> {
    let stg_address_id = Uuid::new_v4();
    sqlx::query(
        r#"
        INSERT INTO ingest_staging.stg_tx_addresses (id, line_1, city, state, country, politician_id)
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
            r#"UPDATE ingest_staging.stg_tx_politicians SET residence_address_id = $1 WHERE id = $2"#,
        )
        .bind(stg_address_id)
        .bind(politician_id)
        .execute(pool)
        .await?;
    }
    Ok(stg_address_id)
}

/// Apply updates (email/address) for existing row when we determined same person, then record dupes and return.
async fn apply_same_person_updates_and_dupes(
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
    if emails_equal_not_both_empty {
        // Saves address if the politician doesn't have one yet
        if existing_residence_address_id.is_none() {
            if let Some(addr) = incoming_address {
                insert_staging_address_for_politician(pool, addr, existing_id, true).await?;
            }
        }
    }
    // Always overwrite existing email with incoming when incoming is not empty (incoming wins)
    if let Some(incoming_trimmed) = politician
        .email
        .as_deref()
        .map(|e| e.trim())
        .filter(|e| !e.is_empty())
    {
    sqlx::query(
        r#"UPDATE ingest_staging.stg_tx_politicians SET email = $1 WHERE id = $2"#,
        )
        .bind(incoming_trimmed)
        .bind(existing_id)
        .execute(pool)
        .await?;
    }
    record_politician_dupe(
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

/// Returns true if the politician was inserted (either on first try or after slug increment), false if not (conflict and same person).
/// If address is Some and the politician is inserted, inserts into stg_tx_addresses and sets politician.residence_address_id to that staging address id (overwritten during merge).
async fn insert_staging_politician(
    pool: &PgPool,
    politician: &mut Politician,
    address: Option<TxStagingAddress>,
) -> Result<bool, Box<dyn Error>> {

    // Tries to insert the politician into the staging table; returns rows affected (0 on slug conflict).
    let mut rows_affected = execute_staging_politician_insert(pool, politician).await?;

    // Handle Address: if the politician was inserted, insert the address and link it
    if rows_affected > 0 {
        if let Some(addr) = &address {
            let stg_address_id =
                insert_staging_address_for_politician(pool, addr, politician.id, true).await?;
            politician.residence_address_id = Some(stg_address_id);
        }
        return Ok(true);
    }

    // Slug conflict: fetch existing row (including residence_address_id) and compare emails + addresses
    let existing: (uuid::Uuid, String, Option<String>, Option<String>, Option<uuid::Uuid>) = sqlx::query_as(
        r#"
        SELECT id, slug, email, ref_key, residence_address_id
        FROM ingest_staging.stg_tx_politicians
        WHERE slug = $1
        "#,
    )
    .bind(&politician.slug)
    .fetch_optional(pool)
    .await?
    .ok_or_else(|| "Existing row not found for slug conflict".to_string())?;

    let (existing_id, existing_slug, existing_email, existing_ref_key, existing_residence_address_id) = existing;
    let existing_email_trimmed = existing_email.as_deref().map(|e| e.trim()).filter(|e| !e.is_empty());
    let incoming_email_trimmed = politician.email.as_deref().map(|e| e.trim()).filter(|e| !e.is_empty());

    let existing_address = fetch_staging_address(pool, existing_residence_address_id).await?;
    let incoming_address = address.as_ref();
    
    // Check if it's the same person by comparing emails and addresses.
    let (same_person, emails_equal_not_both_empty) = is_same_person_from_emails_and_addresses(
        existing_email_trimmed,
        existing_address.as_ref(),
        incoming_email_trimmed,
        incoming_address,
    );

    if same_person {
        apply_same_person_updates_and_dupes(
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

    // Not same person: find next free slug. For each candidate slug that already exists, run email+address conflict test again.
    let base_slug = &politician.slug;
    let mut final_incoming_slug = politician.slug.clone();
    let mut n = 1u32;
    loop {
        let candidate = format!("{}-{}", base_slug, n);
        let existing: Option<(uuid::Uuid, String, Option<String>, Option<String>, Option<uuid::Uuid>)> = sqlx::query_as(
            r#"
            SELECT id, slug, email, ref_key, residence_address_id
            FROM ingest_staging.stg_tx_politicians
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
                let e_email_trimmed = eemail.as_deref().map(|e| e.trim()).filter(|e| !e.is_empty());
                let e_address = fetch_staging_address(pool, eaddr_id).await?;
                let (same, emails_eq) = is_same_person_from_emails_and_addresses(
                    e_email_trimmed,
                    e_address.as_ref(),
                    incoming_email_trimmed,
                    incoming_address,
                );
                if same {
                    apply_same_person_updates_and_dupes(
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
    
    // Saves address if address exists
    if let Some(addr) = &address {
        let stg_address_id =
            insert_staging_address_for_politician(pool, addr, politician.id, false).await?;
        incoming_with_new_slug.residence_address_id = Some(stg_address_id);
        politician.residence_address_id = Some(stg_address_id);
    }
    incoming_with_new_slug.slug = final_incoming_slug.clone();
    rows_affected = execute_staging_politician_insert(pool, &incoming_with_new_slug).await?;

    let incoming_inserted = rows_affected > 0;
    record_politician_dupe(
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

async fn fetch_staging_address(
    pool: &PgPool,
    residence_address_id: Option<uuid::Uuid>,
) -> Result<Option<TxStagingAddress>, Box<dyn Error>> {
    let addr_id = match residence_address_id {
        Some(id) => id,
        None => return Ok(None),
    };
    let row: Option<(String, String, String, String)> = sqlx::query_as(
        r#"SELECT line_1, city, state, country FROM ingest_staging.stg_tx_addresses WHERE id = $1"#,
    )
    .bind(addr_id)
    .fetch_optional(pool)
    .await?;
    Ok(row.map(|(line_1, city, state, country)| TxStagingAddress { line_1, city, state, country }))
}

async fn insert_staging_race(pool: &PgPool, race: &Race) -> Result<(), Box<dyn Error>> {
    sqlx::query(
        r#"
        INSERT INTO ingest_staging.stg_tx_races (
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

async fn insert_staging_race_candidate(
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
        FROM ingest_staging.stg_tx_race_candidates
        WHERE ref_key = $1
        "#,
    )
    .bind(ref_key)
    .fetch_optional(pool)
    .await?;

    if let Some(row) = existing {
        sqlx::query(
            r#"
            INSERT INTO ingest_staging.stg_tx_race_candidates_process_dupes
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
        INSERT INTO ingest_staging.stg_tx_race_candidates (race_id, candidate_id, ref_key)
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
