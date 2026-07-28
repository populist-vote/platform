//! Merges staging data from ingest_staging.stg_mn_* into production tables (office, politician, race, race_candidates).
//! Run after process_mn_candidate_filings. Resolves by slug for offices/races and by
//! email/phone/address for politicians. Addresses: either staging residence or campaign may match
//! either production residence or campaign; staging addresses are upserted into production address
//! and linked on the politician.

use db::{
    Address, Chamber, DistrictType, ElectionScope, InsertAddressInput, Office, PoliticalScope,
    Politician, Race, RaceCandidate, RaceType, State, UpdatePoliticianInput, UpsertOfficeInput,
    UpsertPoliticianInput, UpsertRaceCandidateInput, UpsertRaceInput, VoteType,
};
use serde_json::Value as JSON;
use sqlx::PgPool;
use std::collections::HashMap;
use std::str::FromStr;

#[derive(sqlx::FromRow, Debug)]
struct StgOffice {
    id: uuid::Uuid,
    slug: String,
    name: Option<String>,
    title: Option<String>,
    subtitle: Option<String>,
    subtitle_short: Option<String>,
    office_type: Option<String>,
    chamber: Option<String>,
    district_type: Option<String>,
    political_scope: Option<String>,
    election_scope: Option<String>,
    state: Option<String>,
    county: Option<String>,
    municipality: Option<String>,
    term_length: Option<i32>,
    district: Option<String>,
    seat: Option<String>,
    school_district: Option<String>,
    hospital_district: Option<String>,
    priority: Option<i32>,
}

#[derive(sqlx::FromRow, Debug)]
struct StgPolitician {
    id: uuid::Uuid,
    slug: String,
    ref_key: Option<String>,
    first_name: String,
    middle_name: Option<String>,
    last_name: String,
    suffix: Option<String>,
    preferred_name: Option<String>,
    full_name: Option<String>,
    home_state: Option<String>,
    party_id: Option<uuid::Uuid>,
    email: Option<String>,
    phone: Option<String>,
    campaign_website_url: Option<String>,
    assets: Option<JSON>,
    residence_address_id: Option<uuid::Uuid>,
    campaign_address_id: Option<uuid::Uuid>,
}

#[derive(Clone, sqlx::FromRow, Debug)]
struct StgAddress {
    line_1: String,
    city: String,
    state: String,
    postal_code: String,
    country: String,
}

#[derive(sqlx::FromRow, Debug)]
struct StgRace {
    id: uuid::Uuid,
    slug: String,
    title: String,
    office_id: uuid::Uuid,
    state: Option<String>,
    race_type: Option<String>,
    vote_type: Option<String>,
    party_id: Option<uuid::Uuid>,
    election_id: Option<uuid::Uuid>,
    is_special_election: bool,
    num_elect: Option<i32>,
}

#[derive(sqlx::FromRow, Debug)]
struct StgRaceCandidate {
    race_id: uuid::Uuid,
    candidate_id: uuid::Uuid,
    ref_key: Option<String>,
}

#[tokio::main]
async fn main() {
    db::init_pool().await.unwrap();
    let pool = db::pool().await;
    let db = &pool.connection;

    println!("=== Merge MN staging → production ===\n");

    if let Err(e) = run_merge(db).await {
        eprintln!("Merge failed: {}", e);
        std::process::exit(1);
    }
    println!("\n✓ Merge completed successfully.");
}

