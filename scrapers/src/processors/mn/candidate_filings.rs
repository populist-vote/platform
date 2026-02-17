use crate::extractors;
use crate::generators;
use db::{Office, Politician, Race, RaceType, State, VoteType};
use serde_json::Value as JSON;
use slugify::slugify;
use sqlx::PgPool;
use std::error::Error;
use std::str::FromStr;
use uuid::Uuid;

pub struct CandidateFiling {
    pub office_title: Option<String>,
    pub office_id: Option<String>,
    pub office_code: Option<String>,
    pub candidate_name: Option<String>,
    pub party_abbreviation: Option<String>,
    pub campaign_phone: Option<String>,
    pub campaign_email: Option<String>,
    pub campaign_website: Option<String>,
    pub county_id: Option<String>,
    pub county_name: Option<String>,
    pub residence_street_address: Option<String>,
    pub residence_city: Option<String>,
    pub residence_state: Option<String>,
    pub residence_zip: Option<String>,
    pub campaign_address: Option<String>,
    pub campaign_city: Option<String>,
    pub campaign_state: Option<String>,
    pub campaign_zip: Option<String>,
}

pub async fn process_mn_candidate_filings(
    pool: &PgPool,
    source_table: &str,
    race_type: &str,
) -> Result<(), Box<dyn Error>> {
    println!("Starting MN candidate filings processor...");
    println!("Source table: {}", source_table);
    println!("Race type: {}", race_type);

    // 1. Create staging tables in ingest_staging schema
    create_staging_tables(pool).await?;

    // 2. Get raw filings from source table
    println!("Fetching raw candidate filings...");
    let filings = sqlx::query_as!(
        CandidateFiling,
        r#"
        SELECT DISTINCT
            raw.office_title,
            raw.office_id,
            raw.office_code,
            raw.candidate_name,
            raw.party_abbreviation,
            raw.campaign_phone,
            raw.campaign_email,
            raw.campaign_website,
            raw.county_id,
            vd.countyname as county_name,
            raw.residence_street_address,
            raw.residence_city,
            raw.residence_state,
            raw.residence_zip,
            raw.campaign_address,
            raw.campaign_city,
            raw.campaign_state,
            raw.campaign_zip
        FROM p6t_state_mn.mn_candidate_filings_local_2025 raw
        LEFT JOIN (
            SELECT DISTINCT countycode, countyname 
            FROM p6t_state_mn.bdry_votingdistricts
        ) vd ON raw.county_id IS NOT NULL AND REGEXP_REPLACE(raw.county_id, '^0+', '') = vd.countycode
        WHERE raw.office_title IS NOT NULL AND raw.office_title != 'U.S. President & Vice President'
        "#
    )
    .fetch_all(pool)
    .await?;

    println!("Found {} candidate filings to process", filings.len());

    // 3. Process each filing and insert into staging tables
    let mut processed_count = 0;
    let mut error_count = 0;

    for (index, filing) in filings.iter().enumerate() {
        if index % 100 == 0 {
            println!("Processing filing {}/{}...", index + 1, filings.len());
        }

        match process_and_insert_filing(pool, filing, race_type).await {
            Ok(_) => processed_count += 1,
            Err(e) => {
                error_count += 1;
                eprintln!(
                    "Error processing filing: {} - {}",
                    filing
                        .candidate_name
                        .as_ref()
                        .unwrap_or(&"Unknown".to_string()),
                    e
                );
            }
        }
    }

    println!("\n=== Processing Complete ===");
    println!("Successfully processed: {}", processed_count);
    println!("Errors: {}", error_count);
    println!("\nStaging tables created in ingest_staging schema:");
    println!("  - ingest_staging.stg_mn_offices");
    println!("  - ingest_staging.stg_mn_politicians");
    println!("  - ingest_staging.stg_mn_races");
    println!("  - ingest_staging.stg_mn_race_candidates");

    Ok(())
}

