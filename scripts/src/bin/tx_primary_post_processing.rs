//! Post-primary script: (1) Find races that require a runoff, create runoff races and candidates;
//! (2) Create general election races and candidates for primary winners (no runoff).
//!
//! Only runs for races in STATE with election_id = PRIMARY_ELECTION_ID. Use `--dry-run` to write
//! to ingest_staging tables instead of production.

use std::error::Error;
use std::process;

/// State to run for (e.g. only races where state = STATE).
const STATE: &str = "TX";
/// Election id for the primary election; only races with this election_id are considered.
const PRIMARY_ELECTION_ID: &str = "0d586931-c119-4fe7-814f-f679e91282a8";
/// Election id assigned to newly created runoff races.
const PRIMARY_RUNOFF_ELECTION_ID: &str = "569fe90b-bbb2-445a-91a2-08d6d0215898";
/// Election id for the general election; new general races get this election_id.
const GENERAL_ELECTION_ID: &str = "6138cc76-f273-43cf-a017-a98d1119b0c3";

#[derive(Debug, sqlx::FromRow)]
struct ProcessingResult {
    n_runoff_races: i64,
    n_runoff_candidates: i64,
    n_general_races: i64,
    n_general_candidates: i64,
}

async fn ensure_staging_tables(pool: &sqlx::PgPool) -> Result<(), Box<dyn Error>> {
    sqlx::query("CREATE SCHEMA IF NOT EXISTS ingest_staging")
        .execute(pool)
        .await?;

    sqlx::query("DROP TABLE IF EXISTS ingest_staging.tx_primary_runoff_race_candidates")
        .execute(pool)
        .await?;
    sqlx::query("DROP TABLE IF EXISTS ingest_staging.tx_primary_runoff_races")
        .execute(pool)
        .await?;

    sqlx::query(
        r#"
        CREATE TABLE ingest_staging.tx_primary_runoff_races (
            id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
            source_race_id UUID NOT NULL,
            title TEXT NOT NULL,
            slug TEXT NOT NULL,
            office_id UUID NOT NULL,
            race_type TEXT NOT NULL,
            election_id UUID NOT NULL,
            created_at TIMESTAMPTZ NOT NULL DEFAULT now()
        )
        "#,
    )
    .execute(pool)
    .await?;

    sqlx::query(
        r#"
        CREATE TABLE ingest_staging.tx_primary_runoff_race_candidates (
            race_id UUID NOT NULL,
            candidate_id UUID NOT NULL,
            PRIMARY KEY (race_id, candidate_id)
        )
        "#,
    )
    .execute(pool)
    .await?;

    sqlx::query("DROP TABLE IF EXISTS ingest_staging.tx_general_race_candidates")
        .execute(pool)
        .await?;
    sqlx::query("DROP TABLE IF EXISTS ingest_staging.tx_general_races")
        .execute(pool)
        .await?;
    sqlx::query(
        r#"
        CREATE TABLE ingest_staging.tx_general_races (
            id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
            source_race_id UUID NOT NULL,
            title TEXT NOT NULL,
            slug TEXT NOT NULL UNIQUE,
            office_id UUID NOT NULL,
            race_type TEXT NOT NULL,
            election_id UUID NOT NULL,
            num_elect INT,
            created_at TIMESTAMPTZ NOT NULL DEFAULT now()
        )
        "#,
    )
    .execute(pool)
    .await?;
    sqlx::query(
        r#"
        CREATE TABLE ingest_staging.tx_general_race_candidates (
            race_id UUID NOT NULL,
            candidate_id UUID NOT NULL,
            PRIMARY KEY (race_id, candidate_id)
        )
        "#,
    )
    .execute(pool)
    .await?;

    Ok(())
}