async fn run_merge(pool: &PgPool) -> Result<(), Box<dyn std::error::Error>> {
    // 1. Offices: upsert by slug, build stg_office_id -> prod_office_id
    println!("Merging offices...");
    let stg_offices: Vec<StgOffice> = sqlx::query_as(
        "SELECT id, slug, name, title, subtitle, subtitle_short, office_type, chamber, district_type, political_scope, election_scope, state, county, municipality, term_length, district, seat, school_district, hospital_district, priority FROM ingest_staging.stg_mn_offices",
    )
    .fetch_all(pool)
    .await?;

    let mut stg_to_prod_office: HashMap<uuid::Uuid, uuid::Uuid> = HashMap::new();
    let mut offices_existing = 0usize;
    let mut offices_new = 0usize;
    for stg in &stg_offices {
        let existed = office_slug_exists(pool, &stg.slug).await?;
        if existed {
            offices_existing += 1;
        } else {
            offices_new += 1;
        }
        let input = stg_office_to_upsert(stg);
        let prod = Office::upsert_from_source(pool, &input).await?;
        stg_to_prod_office.insert(stg.id, prod.id);
    }
    println!(
        "  Offices: {} existing (matched by slug), {} new",
        offices_existing, offices_new
    );

    // 2. Politicians: resolve by email / phone / address, else upsert
    println!("Merging politicians...");
    let stg_politicians: Vec<StgPolitician> = sqlx::query_as(
        "SELECT id, slug, ref_key, first_name, middle_name, last_name, suffix, preferred_name, full_name, home_state, party_id, email, phone, campaign_website_url, assets, residence_address_id, campaign_address_id FROM ingest_staging.stg_mn_politicians",
    )
    .fetch_all(pool)
    .await?;

    create_merge_tables(pool).await?;

    let mut stg_to_prod_politician: HashMap<uuid::Uuid, uuid::Uuid> = HashMap::new();
    let mut politicians_existing = 0usize;
    let mut politicians_new = 0usize;
    let mut addresses_inserted = 0usize;
    let mut addresses_reused = 0usize;
    for stg in &stg_politicians {
        let (prod_id, was_existing) = resolve_or_upsert_politician(
            pool,
            stg,
            &mut addresses_inserted,
            &mut addresses_reused,
        )
        .await?;
        if was_existing {
            politicians_existing += 1;
        } else {
            politicians_new += 1;
        }
        stg_to_prod_politician.insert(stg.id, prod_id);
    }
    println!(
        "  Politicians: {} existing (matched by email/phone/address), {} new",
        politicians_existing, politicians_new
    );
    println!(
        "  Addresses: {} inserted, {} existing (reused)",
        addresses_inserted, addresses_reused
    );
    println!("  Audit tables: ingest_staging.politician_merge_dupes, politician_merge_updated,");
    println!("                politician_merge_overwritten_data, inserted_politicians_with_same_slug");

    // 3. Races: upsert by slug with prod office_id; build stg_race_id -> prod_race_id
    println!("Merging races...");
    let stg_races: Vec<StgRace> = sqlx::query_as(
        "SELECT id, slug, title, office_id, state, race_type, vote_type, party_id, election_id, is_special_election, num_elect FROM ingest_staging.stg_mn_races",
    )
    .fetch_all(pool)
    .await?;

    let mut stg_to_prod_race: HashMap<uuid::Uuid, uuid::Uuid> = HashMap::new();
    for stg in &stg_races {
        let prod_office_id = stg_to_prod_office
            .get(&stg.office_id)
            .copied()
            .ok_or_else(|| {
                format!(
                    "Staging race {} references unknown office {}",
                    stg.slug, stg.office_id
                )
            })?;
        let input = stg_race_to_upsert(stg, prod_office_id);
        let prod = Race::upsert_from_source(pool, &input).await?;
        stg_to_prod_race.insert(stg.id, prod.id);
    }
    println!("  Races: {} merged", stg_to_prod_race.len());

    // 4. Race candidates: insert (prod_race_id, prod_candidate_id)
    println!("Merging race_candidates...");
    let stg_rcs: Vec<StgRaceCandidate> = sqlx::query_as(
        "SELECT race_id, candidate_id, ref_key FROM ingest_staging.stg_mn_race_candidates",
    )
    .fetch_all(pool)
    .await?;

    let mut inserted = 0usize;
    for rc in &stg_rcs {
        let prod_race_id = match stg_to_prod_race.get(&rc.race_id) {
            Some(id) => *id,
            None => continue,
        };
        let prod_candidate_id = match stg_to_prod_politician.get(&rc.candidate_id) {
            Some(id) => *id,
            None => continue,
        };
        let input = UpsertRaceCandidateInput {
            race_id: prod_race_id,
            candidate_id: prod_candidate_id,
            ref_key: rc.ref_key.clone(),
            is_running: None,
        };
        if RaceCandidate::upsert_from_source(pool, &input)
            .await?
            .is_some()
        {
            inserted += 1;
        }
    }
    println!("  Race_candidates: {} new links", inserted);

    Ok(())
}

fn parse_state(s: Option<&String>) -> Option<State> {
    s.and_then(|s| State::from_str(s.trim()).ok())
}

fn parse_scope(s: Option<&String>) -> Option<PoliticalScope> {
    s.and_then(|s| PoliticalScope::from_str(s.trim()).ok())
}

fn parse_election_scope(s: Option<&String>) -> Option<ElectionScope> {
    s.and_then(|s| ElectionScope::from_str(s.trim()).ok())
}

fn parse_chamber(s: Option<&String>) -> Option<Chamber> {
    s.and_then(|s| Chamber::from_str(s.trim()).ok())
}

fn parse_district_type(s: Option<&String>) -> Option<DistrictType> {
    s.and_then(|s| DistrictType::from_str(s.trim()).ok())
}

fn parse_race_type(s: Option<&String>) -> Option<RaceType> {
    s.and_then(|s| RaceType::from_str(s.trim()).ok())
}

fn parse_vote_type(s: Option<&String>) -> Option<VoteType> {
    s.and_then(|s| VoteType::from_str(s.trim()).ok())
}

fn base_slug_if_increment(slug: &str) -> Option<String> {
    let Some(dash_pos) = slug.rfind('-') else {
        return None;
    };
    let suffix = &slug[dash_pos + 1..];
    if suffix.chars().all(|c| c.is_ascii_digit()) && !suffix.is_empty() {
        Some(slug[..dash_pos].to_string())
    } else {
        None
    }
}

