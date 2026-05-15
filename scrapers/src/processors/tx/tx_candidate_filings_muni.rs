//! Texas **municipal** candidate filings processor.
//!
//! Reads `p6t_state_tx.tx_2026_municipal_filings` into `ingest_staging.stg_tx_muni_*` tables.
//! Merge to production is **not** wired to `tx_merge_filings` yet (that binary reads `stg_tx_*`).

use crate::extractors;
use crate::generators;
use db::{Office, Politician, Race, RaceType, State, VoteType};
use serde_json::Value as JSON;
use sqlx::FromRow;
use sqlx::PgPool;
use std::error::Error;
use uuid::Uuid;

/// Staging address built from a filing; inserted into `stg_tx_muni_addresses` only when the politician is inserted.
#[derive(Debug, Clone)]
pub struct TxStagingAddress {
    pub line_1: String,
    pub city: String,
    pub state: String,
    pub country: String,
}

/// One row from `p6t_state_tx.tx_2026_municipal_filings`.
#[derive(Debug, FromRow)]
pub struct TxMunicipalFiling {
    pub county: Option<String>,
    pub municipality: Option<String>,
    pub office_name: Option<String>,
    pub district_type: Option<String>,
    pub district: Option<String>,
    pub school_district: Option<String>,
    pub seat_label: Option<String>,
    pub seat: Option<String>,
    pub candidate_name: Option<String>,
    pub incumbent: Option<String>,
    pub is_special_election: Option<bool>,
    pub status: Option<String>,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub campaign_website: Option<String>,
    pub place_fips: Option<String>,
    pub state: Option<String>,
}

const TX_MUNI_SOURCE_TABLE: &str = "p6t_state_tx.tx_2026_municipal_filings";
const ELECTION_YEAR: i32 = 2026;

/// 2026 TX municipal election (replace if your ingest targets a different election).
pub const TX_MUNI_ELECTION_ID: &str = "dce8e1a4-dcdb-4f3e-9cfa-11f96ae86007";

