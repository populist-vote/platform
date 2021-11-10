# Populist Platform

Populist Database Interface and GraphQL API Server

## Getting Started
Make sure you have [Rust installed] on your machine.  Next, you'll need the [`sqlx-cli`] installed to manage the database connection and run migrations.  To do so, run `cargo install sqlx-cli --features postgres` 

First copy the `.env.example` file to `.env` which is .gitignored.  
`cp .env.example .env` For local development, you can then run `sqlx db create` to create a new Postgres database at the url defined in your new `.env` 

Next, you'll need to run the migrations with `sqlx migrate run`

## SQLx
[`sqlx`] is an amazing tool for managing asyncronous database operations.  This repository relies heavily on compile-time query verification using `sqlx` macros, namely `query_as!`  If you do not have a DATABASE_URL specified in your .env file, you will not be able to compile the binary for this crate.  You can run sqlx in offline mode by setting SQLX_OFFLINE=true.  You can enable "offline mode" to cache the results of the SQL query analysis using the sqlx-cli.  If you make schema alterations, run the command `cargo sqlx prepare` which will write your query data to `sqlx-data.json` at the `/db` root

## 









[Rust installed]: https://www.rust-lang.org/tools/install
[`sqlx-cli`]: https://crates.io/crates/sqlx-cli