fn stg_office_to_upsert(stg: &StgOffice) -> UpsertOfficeInput {
    UpsertOfficeInput {
        id: None,
        slug: Some(stg.slug.clone()),
        title: stg.title.clone(),
        subtitle: stg.subtitle.clone(),
        subtitle_short: stg.subtitle_short.clone(),
        name: stg.name.clone(),
        office_type: stg.office_type.clone(),
        district: stg.district.clone(),
        district_type: parse_district_type(stg.district_type.as_ref()),
        hospital_district: stg.hospital_district.clone(),
        school_district: stg.school_district.clone(),
        chamber: parse_chamber(stg.chamber.as_ref()),
        political_scope: parse_scope(stg.political_scope.as_ref()),
        election_scope: parse_election_scope(stg.election_scope.as_ref()),
        state: parse_state(stg.state.as_ref()),
        county: stg.county.clone(),
        municipality: stg.municipality.clone(),
        term_length: stg.term_length,
        seat: stg.seat.clone(),
        priority: stg.priority,
    }
}

fn addresses_equal(stg: &StgAddress, prod: &Address) -> bool {
    let norm = |s: &str| s.trim().to_lowercase();
    let stg_state = parse_state(Some(&stg.state));
    norm(&stg.line_1) == norm(&prod.line_1)
        && norm(&stg.city) == norm(&prod.city)
        && stg_state == Some(prod.state)
        && norm(&stg.country) == norm(&prod.country)
        && norm(&stg.postal_code) == norm(&prod.postal_code)
}

async fn fetch_stg_mn_address(
    pool: &PgPool,
    address_id: uuid::Uuid,
) -> Result<Option<StgAddress>, Box<dyn std::error::Error>> {
    let row = sqlx::query_as(
        "SELECT line_1, city, state, postal_code, country FROM ingest_staging.stg_mn_address WHERE id = $1",
    )
    .bind(address_id)
    .fetch_optional(pool)
    .await?;
    Ok(row)
}

/// Merge staging address into production: if an address with the same key already exists, return
/// its id and false (reused); otherwise upsert and return the address id and true (inserted).
async fn merge_staging_address_to_production(
    pool: &PgPool,
    stg: &StgAddress,
) -> Result<(uuid::Uuid, bool), Box<dyn std::error::Error>> {
    let state = parse_state(Some(&stg.state)).unwrap_or(State::MN);
    let postal_code = stg.postal_code.trim().to_string();
    if let Some(addr) = Address::find_by_unique_key(
        pool,
        &stg.line_1,
        None,
        &stg.city,
        &state,
        &stg.country,
        &postal_code,
    )
    .await?
    {
        return Ok((addr.id, false));
    }
    let input = InsertAddressInput {
        line_1: stg.line_1.clone(),
        line_2: None,
        city: stg.city.clone(),
        state,
        country: stg.country.clone(),
        postal_code,
        county: None,
        congressional_district: None,
        state_senate_district: None,
        state_house_district: None,
        lon: None,
        lat: None,
    };
    let addr = Address::upsert(pool, &input).await?;
    Ok((addr.id, true))
}

/// Resolve staging residence and campaign addresses into production address ids.
async fn resolve_stg_addresses_to_production(
    pool: &PgPool,
    stg: &StgPolitician,
    address_inserted: &mut usize,
    address_reused: &mut usize,
) -> Result<(Option<uuid::Uuid>, Option<uuid::Uuid>), Box<dyn std::error::Error>> {
    let mut residence_address_id = None;
    let mut campaign_address_id = None;

    if let Some(stg_addr_id) = stg.residence_address_id {
        if let Some(addr) = fetch_stg_mn_address(pool, stg_addr_id).await? {
            let (id, was_inserted) = merge_staging_address_to_production(pool, &addr).await?;
            if was_inserted {
                *address_inserted += 1;
            } else {
                *address_reused += 1;
            }
            residence_address_id = Some(id);
        }
    }

    if let Some(stg_addr_id) = stg.campaign_address_id {
        if let Some(addr) = fetch_stg_mn_address(pool, stg_addr_id).await? {
            let (id, was_inserted) = merge_staging_address_to_production(pool, &addr).await?;
            if was_inserted {
                *address_inserted += 1;
            } else {
                *address_reused += 1;
            }
            campaign_address_id = Some(id);
        }
    }

    Ok((residence_address_id, campaign_address_id))
}

