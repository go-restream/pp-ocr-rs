#[cfg(feature = "server")]
use axum::response::Json;
#[cfg(feature = "server")]
use std::time::{SystemTime, UNIX_EPOCH};

#[cfg(feature = "server")]
use super::super::models::HealthResponse;

#[cfg(feature = "server")]
pub async fn health() -> Json<HealthResponse> {
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();

    Json(HealthResponse {
        status: "healthy".to_string(),
        timestamp,
    })
}