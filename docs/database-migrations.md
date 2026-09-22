# Database migration safety

The September 21, 2026 outage was a startup-history mismatch: production's
Postgres was migrated while its August binary stayed deployed. A restart then
failed with SQLx `VersionMissing`. Deployment and database changes are separate
operations; an unchanged Heroku release does not imply an unchanged schema.

## Execution model

```text
build (embed committed migrations)
  -> release: server migrate
       validate all applied history under SQLx's advisory lock
       apply pending transactional migrations
       validate using runtime credentials
  -> web: server
       read-only schema validation
       start jobs/listeners, then accept requests
```

`server/build.rs` tracks the migration directory, including newly added files,
so a migration-only change rebuilds the embedded manifest. The same server
executable handles release and web modes; no second binary or buildpack build
flag is required.

Web startup uses a read-only transaction and does not acquire the migration
advisory lock, create `_sqlx_migrations`, or apply pending SQL. It can operate
with SELECT-only access to the ledger. Application requests still need their
normal DML permissions.

The release command uses a dedicated connection. It holds SQLx's advisory lock
through preflight, migration application, and validation, and closes that
connection on success or failure. Connections time out after 10 seconds; lock
waits are capped at 5 seconds and individual SQL statements at 5 minutes. This
prevents an ALTER waiting indefinitely while requests queue behind it. A
long-running operation needs a separately planned maintenance procedure.

The migrator checks all known checksums before any pending SQL. Applied
versions absent from the binary are allowed only if strictly newer than its
last embedded up migration. An older release can therefore validate an already
advanced database without applying down migrations. Historical gaps and
out-of-order pending migrations are rejected.

## Failure behavior

| Database state | Web startup | Release command |
| --- | --- | --- |
| All required migrations applied and unchanged | Starts; no writes | No-op |
| Additional successful newer migrations | Warns and starts | Allows compatible rollback; no down migrations |
| Required migration missing | Refuses startup without mutating DB | Applies it only if it is an append to history |
| Known checksum changed | Refuses startup | Rejects before any pending SQL |
| Dirty migration required by this binary | Refuses startup | Rejects |
| Dirty migration strictly newer than this binary | Warns; previous binary can still start | Rejects until repaired |
| Unknown migration inside embedded historical range | Refuses startup | Rejects |
| Pending non-transactional migration | Refuses startup because requirement is missing | Rejects; maintenance plan required |
| Transactional migration SQL fails | Previous compatible binary remains usable | Exits nonzero; failing migration rolls back |
| Lock or statement timeout | Web startup does not wait on migration lock | Exits nonzero and closes its connection |

A dirty future migration is not evidence that the old schema was destroyed.
The preceding binary checks its own requirements instead of becoming hostage
to a failed future release. This cannot prove that arbitrary DDL left those
requirements semantically intact; backward compatibility remains mandatory.