/// True if either staging residence or campaign address values match either production
/// residence or campaign address values.
async fn do_mn_addresses_match(
    pool: &PgPool,
    stg: &StgPolitician,
    prod_residence_address_id: Option<uuid::Uuid>,
    prod_campaign_address_id: Option<uuid::Uuid>,
) -> Result<bool, Box<dyn std::error::Error>> {
    let mut stg_addrs: Vec<StgAddress> = Vec::new();
    if let Some(id) = stg.residence_address_id {
        if let Some(addr) = fetch_stg_mn_address(pool, id).await? {
            stg_addrs.push(addr);
        }
    }
    if let Some(id) = stg.campaign_address_id {
        if let Some(addr) = fetch_stg_mn_address(pool, id).await? {
            stg_addrs.push(addr);
        }
    }
    if stg_addrs.is_empty() {
        return Ok(false);
    }

    let mut prod_addrs: Vec<Address> = Vec::new();
    if let Some(id) = prod_residence_address_id {
        if let Some(addr) = Address::find_by_id(pool, &id).await? {
            prod_addrs.push(addr);
        }
    }
    if let Some(id) = prod_campaign_address_id {
        if let Some(addr) = Address::find_by_id(pool, &id).await? {
            prod_addrs.push(addr);
        }
    }
    if prod_addrs.is_empty() {
        return Ok(false);
    }

    for stg_addr in &stg_addrs {
        for prod_addr in &prod_addrs {
            if addresses_equal(stg_addr, prod_addr) {
                return Ok(true);
            }
        }
    }
    Ok(false)
}

/// Create or recreate all ingest_staging tables used by the MN merge (politician audit tables).
/// Drops tables if they exist, then creates them. Call once before merging politicians.
async fn create_merge_tables(pool: &PgPool) -> Result<(), Box<dyn std::error::Error>> {
    sqlx::query("CREATE SCHEMA IF NOT EXISTS ingest_staging")
        .execute(pool)
        .await?;

    sqlx::query("DROP TABLE IF EXISTS ingest_staging.politician_merge_overwritten_data CASCADE")
        .execute(pool)
        .await?;
    sqlx::query("DROP TABLE IF EXISTS ingest_staging.inserted_politicians_with_same_slug CASCADE")
        .execute(pool)
        .await?;
    sqlx::query("DROP TABLE IF EXISTS ingest_staging.politician_merge_updated CASCADE")
        .execute(pool)
        .await?;
    sqlx::query("DROP TABLE IF EXISTS ingest_staging.politician_merge_dupes CASCADE")
        .execute(pool)
        .await?;

    sqlx::query(
        r#"
        CREATE TABLE ingest_staging.politician_merge_dupes (
            id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
            stg_first_name TEXT NOT NULL,
            stg_middle_name TEXT,
            stg_last_name TEXT NOT NULL,
            stg_suffix TEXT,
            prod_first_name TEXT NOT NULL,
            prod_middle_name TEXT,
            prod_last_name TEXT NOT NULL,
            prod_suffix TEXT,
            stg_id UUID NOT NULL,
            prod_id UUID NOT NULL,
            stg_ref_key TEXT,
            prod_ref_key TEXT,
            stg_address_line_1 TEXT,
            stg_address_city TEXT,
            prod_address_line_1 TEXT,
            prod_address_city TEXT,
            stg_slug TEXT NOT NULL,
            prod_slug TEXT NOT NULL,
            stg_email TEXT,
            prod_email TEXT,
            stg_phone TEXT,
            prod_phone TEXT,
            prod_created_at TIMESTAMPTZ,
            stg_home_state TEXT,
            prod_home_state TEXT,
            stg_full_name TEXT,
            prod_full_name TEXT,
            was_inserted BOOLEAN NOT NULL,
            match_type TEXT NOT NULL
        )
        "#,
    )
    .execute(pool)
    .await?;

    sqlx::query(
        r#"
        CREATE TABLE ingest_staging.politician_merge_updated (
            id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
            stg_politician_id UUID NOT NULL,
            prod_politician_id UUID NOT NULL,
            stg_full_name TEXT,
            prod_full_name TEXT,
            match_type TEXT NOT NULL,
            created_at TIMESTAMPTZ NOT NULL DEFAULT now()
        )
        "#,
    )
    .execute(pool)
    .await?;

    sqlx::query(
        r#"
        CREATE TABLE ingest_staging.inserted_politicians_with_same_slug (
            id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
            stg_politician_id UUID NOT NULL,
            prod_politician_id UUID NOT NULL,
            full_name TEXT,
            slug TEXT NOT NULL,
            email TEXT,
            phone TEXT,
            address_line_1 TEXT,
            address_city TEXT,
            address_state TEXT,
            address_country TEXT,
            created_at TIMESTAMPTZ NOT NULL DEFAULT now()
        )
        "#,
    )
    .execute(pool)
    .await?;

    sqlx::query(
        r#"
        CREATE TABLE ingest_staging.politician_merge_overwritten_data (
            id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
            prod_politician_id UUID NOT NULL,
            stg_politician_id UUID NOT NULL,
            match_type TEXT NOT NULL,
            prod_slug TEXT,
            prod_full_name TEXT,
            created_at TIMESTAMPTZ NOT NULL DEFAULT now()
        )
        "#,
    )
    .execute(pool)
    .await?;

    Ok(())
}

