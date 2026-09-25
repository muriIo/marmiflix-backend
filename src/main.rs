use std::time::Duration;

use axum::{Json, Router, http::StatusCode, routing::get};

use serde::{Deserialize, Serialize};
use tower::ServiceBuilder;
use tower_http::{
    ServiceBuilderExt, compression::CompressionLayer, cors::CorsLayer,
    limit::RequestBodyLimitLayer, request_id::MakeRequestUuid, timeout::TimeoutLayer,
    trace::TraceLayer,
};

mod queue;

#[derive(Clone)]
struct AppState {
    config: Config,
}

#[derive(Debug, Deserialize, Clone)]
struct Config {
    upstash_redis_rest_url: String,
    upstash_redis_rest_token: String,
    vapid_public_key: String,
    vapid_private_key: String,
    vapid_subject: String,
    queue_confirm_window_seconds: String,
    queue_heating_nominal_seconds: String,
    queue_heating_urgency_seconds: String,
    queue_per_person_wait_seconds: String,
    sentry_dsn: String,
}

impl Config {
    fn from_env() -> Result<Self, Box<dyn std::error::Error>> {
        dotenvy::dotenv_override()?;

        let config: Config = envy::from_env().expect("invalid config");

        Ok(config)
    }
}

#[derive(Serialize)]
struct HealthResponse {
    status_code: u16,
}

async fn root() -> &'static str {
    "hello world"
}

async fn health() -> Json<HealthResponse> {
    Json(HealthResponse {
        status_code: StatusCode::OK.as_u16(),
    })
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter("info,tower_http=debug")
        .init();

    let config: Config = match Config::from_env() {
        Ok(loaded) => loaded,
        Err(err) => {
            eprintln!("{:#?}", err);

            return;
        }
    };

    let state = AppState { config };

    let app = Router::new()
        .route("/", get(root))
        .route("/health", get(health))
        .with_state(state)
        .layer(
            ServiceBuilder::new()
                .set_x_request_id(MakeRequestUuid::default())
                .propagate_x_request_id()
                .layer(TraceLayer::new_for_http())
                .layer(CorsLayer::permissive()) // troque por política real
                .layer(TimeoutLayer::with_status_code(
                    StatusCode::REQUEST_TIMEOUT,
                    Duration::from_secs(10),
                ))
                .layer(RequestBodyLimitLayer::new(1024 * 1024)) // 1 MiB
                .layer(CompressionLayer::new()),
        );

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();

    axum::serve(listener, app).await.unwrap();
}
