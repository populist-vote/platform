# Populist Platform

Populist Database Interface, GraphQL API Server, and Command Line Utilities

## Getting Started

To clone this repository, run `git clone --recurse-submodules -j8 https://github.com/populist-vote/platform.git`
Make sure you have [Rust installed] on your machine. Next, you'll need the [sqlx-cli] installed to manage the database connection and run migrations. To do so, run `cargo install sqlx-cli --features postgres`

First copy the `.env.example` file to `.env` which is .gitignored.

```bash
cp .env.example .env
```

For local development, its best to create a local copy of our staging database on Heroku. Once you have access to our Heroku account and have logged in with the Heroku CLI, you can do so by running

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

To start the api server, run `cargo watch -x run` which will type check, compile, and run your code. The GraphQL playground will then be live at https://localhost:1234 for you to execute queries and mutations against the specified database.

To run certain mutations and queries which require staff or superuser permissions, you can add an `Authorization` token to the HTTP headers section of the playground. You can login to `https://staging.populist.us` or `https://populist.us` and grab the value from the `access_token` cookie in your browsers developer tools. Add this to the http headers like so: `"Authorization" : "Bearer <TOKEN>"`

## Testing

`cargo test`

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