/// Process municipal filings into `stg_tx_muni_*` staging tables.
pub async fn process_tx_municipal_filings(pool: &PgPool) -> Result<(), Box<dyn Error>> {
    println!("Starting TX municipal candidate filings processor...");
    println!("Source table: {}", TX_MUNI_SOURCE_TABLE);

    create_muni_staging_tables(pool).await?;

    let query = format!(
        r#"
        SELECT
            county,
            municipality,
            office_name,
            district_type,
            district,
            school_district,
            seat_label,
            seat,
            candidate_name,
            incumbent,
            is_special_election,
            status,
            email,
            phone,
            campaign_website,
            place_fips,
            state
        FROM {}
        "#,
        TX_MUNI_SOURCE_TABLE
    );

    println!("Fetching municipal filings...");
    let filings: Vec<TxMunicipalFiling> = sqlx::query_as::<_, TxMunicipalFiling>(&query)
        .fetch_all(pool)
        .await?;

    println!("Found {} municipal filings to process", filings.len());

    let mut processed_count = 0usize;
    let mut skipped_status_count = 0usize;
    let mut error_count = 0usize;

    for (index, filing) in filings.iter().enumerate() {
        if index % 100 == 0 {
            println!("Processing filing {}/{}...", index + 1, filings.len());
        }
        match process_and_insert_tx_muni_filing(pool, filing).await {
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

    println!("\n=== TX municipal processing complete ===");
    println!("Successfully processed: {}", processed_count);
    println!("Skipped (status not Active/Unopposed): {}", skipped_status_count);
    println!("Errors: {}", error_count);
    println!("\nStaging tables (merge not in tx_merge_filings yet):");
    println!("  - ingest_staging.stg_tx_muni_offices");
    println!("  - ingest_staging.stg_tx_muni_politicians");
    println!("  - ingest_staging.stg_tx_muni_politician_process_dupes");
    println!("  - ingest_staging.stg_tx_muni_addresses");
    println!("  - ingest_staging.stg_tx_muni_races");
    println!("  - ingest_staging.stg_tx_muni_race_candidates");
    println!("  - ingest_staging.stg_tx_muni_race_candidates_process_dupes");

    Ok(())
}

async fn create_muni_staging_tables(pool: &PgPool) -> Result<(), Box<dyn Error>> {
    println!("Creating ingest_staging.stg_tx_muni_* tables...");

    sqlx::query("CREATE SCHEMA IF NOT EXISTS ingest_staging")
        .execute(pool)
        .await?;

    sqlx::query(
        "DROP TABLE IF EXISTS ingest_staging.stg_tx_muni_race_candidates_process_dupes CASCADE",
    )
    .execute(pool)
    .await?;
    sqlx::query("DROP TABLE IF EXISTS ingest_staging.stg_tx_muni_race_candidates CASCADE")
        .execute(pool)
        .await?;
    sqlx::query("DROP TABLE IF EXISTS ingest_staging.stg_tx_muni_races CASCADE")
        .execute(pool)
        .await?;
    sqlx::query("DROP TABLE IF EXISTS ingest_staging.stg_tx_muni_addresses CASCADE")
        .execute(pool)
        .await?;
    sqlx::query("DROP TABLE IF EXISTS ingest_staging.stg_tx_muni_politician_process_dupes CASCADE")
        .execute(pool)
        .await?;
    sqlx::query("DROP TABLE IF EXISTS ingest_staging.stg_tx_muni_politicians CASCADE")
        .execute(pool)
        .await?;
    sqlx::query("DROP TABLE IF EXISTS ingest_staging.stg_tx_muni_offices CASCADE")
        .execute(pool)
        .await?;

    sqlx::query(
        r#"
        CREATE TABLE ingest_staging.stg_tx_muni_offices (
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
    "#,
    )
    .execute(pool)
    .await?;

    sqlx::query(
        r#"
        CREATE TABLE ingest_staging.stg_tx_muni_politicians (
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
        CREATE TABLE ingest_staging.stg_tx_muni_addresses (
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
        CREATE TABLE ingest_staging.stg_tx_muni_politician_process_dupes (
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
        CREATE TABLE ingest_staging.stg_tx_muni_races (
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
        CREATE TABLE ingest_staging.stg_tx_muni_race_candidates (
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
        CREATE TABLE ingest_staging.stg_tx_muni_race_candidates_process_dupes (
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

    println!("Municipal staging tables created.");
    Ok(())
}

/// Ingest only rows whose `status` is **Active** or **Unopposed** (trimmed; case-insensitive).
fn should_ingest_municipal_filing_status(status: Option<&str>) -> bool {
    status
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .is_some_and(|s| {
            s.eq_ignore_ascii_case("Active") || s.eq_ignore_ascii_case("Unopposed")
        })
}

/// Returns `Ok(false)` if the row is skipped because `status` is not Active or Unopposed.
async fn process_and_insert_tx_muni_filing(
    pool: &PgPool,
    filing: &TxMunicipalFiling,
) -> Result<bool, Box<dyn Error>> {
    if !should_ingest_municipal_filing_status(filing.status.as_deref()) {
        return Ok(false);
    }

    let office = process_tx_muni_office(filing)?;
    let office_id = get_staging_muni_office_id_by_slug(pool, &office.slug).await?;
    if office_id.is_none() {
        // TODO(custom): derive `state_id` for municipal offices if needed (primary uses SOS office title).
        let state_id: Option<String> = None;
        insert_staging_muni_office(pool, &office, state_id.as_ref()).await?;
    }
    let resolved_office_id = office_id.unwrap_or(office.id);

    let mut politician = process_tx_muni_politician(pool, filing, resolved_office_id).await?;

    let race = process_tx_muni_race(filing, &office, office_id, None)?;
    let address: Option<TxStagingAddress> = None;
    // TODO(custom): derive address from `place_fips` or other columns if desired.

    let politician_inserted = insert_staging_muni_politician(pool, &mut politician, address).await?;

    if !politician_inserted {
        let name = politician.full_name.as_deref().unwrap_or("(no name)");
        eprintln!(
            "Politician not inserted (slug conflict, emails equal); skipping race/race_candidate: {}",
            name
        );
        return Ok(true);
    }

    insert_staging_muni_race(pool, &race).await?;

    let race_id = get_staging_muni_race_id_by_slug(pool, &race.slug)
        .await?
        .unwrap_or(race.id);
    let office_identity = municipal_office_identity_for_ref_key(filing);
    let candidate_name = filing.candidate_name.as_deref().unwrap_or("");
    let race_candidate_ref_key = generators::politician::PoliticianRefKeyGenerator::new(
        "tx-municipal",
        ELECTION_YEAR,
        &office_identity,
        Some(candidate_name),
    )
    .generate();
    insert_staging_muni_race_candidate(pool, race_id, &politician, &race_candidate_ref_key).await?;

    Ok(true)
}

/// Maps municipal filing columns to [`Office`] using the same TX office extractors as primary
/// filings, with raw title from [`TxMunicipalFiling::office_name`]. There is no party on municipal
/// rows, so [`extract_office_name`](crate::extractors::tx::tx_office::extract_office_name) is
/// called with `party: None`. County is taken only from [`TxMunicipalFiling::county`], not parsed
/// from the office title. Seat and district come from [`TxMunicipalFiling::seat`] and
/// [`TxMunicipalFiling::district`]. Chamber is left unset (not applicable to municipal offices).
fn process_tx_muni_office(filing: &TxMunicipalFiling) -> Result<Office, Box<dyn Error>> {
    use crate::extractors::tx::tx_office as office;

    let raw_filing_title = filing.office_name.as_ref().ok_or("Missing office name")?;

    let name = office::extract_office_name(raw_filing_title, None)
        .ok_or("Failed to extract office name")?;

    let county = filing
        .county
        .clone()
        .filter(|c| !c.trim().is_empty());

    let seat = filing
        .seat
        .clone()
        .filter(|s| !s.trim().is_empty());

    let district = filing
        .district
        .clone()
        .filter(|d| !d.trim().is_empty());

    let title = office::extract_office_title(&name).unwrap_or_default();

    let (political_scope, election_scope, district_type) = office::extract_office_scope(
        &name,
        county.as_deref(),
        seat.as_deref(),
        district.as_deref(),
    )
    .ok_or("Failed to extract office scope")?;

    let municipality = filing
        .municipality
        .clone()
        .filter(|m| !m.trim().is_empty())
        .map(|m| extractors::politician::title_case(&m));

    let school_district = filing
        .school_district
        .clone()
        .filter(|s| !s.trim().is_empty())
        .map(|s| extractors::politician::title_case(&s));

    let slug = generators::tx::tx_office::OfficeSlugGenerator {
        state: &State::TX,
        name: &name,
        county: county.as_deref(),
        district: district.as_deref(),
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
        seat: seat.as_deref(),
        school_district: school_district.as_deref(),
        hospital_district: None,
        municipality: municipality.as_deref(),
    }
    .generate();

    let priority = generators::tx::tx_office::office_priority(&title, county.as_deref(), district.as_deref());

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
        seat,
        school_district,
        hospital_district: None,
        priority,
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    })
}

fn process_tx_muni_race(
    filing: &TxMunicipalFiling,
    office: &Office,
    office_id: Option<Uuid>,
    party_id: Option<Uuid>,
) -> Result<Race, Box<dyn Error>> {
    let election_id = Uuid::parse_str(TX_MUNI_ELECTION_ID).map_err(|e| e.to_string())?;

    let is_special_election = filing.is_special_election.unwrap_or(false);

    // TODO(custom): derive `num_elect` from municipal data when available.
    let num_elect: Option<i32> = None;

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

/// **TODO(custom):** Replace with a normalized, stable composite for deduplication across runs.
fn municipal_office_identity_for_ref_key(filing: &TxMunicipalFiling) -> String {
    [
        filing.municipality.as_deref().unwrap_or(""),
        filing.office_name.as_deref().unwrap_or(""),
        filing.district.as_deref().unwrap_or(""),
        filing.seat.as_deref().unwrap_or(""),
        filing.school_district.as_deref().unwrap_or(""),
        filing.place_fips.as_deref().unwrap_or(""),
    ]
    .join("|")
}

async fn process_tx_muni_politician(
    pool: &PgPool,
    filing: &TxMunicipalFiling,
    current_office_id: Uuid,
) -> Result<Politician, Box<dyn Error>> {
    let candidate_name_raw = filing
        .candidate_name
        .as_ref()
        .ok_or("Missing candidate name")?;
    let candidate_name = extractors::politician::normalize_name(candidate_name_raw);

    let office_identity = municipal_office_identity_for_ref_key(filing);
    let ref_key = generators::politician::PoliticianRefKeyGenerator::new(
        "TX-MUNI",
        ELECTION_YEAR,
        &office_identity,
        Some(&candidate_name),
    )
    .generate();

    let fec_code = "UN";
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
    let middle_name = name_parts.middle.as_deref().map(|s| title(s));
    let last_name = name_parts
        .last
        .as_deref()
        .map(|s| title(s))
        .unwrap_or_default();
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

async fn get_staging_muni_office_id_by_slug(
    pool: &PgPool,
    slug: &str,
) -> Result<Option<Uuid>, Box<dyn Error>> {
    let row: Option<(Uuid,)> = sqlx::query_as(
        "SELECT id FROM ingest_staging.stg_tx_muni_offices WHERE slug = $1",
    )
    .bind(slug)
    .fetch_optional(pool)
    .await?;
    Ok(row.map(|(id,)| id))
}

async fn get_staging_muni_race_id_by_slug(
    pool: &PgPool,
    slug: &str,
) -> Result<Option<Uuid>, Box<dyn Error>> {
    let row: Option<(Uuid,)> = sqlx::query_as(
        "SELECT id FROM ingest_staging.stg_tx_muni_races WHERE slug = $1",
    )
    .bind(slug)
    .fetch_optional(pool)
    .await?;
    Ok(row.map(|(id,)| id))
}

async fn insert_staging_muni_office(
    pool: &PgPool,
    office: &Office,
    state_id: Option<&String>,
) -> Result<(), Box<dyn Error>> {
    sqlx::query(
        r#"
        INSERT INTO ingest_staging.stg_tx_muni_offices (
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

async fn execute_staging_muni_politician_insert(
    pool: &PgPool,
    politician: &Politician,
) -> Result<u64, sqlx::Error> {
    let result = sqlx::query(
        r#"
        INSERT INTO ingest_staging.stg_tx_muni_politicians (
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
    .bind(&politician.votesmart_candidate_id)
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

async fn record_politician_muni_dupe(
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
        INSERT INTO ingest_staging.stg_tx_muni_politician_process_dupes
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

async fn insert_staging_muni_address_for_politician(
    pool: &PgPool,
    addr: &TxStagingAddress,
    politician_id: Uuid,
    update_politician_row: bool,
) -> Result<Uuid, Box<dyn Error>> {
    let stg_address_id = Uuid::new_v4();
    sqlx::query(
        r#"
        INSERT INTO ingest_staging.stg_tx_muni_addresses (id, line_1, city, state, country, politician_id)
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
            r#"UPDATE ingest_staging.stg_tx_muni_politicians SET residence_address_id = $1 WHERE id = $2"#,
        )
        .bind(stg_address_id)
        .bind(politician_id)
        .execute(pool)
        .await?;
    }
    Ok(stg_address_id)
}

async fn apply_same_person_updates_and_dupes_muni(
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
        if existing_residence_address_id.is_none() {
            if let Some(addr) = incoming_address {
                insert_staging_muni_address_for_politician(pool, addr, existing_id, true).await?;
            }
        }
    }
    if let Some(incoming_trimmed) = politician
        .email
        .as_deref()
        .map(|e| e.trim())
        .filter(|e| !e.is_empty())
    {
        sqlx::query(r#"UPDATE ingest_staging.stg_tx_muni_politicians SET email = $1 WHERE id = $2"#)
            .bind(incoming_trimmed)
            .bind(existing_id)
            .execute(pool)
            .await?;
    }
    record_politician_muni_dupe(
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

async fn insert_staging_muni_politician(
    pool: &PgPool,
    politician: &mut Politician,
    address: Option<TxStagingAddress>,
) -> Result<bool, Box<dyn Error>> {
    let mut rows_affected = execute_staging_muni_politician_insert(pool, politician).await?;

    if rows_affected > 0 {
        if let Some(addr) = &address {
            let stg_address_id =
                insert_staging_muni_address_for_politician(pool, addr, politician.id, true).await?;
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
        FROM ingest_staging.stg_tx_muni_politicians
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

    let existing_address = fetch_staging_muni_address(pool, existing_residence_address_id).await?;
    let incoming_address = address.as_ref();

    let (same_person, emails_equal_not_both_empty) = is_same_person_from_emails_and_addresses(
        existing_email_trimmed,
        existing_address.as_ref(),
        incoming_email_trimmed,
        incoming_address,
    );

    if same_person {
        apply_same_person_updates_and_dupes_muni(
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
            FROM ingest_staging.stg_tx_muni_politicians
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
                let e_address = fetch_staging_muni_address(pool, eaddr_id).await?;
                let (same, emails_eq) = is_same_person_from_emails_and_addresses(
                    e_email_trimmed,
                    e_address.as_ref(),
                    incoming_email_trimmed,
                    incoming_address,
                );
                if same {
                    apply_same_person_updates_and_dupes_muni(
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
            insert_staging_muni_address_for_politician(pool, addr, politician.id, false).await?;
        incoming_with_new_slug.residence_address_id = Some(stg_address_id);
        politician.residence_address_id = Some(stg_address_id);
    }
    incoming_with_new_slug.slug = final_incoming_slug.clone();
    rows_affected = execute_staging_muni_politician_insert(pool, &incoming_with_new_slug).await?;

    let incoming_inserted = rows_affected > 0;
    record_politician_muni_dupe(
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

async fn fetch_staging_muni_address(
    pool: &PgPool,
    residence_address_id: Option<uuid::Uuid>,
) -> Result<Option<TxStagingAddress>, Box<dyn Error>> {
    let addr_id = match residence_address_id {
        Some(id) => id,
        None => return Ok(None),
    };
    let row: Option<(String, String, String, String)> = sqlx::query_as(
        r#"SELECT line_1, city, state, country FROM ingest_staging.stg_tx_muni_addresses WHERE id = $1"#,
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

async fn insert_staging_muni_race(pool: &PgPool, race: &Race) -> Result<(), Box<dyn Error>> {
    sqlx::query(
        r#"
        INSERT INTO ingest_staging.stg_tx_muni_races (
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

async fn insert_staging_muni_race_candidate(
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
        FROM ingest_staging.stg_tx_muni_race_candidates
        WHERE ref_key = $1
        "#,
    )
    .bind(ref_key)
    .fetch_optional(pool)
    .await?;

    if let Some(row) = existing {
        sqlx::query(
            r#"
            INSERT INTO ingest_staging.stg_tx_muni_race_candidates_process_dupes
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
        INSERT INTO ingest_staging.stg_tx_muni_race_candidates (race_id, candidate_id, ref_key)
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
