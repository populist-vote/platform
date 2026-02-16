//! Merges staging data from ingest_staging.stg_tx_* into production tables (office, politician, race, race_candidates).
//! Run after process_tx_candidate_filings. Resolves by slug for offices/races and by ref_key/slug/email/phone for politicians.

use std::collections::HashMap;
use std::str::FromStr;
use sqlx::PgPool;
use db::{
    Address, Chamber, DistrictType, ElectionScope, InsertAddressInput, Office, Politician,
    PoliticalScope, Race, RaceCandidate, RaceType, State, UpdatePoliticianInput, UpsertOfficeInput,
    UpsertPoliticianInput, UpsertRaceCandidateInput, UpsertRaceInput, VoteType,
};

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
    residence_address_id: Option<uuid::Uuid>,
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

/// Outcome of resolving a staging politician: exact match (email/phone/slug), new insert, or questionable match skipped.
#[derive(Debug)]
enum PoliticianOutcome {
    ExactMatch { match_type: &'static str },
    NewInsert,
    /// Same slug + same home_state but address didn't match; saved to dupe_politicians_during_merge, not merged.
    QuestionableSlugMatchSkipped,
    /// Phone match but incoming slug matches (or is increment of) existing slug; saved to dupe table, not merged.
    QuestionablePhoneMatchSkipped,
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
        "SELECT id, slug, ref_key, first_name, middle_name, last_name, suffix, preferred_name, full_name, home_state, party_id, email, phone, campaign_website_url, residence_address_id FROM ingest_staging.stg_tx_politicians",
    )
    .fetch_all(pool)
    .await?;

    ensure_politician_phase2_dupes_table(pool).await?;
    ensure_dupe_politicians_during_merge_table(pool).await?;

    let mut stg_to_prod_politician: HashMap<uuid::Uuid, uuid::Uuid> = HashMap::new();
    let mut exact_match_count = 0usize;
    let mut questionable_slug_skipped_count = 0usize;
    let mut questionable_phone_skipped_count = 0usize;
    for stg in &stg_politicians {
        let results = resolve_or_upsert_politician(pool, stg).await?;
        for (prod_id_opt, outcome) in results {
            match &outcome {
                PoliticianOutcome::ExactMatch { .. } => exact_match_count += 1,
                PoliticianOutcome::QuestionableSlugMatchSkipped => questionable_slug_skipped_count += 1,
                PoliticianOutcome::QuestionablePhoneMatchSkipped => questionable_phone_skipped_count += 1,
                PoliticianOutcome::NewInsert => {}
            }
            if let Some(pid) = prod_id_opt {
                stg_to_prod_politician.insert(stg.id, pid);
            }
        }
    }
    println!("  Exact matches (email, phone, or slug+address → updated): {}", exact_match_count);
    println!("  Questionable slug matches skipped (saved to dupe_politicians_during_merge): {}", questionable_slug_skipped_count);
    println!("  Questionable phone matches skipped (slug overlap, saved to dupe_politicians_during_merge): {}", questionable_phone_skipped_count);

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
        if RaceCandidate::upsert_from_source(pool, &input).await?.is_some() {
            inserted += 1;
        }
    }
    println!("  Race_candidates: {} new links", inserted);

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
async fn update_matched_politician_from_staging(
    pool: &PgPool,
    id: uuid::Uuid,
    stg: &StgPolitician,
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
        assets: None,
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
        residence_address_id: None,
        campaign_address_id: None,
    };
    Politician::update(pool, &input).await?;
    Ok(())
}

/// Compare staging and production residence addresses. Returns (address_match, stg_addr_opt, prod_addr_opt)
/// so the caller can reuse the fetched addresses (e.g. for record_phase2_dupe) and avoid re-fetching.
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