/// Shared CTE: ranked candidates and runoff_races (same for both dry-run and production).
/// Only considers races with election_id = PRIMARY_ELECTION_ID.
fn runoff_base_cte() -> String {
    format!(
        r#"
ranked AS (
    SELECT
        rc.race_id,
        rc.candidate_id,
        rc.votes,
        r.total_votes,
        ROW_NUMBER() OVER (
            PARTITION BY rc.race_id
            ORDER BY rc.votes DESC
        ) AS rn
    FROM race_candidates rc
    JOIN race r ON rc.race_id = r.id
    WHERE r.election_id = '{}'::uuid
      AND r.state::text = '{}'
      AND r.num_precincts_reporting = r.total_precincts
      AND r.total_precincts IS NOT NULL
      AND r.num_precincts_reporting IS NOT NULL
      AND rc.votes IS NOT NULL
      AND rc.votes > 0
),
runoff_races AS (
    SELECT
        r.id AS race_id,
        r.title,
        r.slug,
        r.office_id,
        r.race_type,
        r.party_id,
        r.state,
        ARRAY_AGG(ranked.candidate_id ORDER BY ranked.votes DESC) AS top_two,
        COUNT(DISTINCT rc_all.candidate_id) AS total_candidates
    FROM ranked
    JOIN race r ON ranked.race_id = r.id
    JOIN race_candidates rc_all ON rc_all.race_id = r.id
    WHERE ranked.rn <= 2
    GROUP BY r.id, r.title, r.slug, r.office_id, r.race_type, r.party_id, r.state
    HAVING MAX(ranked.votes::float / NULLIF(ranked.total_votes, 0)) < 0.50
       AND COUNT(DISTINCT rc_all.candidate_id) > 2
)"#,
        PRIMARY_ELECTION_ID,
        STATE
    )
}

fn runoff_cte_dry_run_sql() -> String {
    format!(
        r#"WITH
{}
,
inserted_runoff_races AS (
    INSERT INTO ingest_staging.tx_primary_runoff_races
        (id, source_race_id, title, slug, office_id, race_type, election_id)
    SELECT
        gen_random_uuid(),
        rr.race_id,
        trim(rr.title) || ' - Runoff',
        rtrim(rtrim(rr.slug), '- ') || '-runoff',
        rr.office_id,
        rr.race_type::TEXT,
        $1::uuid
    FROM runoff_races rr
    RETURNING id, source_race_id
),
update_winners AS (
    UPDATE race r SET winner_ids = r.winner_ids WHERE false
    RETURNING r.id
),
inserted_candidates AS (
    INSERT INTO ingest_staging.tx_primary_runoff_race_candidates (race_id, candidate_id)
    SELECT irr.id, u.candidate_id
    FROM inserted_runoff_races irr
    JOIN runoff_races rr ON rr.race_id = irr.source_race_id
    CROSS JOIN LATERAL unnest(rr.top_two) AS u(candidate_id)
    ON CONFLICT (race_id, candidate_id) DO NOTHING
    RETURNING race_id
)
{}
SELECT
    (SELECT count(*) FROM inserted_runoff_races)::bigint AS n_runoff_races,
    (SELECT count(*) FROM inserted_candidates)::bigint AS n_runoff_candidates,
    (SELECT count(*) FROM inserted_general_races)::bigint AS n_general_races,
    (SELECT count(*) FROM inserted_general_candidates)::bigint AS n_general_candidates
"#,
        runoff_base_cte(),
        general_stage_cte_sql(true)
    )
}

