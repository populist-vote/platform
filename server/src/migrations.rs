use sqlx::{migrate::MigrateError, migrate::Migrator, PgPool};

pub(crate) async fn run(pool: &PgPool) -> Result<(), MigrateError> {
    run_with(pool, sqlx::migrate!("../db/migrations")).await
}

async fn run_with(pool: &PgPool, mut migrator: Migrator) -> Result<(), MigrateError> {
    // A newer release or a manual migration can advance the database before this
    // binary is replaced. Do not crash on restart solely because its embedded
    // history is older. SQLx still checks known checksums, dirty migrations, and
    // errors applying pending migrations. Schema changes must remain compatible
    // with every release that can still be running.
    migrator.set_ignore_missing(true).run(pool).await?;

    let applied_versions: Vec<i64> =
        sqlx::query_scalar("SELECT version FROM _sqlx_migrations ORDER BY version")
            .fetch_all(pool)
            .await?;
    for version in applied_versions {
        if !migrator.version_exists(version) {
            tracing::warn!(
                migration_version = version,
                "Database contains a migration absent from this binary; continuing startup. Verify schema compatibility and deploy the matching release."
            );
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::migrate::{Migration, MigrationType};
    use std::borrow::Cow;

    fn history(migrations: &[(i64, &str)]) -> Migrator {
        Migrator {
            migrations: Cow::Owned(
                migrations
                    .iter()
                    .map(|(version, sql)| {
                        Migration::new(
                            *version,
                            "startup regression".into(),
                            MigrationType::Simple,
                            sql.to_string().into(),
                            false,
                        )
                    })
                    .collect(),
            ),
            ..Migrator::DEFAULT
        }
    }

    const BASE: (i64, &str) = (1, "CREATE TABLE startup_probe (id INTEGER PRIMARY KEY);");
    const NEXT: (i64, &str) = (2, "ALTER TABLE startup_probe ADD COLUMN note TEXT;");

    #[sqlx::test(migrations = false)]
    async fn older_binary_restarts_after_database_advances(pool: PgPool) {
        // Fresh startup still applies the binary's migrations.
        run_with(&pool, history(&[BASE])).await.unwrap();
        sqlx::query("INSERT INTO startup_probe (id) VALUES (42)")
            .execute(&pool)
            .await
            .unwrap();

        // A separate migration advances the DB, but the deployed binary stays old.
        history(&[BASE, NEXT]).run(&pool).await.unwrap();
        for _ in 0..2 {
            run_with(&pool, history(&[BASE])).await.unwrap();
            let id: i32 = sqlx::query_scalar("SELECT id FROM startup_probe")
                .fetch_one(&pool)
                .await
                .unwrap();
            assert_eq!(id, 42);
        }
        let versions: Vec<i64> =
            sqlx::query_scalar("SELECT version FROM _sqlx_migrations ORDER BY version")
                .fetch_all(&pool)
                .await
                .unwrap();
        assert_eq!(versions, vec![1, 2]);
    }

    #[sqlx::test(migrations = false)]
    async fn known_checksum_mismatch_still_fails(pool: PgPool) {
        history(&[BASE, NEXT]).run(&pool).await.unwrap();
        let changed = (1, "CREATE TABLE startup_probe (id BIGINT PRIMARY KEY);");
        assert!(matches!(
            run_with(&pool, history(&[changed])).await,
            Err(MigrateError::VersionMismatch(1))
        ));
    }

    #[sqlx::test(migrations = false)]
    async fn dirty_migration_absent_from_binary_still_fails(pool: PgPool) {
        history(&[BASE, NEXT]).run(&pool).await.unwrap();
        sqlx::query("UPDATE _sqlx_migrations SET success = false WHERE version = 2")
            .execute(&pool)
            .await
            .unwrap();
        assert!(matches!(
            run_with(&pool, history(&[BASE])).await,
            Err(MigrateError::Dirty(2))
        ));
    }

    #[sqlx::test(migrations = false)]
    async fn pending_migration_failure_is_not_swallowed(pool: PgPool) {
        run_with(&pool, history(&[BASE])).await.unwrap();
        let broken = (2, "ALTER TABLE nonexistent_table ADD COLUMN note TEXT;");
        assert!(run_with(&pool, history(&[BASE, broken])).await.is_err());
        let versions: Vec<i64> =
            sqlx::query_scalar("SELECT version FROM _sqlx_migrations ORDER BY version")
                .fetch_all(&pool)
                .await
                .unwrap();
        assert_eq!(versions, vec![1]);
    }
}