async fn create_staging_tables(pool: &PgPool) -> Result<(), Box<dyn Error>> {
    println!("Creating staging tables in ingest_staging schema...");

    // Create schema if it doesn't exist
    sqlx::query("CREATE SCHEMA IF NOT EXISTS ingest_staging")
        .execute(pool)
        .await?;

    // Drop existing staging tables
    sqlx::query("DROP TABLE IF EXISTS ingest_staging.stg_mn_race_candidates CASCADE")
        .execute(pool)
        .await?;
    sqlx::query("DROP TABLE IF EXISTS ingest_staging.stg_mn_races CASCADE")
        .execute(pool)
        .await?;
    sqlx::query("DROP TABLE IF EXISTS ingest_staging.stg_mn_politicians CASCADE")
        .execute(pool)
        .await?;
    sqlx::query("DROP TABLE IF EXISTS ingest_staging.stg_mn_offices CASCADE")
        .execute(pool)
        .await?;

    // Create staging offices table
    sqlx::query(
        r#"
        CREATE TABLE ingest_staging.stg_mn_offices (
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

    // Create staging politicians table
    sqlx::query(
        r#"
        CREATE TABLE ingest_staging.stg_mn_politicians (
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
    "#,
    )
    .execute(pool)
    .await?;

    // Create staging races table
    sqlx::query(
        r#"
        CREATE TABLE ingest_staging.stg_mn_races (
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

    // Create staging race_candidates table
    sqlx::query(
        r#"
        CREATE TABLE ingest_staging.stg_mn_race_candidates (
            race_id UUID NOT NULL,
            candidate_id UUID NOT NULL,
            ref_key TEXT,
            PRIMARY KEY (race_id, candidate_id)
        )
    "#,
    )
    .execute(pool)
    .await?;

    println!("Staging tables created successfully!");

    Ok(())
}

async fn process_and_insert_filing(
    pool: &PgPool,
    filing: &CandidateFiling,
    race_type: &str,
) -> Result<(), Box<dyn Error>> {
    // Process office data
    let office = process_office(filing)?;

    // Resolve office id: use existing row in stg_mn_offices if slug already exists, else None (we'll use office.id in process_race)
    let office_id = get_staging_office_id_by_slug(pool, &office.slug).await?;

    // Insert office into staging table only if it doesn't already exist
    if office_id.is_none() {
        insert_staging_office(pool, &office, filing.office_code.as_ref()).await?;
    }

    // Process politician data
    let politician = process_politician(pool, filing).await?;

    // Process race data: use resolved office_id if present, otherwise office.id
    let race = process_race(filing, &office, office_id, race_type)?;

    // Insert politician into staging table
    insert_staging_politician(pool, &politician).await?;

    // Insert race into staging table (or skip if slug already exists)
    insert_staging_race(pool, &race).await?;

    // Resolve the race id from stg_mn_races by slug so we use the existing row's id when there was a slug conflict
    let race_id = get_staging_race_id_by_slug(pool, &race.slug)
        .await?
        .unwrap_or(race.id);

    // Process race candidate ref_key from filing, then insert into staging table
    let race_candidate_ref_key = process_race_candidate(filing);
    insert_staging_race_candidate(pool, race_id, &politician, &race_candidate_ref_key).await?;

    Ok(())
}

async fn insert_staging_office(
    pool: &PgPool,
    office: &Office,
    state_id: Option<&String>,
) -> Result<(), Box<dyn Error>> {
    sqlx::query(
        r#"
        INSERT INTO ingest_staging.stg_mn_offices (
            id, slug, name, title, subtitle, subtitle_short, office_type, chamber,
            district_type, political_scope, election_scope, state, state_id, county, municipality,
            term_length, district, seat, school_district, hospital_district, priority,
            created_at, updated_at
        ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18, $19, $20, $21, $22, $23)
        ON CONFLICT (slug) DO NOTHING
        "#
    )
    .bind(office.id)
    .bind(&office.slug)
    .bind(&office.name)
    .bind(&office.title)
    .bind(&office.subtitle)
    .bind(&office.subtitle_short)
    .bind(office.office_type.as_ref().map(|o| o.as_str()))
    .bind(office.chamber.map(|c| format!("{:?}", c)))
    .bind(office.district_type.map(|d| format!("{:?}", d)))
    .bind(format!("{:?}", office.political_scope))
    .bind(format!("{:?}", office.election_scope))
    .bind(office.state.map(|s| s.as_ref().to_string()))
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

async fn insert_staging_politician(
    pool: &PgPool,
    politician: &Politician,
) -> Result<(), Box<dyn Error>> {
    sqlx::query(
        r#"
        INSERT INTO ingest_staging.stg_mn_politicians (
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
    .bind(politician.home_state.map(|s| s.as_ref().to_string()))
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
        INSERT INTO ingest_staging.stg_mn_races (
            id, title, slug, office_id, state, race_type, vote_type, party_id,
            description, ballotpedia_link, early_voting_begins_date, official_website,
            election_id, winner_ids, total_votes, num_precincts_reporting, total_precincts,
            is_special_election, num_elect, created_at, updated_at
        ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18, $19, $20, $21)
        ON CONFLICT (slug) DO NOTHING
        "#
    )
    .bind(race.id)
    .bind(&race.title)
    .bind(&race.slug)
    .bind(race.office_id)
    .bind(race.state.map(|s| s.as_ref().to_string()))
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

async fn get_staging_office_id_by_slug(
    pool: &PgPool,
    slug: &str,
) -> Result<Option<Uuid>, Box<dyn Error>> {
    let row: Option<(Uuid,)> =
        sqlx::query_as("SELECT id FROM ingest_staging.stg_mn_offices WHERE slug = $1")
            .bind(slug)
            .fetch_optional(pool)
            .await?;
    Ok(row.map(|(id,)| id))
}

async fn get_staging_race_id_by_slug(
    pool: &PgPool,
    slug: &str,
) -> Result<Option<Uuid>, Box<dyn Error>> {
    let row: Option<(Uuid,)> =
        sqlx::query_as("SELECT id FROM ingest_staging.stg_mn_races WHERE slug = $1")
            .bind(slug)
            .fetch_optional(pool)
            .await?;
    Ok(row.map(|(id,)| id))
}

fn process_race_candidate(filing: &CandidateFiling) -> String {
    let office_title = filing.office_title.as_deref().unwrap_or("");
    let candidate_name = filing.candidate_name.as_deref().unwrap_or("");
    slugify!(&format!("mn-sos-{}-{}", office_title, candidate_name))
}

async fn insert_staging_race_candidate(
    pool: &PgPool,
    race_id: Uuid,
    politician: &Politician,
    ref_key: &str,
) -> Result<(), Box<dyn Error>> {
    sqlx::query(
        r#"
        INSERT INTO ingest_staging.stg_mn_race_candidates (race_id, candidate_id, ref_key)
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

fn process_office(filing: &CandidateFiling) -> Result<Office, Box<dyn Error>> {
    let office_title = filing.office_title.as_ref().ok_or("Missing office title")?;

    // Extract office attributes
    let name = extractors::mn::office::extract_office_name(office_title);
    let title = extractors::mn::office::extract_office_title(office_title)
        .ok_or("Failed to extract office title")?;
    let chamber = extractors::mn::office::extract_office_chamber(office_title);
    let county_id_int = filing
        .county_id
        .as_ref()
        .and_then(|id| id.parse::<i32>().ok());

    // Extract district_type and election_scope first (required for political_scope)
    let district_type =
        extractors::mn::office::extract_office_district_type(office_title, county_id_int);
    let election_scope =
        extractors::mn::office::extract_office_election_scope(office_title, county_id_int)
            .ok_or("Failed to extract election scope")?;

    // Extract political_scope using election_scope, name, and district_type
    let political_scope = extractors::mn::office::extract_office_political_scope(
        name.as_deref(),
        &election_scope,
        &district_type,
    );

    // Extract district and seat
    let district = extractors::mn::office::extract_office_district(office_title);
    let seat = extractors::mn::office::extract_office_seat(office_title);

    // Extract school and hospital districts
    let school_district = extractors::mn::office::extract_school_district(office_title);
    let hospital_district = extractors::mn::office::extract_hospital_district(office_title);

    // Extract municipality
    let municipality =
        extractors::mn::office::extract_municipality(office_title, &election_scope, &district_type);

    // Generate slug and subtitle
    let slug = generators::mn::office::OfficeSlugGenerator {
        state: &State::MN,
        name: name.as_deref().unwrap_or_default(),
        county: filing.county_name.as_deref(),
        district: district.as_deref(),
        seat: seat.as_deref(),
        school_district: school_district.as_deref(),
        hospital_district: hospital_district.as_deref(),
        municipality: municipality.as_deref(),
        election_scope: Some(&election_scope),
        district_type: district_type.as_ref(),
    }
    .generate();

    let (subtitle, subtitle_short) = generators::mn::office::OfficeSubtitleGenerator {
        state: &State::MN,
        office_name: name.as_deref(),
        election_scope: &election_scope,
        district_type: district_type.as_ref(),
        county: filing.county_name.as_deref(),
        district: district.as_deref(),
        seat: seat.as_deref(),
        school_district: school_district.as_deref(),
        hospital_district: hospital_district.as_deref(),
        municipality: municipality.as_deref(),
    }
    .generate();

    // Create office record
    Ok(Office {
        id: Uuid::new_v4(),
        slug,
        name,
        title,
        subtitle: Some(subtitle),
        subtitle_short: Some(subtitle_short),
        office_type: None,
        chamber,
        district_type,
        political_scope,
        election_scope,
        state: Some(State::MN),
        county: filing.county_name.clone(),
        municipality,
        term_length: None,
        district,
        seat,
        school_district,
        hospital_district,
        priority: None,
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    })
}

async fn process_politician(
    pool: &PgPool,
    filing: &CandidateFiling,
) -> Result<Politician, Box<dyn Error>> {
    let candidate_name = filing
        .candidate_name
        .as_ref()
        .ok_or("Missing candidate name")?;

    // Generate politician slug
    let slug = generators::politician::PoliticianSlugGenerator::new(candidate_name).generate();

    // Generate ref key
    let ref_key =
        generators::politician::PoliticianRefKeyGenerator::new("MN-SOS", candidate_name).generate();

    // Resolve party_id from production party table
    // If no party_abbreviation or empty, use "UN" (unaffiliated)
    let fec_code = if let Some(party_abbrev) = &filing.party_abbreviation {
        if party_abbrev.trim().is_empty() {
            "UN".to_string() // Empty/whitespace = unaffiliated
        } else {
            // Try to extract the FEC code, default to "UN" if not found
            extractors::party::extract_party_fec_code(party_abbrev)
                .unwrap_or_else(|| "UN".to_string())
        }
    } else {
        "UN".to_string() // No party abbreviation = unaffiliated
    };

    // Query the production party table to get the party.id
    let party_id = sqlx::query_scalar!(r#"SELECT id FROM party WHERE fec_code = $1"#, fec_code)
        .fetch_optional(pool)
        .await?;

    // Parse name using extractor (with fallback to simple split)
    let name_parts = extractors::politician::extract_politician_name(candidate_name)
        .or_else(|| {
            // Fallback to simple split if extractor fails
            let parts: Vec<&str> = candidate_name.split_whitespace().collect();
            if let (Some(first), Some(last)) = (parts.first(), parts.last()) {
                Some(extractors::politician::PoliticianName {
                    first: first.to_string(),
                    last: Some(last.to_string()),
                    middle: if parts.len() > 2 {
                        Some(parts[1..parts.len() - 1].join(" "))
                    } else {
                        None
                    },
                    preferred: None,
                    suffix: None,
                })
            } else {
                None
            }
        })
        .ok_or("Failed to parse candidate name")?;

    // Create politician record
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
        home_state: Some(State::MN),
        party_id,
        date_of_birth: None,
        office_id: None,
        upcoming_race_id: None,
        thumbnail_image_url: None,
        assets: JSON::Object(serde_json::Map::new()),
        official_website_url: None,
        campaign_website_url: filing.campaign_website.clone(),
        facebook_url: None,
        twitter_url: None,
        instagram_url: None,
        youtube_url: None,
        linkedin_url: None,
        tiktok_url: None,
        email: filing.campaign_email.clone(),
        phone: filing.campaign_phone.clone(),
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

fn process_race(
    filing: &CandidateFiling,
    office: &Office,
    office_id: Option<Uuid>,
    race_type: &str,
) -> Result<Race, Box<dyn Error>> {
    // Hardcoded election ID for 2025 General Election
    let election_id =
        Uuid::parse_str("a81f4a62-69d6-48f9-b704-c0151a42b8c8").expect("Invalid election UUID");

    // Hardcoded year for race titles
    let election_year = 2025;

    // Extract if this is a special election
    let is_special_election = filing
        .office_title
        .as_ref()
        .map(|title| extractors::mn::race::extract_is_special_election(title))
        .unwrap_or(false);

    // Extract number of positions to elect
    let num_elect = filing
        .office_title
        .as_ref()
        .and_then(|title| extractors::mn::race::extract_num_elect(title));

    // Extract vote type (ranked choice if "First Choice" is present)
    let vote_type = filing
        .office_title
        .as_ref()
        .map(|title| {
            if extractors::mn::race::extract_is_ranked_choice(title) {
                VoteType::RankedChoice
            } else {
                VoteType::Plurality
            }
        })
        .unwrap_or(VoteType::Plurality);

    // Get party abbreviation for primary races
    let party = filing.party_abbreviation.as_deref();

    // Generate race title and slug
    let (title, slug) = generators::mn::race::RaceTitleGenerator::from_source(
        &RaceType::from_str(race_type)?,
        office,
        is_special_election,
        party,
        election_year,
    )
    .generate();

    // Create race record: use resolved office_id from stg_mn_offices if present, otherwise office.id
    let resolved_office_id = office_id.unwrap_or(office.id);
    Ok(Race {
        id: Uuid::new_v4(),
        title,
        slug,
        office_id: resolved_office_id,
        state: Some(State::MN),
        race_type: RaceType::from_str(race_type)?,
        vote_type,
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
