use axum::{Json, Router, http::StatusCode, routing::get};

use serde::Serialize;

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
    let app = Router::<()>::new()
        .route("/", get(root))
        .route("/health", get(health));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();

    axum::serve(listener, app).await.unwrap();
}