/// Stage 2: general election CTEs (primary_races, general races + candidates). Depends on runoff_races.
fn general_stage_cte_sql(dry_run: bool) -> String {
    let (insert_general_races, general_candidates_select_from) = if dry_run {
        (
            r#"INSERT INTO ingest_staging.tx_general_races (id, source_race_id, title, slug, office_id, race_type, election_id, num_elect)
    SELECT gen_random_uuid(), gs.primary_race_id, gs.general_title, gs.general_slug, gs.office_id, 'general', $2::uuid, gs.num_elect
    FROM general_slugs gs
    ON CONFLICT (slug) DO NOTHING
    RETURNING id, slug, source_race_id"#,
            r#"SELECT agr.id, u.candidate_id
    FROM all_general_races agr
    JOIN general_slugs gs ON gs.general_slug = agr.slug
    JOIN advancing_winners aw ON aw.primary_race_id = gs.primary_race_id
    CROSS JOIN LATERAL unnest(aw.winner_ids) AS u(candidate_id)"#,
        )
    } else {
        (
            r#"INSERT INTO race (id, slug, title, office_id, race_type, vote_type, party_id, state, election_id, is_special_election, num_elect)
    SELECT gen_random_uuid(), gs.general_slug, gs.general_title, gs.office_id, 'general'::race_type, 'plurality'::vote_type, NULL, gs.state, $2::uuid, false, gs.num_elect
    FROM general_slugs gs
    ON CONFLICT (slug) DO NOTHING
    RETURNING id, slug"#,
            r#"SELECT agr.id, u.candidate_id
    FROM all_general_races agr
    JOIN general_slugs gs ON gs.general_slug = agr.slug
    JOIN advancing_winners aw ON aw.primary_race_id = gs.primary_race_id
    CROSS JOIN LATERAL unnest(aw.winner_ids) AS u(candidate_id)"#,
        )
    };
    let (general_candidates_insert, all_general_races_cte, update_primary_winners_sql) = if dry_run {
        (
            "INSERT INTO ingest_staging.tx_general_race_candidates (race_id, candidate_id)",
            r#",
all_general_races AS (
    SELECT id, slug FROM inserted_general_races
    UNION ALL
    SELECT tgr.id, tgr.slug FROM ingest_staging.tx_general_races tgr
    WHERE tgr.slug IN (SELECT general_slug FROM general_slugs)
      AND tgr.slug NOT IN (SELECT slug FROM inserted_general_races)
)"#,
            "UPDATE race r SET winner_ids = r.winner_ids WHERE false",
        )
    } else {
        (
            "INSERT INTO race_candidates (race_id, candidate_id)",
            r#",
all_general_races AS (
    SELECT id, slug FROM inserted_general_races
    UNION ALL
    SELECT r.id, r.slug FROM race r
    WHERE r.slug IN (SELECT general_slug FROM general_slugs)
      AND r.slug NOT IN (SELECT slug FROM inserted_general_races)
)"#,
            r#"UPDATE race r SET winner_ids = aw.winner_ids
    FROM advancing_winners aw
    WHERE r.id = aw.primary_race_id AND (r.winner_ids IS NULL OR r.winner_ids = '{}')"#,
        )
    };
    format!(
        r#",
primary_races AS (
    SELECT r.id, r.title, r.slug, r.office_id, r.num_elect, r.winner_ids, r.state, r.num_precincts_reporting, r.total_precincts, r.total_votes
    FROM race r
    WHERE r.election_id = '{}'::uuid AND r.state::text = '{}' AND r.race_type = 'primary'
),
general_slugs AS (
    SELECT pr.id AS primary_race_id, pr.office_id, pr.state, pr.num_elect,
        replace(replace(replace(trim(pr.title), ' Primary', ' General'), ' Democratic -', ''), ' Republican -', '') AS general_title,
        rtrim(regexp_replace(regexp_replace(regexp_replace(pr.slug, '-primary', '-general', 'i'), '-democratic-', '-', 'gi'), '-republican-', '-', 'gi'), '- ') AS general_slug
    FROM primary_races pr
),
inserted_general_races AS (
    {}
){},
primary_races_no_runoff AS (
    SELECT pr.* FROM primary_races pr
    WHERE pr.id NOT IN (SELECT race_id FROM runoff_races)
      AND pr.num_precincts_reporting = pr.total_precincts
      AND pr.total_precincts IS NOT NULL
      AND (pr.total_votes > 0 OR pr.total_votes IS NOT NULL)
),
advancing_winners AS (
    SELECT pr.id AS primary_race_id,
        COALESCE(
            CASE WHEN pr.winner_ids IS NOT NULL AND array_length(pr.winner_ids, 1) > 0 THEN pr.winner_ids
                 ELSE (SELECT array_agg(sub.candidate_id ORDER BY sub.votes DESC NULLS LAST) FROM (SELECT rc.candidate_id, rc.votes FROM race_candidates rc WHERE rc.race_id = pr.id ORDER BY rc.votes DESC NULLS LAST LIMIT COALESCE(pr.num_elect::int, 1)) sub)
            END,
            '{{}}'::uuid[]
        ) AS winner_ids
    FROM primary_races_no_runoff pr
),
update_primary_winners AS (
    {}
    RETURNING r.id
),
inserted_general_candidates AS (
    {} {}
    ON CONFLICT (race_id, candidate_id) DO NOTHING
    RETURNING race_id
)
"#,
        PRIMARY_ELECTION_ID,
        STATE,
        insert_general_races,
        all_general_races_cte,
        update_primary_winners_sql,
        general_candidates_insert,
        general_candidates_select_from
    )
}

