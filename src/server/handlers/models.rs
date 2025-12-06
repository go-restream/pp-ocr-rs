#[cfg(feature = "server")]
use axum::response::Json;
#[cfg(feature = "server")]
use std::time::{SystemTime, UNIX_EPOCH};

#[cfg(feature = "server")]
use super::super::models::{ModelsResponse, ModelInfo};

#[cfg(feature = "server")]
pub async fn get_models() -> Json<ModelsResponse> {
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();

    let model_list = vec![
        ModelInfo {
            id: "ch_pp_ocr_v5_mobile".to_string(),
            object: "model".to_string(),
            created: timestamp,
            owned_by: "pp-ocr".to_string(),
        },
        ModelInfo {
            id: "ch_pp_ocr_v5_server".to_string(),
            object: "model".to_string(),
            created: timestamp,
            owned_by: "pp-ocr".to_string(),
        },
    ];

    Json(ModelsResponse {
        object: "list".to_string(),
        data: model_list,
    })
}