async fn record_overwritten_politician(
    pool: &PgPool,
    prod_id: uuid::Uuid,
    stg_id: uuid::Uuid,
    match_type: &'static str,
    pre_fetched_prod: Option<&Politician>,
) -> Result<(), Box<dyn std::error::Error>> {
    let prod_owned = if pre_fetched_prod.is_none() {
        Some(Politician::find_by_id(pool, prod_id).await?)
    } else {
        None
    };
    let prod = pre_fetched_prod.unwrap_or_else(|| prod_owned.as_ref().unwrap());
    sqlx::query(
        r#"
        INSERT INTO ingest_staging.politician_merge_overwritten_data (
            prod_politician_id, stg_politician_id, match_type, prod_slug, prod_full_name
        ) VALUES ($1, $2, $3, $4, $5)
        "#,
    )
    .bind(prod_id)
    .bind(stg_id)
    .bind(match_type)
    .bind(&prod.slug)
    .bind(prod.full_name.as_deref())
    .execute(pool)
    .await?;
    Ok(())
}

async fn record_updated_politician(
    pool: &PgPool,
    stg_id: uuid::Uuid,
    stg_full_name: Option<&str>,
    prod_id: uuid::Uuid,
    match_type: &'static str,
    pre_fetched_prod_full_name: Option<&str>,
) -> Result<(), Box<dyn std::error::Error>> {
    let prod_full_name = match pre_fetched_prod_full_name {
        Some(name) => Some(name.to_string()),
        None => Politician::find_by_id(pool, prod_id).await?.full_name,
    };
    sqlx::query(
        r#"
        INSERT INTO ingest_staging.politician_merge_updated (
            stg_politician_id, prod_politician_id, stg_full_name, prod_full_name, match_type
        ) VALUES ($1, $2, $3, $4, $5)
        "#,
    )
    .bind(stg_id)
    .bind(prod_id)
    .bind(stg_full_name)
    .bind(prod_full_name.as_deref())
    .bind(match_type)
    .execute(pool)
    .await?;
    Ok(())
}

