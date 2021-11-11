# Populist Platform

Populist Database Interface and GraphQL API Server

## Getting Started
Make sure you have [Rust installed] on your machine.  Next, you'll need the [sqlx-cli] installed to manage the database connection and run migrations.  To do so, run `cargo install sqlx-cli --features postgres` 

First copy the `.env.example` file to `.env` which is .gitignored.  
`cp .env.example .env` For local development, you can then run `sqlx db create` to create a new Postgres database at the url defined in your new `.env` 

Next, you'll need to run the migrations with `sqlx migrate run`

## Database
[sqlx] is used for managing asyncronous database operations.  This project relies heavily on compile-time query verification using `sqlx` macros, namely `query_as!`  If you do not have a DATABASE_URL specified in your .env file, you will not be able to compile the binary for this crate.  You can run sqlx in offline mode by setting SQLX_OFFLINE=true.  You can enable "offline mode" to cache the results of the SQL query analysis using the sqlx-cli.  If you make schema alterations, run the command `cargo sqlx prepare` which will write your query data to `sqlx-data.json` at the `/db` root.

## API Server
To start the api server, run `cargo watch -x run` which will type check, compile, and run your code.  The GraphQL playground will then be live at http://localhost:3000 for you to execute queries and mutations against the specified database.  

## Architecture
todo!()

## Testing
`cargo test`

## Deploying
Deploys happen automatically when changes are pushed or merged to the `main` branch.   Be sure to run `cargo sqlx prepare` from the `/db` root and commit the changes to the `sqlx-data.json` file if you change the schema.  This file is used during build time to validate the SQL queries. 

Ultimately a staging environment will be setup with automatic deployments from the `xyz` branch.  To run the migrations, **make sure you're on branch `main`** and set the `DATABASE_URL` to the URI found on our [Heroku datastore dashboard], under "View Credentials."  Then run `sqlx migrate run` from your local machine.  This is a temporary solution until we figure out how to automatically run the migrations on each deploy.




[Rust installed]: https://www.rust-lang.org/tools/install
[`sqlx-cli`]: https://crates.io/crates/sqlx-cli
[Heroku datastore dashboard]: https://data.heroku.com/datastores/35cb347f-6fb1-488f-8f21-02bbd726f5a8#administration