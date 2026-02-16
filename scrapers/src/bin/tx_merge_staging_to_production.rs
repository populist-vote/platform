//! Merges staging data from ingest_staging.stg_tx_* into production tables (office, politician, race, race_candidates).
//! Run after process_tx_candidate_filings. Resolves by slug for offices/races and by ref_key/slug/email/phone for politicians.

use std::collections::HashMap;
use std::str::FromStr;
use serde_json::Value as JSON;
use slugify::slugify;
use sqlx::PgPool;
use db::{
    Address, Chamber, DistrictType, ElectionScope, InsertAddressInput, Office, Politician,
    PoliticalScope, Race, RaceCandidate, RaceType, State, UpdatePoliticianInput, UpsertOfficeInput,
    UpsertPoliticianInput, UpsertRaceCandidateInput, UpsertRaceInput, VoteType,
};

const ELECTION_YEAR: i32 = 2026;

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
    office_id: Option<uuid::Uuid>,
    assets: Option<JSON>,
    email: Option<String>,
    phone: Option<String>,
    campaign_website_url: Option<String>,
    residence_address_id: Option<uuid::Uuid>,
    /// When true, exact slug match (stg.slug == prod.slug) is treated as same person without requiring email/phone/address.
    treat_exact_slug_as_same_person: bool,
}

#[derive(Clone, sqlx::FromRow, Debug)]
struct StgAddress {
    line_1: String,
    city: String,
    state: String,
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

/// Outcome of resolving a staging politician: exact match (slug+email/phone/address) or new insert.
#[derive(Debug)]
enum PoliticianOutcome {
    ExactMatch { match_type: &'static str },
    NewInsert,
}

#[tokio::main]
async fn main() {
    db::init_pool().await.unwrap();
    let pool = db::pool().await;
    let db = &pool.connection;

    println!("=== Merge TX staging → production ===\n");

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
        "SELECT id, slug, name, title, subtitle, subtitle_short, office_type, chamber, district_type, political_scope, election_scope, state, county, municipality, term_length, district, seat, school_district, hospital_district, priority FROM ingest_staging.stg_tx_offices",
    )
    .fetch_all(pool)
    .await?;

    // merge offices
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
    println!("  Offices: {} existing (matched by slug), {} new", offices_existing, offices_new);

    // 2. Politicians: resolve by email / phone, else upsert; build stg_politician_id -> prod_politician_id
    println!("Merging politicians...");
    let stg_politicians: Vec<StgPolitician> = sqlx::query_as(
        "SELECT id, slug, ref_key, first_name, middle_name, last_name, suffix, preferred_name, full_name, home_state, party_id, office_id, assets, email, phone, campaign_website_url, residence_address_id, treat_exact_slug_as_same_person FROM ingest_staging.stg_tx_politicians",
    )
    .fetch_all(pool)
    .await?;

    create_merge_tables(pool).await?;

    let mut stg_to_prod_politician: HashMap<uuid::Uuid, uuid::Uuid> = HashMap::new();
    let mut exact_match_count = 0usize;
    let mut exact_flagged_count = 0usize;
    let mut addresses_inserted = 0usize;
    let mut addresses_reused = 0usize;
    for stg in &stg_politicians {
        let results = resolve_or_upsert_politician(
            pool,
            stg,
            &stg_to_prod_office,
            &mut addresses_inserted,
            &mut addresses_reused,
        )
        .await?;
        for (prod_id_opt, outcome) in results {
            match &outcome {
                PoliticianOutcome::ExactMatch { match_type } => {
                    exact_match_count += 1;
                    if *match_type == "slug exact + flagged" {
                        exact_flagged_count += 1;
                    }
                }
                PoliticianOutcome::NewInsert => {}
            }
            if let Some(pid) = prod_id_opt {
                stg_to_prod_politician.insert(stg.id, pid);
            }
        }
    }
    println!("  Exact matches (slug+email/phone/address → updated): {}", exact_match_count);
    println!("  Slug exact + flagged: {}", exact_flagged_count);
    println!("  Addresses: {} inserted, {} existing (reused)", addresses_inserted, addresses_reused);

