# Populist Platform

Populist Database Interface, GraphQL API Server, and Command Line Utility

## Getting Started

Make sure you have [Rust installed] on your machine. Next, you'll need the [sqlx-cli] installed to manage the database connection and run migrations. To do so, run `cargo install sqlx-cli --features postgres`

First copy the `.env.example` file to `.env` which is .gitignored.  
`cp .env.example .env` For local development, you can then run `sqlx db create` to create a new Postgres database at the url defined in your new `.env`

Next, you'll need to run the migrations with `sqlx migrate run`

## Database

[sqlx] is used for managing asynchronous database operations. This project relies heavily on compile-time query verification using `sqlx` macros, namely `query_as!` If you do not have a DATABASE_URL specified in your .env file, you will not be able to compile the binary for this crate. You can run sqlx in offline mode by setting SQLX_OFFLINE=true. You can enable "offline mode" to cache the results of the SQL query analysis using the sqlx-cli. If you make schema alterations, run the command `cargo sqlx prepare` which will write your query data to `sqlx-data.json` at the `/db` root.

## API Server

To start the api server, run `cargo watch -x run` which will type check, compile, and run your code. The GraphQL playground will then be live at http://localhost:1234 for you to execute queries and mutations against the specified database.

To run certain mutations and queries which require staff or superuser permissions, you can add an `Authorization` token to the HTTP headers section of the playground. You can login to `https://staging.populist.us` or `https://populist.us` and grab the value from the `access_token` cookie in your browsers developer tools. Add this to the http headers like so: `"Authorization" : "Bearer <TOKEN>"`

## Command Line

The `/cli` crate compiles an executable binary that serves as the Populist CLI. You can run the cli locally and learn more about usage with `./target/debug/cli --help`

Here are a few example commands to get you started:

```bash
./target/debug/cli proxy votesmart get-politician-bio 169020 --create-record --pretty-print
```

This will fetch the candidate bio data from Votesmart for Cori Bush, the Democratic Representative from Missouri, with the Votesmart candidate_id of 169020. The `--create--record` flag, or `-c` for short, will create a new record for the fetched politician and write the votesmart data to the `votesmart_candidate_bio` jsonb column in the candidate table. The `--pretty-print` flag, or `-p` for short, will simply print the fetched json data to the console once it has been fetched.

If a politician already exists in our database but does not yet have `votesmart_candidate_bio` data, you can add their votesmart_candidate_id to their row in the politician table, and run the above command with the `--update-record flag`, or `-u` for short. (Instead of the `-c` flag)

If you want to explore the command line api proxy utility further, you can run:

```bash
./target/debug/cli proxy --help
```

## Testing

`cargo test`

## Deploying

Deploys to the staging environment happen automatically when changes are pushed or merged to the `main` branch. Be sure to run `cargo sqlx prepare` from the `/db` root and commit the changes to the `sqlx-data.json` file if you change the schema. This file is used during build time to validate the SQL queries.

To run the migrations, **make sure you're on branch `main`** and set the `DATABASE_URL` to the URI found on our [Heroku datastore dashboard], under "View Credentials." Then run `sqlx migrate run` from your local machine. This is a temporary solution until we figure out how to automatically run the migrations on each deploy.

Deploys to production happen manually via the Heroku dashboard. Press the "Promote to Production" button on the staging app in the [pipeline view]. You can access logs to the production server by running `heroku logs --tail -a populist-api-production`

[rust installed]: https://www.rust-lang.org/tools/install
[sqlx-cli]: https://crates.io/crates/sqlx-cli
[sqlx]: https://crates.io/crates/sqlx
[heroku datastore dashboard]: https://data.heroku.com/datastores/35cb347f-6fb1-488f-8f21-02bbd726f5a8#administration
[pipeline view]: https://dashboard.heroku.com/pipelines/3ce13ae5-d2aa-4522-b513-3b3ba0e6f179
