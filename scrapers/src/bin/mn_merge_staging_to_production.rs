//! Merges staging data from ingest_staging.stg_mn_* into production tables (office, politician, race, race_candidates).
//! Run after process_mn_candidate_filings. Resolves by slug for offices/races and by ref_key/slug/email/phone for politicians.

use std::collections::HashMap;
use std::str::FromStr;
use sqlx::PgPool;
use db::{
    Chamber, DistrictType, ElectionScope, Office, Politician, PoliticalScope, Race, RaceCandidate,
    RaceType, State, UpdatePoliticianInput, UpsertOfficeInput, UpsertPoliticianInput,
    UpsertRaceCandidateInput, UpsertRaceInput, VoteType,
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
    println!("  Offices: {} existing (matched by slug), {} new", offices_existing, offices_new);

    // 2. Politicians: resolve by email / phone, else upsert; build stg_politician_id -> prod_politician_id
    println!("Merging politicians...");
    let stg_politicians: Vec<StgPolitician> = sqlx::query_as(
        "SELECT id, slug, ref_key, first_name, middle_name, last_name, suffix, preferred_name, full_name, home_state, party_id, email, phone, campaign_website_url FROM ingest_staging.stg_mn_politicians",
    )
    .fetch_all(pool)
    .await?;

    let mut stg_to_prod_politician: HashMap<uuid::Uuid, uuid::Uuid> = HashMap::new();
    let mut politicians_existing = 0usize;
    let mut politicians_new = 0usize;
    for stg in &stg_politicians {
        let (prod_id, was_existing) = resolve_or_upsert_politician(pool, stg).await?;
        if was_existing {
            politicians_existing += 1;
        } else {
            politicians_new += 1;
        }
        stg_to_prod_politician.insert(stg.id, prod_id);
    }
    println!("  Politicians: {} existing (matched by email/phone), {} new", politicians_existing, politicians_new);

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
            .ok_or_else(|| format!("Staging race {} references unknown office {}", stg.slug, stg.office_id))?;
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
/// update the production row with staging data. first_name, middle_name, last_name, suffix,
/// preferred_name, full_name, home_state, party_id, campaign_website_url are always updated.
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
    let input = UpdatePoliticianInput {
        id,
        slug: None,
        first_name: Some(stg.first_name.clone()),
        middle_name: stg.middle_name.clone(),
        last_name: Some(stg.last_name.clone()),
        suffix: stg.suffix.clone(),
        preferred_name: stg.preferred_name.clone(),
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
    };
    Politician::update(pool, &input).await?;
    // full_name is not on UpdatePoliticianInput; update via raw SQL
    if stg.full_name.is_some() {
        sqlx::query("UPDATE politician SET full_name = $1 WHERE id = $2")
            .bind(&stg.full_name)
            .bind(id)
            .execute(pool)
            .await?;
    }
    Ok(())
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

/// Returns (production_politician_id, was_existing). was_existing is true when matched by email or phone.
async fn resolve_or_upsert_politician(
    pool: &PgPool,
    stg: &StgPolitician,
) -> Result<(uuid::Uuid, bool), Box<dyn std::error::Error>> {
    // 1. By email (non-empty) — update existing politician with staging data
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
                update_matched_politician_from_staging(pool, id, stg).await?;
                return Ok((id, true));
            }
        }
    }

    // 2. By phone (non-empty) — update existing politician with staging data
    if let Some(phone) = &stg.phone {
        let phone = phone.trim();
        if !phone.is_empty() {
            let row: Option<(uuid::Uuid,)> = sqlx::query_as(
                "SELECT id FROM politician WHERE phone = $1",
            )
            .bind(phone)
            .fetch_optional(pool)
            .await?;
            if let Some((id,)) = row {
                update_matched_politician_from_staging(pool, id, stg).await?;
                return Ok((id, true));
            }
        }
    }

    // 3. Insert/update via upsert_from_source (requires ref_key and slug) — new politician
    // If staging slug already exists in production, use slug-1, slug-2, ... until unique
    let slug = resolve_unique_politician_slug(pool, &stg.slug).await?;
    let ref_key = stg
        .ref_key
        .clone()
        .unwrap_or_else(|| format!("mn-sos|{}", slug));
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
    };
    let prod = Politician::upsert_from_source(pool, &input).await?;
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
