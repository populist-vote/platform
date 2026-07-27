# Populist Platform

Populist Database Interface, GraphQL API Server, and Command Line Utilities

## Getting Started

To clone this repository, run `git clone --recurse-submodules -j8 https://github.com/populist-vote/platform.git`.
Make sure you have [Rust installed] on your machine. On macOS, install the
remaining local dependencies with:

```bash
brew install postgresql@17 postgis mkcert
cargo install sqlx-cli --no-default-features --features rustls,postgres
cargo install cargo-watch
mkcert -install
```

First copy the `.env.example` file to `.env` which is .gitignored.

```bash
cp .env.example .env
```

Set `DATABASE_URL` to your local database. A fresh database can be prepared with:

```bash
sqlx database create
sqlx migrate run --source db/migrations
psql "$DATABASE_URL" -f db/scripts/bootstrap_local_gis.sql
```

The GIS bootstrap creates empty Texas boundary tables required by SQLx's
compile-time validation. Load the official shapefiles separately when testing
Texas address-to-district lookups.

Alternatively, create a local copy of the staging database from Heroku. Once
you have access to the Heroku account and have logged in with the Heroku CLI,
run:

```bash
./scripts/refresh_local_db.sh populist-api-staging
```

from the root of the project. This will download the latest backup from Heroku and restore it locally in a database called populist-platform-dev. You can then run `cargo sqlx prepare` to generate the sqlx-data.json file which is used to validate SQL queries at compile time.

## Database

[sqlx] is used for managing asynchronous database operations. This project relies heavily on compile-time query verification using `sqlx` macros, namely `query_as!` If you do not have a DATABASE_URL specified in your .env file, you will not be able to compile the binary for this crate. You can run sqlx in offline mode by setting SQLX_OFFLINE=true. You can enable "offline mode" to cache the results of the SQL query analysis using the sqlx-cli. If you make schema alterations, run the command `cargo sqlx prepare` which will write your query data to `sqlx-data.json` at the `/db` root.

### Running Migrations

We can easily create SQL migration files using the sqlx-cli. From the /db directory, you can run `sqlx migrate add -r DescriptiveMigrationName` to create up and down migration files in the /migrations folder. You can write SQL in these files and use `sqlx migrate run` and `sqlx migrate revert` respectively.

Prior to pushing to staging, if you have any migrations you will want to run `DATABASE_URL=$PRODUCTION_DATABASE_URL sqlx migrate run` to run the migrations in the staging environment. Then the compile time query validation will be able to verify the queries against the staging database. For pushing to production using the 'Promote to Production' button in the Heroku pipeline, you do not need to run the migrations manually because they are [embedded into the binary] and will run as part of the deploy process.

## API Server

Locally, we need to generate self-signed TLS certificates to run the server in https mode. You can do so by running `mkcert localhost 127.0.0.1 ::1` in the /server/src/certs directory (which is gitignored).
To start the api server, run `cargo watch -x run` which will type check, compile, and run your code. The GraphQL playground will then be live at https://localhost:1234 for you to execute queries and mutations against the specified database.

To run certain mutations and queries which require staff or superuser permissions, you can add an `Authorization` token to the HTTP headers section of the playground. You can login to `https://staging.populist.us` or `https://populist.us` and grab the value from the `access_token` cookie in your browsers developer tools. Add this to the http headers like so: `"Authorization" : "Bearer <TOKEN>"`

### REST API

Versioned REST endpoints are available under `/api/v1`. The API index is at
`GET /api/v1/`.

Resolve the races and ballot measures for an address in a specific election
with:

```http
POST /api/v1/elections/{electionId}/ballot
Content-Type: application/json

{
  "address": {
    "line1": "123 Main St",
    "line2": "Apt 4",
    "city": "Minneapolis",
    "state": "MN",
    "postalCode": "55401",
    "country": "US"
  }
}
```

Add `?endorserId={organizationId}` to restrict each race's candidates to those
endorsed by that organization. The response includes the election, ordered
races with offices, candidates and results, and address-matched ballot
measures. Address data is not returned, and successful responses use
`Cache-Control: no-store`.

REST errors use `application/problem+json` with stable machine-readable `code`
values.

The client integration guide, complete examples, and machine-readable contract
are in:

- [`docs/rest/ballot-by-address.md`](docs/rest/ballot-by-address.md)
- [`docs/rest/openapi.yaml`](docs/rest/openapi.yaml)
- [`docs/rest/examples/`](docs/rest/examples/)

Maintainers can run the deterministic ballot REST gate with
`./scripts/check_ballot_rest.sh`. A bounded, reviewable Codex hardening loop is
documented in [`docs/rest/agent-loop.md`](docs/rest/agent-loop.md).

The guarded platform-and-web staging deployment loop and live verification
suite are documented in
[`docs/rest/staging-deployment-loop.md`](docs/rest/staging-deployment-loop.md).

## Testing

Run the deterministic first-party checks with:

```bash
cargo fmt --all -- --check
cargo check --workspace --all-targets
cargo test --workspace
cargo clippy --workspace --all-targets --no-deps -- -D warnings
```

The Geocodio, LegiScan, OpenSecrets, and VoteSmart clients are workspace
dependencies but are excluded as direct workspace members because their own
tests call live, credentialed APIs. Test those clients from their directories
when valid API credentials are available.

## Deploying

When committing code that manipulates any sqlx query macros such as `query_as!`,
be sure to run `cargo sqlx prepare` from root of each crate affected (likely `/db` or `/graphql`) and commit the changes to the `sqlx-data.json` files. These files are used during build time to validate the SQL queries against the live database.

To deploy the main branch to the staging environment, run `git push heroku`

To run the migrations, **make sure you're on branch `main`** and set the `DATABASE_URL` to the URI found on our [Heroku datastore dashboard], under "View Credentials." Then run `sqlx migrate run` from your local machine. This is a temporary solution until we figure out how to automatically run the migrations on each deploy.

Deploys to production happen manually via the Heroku dashboard. Press the "Promote to Production" button on the staging app in the [pipeline view]. You can access logs to the production server by running `heroku logs --tail -a populist-api-production`

[rust installed]: https://www.rust-lang.org/tools/install
[sqlx-cli]: https://crates.io/crates/sqlx-cli
[sqlx]: https://crates.io/crates/sqlx
[heroku datastore dashboard]: https://data.heroku.com/datastores/35cb347f-6fb1-488f-8f21-02bbd726f5a8#administration
[pipeline view]: https://dashboard.heroku.com/pipelines/3ce13ae5-d2aa-4522-b513-3b3ba0e6f179
[embedded into the binary]: https://docs.rs/sqlx/latest/sqlx/macro.migrate.html