[Heroku release phase](https://devcenter.heroku.com/articles/release-phase)
blocks new web dynos until the command succeeds. This is not a database rollback:
successful earlier migrations remain committed if a later migration fails. A
failed release can also be affected by later add-on config changes, which skip
release phase; promptly fix or roll back a failed release. Do not assume that
an application rollback reverses schema changes.

## Schema compatibility policy

Use expand/contract changes: add the replacement schema, deploy compatible
code, migrate data, then remove the old schema in a later release after the old
code and rollback targets are retired. Test reads and writes from the preceding
release against the expanded schema. Include this evidence and a recovery plan
in any migration PR. Locks, constraints, defaults, type changes, trigger changes,
and data rewrites need review as well as dropped or renamed columns.

Never edit/delete an applied migration, renumber it, remove its ledger row, or
change its checksum to bypass validation. CI rejects changes to migration files
already present on the PR base. New migrations must use later unique versions.
Keep transaction management in SQLx; do not put top-level BEGIN/COMMIT/ROLLBACK
inside new migration files. Historical files are immutable, even where they
predate this policy. Pending `-- no-transaction` migrations are deliberately
refused by the automated runner. Historical non-transactional migrations that
are already applied are still validated normally.

## Commands and credentials

Hosted migrations run as `release: ./target/release/server migrate` from the
Procfile. Running this command from a normal shell, web dyno, or `heroku run`
dyno is refused. For a disposable local DB, use:

```sh
cargo run -p server --bin server -- migrate --local
```

`--local` inspects the parsed effective SQLx connection options and refuses
remote hosts, including a remote `host=` URL override. Loopback addresses and
Unix sockets are accepted. Do not point a local tunnel at production and call
it a local DB. The DYNO/host checks prevent accidents; they are not security
boundaries against someone holding owner credentials.

`DATABASE_URL` is used by web processes. `MIGRATION_DATABASE_URL`, when present,
is used only by the release command and must explicitly name the same database,
host, port, and socket. Without it, releases use `DATABASE_URL` for compatibility
with current Heroku configuration. After migrations, the release verifies the
ledger through `DATABASE_URL` too, so invalid runtime credentials fail the
release before replacing web dynos. Both URLs must be maintained by Heroku
credential attachments, not copied into static config vars that miss rotation.

To enforce a database permission boundary, provision separate roles in a staged
operational rollout using [Heroku-managed credentials](https://devcenter.heroku.com/articles/heroku-postgresql-credentials):

1. Runtime: only the DML, sequence, schema-usage, and function privileges the
   application needs, plus SELECT on `public._sqlx_migrations`. It must not own
   schema objects or inherit the owner role. Exclude ledger writes, DDL, and
   administrative privileges.
2. Developer/query tools: read-only credentials. Do not retain production owner
   credentials in local `.env` files or SQL clients.
3. Release: the schema owner, provided as a managed `MIGRATION_DATABASE_URL`
   attachment. Configure default privileges for new objects so the runtime role
   can use them after each migration.
4. Audit PUBLIC grants and role memberships. Historical migrations grant schema
   privileges to PUBLIC; revoking CREATE only from a named runtime role does not
   remove access inherited from PUBLIC. Review SECURITY DEFINER functions too.
5. Exercise authentication, writes, jobs, listeners, and future-object grants on
   staging before changing production's runtime credentials. Attach/promote
   managed credentials in a coordinated rollout; retain a tested recovery path.

Support for custom credentials depends on the Heroku Postgres plan. Both
credentials being available in one Heroku app is not secret isolation against
an app administrator or compromised process; deployment-only credential access
requires infrastructure/access controls beyond this repository. This PR does
not provision roles, revoke production grants, rotate credentials, or claim that
raw `psql`/`sqlx` owner access is blocked.

## Rollout and verification

Deploy this change to staging and verify release logs, web startup, requests,
and a restart. Promote that exact tested build to production. Install this
baseline before adding more migrations; pre-baseline binaries still have the
old fatal startup migrator. Do not roll back to those builds after advancing
the database. No schema migration is needed to install this lifecycle change.

For any failed release, inspect `heroku releases:output` before retrying. A
failed non-transactional maintenance operation may need explicit cleanup (for
example, an invalid concurrent index); do not conceal it by editing the ledger.

The `Migration safety` GitHub Actions workflow prepares an isolated Postgres
17/PostGIS database and runs the lifecycle tests and the actual binary smoke
check. It uses no hosted credentials. For local reproduction, use the README's
fresh database setup with a database named `migration_safety_compile`, then:

```sh
export DATABASE_URL=postgres://localhost/migration_safety_compile
export SQLX_OFFLINE=false
cargo test --locked -p server --lib migrations::tests
cargo build --locked -p server --bin server
python3 scripts/smoke_migration_lifecycle.py target/debug/server
```

The smoke check refuses remote and non-test database URLs. It intentionally
advances the disposable database after compilation, verifies two server starts
and database-backed HTTP requests, checks command guards and warning output,
and runs a rollback release without changing history. Unit/integration tests
also cover SELECT-only roles, dirty histories, missing requirements, checksums,
transaction rollback, concurrent releases, and advisory/DDL/statement timeouts.
