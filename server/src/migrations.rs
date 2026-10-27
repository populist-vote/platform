//! Release-time schema changes and read-only web startup validation.
use anyhow::{bail, Context, Result};
use sqlx::{
    migrate::{Migrate, MigrateError, Migrator},
    postgres::{PgConnectOptions, PgPoolOptions},
    Connection, PgConnection, PgPool,
};
use std::{collections::BTreeMap, net::IpAddr, str::FromStr, time::Duration};

// Keep lock waits short so a queued ALTER does not stall requests behind it.
const LOCK_TIMEOUT: &str = "5s";
const STATEMENT_TIMEOUT: &str = "5min";
type Applied = (i64, bool, Vec<u8>);

fn embedded() -> Migrator {
    sqlx::migrate!("../db/migrations")
}

#[derive(Clone, Copy, PartialEq)]
enum Purpose {
    Startup,
    Release,
}

/// The web process must never create the migration table or apply DDL.
/// Run before starting background jobs or accepting requests.
pub(crate) async fn check(pool: &PgPool) -> Result<()> {
    check_with(pool, &embedded()).await
}

async fn check_with(pool: &PgPool, migrator: &Migrator) -> Result<()> {
    let mut tx = pool.begin().await?;
    sqlx::query("SET TRANSACTION ISOLATION LEVEL REPEATABLE READ, READ ONLY")
        .execute(&mut *tx)
        .await?;
    let applied = history(&mut tx).await?;
    validate(migrator, &applied, Purpose::Startup)?;
    tx.commit().await?;
    Ok(())
}

async fn history(conn: &mut PgConnection) -> Result<Vec<Applied>> {
    // Checking existence first also works inside a read-only transaction on a
    // fresh DB; catching undefined_table after SELECT would abort the transaction.
    let exists: bool =
        sqlx::query_scalar("SELECT to_regclass('public._sqlx_migrations') IS NOT NULL")
            .fetch_one(&mut *conn)
            .await?;
    if !exists {
        return Ok(Vec::new());
    }
    Ok(sqlx::query_as(
        "SELECT version, success, checksum FROM public._sqlx_migrations ORDER BY version",
    )
    .fetch_all(conn)
    .await?)
}

fn validate(migrator: &Migrator, applied: &[Applied], purpose: Purpose) -> Result<()> {
    let mut required = BTreeMap::new();
    for migration in migrator
        .iter()
        .filter(|m| !m.migration_type.is_down_migration())
    {
        if required.insert(migration.version, migration).is_some() {
            bail!(
                "Duplicate embedded up migration version {}; assign unique migration versions",
                migration.version
            );
        }
    }
    let newest_required = required.keys().next_back().copied().unwrap_or(0);
    let newest_applied = applied.iter().map(|m| m.0).max().unwrap_or(0);
    let by_version: BTreeMap<_, _> = applied.iter().map(|m| (m.0, m)).collect();

    // Validate the entire existing history before executing ANY pending SQL.
    // SQLx alone checks known checksums as it iterates through migrations.
    for (version, success, checksum) in applied {
        if let Some(migration) = required.get(version) {
            if !success {
                return Err(MigrateError::Dirty(*version).into());
            }
            if migration.checksum.as_ref() != checksum {
                return Err(MigrateError::VersionMismatch(*version).into());
            }
        } else if *version <= newest_required {
            bail!("Applied migration {version} is absent from this binary's historical migration sequence; restore the original migration file");
        } else {
            // A failed future release must not prevent the preceding compatible
            // binary from restarting. Release commands still reject ALL dirty rows.
            if !success && purpose == Purpose::Release {
                return Err(MigrateError::Dirty(*version).into());
            }
            tracing::warn!(
                migration_version = version,
                success,
                "Database migration is newer than this binary; verify backward compatibility"
            );
        }
    }
    for (version, migration) in &required {
        if !by_version.contains_key(version) {
            if purpose == Purpose::Startup {
                bail!("Required migration {version} is not applied; run the release migration command before starting this binary");
            }
            if *version < newest_applied {
                bail!("Refusing out-of-order migration {version}: database already contains {newest_applied}");
            }
            if migration.no_tx {
                bail!("Migration {version} is non-transactional; automatic release execution is refused. Use a separately reviewed maintenance operation with a recovery plan.");
            }
        }
    }
    Ok(())
}

