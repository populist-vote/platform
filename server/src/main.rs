use async_graphql::{
    extensions::ApolloTracing,
    http::{playground_source, GraphQLPlaygroundConfig},
};
use async_graphql_poem::{GraphQLRequest, GraphQLResponse, GraphQLSubscription};
use auth::jwt;
use config::Environment;
use dotenv::dotenv;
use graphql::{context::ApiContext, new_schema, PopulistSchema};
use poem::{
    get, handler,
    http::HeaderMap,
    listener::TcpListener,
    middleware::{Compression, CookieJarManager, Cors},
    web::{cookie::CookieJar, Data, Html},
    EndpointExt, IntoResponse, Route, Server,
};
use regex::Regex;
use std::str::FromStr;
use tracing::info;
use tracing_subscriber::EnvFilter;

#[handler]
fn root() -> impl IntoResponse {
    Html(r#"<h1>Populist API Docs</h1>"#)
}

#[handler]
async fn graphql_handler(
    schema: Data<&PopulistSchema>,
    req: GraphQLRequest,
    headers: &HeaderMap,
    cookie_jar: &CookieJar,
) -> GraphQLResponse {
    let bearer_token = headers
        .get("authorization")
        .and_then(|header| header.to_str().ok())
        .and_then(|header| header.split_whitespace().nth(1));

    let bearer_token_data = if let Some(token) = bearer_token {
        let token_data = jwt::validate_token(token);
        if let Ok(token_data) = token_data {
            Some(token_data)
        } else {
            None
        }
    } else {
        None
    };

    let cookie = cookie_jar.get("access_token");
    let cookie_token_data = if let Some(cookie) = cookie {
        let token_data = jwt::validate_token(cookie.value_str());
        if let Ok(token_data) = token_data {
            Some(token_data)
        } else {
            None
        }
    } else {
        None
    };

    // Use the bearer token if it's present, otherwise use the cookie
    let token_data = bearer_token_data.or(cookie_token_data);
    schema.execute(req.0.data(token_data)).await.into()
}

#[handler]
fn graphql_playground() -> impl IntoResponse {
    Html(playground_source(
        GraphQLPlaygroundConfig::new("/").subscription_endpoint("/ws"),
    ))
}

pub fn cors(environment: Environment) -> Cors {
    let cors = Cors::new().allow_credentials(true);

    fn allowed_staging_origins(origin: &str) -> bool {
        let staging_origins = vec![
            "https://populist-api-staging.herokuapp.com",
            "https://api.staging.populist.us",
            "https://staging.populist.us",
            "http://localhost:3030",
        ];
        let re = Regex::new(r"https://web-.*?-populist\.vercel\.app$").unwrap();
        re.is_match(origin) || staging_origins.contains(&origin)
    }

    match environment {
        Environment::Local => cors,
        Environment::Staging => cors.allow_origins_fn(allowed_staging_origins),
        Environment::Production => cors.allow_origins(vec![
            "http://localhost:3030",
            "https://populist-api-production.herokuapp.com",
            "https://api.populist.us",
            "https://populist.us",
            "https://www.populist.us",
            "https://web-five-kohl.vercel.app",
            "https://web-populist.vercel.app",
            "https://web-git-main-populist.vercel.app",
        ]),
        _ => cors,
    }
}

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    dotenv().ok();

    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .with_writer(std::io::stderr)
        .init();

    db::init_pool().await.unwrap();
    let pool = db::pool().await;

    // Embed migrations into binary
    let migrator = pool.connection.clone();
    sqlx::migrate!("../db/migrations")
        .run(&migrator)
        .await
        .unwrap();

    let context = ApiContext::new(pool.connection.clone());

    let schema = new_schema().data(context).extension(ApolloTracing).finish();

    let environment = Environment::from_str(&std::env::var("ENVIRONMENT").unwrap()).unwrap();
    let port = std::env::var("PORT").unwrap_or_else(|_| "1234".to_string());

    let app = Route::new()
        .at("/", get(graphql_playground).post(graphql_handler))
        .at("/ws", get(GraphQLSubscription::new(schema.clone())))
        .data(schema)
        .with(cors(environment))
        // Will need to implement a custom X-Forwarded-Proto header for Heroku to
        // get https redirects to work
        // https://help.heroku.com/VKLSBMJS/why-am-i-getting-a-message-too-many-redirects
        // .with_if(environment != Environment::Local, ForceHttps::default())
        .with(Compression::default())
        .with(CookieJarManager::default());

    let address = format!("0.0.0.0:{}", port);
    info!("GraphQL Playground live at http://localhost:{}", &port);
    let listener = TcpListener::bind(&address);

    Server::new(listener).run(app).await
}