async fn record_inserted_politician_with_same_slug(
    pool: &PgPool,
    stg_id: uuid::Uuid,
    prod_id: uuid::Uuid,
    full_name: Option<&str>,
    slug: &str,
    email: Option<&str>,
    phone: Option<&str>,
    address_line_1: Option<&str>,
    address_city: Option<&str>,
    address_state: Option<&str>,
    address_country: Option<&str>,
) -> Result<(), Box<dyn std::error::Error>> {
    sqlx::query(
        r#"
        INSERT INTO ingest_staging.inserted_politicians_with_same_slug (
            stg_politician_id, prod_politician_id, full_name, slug, email, phone,
            address_line_1, address_city, address_state, address_country
        ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
        "#,
    )
    .bind(stg_id)
    .bind(prod_id)
    .bind(full_name)
    .bind(slug)
    .bind(email)
    .bind(phone)
    .bind(address_line_1)
    .bind(address_city)
    .bind(address_state)
    .bind(address_country)
    .execute(pool)
    .await?;
    Ok(())
}

/// Prefer residence address for audit display; fall back to campaign.
async fn fetch_stg_address_for_audit(
    pool: &PgPool,
    stg: &StgPolitician,
) -> Result<Option<StgAddress>, Box<dyn std::error::Error>> {
    if let Some(id) = stg.residence_address_id {
        if let Some(addr) = fetch_stg_mn_address(pool, id).await? {
            return Ok(Some(addr));
        }
    }
    if let Some(id) = stg.campaign_address_id {
        return fetch_stg_mn_address(pool, id).await;
    }
    Ok(None)
}

async fn fetch_prod_address_for_audit(
    pool: &PgPool,
    prod: &Politician,
) -> Result<Option<Address>, Box<dyn std::error::Error>> {
    if let Some(id) = prod.residence_address_id {
        if let Some(addr) = Address::find_by_id(pool, &id).await? {
            return Ok(Some(addr));
        }
    }
    if let Some(id) = prod.campaign_address_id {
        return Address::find_by_id(pool, &id)
            .await
            .map_err(|e| e.into());
    }
    Ok(None)
}

async fn record_merge_dupe(
    pool: &PgPool,
    stg: &StgPolitician,
    prod_id: uuid::Uuid,
    match_type: &'static str,
    pre_fetched_prod_politician: Option<Politician>,
    pre_fetched_stg_address: Option<StgAddress>,
    pre_fetched_prod_address: Option<Address>,
) -> Result<(), Box<dyn std::error::Error>> {
    let prod = match pre_fetched_prod_politician {
        Some(p) => p,
        None => Politician::find_by_id(pool, prod_id).await?,
    };
    let stg_addr = match pre_fetched_stg_address {
        Some(a) => Some(a),
        None => fetch_stg_address_for_audit(pool, stg).await?,
    };
    let prod_addr = match pre_fetched_prod_address {
        Some(a) => Some(a),
        None => fetch_prod_address_for_audit(pool, &prod).await?,
    };
    let (stg_line_1, stg_city) = match &stg_addr {
        Some(a) => (Some(a.line_1.as_str()), Some(a.city.as_str())),
        None => (None, None),
    };
    let (prod_line_1, prod_city) = match &prod_addr {
        Some(a) => (Some(a.line_1.as_str()), Some(a.city.as_str())),
        None => (None, None),
    };
    let prod_home_state_text = prod.home_state.as_ref().map(|s| s.to_string());
    sqlx::query(
        r#"
        INSERT INTO ingest_staging.politician_merge_dupes (
            stg_first_name, stg_middle_name, stg_last_name, stg_suffix,
            prod_first_name, prod_middle_name, prod_last_name, prod_suffix,
            stg_id, prod_id, stg_ref_key, prod_ref_key,
            stg_address_line_1, stg_address_city, prod_address_line_1, prod_address_city,
            stg_slug, prod_slug, stg_email, prod_email, stg_phone, prod_phone,
            prod_created_at, stg_home_state, prod_home_state, stg_full_name, prod_full_name,
            was_inserted, match_type
        ) VALUES (
            $1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16,
            $17, $18, $19, $20, $21, $22, $23, $24, $25, $26, $27, $28, $29
        )
        "#,
    )
    .bind(&stg.first_name)
    .bind(&stg.middle_name)
    .bind(&stg.last_name)
    .bind(&stg.suffix)
    .bind(&prod.first_name)
    .bind(&prod.middle_name)
    .bind(&prod.last_name)
    .bind(&prod.suffix)
    .bind(stg.id)
    .bind(prod.id)
    .bind(&stg.ref_key)
    .bind(&prod.ref_key)
    .bind(stg_line_1)
    .bind(stg_city)
    .bind(prod_line_1)
    .bind(prod_city)
    .bind(&stg.slug)
    .bind(&prod.slug)
    .bind(&stg.email)
    .bind(&prod.email)
    .bind(&stg.phone)
    .bind(&prod.phone)
    .bind(prod.created_at)
    .bind(&stg.home_state)
    .bind(prod_home_state_text.as_deref())
    .bind(&stg.full_name)
    .bind(&prod.full_name)
    .bind(false)
    .bind(match_type)
    .execute(pool)
    .await?;
    Ok(())
}

/// Record dupe/overwritten/updated audit rows, then update the production politician from staging.
async fn apply_match_and_record(
    pool: &PgPool,
    stg: &StgPolitician,
    prod_id: uuid::Uuid,
    match_type: &'static str,
    address_inserted: &mut usize,
    address_reused: &mut usize,
) -> Result<(), Box<dyn std::error::Error>> {
    let prod = Politician::find_by_id(pool, prod_id).await?;
    record_merge_dupe(pool, stg, prod_id, match_type, None, None, None).await?;
    record_overwritten_politician(pool, prod_id, stg.id, match_type, Some(&prod)).await?;
    update_matched_politician_from_staging(
        pool,
        prod_id,
        stg,
        Some(&prod.assets),
        address_inserted,
        address_reused,
    )
    .await?;
    record_updated_politician(
        pool,
        stg.id,
        stg.full_name.as_deref(),
        prod_id,
        match_type,
        Some(prod.full_name.as_deref().unwrap_or("")),
    )
    .await?;
    Ok(())
}

/// When a staging politician matches an existing production politician (by email, phone, or address),
/// update the production row with staging data. first_name, middle_name, last_name, suffix,
/// preferred_name, full_name, home_state, party_id, campaign_website_url are always updated.
/// middle_name is only updated when staging middle_name is non-empty (otherwise existing value is kept).
/// email and phone are only updated when the staging value is not empty or null.
/// Staging residence/campaign addresses are merged to production and linked on the politician.
/// Assets are only updated from staging when current production assets are the empty object '{}'.
async fn update_matched_politician_from_staging(
    pool: &PgPool,
    id: uuid::Uuid,
    stg: &StgPolitician,
    prod_assets: Option<&JSON>,
    address_inserted: &mut usize,
    address_reused: &mut usize,
) -> Result<(), Box<dyn std::error::Error>> {
    let email = stg
        .email
        .as_ref()
        .map(|s| s.trim())
        .filter(|s| !s.is_empty());
    let phone = stg
        .phone
        .as_ref()
        .map(|s| s.trim())
        .filter(|s| !s.is_empty());
    // If stg.middle_name is empty, pass None so existing politician.middle_name is not overwritten
    let middle_name = stg
        .middle_name
        .as_ref()
        .filter(|s| !s.trim().is_empty())
        .cloned();
    let (residence_address_id, campaign_address_id) =
        resolve_stg_addresses_to_production(pool, stg, address_inserted, address_reused).await?;
    let assets = if prod_assets
        .map(|a| a == &serde_json::json!({}))
        .unwrap_or(false)
    {
        stg.assets.clone()
    } else if prod_assets.is_none() {
        let prod = Politician::find_by_id(pool, id).await?;
        if prod.assets == serde_json::json!({}) {
            stg.assets.clone()
        } else {
            None
        }
    } else {
        None
    };
    let input = UpdatePoliticianInput {
        id,
        ref_key: stg.ref_key.clone(),
        slug: None,
        first_name: Some(stg.first_name.clone()),
        middle_name,
        last_name: Some(stg.last_name.clone()),
        suffix: stg.suffix.clone(),
        preferred_name: stg.preferred_name.clone(),
        full_name: stg.full_name.clone(),
        biography: None,
        biography_source: None,
        home_state: parse_state(stg.home_state.as_ref()),
        date_of_birth: None,
        office_id: None,
        upcoming_race_id: None,
        thumbnail_image_url: None,
        assets,
        official_website_url: None,
        ballotpedia_url: None,
        campaign_website_url: stg.campaign_website_url.clone(),
        facebook_url: None,
        twitter_url: None,
        instagram_url: None,
        youtube_url: None,
        linkedin_url: None,
        tiktok_url: None,
        email: email.map(|s| s.to_string()),
        phone: phone.map(|s| s.to_string()),
        party_id: stg.party_id,
        issue_tags: None,
        organization_endorsements: None,
        politician_endorsements: None,
        votesmart_candidate_id: None,
        votesmart_candidate_bio: None,
        votesmart_candidate_ratings: None,
        legiscan_people_id: None,
        crp_candidate_id: None,
        fec_candidate_id: None,
        race_wins: None,
        race_losses: None,
        residence_address_id: None,
        campaign_address_id: None,
    };
    Politician::update(pool, &input).await?;
    Ok(())
}

async fn office_slug_exists(pool: &PgPool, slug: &str) -> Result<bool, Box<dyn std::error::Error>> {
    let row: Option<(uuid::Uuid,)> = sqlx::query_as("SELECT id FROM office WHERE slug = $1")
        .bind(slug)
        .fetch_optional(pool)
        .await?;
    Ok(row.is_some())
}

async fn politician_slug_exists(
    pool: &PgPool,
    slug: &str,
) -> Result<bool, Box<dyn std::error::Error>> {
    let row: Option<(uuid::Uuid,)> = sqlx::query_as("SELECT id FROM politician WHERE slug = $1")
        .bind(slug)
        .fetch_optional(pool)
        .await?;
    Ok(row.is_some())
}

async fn resolve_unique_politician_slug(
    pool: &PgPool,
    base_slug: &str,
) -> Result<String, Box<dyn std::error::Error>> {
    if !politician_slug_exists(pool, base_slug).await? {
        return Ok(base_slug.to_string());
    }
    let mut n = 1u32;
    loop {
        let candidate = format!("{}-{}", base_slug, n);
        if !politician_slug_exists(pool, &candidate).await? {
            return Ok(candidate);
        }
        n += 1;
    }
}

/// Returns (production_politician_id, was_existing).
/// was_existing is true when matched by email, phone, or address.
async fn resolve_or_upsert_politician(
    pool: &PgPool,
    stg: &StgPolitician,
    address_inserted: &mut usize,
    address_reused: &mut usize,
) -> Result<(uuid::Uuid, bool), Box<dyn std::error::Error>> {
    // 1. By email (non-empty) — update existing politician with staging data
    if let Some(email) = &stg.email {
        let email = email.trim();
        if !email.is_empty() {
            let row: Option<(uuid::Uuid,)> = sqlx::query_as(
                "SELECT id FROM politician WHERE LOWER(TRIM(email)) = LOWER($1)",
            )
            .bind(email)
            .fetch_optional(pool)
            .await?;
            if let Some((id,)) = row {
                apply_match_and_record(
                    pool,
                    stg,
                    id,
                    "email",
                    address_inserted,
                    address_reused,
                )
                .await?;
                return Ok((id, true));
            }
        }
    }

    // 2. By phone (non-empty) — update existing politician with staging data
    if let Some(phone) = &stg.phone {
        let phone = phone.trim();
        if !phone.is_empty() {
            let row: Option<(uuid::Uuid,)> =
                sqlx::query_as("SELECT id FROM politician WHERE phone = $1")
                    .bind(phone)
                    .fetch_optional(pool)
                    .await?;
            if let Some((id,)) = row {
                apply_match_and_record(
                    pool,
                    stg,
                    id,
                    "phone",
                    address_inserted,
                    address_reused,
                )
                .await?;
                return Ok((id, true));
            }
        }
    }

    // 3. By address among slug / slug-% candidates — either stg residence or campaign
    // may match either prod residence or campaign
    let base_slug: String =
        base_slug_if_increment(&stg.slug).unwrap_or_else(|| stg.slug.to_string());
    let candidates: Vec<(uuid::Uuid, Option<uuid::Uuid>, Option<uuid::Uuid>)> = sqlx::query_as(
        r#"SELECT id, residence_address_id, campaign_address_id
           FROM politician
           WHERE slug = $1 OR slug LIKE $1 || '-%'"#,
    )
    .bind(&base_slug)
    .fetch_all(pool)
    .await?;

    for (id, prod_residence_address_id, prod_campaign_address_id) in &candidates {
        if do_mn_addresses_match(
            pool,
            stg,
            *prod_residence_address_id,
            *prod_campaign_address_id,
        )
        .await?
        {
            apply_match_and_record(
                pool,
                stg,
                *id,
                "slug + address",
                address_inserted,
                address_reused,
            )
            .await?;
            return Ok((*id, true));
        }
    }

    // 4. Insert via upsert_from_source — new politician
    // If staging slug already exists in production, use slug-1, slug-2, ... until unique
    let (residence_address_id, campaign_address_id) =
        resolve_stg_addresses_to_production(pool, stg, address_inserted, address_reused).await?;
    let slug = resolve_unique_politician_slug(pool, &stg.slug).await?;
    let ref_key = stg
        .ref_key
        .clone()
        .unwrap_or_else(|| format!("mn-sos|{}", slug));
    let input = UpsertPoliticianInput {
        id: None,
        slug: Some(slug.clone()),
        ref_key: Some(ref_key),
        first_name: Some(stg.first_name.clone()),
        middle_name: stg.middle_name.clone(),
        last_name: Some(stg.last_name.clone()),
        suffix: stg.suffix.clone(),
        preferred_name: stg.preferred_name.clone(),
        full_name: stg.full_name.clone(),
        biography: None,
        biography_source: None,
        home_state: parse_state(stg.home_state.as_ref()),
        date_of_birth: None,
        office_id: None,
        upcoming_race_id: None,
        thumbnail_image_url: None,
        assets: stg.assets.clone(),
        official_website_url: None,
        ballotpedia_url: None,
        campaign_website_url: stg.campaign_website_url.clone(),
        facebook_url: None,
        twitter_url: None,
        instagram_url: None,
        youtube_url: None,
        linkedin_url: None,
        tiktok_url: None,
        email: stg.email.clone(),
        phone: stg.phone.clone(),
        party_id: stg.party_id,
        issue_tags: None,
        organization_endorsements: None,
        politician_endorsements: None,
        votesmart_candidate_id: None,
        votesmart_candidate_bio: None,
        votesmart_candidate_ratings: None,
        legiscan_people_id: None,
        crp_candidate_id: None,
        fec_candidate_id: None,
        race_wins: None,
        race_losses: None,
        residence_address_id: None,
        campaign_address_id: None,
    };
    let prod = Politician::upsert_from_source(pool, &input).await?;

    // Record when we inserted alongside existing same-base-slug politicians
    if !candidates.is_empty() || slug != stg.slug {
        let (addr_line_1, addr_city, addr_state, addr_country) = match residence_address_id
            .or(campaign_address_id)
        {
            Some(addr_id) => {
                let addr = Address::find_by_id(pool, &addr_id).await?;
                match addr {
                    Some(a) => (
                        Some(a.line_1.clone()),
                        Some(a.city.clone()),
                        Some(format!("{:?}", a.state)),
                        Some(a.country.clone()),
                    ),
                    None => (None, None, None, None),
                }
            }
            None => (None, None, None, None),
        };
        record_inserted_politician_with_same_slug(
            pool,
            stg.id,
            prod.id,
            stg.full_name.as_deref(),
            &prod.slug,
            stg.email.as_deref(),
            stg.phone.as_deref(),
            addr_line_1.as_deref(),
            addr_city.as_deref(),
            addr_state.as_deref(),
            addr_country.as_deref(),
        )
        .await?;
    }

    Ok((prod.id, false))
}

fn stg_race_to_upsert(stg: &StgRace, prod_office_id: uuid::Uuid) -> UpsertRaceInput {
    UpsertRaceInput {
        id: None,
        slug: Some(stg.slug.clone()),
        title: Some(stg.title.clone()),
        office_id: Some(prod_office_id),
        race_type: parse_race_type(stg.race_type.as_ref()),
        vote_type: parse_vote_type(stg.vote_type.as_ref()),
        party_id: stg.party_id,
        description: None,
        ballotpedia_link: None,
        early_voting_begins_date: None,
        official_website: None,
        state: parse_state(stg.state.as_ref()),
        election_id: stg.election_id,
        winner_ids: None,
        total_votes: None,
        is_special_election: stg.is_special_election,
        num_elect: stg.num_elect,
    }
}