/// Production: write to race and race_candidates. ON CONFLICT (slug) DO NOTHING; use existing race.id when conflict.
fn runoff_cte_production_sql() -> String {
    format!(
        r#"WITH
{}
,
runoff_slugs AS (
    SELECT rr.race_id, rtrim(rtrim(rr.slug), '- ') || '-runoff' AS runoff_slug
    FROM runoff_races rr
),
inserted_runoff_races AS (
    INSERT INTO race (id, slug, title, office_id, race_type, vote_type, party_id, state, election_id, is_special_election)
    SELECT
        gen_random_uuid(),
        rs.runoff_slug,
        trim(rr.title) || ' - Runoff',
        rr.office_id,
        rr.race_type,
        'plurality'::vote_type,
        rr.party_id,
        rr.state,
        $1::uuid,
        false
    FROM runoff_slugs rs
    JOIN runoff_races rr ON rr.race_id = rs.race_id
    ON CONFLICT (slug) DO NOTHING
    RETURNING id, slug
),
all_runoff_races AS (
    SELECT id, slug FROM inserted_runoff_races
    UNION ALL
    SELECT r.id, r.slug
    FROM race r
    WHERE r.slug IN (SELECT runoff_slug FROM runoff_slugs)
      AND r.slug NOT IN (SELECT slug FROM inserted_runoff_races)
),
update_winners AS (
    UPDATE race r
    SET winner_ids = rr.top_two
    FROM runoff_races rr
    WHERE r.id = rr.race_id
      AND (r.winner_ids IS NULL OR r.winner_ids = '{{}}')
    RETURNING r.id
),
inserted_candidates AS (
    INSERT INTO race_candidates (race_id, candidate_id)
    SELECT arr.id, u.candidate_id
    FROM all_runoff_races arr
    JOIN runoff_races rr ON arr.slug = (rtrim(rtrim(rr.slug), '- ') || '-runoff')
    CROSS JOIN LATERAL unnest(rr.top_two) AS u(candidate_id)
    ON CONFLICT (race_id, candidate_id) DO NOTHING
    RETURNING race_id
)
{}
SELECT
    (SELECT count(*) FROM inserted_runoff_races)::bigint AS n_runoff_races,
    (SELECT count(*) FROM inserted_candidates)::bigint AS n_runoff_candidates,
    (SELECT count(*) FROM inserted_general_races)::bigint AS n_general_races,
    (SELECT count(*) FROM inserted_general_candidates)::bigint AS n_general_candidates
"#,
        runoff_base_cte(),
        general_stage_cte_sql(false)
    )
}

async fn run_post_processing(pool: &sqlx::PgPool, dry_run: bool) -> Result<(), Box<dyn Error>> {
    let runoff_election_id: uuid::Uuid = PRIMARY_RUNOFF_ELECTION_ID.parse()?;
    let general_election_id: uuid::Uuid = GENERAL_ELECTION_ID.parse()?;
    let sql = if dry_run {
        runoff_cte_dry_run_sql()
    } else {
        runoff_cte_production_sql()
    };

    let r = sqlx::query_as::<_, ProcessingResult>(&sql)
        .bind(runoff_election_id)
        .bind(general_election_id)
        .fetch_one(pool)
        .await?;

    let dest = if dry_run { "staging" } else { "production (race, race_candidates)" };
    println!("--- Summary ---");
    println!("  Runoff races inserted:        {}", r.n_runoff_races);
    println!("  Runoff race_candidates:       {}", r.n_runoff_candidates);
    println!("  General races inserted:      {}", r.n_general_races);
    println!("  General race_candidates:     {}", r.n_general_candidates);
    println!("  Destination:                 {}", dest);
    if r.n_runoff_races == 0 && r.n_runoff_candidates == 0 && r.n_general_races == 0 && r.n_general_candidates == 0 {
        println!("  (No races found for state {} and primary election.)", STATE);
    }
    if dry_run {
        println!("\nReview: SELECT * FROM ingest_staging.tx_primary_runoff_races; SELECT * FROM ingest_staging.tx_primary_runoff_race_candidates;");
        println!("        SELECT * FROM ingest_staging.tx_general_races; SELECT * FROM ingest_staging.tx_general_race_candidates;");
    }
    Ok(())
}

#[tokio::main]
async fn main() {
    let args: Vec<String> = std::env::args().collect();
    let dry_run = args.iter().any(|a| a == "--dry-run" || a == "-n");

    if dry_run {
        eprintln!("Dry run: writing to ingest_staging only.");
    }

    dotenv::dotenv().ok();
    db::init_pool().await.expect("db pool");
    let pool = db::pool().await;

    if dry_run {
        if let Err(e) = ensure_staging_tables(&pool.connection).await {
            eprintln!("Failed to ensure staging tables: {}", e);
            process::exit(1);
        }
    }
    if let Err(e) = run_post_processing(&pool.connection, dry_run).await {
        eprintln!("Post-processing failed: {}", e);
        process::exit(1);
    }
}