async fn ensure_politician_phase2_dupes_table(pool: &PgPool) -> Result<(), Box<dyn std::error::Error>> {
    sqlx::query("DROP TABLE IF EXISTS ingest_staging.politician_phase2_dupes")
        .execute(pool)
        .await?;
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS ingest_staging.politician_phase2_dupes (
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
    Ok(())
}

async fn ensure_dupe_politicians_during_merge_table(pool: &PgPool) -> Result<(), Box<dyn std::error::Error>> {
    sqlx::query("DROP TABLE IF EXISTS ingest_staging.dupe_politicians_during_merge")
        .execute(pool)
        .await?;
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS ingest_staging.dupe_politicians_during_merge (
            id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
            stg_politician_id UUID NOT NULL,
            prod_politician_id UUID NOT NULL,
            stg_slug TEXT NOT NULL,
            stg_full_name TEXT,
            stg_email TEXT,
            stg_phone TEXT,
            stg_home_state TEXT,
            prod_slug TEXT NOT NULL,
            prod_full_name TEXT,
            created_at TIMESTAMPTZ NOT NULL DEFAULT now()
        )
        "#,
    )
    .execute(pool)
    .await?;
    Ok(())
}

async fn record_questionable_slug_match(
    pool: &PgPool,
    stg: &StgPolitician,
    prod_id: uuid::Uuid,
) -> Result<(), Box<dyn std::error::Error>> {
    let prod = Politician::find_by_id(pool, prod_id).await?;
    sqlx::query(
        r#"
        INSERT INTO ingest_staging.dupe_politicians_during_merge (
            stg_politician_id, prod_politician_id, stg_slug, stg_full_name, stg_email, stg_phone, stg_home_state,
            prod_slug, prod_full_name
        ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
        "#,
    )
    .bind(stg.id)
    .bind(prod_id)
    .bind(&stg.slug)
    .bind(&stg.full_name)
    .bind(&stg.email)
    .bind(&stg.phone)
    .bind(&stg.home_state)
    .bind(&prod.slug)
    .bind(&prod.full_name)
    .execute(pool)
    .await?;
    Ok(())
}

/// Record a phase2 dupe row. Pass pre-fetched address data when already loaded (e.g. from do_addresses_match)
/// to avoid re-fetching; pass None for any not yet loaded and they will be fetched here.
async fn record_phase2_dupe(
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
        INSERT INTO ingest_staging.politician_phase2_dupes (
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

/// Insert the staging address into production address (or return existing id by unique key).
/// Returns the production address id.
async fn merge_staging_address_to_production(
    pool: &PgPool,
    stg: &StgAddress,
) -> Result<uuid::Uuid, Box<dyn std::error::Error>> {
    let state = parse_state(Some(&stg.state)).unwrap_or(State::TX);
    let input = InsertAddressInput {
        line_1: stg.line_1.clone(),
        line_2: None,
        city: stg.city.clone(),
        state,
        country: stg.country.clone(),
        postal_code: String::new(),
        county: None,
        congressional_district: None,
        state_senate_district: None,
        state_house_district: None,
        lon: None,
        lat: None,
    };
    let addr = Address::upsert(pool, &input).await?;
    Ok(addr.id)
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

async fn politician_slug_exists(pool: &PgPool, slug: &str) -> Result<bool, Box<dyn std::error::Error>> {
    let row: Option<(uuid::Uuid,)> = sqlx::query_as(
        "SELECT id FROM politician WHERE slug = $1",
    )
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

/// Returns a list of (prod_id_opt, outcome). Usually one element; step 3 (slug) can return multiple when several slug-match candidates are questionable.
async fn resolve_or_upsert_politician(
    pool: &PgPool,
    stg: &StgPolitician,
) -> Result<Vec<(Option<uuid::Uuid>, PoliticianOutcome)>, Box<dyn std::error::Error>> {
    // 1. Match by email (non-empty) — record dupe, update existing
    if let Some(email) = &stg.email {
        let email = email.trim();
        if !email.is_empty() {
            let row: Option<(uuid::Uuid,)> = sqlx::query_as(
                "SELECT id FROM politician WHERE email = $1",
            )
            .bind(email)
            .fetch_optional(pool)
            .await?;
            if let Some((id,)) = row {
                record_phase2_dupe(pool, stg, id, "email", None, None, None).await?;
                update_matched_politician_from_staging(pool, id, stg).await?;
                return Ok(vec![(Some(id), PoliticianOutcome::ExactMatch { match_type: "email" })]);
            }
        }
    }

    // 2. Match by phone (non-empty) — if prod slug matches or is increment of stg slug, same person; else questionable
    if let Some(phone) = &stg.phone {
        let phone = phone.trim();
        if !phone.is_empty() {
            let row: Option<(uuid::Uuid, String)> = sqlx::query_as(
                "SELECT id, slug FROM politician WHERE phone = $1",
            )
            .bind(phone)
            .fetch_optional(pool)
            .await?;
            if let Some((id, prod_slug)) = row {
                if same_slug_or_increment(&stg.slug, &prod_slug) {
                    record_phase2_dupe(pool, stg, id, "phone", None, None, None).await?;
                    update_matched_politician_from_staging(pool, id, stg).await?;
                    return Ok(vec![(Some(id), PoliticianOutcome::ExactMatch { match_type: "phone" })]);
                }
                record_questionable_slug_match(pool, stg, id).await?;
                record_phase2_dupe(pool, stg, id, "phone", None, None, None).await?;
                return Ok(vec![(None, PoliticianOutcome::QuestionablePhoneMatchSkipped)]);
            }
        }
    }

    // 3. After email and phone: find ALL slug matches using base of stg.slug (base, base-1, base-2, ...), then test home_state and address on each
    let stg_home_state = parse_state(stg.home_state.as_ref());
    // use the base slug of the staging politician's slug, or the slug itself if it doesn't have a numeric suffix
    let base_slug: String = base_slug_if_increment(&stg.slug).unwrap_or_else(|| stg.slug.to_string());
    // find all slug matches in production politicians using the base slug (base, base-1, base-2, ...)
    let candidates: Vec<(uuid::Uuid, String, Option<State>, Option<uuid::Uuid>)> = sqlx::query_as(
        r#"SELECT id, slug, home_state AS "home_state:State", residence_address_id FROM politician WHERE slug = $1 OR slug LIKE $1 || '-%'"#,
    )
    .bind(&base_slug)
    .fetch_all(pool)
    .await?;

    let mut pre_fetched_stg_addr: Option<StgAddress> = None;
    let mut questionable_slug_results: Vec<(Option<uuid::Uuid>, PoliticianOutcome)> = Vec::new();
    
    // loop through every candidate with a matching slug (or increment) and tests home_state and address matching
    for (id, _prod_slug, prod_home_state, prod_residence_address_id) in &candidates {
        if same_home_state(stg_home_state, *prod_home_state) {
            let (address_match, stg_addr_opt, prod_addr_opt) =
                do_addresses_match(pool, stg.residence_address_id, *prod_residence_address_id).await?;
            if address_match {
                record_phase2_dupe(
                    pool,
                    stg,
                    *id,
                    "slug + address",
                    None,
                    stg_addr_opt,
                    prod_addr_opt,
                )
                .await?;
                update_matched_politician_from_staging(pool, *id, stg).await?;
                // we found the exact person based on our accepted criteria, other questionable slug matches do not matter and are not logged
                return Ok(vec![(Some(*id), PoliticianOutcome::ExactMatch { match_type: "slug + address" })]);
            }
            if pre_fetched_stg_addr.is_none() {
                pre_fetched_stg_addr = stg_addr_opt.clone();
            }
            record_phase2_dupe(
                pool,
                stg,
                *id,
                "slug",
                None,
                stg_addr_opt,
                prod_addr_opt,
            )
            .await?;
            record_questionable_slug_match(pool, stg, *id).await?;
            questionable_slug_results.push((None, PoliticianOutcome::QuestionableSlugMatchSkipped));
        }
    }
    if !questionable_slug_results.is_empty() {
        return Ok(questionable_slug_results);
    }
    // No exact match and no questionables → insert incoming politician

    // 4. Insert/update via upsert_from_source — new politician

    // start with inserting the staging address into production
    let residence_address_id = if let Some(addr) = pre_fetched_stg_addr {
        Some(merge_staging_address_to_production(pool, &addr).await?)
    } else {
        match stg.residence_address_id {
            Some(stg_addr_id) => {
                let stg_addr = fetch_stg_tx_address(pool, stg_addr_id).await?;
                match stg_addr {
                    Some(addr) => Some(merge_staging_address_to_production(pool, &addr).await?),
                    None => None,
                }
            }
            None => None,
        }
    };

    let slug = resolve_unique_politician_slug(pool, &stg.slug).await?;
    let ref_key = stg
        .ref_key
        .clone()
        .unwrap_or_else(|| format!("tx-sos|{}", slug));
    let input = UpsertPoliticianInput {
        id: None,
        slug: Some(slug),
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
        assets: None,
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
