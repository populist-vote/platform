//! Texas candidate filings processor.
//!
//! Reads from existing table `p6t_state_tx.tx_primaries_20260209`.
//! Creates staging tables (ingest_staging.stg_tx_offices, stg_tx_politicians, stg_tx_races, stg_tx_race_candidates)
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
const TX_SOURCE_TABLE: &str = "p6t_state_tx.tx_primaries_20260209";

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
    println!("  - ingest_staging.stg_tx_races");
    println!("  - ingest_staging.stg_tx_race_candidates");

    Ok(())
}

async fn create_staging_tables(pool: &PgPool) -> Result<(), Box<dyn Error>> {
    println!("Creating staging tables in ingest_staging schema...");

    sqlx::query("CREATE SCHEMA IF NOT EXISTS ingest_staging")
        .execute(pool)
        .await?;

    sqlx::query("DROP TABLE IF EXISTS ingest_staging.stg_tx_race_candidates CASCADE")
        .execute(pool)
        .await?;
    sqlx::query("DROP TABLE IF EXISTS ingest_staging.stg_tx_races CASCADE")
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
            created_at TIMESTAMPTZ NOT NULL,
            updated_at TIMESTAMPTZ NOT NULL
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

    println!("Staging tables created successfully!");
    Ok(())
}

/// Process one TX filing and insert into staging tables.
async fn process_and_insert_tx_filing(
    pool: &PgPool,
    filing: &TxCandidateFiling,
    race_type: &str,
) -> Result<(), Box<dyn Error>> {
    let office = process_tx_office(filing)?;
    let office_id = get_staging_office_id_by_slug(pool, &office.slug).await?;
    if office_id.is_none() {
        // Replace none with a state_id once we decide what the state_id is for TX
        insert_staging_office(pool, &office, None).await?;
    }

    let politician = process_tx_politician(pool, filing).await?;
    let race = process_tx_race(filing, &office, office_id, race_type)?;

    insert_staging_politician(pool, &politician).await?;
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
    let county = filing.county.clone().or(extracted_county);
    let office_title = office_title_stripped.as_str();

    let title = office::extract_office_title(&name).unwrap_or_default();
    let chamber = office::extract_office_chamber(&name);
    let (political_scope, election_scope, district_type) = office::extract_office_scope(&name, county.as_deref())
        .ok_or("Failed to extract office scope")?;

    // Extract and strip the seat from the office title, then use that to extract the district
    // This is done to help simplify the district extraction
    let (seat, office_title_no_seat) = office::extract_office_seat(office_title);
    let district = office::extract_office_district(&office_title_no_seat);

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
        priority: None,
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    })
}

async fn process_tx_politician(
    pool: &PgPool,
    filing: &TxCandidateFiling,
) -> Result<Politician, Box<dyn Error>> {
    let candidate_name = filing.candidate_name.as_ref().ok_or("Missing candidate name")?;
    let slug = generators::politician::PoliticianSlugGenerator::new(candidate_name).generate();
    let ref_key = generators::politician::PoliticianRefKeyGenerator::new("TX-SOS", candidate_name).generate();

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

    // Parse name using extractor (with fallback to simple split)
    let name_parts = extractors::politician::extract_politician_name(candidate_name).or_else(|| {
        let parts: Vec<&str> = candidate_name.split_whitespace().collect();
        if let (Some(first), Some(last)) = (parts.first(), parts.last()) {
            Some(extractors::politician::PoliticianName {
                first: (*first).to_string(),
                middle: if parts.len() > 2 {
                    Some(parts[1..parts.len() - 1].join(" "))
                } else {
                    None
                },
                last: Some((*last).to_string()),
                suffix: None,
                preferred: None,
            })
        } else {
            None
        }
    }).ok_or("Failed to parse candidate name")?;

    Ok(Politician {
        id: Uuid::new_v4(),
        slug,
        ref_key: Some(ref_key),
        first_name: name_parts.first,
        middle_name: name_parts.middle,
        last_name: name_parts.last.unwrap_or_default(),
        suffix: name_parts.suffix,
        preferred_name: name_parts.preferred,
        full_name: Some(candidate_name.to_string()),
        biography: None,
        biography_source: None,
        home_state: Some(State::TX),
        party_id,
        date_of_birth: None,
        office_id: None,
        upcoming_race_id: None,
        thumbnail_image_url: None,
        assets: JSON::Object(serde_json::Map::new()),
        official_website_url: None,
        campaign_website_url: None,
        facebook_url: None,
        twitter_url: None,
        instagram_url: None,
        youtube_url: None,
        linkedin_url: None,
        tiktok_url: None,
        email: filing.email.clone(),
        phone: None,
        votesmart_candidate_id: None,
        votesmart_candidate_bio: JSON::Object(serde_json::Map::new()),
        votesmart_candidate_ratings: JSON::Object(serde_json::Map::new()),
        legiscan_people_id: None,
        crp_candidate_id: None,
        fec_candidate_id: None,
        race_wins: None,
        race_losses: None,
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
    let election_year = 2026;

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
        election_year,
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
        party_id: None,
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
    slugify!(&format!("tx-primaries-{}-{}", office_title, candidate_name))
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

async fn insert_staging_politician(pool: &PgPool, politician: &Politician) -> Result<(), Box<dyn Error>> {
    sqlx::query(
        r#"
        INSERT INTO ingest_staging.stg_tx_politicians (
            id, slug, ref_key, first_name, middle_name, last_name, suffix, preferred_name,
            full_name, biography, biography_source, home_state, party_id, date_of_birth,
            office_id, upcoming_race_id, thumbnail_image_url, assets, official_website_url,
            campaign_website_url, facebook_url, twitter_url, instagram_url, youtube_url,
            linkedin_url, tiktok_url, email, phone, votesmart_candidate_id,
            votesmart_candidate_bio, votesmart_candidate_ratings, legiscan_people_id,
            crp_candidate_id, fec_candidate_id, race_wins, race_losses, created_at, updated_at
        ) VALUES (
            $1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18,
            $19, $20, $21, $22, $23, $24, $25, $26, $27, $28, $29, $30, $31, $32, $33, $34,
            $35, $36, $37, $38
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
    .bind(politician.created_at)
    .bind(politician.updated_at)
    .execute(pool)
    .await?;
    Ok(())
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