    // 3. Races: upsert by slug with prod office_id; build stg_race_id -> prod_race_id
    println!("Merging races...");
    let stg_races: Vec<StgRace> = sqlx::query_as(
        "SELECT id, slug, title, office_id, state, race_type, vote_type, party_id, election_id, is_special_election, num_elect FROM ingest_staging.stg_tx_races",
    )
    .fetch_all(pool)
    .await?;

    let mut stg_to_prod_race: HashMap<uuid::Uuid, uuid::Uuid> = HashMap::new();
    for stg in &stg_races {
        let prod_office_id = stg_to_prod_office
            .get(&stg.office_id)
            .copied()
            .ok_or_else(|| format!("Staging race {} references unknown office {}", stg.slug, stg.office_id))?;
        let input = stg_race_to_upsert(stg, prod_office_id);
        let prod = Race::upsert_from_source(pool, &input).await?;
        stg_to_prod_race.insert(stg.id, prod.id);
    }
    println!("  Races: {} merged", stg_to_prod_race.len());

    // 4. Race candidates: insert (prod_race_id, prod_candidate_id)
    println!("Merging race_candidates...");
    let stg_rcs: Vec<StgRaceCandidate> = sqlx::query_as(
        "SELECT race_id, candidate_id, ref_key FROM ingest_staging.stg_tx_race_candidates",
    )
    .fetch_all(pool)
    .await?;

    let mut inserted = 0usize;
    let mut skipped_ref_key = 0usize;
    for rc in &stg_rcs {
        let prod_race_id = match stg_to_prod_race.get(&rc.race_id) {
            Some(id) => *id,
            None => continue,
        };
        let prod_candidate_id = match stg_to_prod_politician.get(&rc.candidate_id) {
            Some(id) => *id,
            None => continue,
        };
        // Skip if incoming ref_key already exists in production (avoid duplicate ref_key)
        if let Some(ref key) = rc.ref_key {
            if !key.trim().is_empty() {
                let exists: Option<(i32,)> = sqlx::query_as(
                    "SELECT 1 FROM race_candidates WHERE ref_key = $1 LIMIT 1",
                )
                .bind(key.as_str())
                .fetch_optional(pool)
                .await?;
                if exists.is_some() {
                    skipped_ref_key += 1;
                    continue;
                }
            }
        }
        let input = UpsertRaceCandidateInput {
            race_id: prod_race_id,
            candidate_id: prod_candidate_id,
            ref_key: rc.ref_key.clone(),
            is_running: None,
        };
        if RaceCandidate::upsert_from_source(pool, &input).await?.is_some() {
            inserted += 1;
        }
    }
    println!("  Race_candidates: {} new links", inserted);
    println!("  Race_candidates skipped (existing ref_key): {}", skipped_ref_key);

