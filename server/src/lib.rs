use async_graphql::extensions::ApolloTracing;
use axum::{
    extract::Host,
    handler::HandlerWithoutStateExt,
    http::{StatusCode, Uri},
    response::Redirect,
    routing::get,
    BoxError,
};
use axum_server::tls_rustls::RustlsConfig;
use dotenv::dotenv;
use graphql::{context::ApiContext, new_schema};
use std::{env, net::SocketAddr, path::PathBuf};
use tower_cookies::CookieManagerLayer;
use tower_http::cors::CorsLayer;
use tracing::info;
use tracing_subscriber::EnvFilter;

mod cron;
pub mod jobs;
pub use cron::init_job_schedule;
pub use jobs::*;
mod handlers;
pub use handlers::{graphql_handler, graphql_playground};

#[allow(dead_code)]
#[derive(Clone, Copy)]
struct Ports {
    http: u16,
    https: u16,
}

pub async fn run() {
    dotenv().ok();

    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .with_writer(std::io::stderr)
        .init();

    db::init_pool().await.unwrap();
    let pool = db::pool().await.to_owned();

    // Run cron jobs in separate thread
    tokio::spawn(cron::init_job_schedule());

    // Embed migrations into binary
    let migrator = pool.connection.clone();
    sqlx::migrate!("../db/migrations")
        .run(&migrator)
        .await
        .unwrap();

    let context = ApiContext::new(pool.connection.clone());

    let schema = new_schema().data(context).extension(ApolloTracing).finish();

    let app = axum::Router::new()
        .route("/", get(graphql_playground).post(graphql_handler))
        .with_state(schema)
        .layer(CorsLayer::very_permissive())
        .layer(CookieManagerLayer::new());

    let env_port = env::var("PORT").unwrap_or_else(|_| "1234".to_string());
    let port = env_port.parse::<u16>().unwrap_or_else(|_| 1234);

    let ports = Ports {
        http: port,
        https: 1235,
    };
    tokio::spawn(redirect_http_to_https(ports));

    // configure certificate and private key used by https
    let config = RustlsConfig::from_pem_file(
        PathBuf::from("certs/cert.pem"),
        PathBuf::from("certs/key.pem"),
    )
    .await
    .unwrap();

    info!("GraphQL Playground live at http://localhost:{}", &port);
    let addr = SocketAddr::from(([127, 0, 0, 1], ports.https));

    axum_server::bind_rustls(addr, config)
        .serve(app.into_make_service_with_connect_info::<SocketAddr>())
        .await
        .unwrap()
}

#[allow(dead_code)]
async fn redirect_http_to_https(ports: Ports) {
    fn make_https(host: String, uri: Uri, ports: Ports) -> Result<Uri, BoxError> {
        let mut parts = uri.into_parts();

        parts.scheme = Some(axum::http::uri::Scheme::HTTPS);

        if parts.path_and_query.is_none() {
            parts.path_and_query = Some("/".parse().unwrap());
        }

        let https_host = host.replace(&ports.http.to_string(), &ports.https.to_string());
        parts.authority = Some(https_host.parse()?);

        Ok(Uri::from_parts(parts)?)
    }

    let redirect = move |Host(host): Host, uri: Uri| async move {
        match make_https(host, uri, ports) {
            Ok(uri) => Ok(Redirect::permanent(&uri.to_string())),
            Err(error) => {
                tracing::warn!(%error, "failed to convert URI to HTTPS");
                Err(StatusCode::BAD_REQUEST)
            }
        }
    };

    let addr = SocketAddr::from(([127, 0, 0, 1], ports.http));
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    tracing::debug!("listening on {}", listener.local_addr().unwrap());
    axum::serve(
        listener,
        redirect.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .await
    .unwrap();
}