/// Explicit command; never invoked by normal web startup.
pub async fn release(local: bool) -> Result<()> {
    dotenv::dotenv().ok();
    let _ = tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()),
        )
        .with_writer(std::io::stderr)
        .try_init();
    let runtime_url = std::env::var("DATABASE_URL").context("DATABASE_URL is required")?;
    let migration_url =
        std::env::var("MIGRATION_DATABASE_URL").unwrap_or_else(|_| runtime_url.clone());
    let runtime = PgConnectOptions::from_str(&runtime_url)
        .map_err(|_| anyhow::anyhow!("Invalid DATABASE_URL"))?;
    let options = PgConnectOptions::from_str(&migration_url)
        .map_err(|_| anyhow::anyhow!("Invalid migration database URL"))?;
    if runtime.get_database().is_none() || options.get_database().is_none() {
        bail!("Runtime and migration URLs must explicitly name the database");
    }
    if runtime.get_host() != options.get_host()
        || runtime.get_port() != options.get_port()
        || runtime.get_database() != options.get_database()
        || runtime.get_socket() != options.get_socket()
    {
        bail!("Migration and runtime credentials must target the same host, port, database, and socket");
    }
    authorize(&options, local, std::env::var("DYNO").ok().as_deref())?;
    apply(&options, embedded(), LOCK_TIMEOUT, STATEMENT_TIMEOUT).await?;
    // Catch bad runtime credentials or missing ledger permissions before Heroku
    // replaces the old web dynos, even when the release uses a different role.
    let pool = PgPoolOptions::new()
        .max_connections(1)
        .acquire_timeout(Duration::from_secs(10))
        .connect_with(runtime)
        .await?;
    let result = check(&pool).await;
    pool.close().await;
    result
}

fn authorize(options: &PgConnectOptions, local: bool, dyno: Option<&str>) -> Result<()> {
    if local {
        let host = options
            .get_host()
            .trim_start_matches('[')
            .trim_end_matches(']');
        let loopback =
            host == "localhost" || host.parse::<IpAddr>().is_ok_and(|ip| ip.is_loopback());
        if options.get_socket().is_some() || loopback {
            return Ok(());
        }
        bail!("--local refuses remote databases; use an isolated loopback database");
    }
    if dyno.is_some_and(|name| {
        name.strip_prefix("release.")
            .is_some_and(|id| !id.is_empty() && id.bytes().all(|c| c.is_ascii_digit()))
    }) {
        return Ok(());
    }
    bail!("Remote migrations require a Heroku release dyno; local development uses `server migrate --local`. This guard is not a substitute for restricted database credentials.");
}

async fn apply(
    options: &PgConnectOptions,
    mut migrator: Migrator,
    lock_timeout: &str,
    statement_timeout: &str,
) -> Result<()> {
    // A dedicated connection is closed on EVERY outcome, releasing advisory locks
    // and aborting open transactions instead of returning them to the web pool.
    let mut conn = tokio::time::timeout(
        Duration::from_secs(10),
        PgConnection::connect_with(
            &options
                .clone()
                .application_name("populist-release-migrations"),
        ),
    )
    .await
    .context("Migration connection timed out")??;
    let result: Result<()> = async {
        sqlx::query("SELECT set_config('lock_timeout', $1, false), set_config('statement_timeout', $2, false), set_config('search_path', 'public', false)")
            .bind(lock_timeout)
            .bind(statement_timeout)
            .execute(&mut conn)
            .await?;
        conn.lock().await?;
        let applied = history(&mut conn).await?;
        validate(&migrator, &applied, Purpose::Release)?;
        // Validation above permits only a strictly newer suffix (for rollback),
        // never arbitrary missing history. Hold SQLx's own lock across preflight
        // and application to serialize against both this command and sqlx-cli.
        migrator.set_ignore_missing(true);
        migrator.set_locking(false);
        migrator.run_direct(&mut conn).await?;
        validate(&migrator, &history(&mut conn).await?, Purpose::Startup)?;
        Ok(())
    }
    .await;
    let closed = conn.close().await;
    result?;
    closed?;
    Ok(())
}

#[cfg(test)]
mod tests;