    Ok(())
}

fn parse_state(s: Option<&String>) -> Option<State> {
    s.and_then(|s| State::from_str(s.trim()).ok())
}

/// Only treat as same person when home states match. If either is missing, we don't reject on state.
fn same_home_state(stg: Option<State>, prod: Option<State>) -> bool {
    match (stg, prod) {
        (Some(a), Some(b)) => a == b,
        _ => true,
    }
}

/// True if slugs are equal or one is the other with a numeric suffix (e.g. "nameslug" and "nameslug-1", or "nameslug-2" and "nameslug").
fn same_slug_or_increment(a: &str, b: &str) -> bool {
    if a == b {
        return true;
    }
    let suffix_is_numeric = |s: &str| s.chars().all(|c| c.is_ascii_digit()) && !s.is_empty();
    // a is "b-N"?
    if a.len() > b.len() + 1 && a.starts_with(b) && a.as_bytes()[b.len()] == b'-' {
        if suffix_is_numeric(&a[b.len() + 1..]) {
            return true;
        }
    }
    // b is "a-N"?
    if b.len() > a.len() + 1 && b.starts_with(a) && b.as_bytes()[a.len()] == b'-' {
        if suffix_is_numeric(&b[a.len() + 1..]) {
            return true;
        }
    }
    false
}

/// If slug has form "base-N" (N numeric), return Some(base); else None.
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

/// When a staging politician matches an existing production politician (by email or phone),
/// update the production row with staging data. first_name, last_name, suffix, preferred_name,
/// full_name, home_state, party_id, campaign_website_url are updated from staging. middle_name
/// is only updated when staging middle_name is non-empty (otherwise existing value is kept).
/// email and phone are only updated when the staging value is not empty or null.
/// If the incoming politician has a non-empty address, merge it to production and set residence_address_id.
/// If stg.office_id is set, converts it to prod office_id via stg_to_prod_office and sets office_id on the update.
/// Assets are only updated from staging when current production assets are the empty object '{}'.
async fn update_matched_politician_from_staging(
    pool: &PgPool,
    id: uuid::Uuid,
    stg: &StgPolitician,
    stg_to_prod_office: &HashMap<uuid::Uuid, uuid::Uuid>,
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
    // If incoming has an address, merge it to production and update the politician's residence_address_id
    let residence_address_id = match stg.residence_address_id {
        Some(stg_addr_id) => {
            let stg_addr = fetch_stg_tx_address(pool, stg_addr_id).await?;
            match stg_addr {
                Some(addr) => {
                    let (addr_id, was_inserted) = merge_staging_address_to_production(pool, &addr).await?;
                    if was_inserted {
                        *address_inserted += 1;
                    } else {
                        *address_reused += 1;
                    }
                    Some(addr_id)
                }
                None => None,
            }
        }
        None => None,
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
        office_id: stg.office_id.and_then(|stg_oid| stg_to_prod_office.get(&stg_oid).copied()),
        upcoming_race_id: None,
        thumbnail_image_url: None,
        assets: if prod_assets.map(|a| a == &serde_json::json!({})).unwrap_or(false) {
            stg.assets.clone()
        } else {
            None
        },
        official_website_url: None,
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
        residence_address_id,
        campaign_address_id: None,
    };
    Politician::update(pool, &input).await?;
    Ok(())
}

/// Compare staging and production residence addresses. Returns (address_match, stg_addr_opt, prod_addr_opt)
/// so the caller can reuse the fetched addresses (e.g. for record_merge_dupe) and avoid re-fetching.
/// address_match: false = reject (addresses missing or differ); true = accept (addresses match).
/// both missing or one missing => (false, None, None)
/// both present and different => (false, Some(stg), Some(prod))
/// both present and same => (true, Some(stg), Some(prod))
async fn do_addresses_match(
    pool: &PgPool,
    stg_residence_address_id: Option<uuid::Uuid>,
    prod_residence_address_id: Option<uuid::Uuid>,
) -> Result<
    (
        bool,
        Option<StgAddress>,
        Option<Address>,
    ),
    Box<dyn std::error::Error>,
> {
    let (Some(stg_id), Some(prod_id)) = (stg_residence_address_id, prod_residence_address_id) else {
        return Ok((false, None, None));
    };
    let stg_addr = match fetch_stg_tx_address(pool, stg_id).await? {
        Some(a) => a,
        None => return Ok((false, None, None)),
    };
    let prod_addr = match Address::find_by_id(pool, &prod_id).await? {
        Some(a) => a,
        None => return Ok((false, None, None)),
    };
    let norm = |s: &str| s.trim().to_lowercase();
    let stg_state = parse_state(Some(&stg_addr.state));
    let same = norm(&stg_addr.line_1) == norm(&prod_addr.line_1)
        && norm(&stg_addr.city) == norm(&prod_addr.city)
        && stg_state == Some(prod_addr.state);
    // Return address_match: true when both present and same (accept), false when both present and different or either missing (reject)
    Ok((same, Some(stg_addr), Some(prod_addr)))
}

/// Create or recreate all ingest_staging tables used by the TX merge (politician audit tables).
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

/// Record a phase2 dupe row. Pass pre-fetched address data when already loaded (e.g. from do_addresses_match)
/// to avoid re-fetching; pass None for any not yet loaded and they will be fetched here.
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
        None => match stg.residence_address_id {
            Some(id) => fetch_stg_tx_address(pool, id).await?,
            None => None,
        },
    };
    let prod_addr = match pre_fetched_prod_address {
        Some(a) => Some(a),
        None => match prod.residence_address_id {
            Some(id) => Address::find_by_id(pool, &id).await?,
            None => None,
        },
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

async fn fetch_stg_tx_address(
    pool: &PgPool,
    residence_address_id: uuid::Uuid,
) -> Result<Option<StgAddress>, Box<dyn std::error::Error>> {
    let row = sqlx::query_as(
        "SELECT line_1, city, state, country FROM ingest_staging.stg_tx_addresses WHERE id = $1",
    )
    .bind(residence_address_id)
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
    let state = parse_state(Some(&stg.state)).unwrap_or(State::TX);
    let postal_code = String::new();
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

async fn office_slug_exists(pool: &PgPool, slug: &str) -> Result<bool, Box<dyn std::error::Error>> {
    let row: Option<(uuid::Uuid,)> = sqlx::query_as(
        "SELECT id FROM office WHERE slug = $1",
    )
    .bind(slug)
    .fetch_optional(pool)
    .await?;
    Ok(row.is_some())
}

/// For a candidate that matched (email, phone, or address): record phase2 dupe, record overwritten,
/// update prod politician from staging, record updated politician, and push to updated list.
/// Fetches the production politician once and passes it to both record_overwritten and record_updated.
async fn apply_match_and_record(
    pool: &PgPool,
    stg: &StgPolitician,
    prod_id: uuid::Uuid,
    match_type: &'static str,
    pre_fetched_stg_address: Option<StgAddress>,
    pre_fetched_prod_address: Option<Address>,
    stg_to_prod_office: &HashMap<uuid::Uuid, uuid::Uuid>,
    address_inserted: &mut usize,
    address_reused: &mut usize,
    updated: &mut Vec<(uuid::Uuid, &'static str)>,
) -> Result<(), Box<dyn std::error::Error>> {
    let prod = Politician::find_by_id(pool, prod_id).await?;
    record_merge_dupe(
        pool,
        stg,
        prod_id,
        match_type,
        None,
        pre_fetched_stg_address,
        pre_fetched_prod_address,
    )
    .await?;
    record_overwritten_politician(pool, prod_id, stg.id, match_type, Some(&prod)).await?;
    update_matched_politician_from_staging(pool, prod_id, stg, stg_to_prod_office, Some(&prod.assets), address_inserted, address_reused).await?;
    record_updated_politician(
        pool,
        stg.id,
        stg.full_name.as_deref(),
        prod_id,
        match_type,
        Some(prod.full_name.as_deref().unwrap_or("")),
    )
    .await?;
    updated.push((prod_id, match_type));
    Ok(())
}

/// Returns a list of (prod_id_opt, outcome). Usually one element.
async fn resolve_or_upsert_politician(
    pool: &PgPool,
    stg: &StgPolitician,
    stg_to_prod_office: &HashMap<uuid::Uuid, uuid::Uuid>,
    address_inserted: &mut usize,
    address_reused: &mut usize,
) -> Result<Vec<(Option<uuid::Uuid>, PoliticianOutcome)>, Box<dyn std::error::Error>> {
    // Find ALL prod politicians with matching slug or slug increment; for each test email, phone, address; record_merge_dupe; if any match → update and record to politician_merge_updated; else insert (and record to inserted_politicians_with_same_slug only when we had slug candidates)
    let base_slug: String = base_slug_if_increment(&stg.slug).unwrap_or_else(|| stg.slug.to_string());
    let candidates: Vec<(uuid::Uuid, String, Option<State>, Option<uuid::Uuid>, Option<String>, Option<String>)> = sqlx::query_as(
        r#"SELECT id, slug, home_state AS "home_state:State", residence_address_id, email, phone FROM politician WHERE slug = $1 OR slug LIKE $1 || '-%'"#,
    )
    .bind(&base_slug)
    .fetch_all(pool)
    .await?;

    let mut pre_fetched_stg_addr: Option<StgAddress> = None;
    let stg_email_trimmed = stg.email.as_ref().map(|s| s.trim()).filter(|s| !s.is_empty());
    let stg_phone_trimmed = stg.phone.as_ref().map(|s| s.trim()).filter(|s| !s.is_empty());
    let mut updated: Vec<(uuid::Uuid, &'static str)> = Vec::new();

    for (id, prod_slug, _prod_home_state, prod_residence_address_id, prod_email, prod_phone) in &candidates {
        if stg.treat_exact_slug_as_same_person && stg.slug == *prod_slug {
            apply_match_and_record(
                pool,
                stg,
                *id,
                "slug exact + flagged",
                None,
                None,
                stg_to_prod_office,
                address_inserted,
                address_reused,
                &mut updated,
            )
            .await?;
            break;
        }

        let prod_email_trimmed = prod_email.as_ref().map(|s| s.trim()).filter(|s| !s.is_empty());
        let prod_phone_trimmed = prod_phone.as_ref().map(|s| s.trim()).filter(|s| !s.is_empty());

        if stg_email_trimmed.is_some() && prod_email_trimmed.is_some() && stg_email_trimmed == prod_email_trimmed {
            apply_match_and_record(pool, stg, *id, "slug + email", None, None, stg_to_prod_office, address_inserted, address_reused, &mut updated).await?;
        } else if stg_phone_trimmed.is_some() && prod_phone_trimmed.is_some() && stg_phone_trimmed == prod_phone_trimmed {
            apply_match_and_record(pool, stg, *id, "slug + phone", None, None, stg_to_prod_office, address_inserted, address_reused, &mut updated).await?;
        } else {
            let (address_match, stg_addr_opt, prod_addr_opt) =
                do_addresses_match(pool, stg.residence_address_id, *prod_residence_address_id).await?;
            if pre_fetched_stg_addr.is_none() {
                pre_fetched_stg_addr = stg_addr_opt.clone();
            }
            if address_match {
                apply_match_and_record(
                    pool,
                    stg,
                    *id,
                    "slug + address",
                    stg_addr_opt,
                    prod_addr_opt,
                    stg_to_prod_office,
                    address_inserted,
                    address_reused,
                    &mut updated,
                )
                .await?;
            }
        }
    }

    if !updated.is_empty() {
        return Ok(updated
            .into_iter()
            .map(|(id, match_type)| (Some(id), PoliticianOutcome::ExactMatch { match_type }))
            .collect());
    }

    // No email/phone/address match for any slug candidate → insert incoming politician (with resolved slug) and record to inserted_politicians_with_same_slug

    // 4. Insert via upsert_from_source — new politician

    // start with inserting the staging address into production
    let residence_address_id = if let Some(addr) = pre_fetched_stg_addr {
        let (id, was_inserted) = merge_staging_address_to_production(pool, &addr).await?;
        if was_inserted {
            *address_inserted += 1;
        } else {
            *address_reused += 1;
        }
        Some(id)
    } else {
        match stg.residence_address_id {
            Some(stg_addr_id) => {
                let stg_addr = fetch_stg_tx_address(pool, stg_addr_id).await?;
                match stg_addr {
                    Some(addr) => {
                        let (id, was_inserted) = merge_staging_address_to_production(pool, &addr).await?;
                        if was_inserted {
                            *address_inserted += 1;
                        } else {
                            *address_reused += 1;
                        }
                        Some(id)
                    }
                    None => None,
                }
            }
            None => None,
        }
    };

    let ref_key = stg.ref_key.clone().unwrap_or_else(|| {
        let name = stg
            .full_name
            .as_deref()
            .filter(|s| !s.trim().is_empty())
            .unwrap_or(&stg.slug);
        format!("tx-sos-{}-{}", ELECTION_YEAR, slugify!(name))
    });
    let input = UpsertPoliticianInput {
        id: None,
        slug: Some(stg.slug.clone()),
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
        office_id: stg.office_id.and_then(|stg_oid| stg_to_prod_office.get(&stg_oid).copied()),
        upcoming_race_id: None,
        thumbnail_image_url: None,
        assets: stg.assets.clone(),
        official_website_url: None,
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
        residence_address_id,
        campaign_address_id: None,
    };
    let prod = Politician::upsert_from_source(pool, &input).await?;
    if !candidates.is_empty() {
        let (addr_line_1, addr_city, addr_state, addr_country) = match residence_address_id {
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
    Ok(vec![(Some(prod.id), PoliticianOutcome::NewInsert)])
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
