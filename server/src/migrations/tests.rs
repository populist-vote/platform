use super::*;
use sqlx::{
    migrate::{Migration, MigrationType},
    postgres::PgPoolOptions,
};
use std::borrow::Cow;

fn manifest(migrations: &[(i64, &str)]) -> Migrator {
    Migrator {
        migrations: Cow::Owned(
            migrations
                .iter()
                .map(|(version, sql)| {
                    Migration::new(
                        *version,
                        "release regression".into(),
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
const BASE: (i64, &str) = (1, "CREATE TABLE release_probe (id INTEGER PRIMARY KEY);");
const NEXT: (i64, &str) = (2, "ALTER TABLE release_probe ADD COLUMN note TEXT;");

#[test]
fn duplicate_embedded_versions_are_rejected_before_application() {
    let duplicate = manifest(&[BASE, BASE]);
    for purpose in [Purpose::Startup, Purpose::Release] {
        let error = validate(&duplicate, &[], purpose).unwrap_err();
        assert!(error
            .to_string()
            .contains("Duplicate embedded up migration version 1"));
    }
}

async fn deploy(pool: &PgPool, migrations: &[(i64, &str)]) -> Result<()> {
    apply(
        &pool.connect_options(),
        manifest(migrations),
        LOCK_TIMEOUT,
        STATEMENT_TIMEOUT,
    )
    .await
}

async fn versions(pool: &PgPool) -> Vec<i64> {
    sqlx::query_scalar("SELECT version FROM _sqlx_migrations ORDER BY version")
        .fetch_all(pool)
        .await
        .unwrap()
}

#[sqlx::test(migrations = false)]
async fn web_startup_never_migrates_a_fresh_or_behind_database(pool: PgPool) {
    assert!(check_with(&pool, &manifest(&[BASE]))
        .await
        .unwrap_err()
        .to_string()
        .contains("Required migration 1"));
    let ledger: bool =
        sqlx::query_scalar("SELECT to_regclass('public._sqlx_migrations') IS NOT NULL")
            .fetch_one(&pool)
            .await
            .unwrap();
    assert!(!ledger);
    deploy(&pool, &[BASE]).await.unwrap();
    assert!(check_with(&pool, &manifest(&[BASE, NEXT])).await.is_err());
    assert_eq!(versions(&pool).await, vec![1]);
}

#[sqlx::test(migrations = false)]
async fn old_binary_restarts_and_rollback_release_preserves_new_schema(pool: PgPool) {
    deploy(&pool, &[BASE]).await.unwrap();
    sqlx::query("INSERT INTO release_probe (id) VALUES (42)")
        .execute(&pool)
        .await
        .unwrap();
    deploy(&pool, &[BASE, NEXT]).await.unwrap();
    for _ in 0..2 {
        check_with(&pool, &manifest(&[BASE])).await.unwrap();
        let id: i32 = sqlx::query_scalar("SELECT id FROM release_probe")
            .fetch_one(&pool)
            .await
            .unwrap();
        assert_eq!(id, 42);
    }
    // Heroku also runs the release command on rollback; it must not revert DDL.
    deploy(&pool, &[BASE]).await.unwrap();
    assert_eq!(versions(&pool).await, vec![1, 2]);
    check_with(&pool, &manifest(&[BASE, NEXT])).await.unwrap();
}

#[sqlx::test(migrations = false)]
async fn startup_works_with_select_only_role(pool: PgPool) {
    deploy(&pool, &[BASE]).await.unwrap();
    let role = format!("migration_test_{}", uuid::Uuid::new_v4().simple());
    sqlx::raw_sql(&format!("CREATE ROLE {role}; GRANT USAGE ON SCHEMA public TO {role}; GRANT SELECT ON _sqlx_migrations, release_probe TO {role};"))
        .execute(&pool).await.unwrap();
    let runtime = PgPoolOptions::new()
        .max_connections(1)
        .connect_with(
            (*pool.connect_options())
                .clone()
                .options([("role", role.as_str())]),
        )
        .await
        .unwrap();
    let user: String = sqlx::query_scalar("SELECT current_user")
        .fetch_one(&runtime)
        .await
        .unwrap();
    assert_eq!(user, role);
    check_with(&runtime, &manifest(&[BASE])).await.unwrap();
    assert!(
        sqlx::query("ALTER TABLE release_probe ADD COLUMN forbidden TEXT")
            .execute(&runtime)
            .await
            .is_err()
    );
    assert!(sqlx::query("DELETE FROM _sqlx_migrations")
        .execute(&runtime)
        .await
        .is_err());
    runtime.close().await;
    sqlx::raw_sql(&format!("DROP OWNED BY {role}; DROP ROLE {role};"))
        .execute(&pool)
        .await
        .unwrap();
}

#[sqlx::test(migrations = false)]
async fn changed_checksum_blocks_startup_and_release_before_any_new_sql(pool: PgPool) {
    deploy(&pool, &[BASE, NEXT]).await.unwrap();
    let changed = (2, "ALTER TABLE release_probe ADD COLUMN different TEXT;");
    let pending = (3, "CREATE TABLE should_not_exist (id INTEGER);");
    for result in [
        check_with(&pool, &manifest(&[BASE, changed])).await,
        deploy(&pool, &[BASE, changed, pending]).await,
    ] {
        assert!(matches!(
            result.unwrap_err().downcast_ref::<MigrateError>(),
            Some(MigrateError::VersionMismatch(2))
        ));
    }
    assert_eq!(versions(&pool).await, vec![1, 2]);
}

#[sqlx::test(migrations = false)]
async fn unknown_historical_migration_and_out_of_order_insert_are_rejected(pool: PgPool) {
    let third = (3, "SELECT 1;");
    deploy(&pool, &[BASE, NEXT, third]).await.unwrap();
    assert!(check_with(&pool, &manifest(&[BASE, third]))
        .await
        .unwrap_err()
        .to_string()
        .contains("historical"));
    assert!(deploy(&pool, &[BASE, third])
        .await
        .unwrap_err()
        .to_string()
        .contains("historical"));
    // Simulate a branch adding a missing earlier timestamp to an advanced DB.
    sqlx::query("DELETE FROM _sqlx_migrations WHERE version=2")
        .execute(&pool)
        .await
        .unwrap();
    assert!(deploy(&pool, &[BASE, NEXT, third])
        .await
        .unwrap_err()
        .to_string()
        .contains("out-of-order"));
    assert_eq!(versions(&pool).await, vec![1, 3]);
}

#[sqlx::test(migrations = false)]
async fn failed_release_rolls_back_its_migration_and_old_binary_still_starts(pool: PgPool) {
    deploy(&pool, &[BASE]).await.unwrap();
    let broken = (2, "CREATE TABLE partial_change (id INTEGER); SELECT 1/0;");
    assert!(deploy(&pool, &[BASE, broken]).await.is_err());
    check_with(&pool, &manifest(&[BASE])).await.unwrap();
    assert!(check_with(&pool, &manifest(&[BASE, broken])).await.is_err());
    let partial: bool =
        sqlx::query_scalar("SELECT to_regclass('public.partial_change') IS NOT NULL")
            .fetch_one(&pool)
            .await
            .unwrap();
    assert!(!partial);
    assert_eq!(versions(&pool).await, vec![1]);
    // Failure closed the connection, releasing its lock; a corrected release runs.
    deploy(&pool, &[BASE, NEXT]).await.unwrap();
}

#[sqlx::test(migrations = false)]
async fn dirty_future_migration_blocks_release_but_not_previous_binary(pool: PgPool) {
    deploy(&pool, &[BASE, NEXT]).await.unwrap();
    sqlx::query("UPDATE _sqlx_migrations SET success=false WHERE version=2")
        .execute(&pool)
        .await
        .unwrap();
    check_with(&pool, &manifest(&[BASE])).await.unwrap();
    for result in [
        check_with(&pool, &manifest(&[BASE, NEXT])).await,
        deploy(&pool, &[BASE]).await,
    ] {
        assert!(matches!(
            result.unwrap_err().downcast_ref::<MigrateError>(),
            Some(MigrateError::Dirty(2))
        ));
    }
}

#[sqlx::test(migrations = false)]
async fn concurrent_releases_apply_each_migration_once(pool: PgPool) {
    let slow = (
        2,
        "SELECT pg_sleep(0.15); ALTER TABLE release_probe ADD COLUMN note TEXT;",
    );
    let migrations = [BASE, slow];
    let (first, second) = tokio::join!(deploy(&pool, &migrations), deploy(&pool, &migrations));
    first.unwrap();
    second.unwrap();
    assert_eq!(versions(&pool).await, vec![1, 2]);
}

#[sqlx::test(migrations = false)]
async fn advisory_lock_wait_is_bounded_and_startup_does_not_take_migration_lock(pool: PgPool) {
    deploy(&pool, &[BASE]).await.unwrap();
    let mut holder = PgConnection::connect_with(&pool.connect_options())
        .await
        .unwrap();
    holder.lock().await.unwrap();
    tokio::time::timeout(
        Duration::from_secs(2),
        check_with(&pool, &manifest(&[BASE])),
    )
    .await
    .unwrap()
    .unwrap();
    let result = tokio::time::timeout(
        Duration::from_secs(2),
        apply(
            &pool.connect_options(),
            manifest(&[BASE, NEXT]),
            "100ms",
            "1s",
        ),
    )
    .await
    .unwrap();
    assert!(result.is_err());
    holder.close().await.unwrap();
    deploy(&pool, &[BASE, NEXT]).await.unwrap();
}

#[sqlx::test(migrations = false)]
async fn ddl_lock_and_statement_timeouts_leave_old_binary_usable(pool: PgPool) {
    deploy(&pool, &[BASE]).await.unwrap();
    let mut held = pool.begin().await.unwrap();
    sqlx::query("SELECT * FROM release_probe")
        .fetch_all(&mut *held)
        .await
        .unwrap();
    let result = apply(
        &pool.connect_options(),
        manifest(&[BASE, NEXT]),
        "100ms",
        "1s",
    )
    .await;
    assert!(result.is_err());
    held.rollback().await.unwrap();
    let slow = (2, "SELECT pg_sleep(5);");
    assert!(apply(
        &pool.connect_options(),
        manifest(&[BASE, slow]),
        "100ms",
        "100ms"
    )
    .await
    .is_err());
    check_with(&pool, &manifest(&[BASE])).await.unwrap();
    assert_eq!(versions(&pool).await, vec![1]);
}

#[test]
fn migration_command_rejects_remote_local_and_non_release_execution() {
    let remote = PgConnectOptions::from_str("postgres://user@database.example/app").unwrap();
    assert!(authorize(&remote, true, None).is_err());
    for dyno in [
        None,
        Some("web.1"),
        Some("run.123"),
        Some("release."),
        Some("release.fake"),
    ] {
        assert!(authorize(&remote, false, dyno).is_err());
    }
    assert!(authorize(&remote, false, Some("release.123")).is_ok());
    for url in [
        "postgres://user@127.0.0.1/app",
        "postgres://user@localhost/app",
        "postgres://user@[::1]/app",
    ] {
        assert!(authorize(&PgConnectOptions::from_str(url).unwrap(), true, None).is_ok());
    }
    let disguised =
        PgConnectOptions::from_str("postgres://user@localhost/app?host=database.example").unwrap();
    assert!(authorize(&disguised, true, None).is_err());
}

#[sqlx::test(migrations = false)]
async fn pending_nontransactional_migration_requires_separate_maintenance(pool: PgPool) {
    deploy(&pool, &[BASE]).await.unwrap();
    let mut migrations = manifest(&[BASE, NEXT]);
    migrations.migrations.to_mut()[1].no_tx = true;
    let error = apply(
        &pool.connect_options(),
        migrations,
        LOCK_TIMEOUT,
        STATEMENT_TIMEOUT,
    )
    .await
    .unwrap_err();
    assert!(error.to_string().contains("non-transactional"));
    assert_eq!(versions(&pool).await, vec![1]);
    check_with(&pool, &manifest(&[BASE])).await.unwrap();
}
