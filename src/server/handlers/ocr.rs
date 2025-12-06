#[cfg(feature = "server")]
use axum::{
    extract::State,
    http::StatusCode,
    response::Json,
};
#[cfg(feature = "server")]
use std::time::{SystemTime, UNIX_EPOCH, Instant};

#[cfg(feature = "server")]
use super::super::{
    models::{
        ChatCompletionRequest, ChatCompletionResponse, Choice, Message, Usage,
        ErrorResponse, ErrorDetail, ContentItem,
    },
    utils::decode_base64_image,
    server::AppState,
};

#[cfg(feature = "server")]
pub async fn chat_completions(
    State(state): State<AppState>,
    Json(request): Json<ChatCompletionRequest>,
) -> Result<Json<ChatCompletionResponse>, (StatusCode, Json<ErrorResponse>)> {
    // Validate model
    if !["ch_pp_ocr_v5_mobile", "ch_pp_ocr_v5_server"].contains(&request.model.as_str()) {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                error: ErrorDetail {
                    message: "Invalid model. Supported models: ch_pp_ocr_v5_mobile, ch_pp_ocr_v5_server".to_string(),
                    error_type: "invalid_request_error".to_string(),
                    code: None,
                },
            }),
        ));
    }

    // Extract image from messages
    let image_data = if let Some(message) = request.messages.last() {
        if let Some(content_item) = message.content.iter().find(|item| item.item_type == "image_url") {
            if let Some(image_url) = &content_item.image_url {
                match decode_base64_image(&image_url.url) {
                    Ok(data) => Some(data),
                    Err(e) => {
                        return Err((
                            StatusCode::BAD_REQUEST,
                            Json(ErrorResponse {
                                error: ErrorDetail {
                                    message: format!("Failed to decode image: {}", e),
                                    error_type: "invalid_request_error".to_string(),
                                    code: None,
                                },
                            }),
                        ));
                    }
                }
            } else {
                None
            }
        } else {
            None
        }
    } else {
        None
    };

    // Check if image was found
    if image_data.is_none() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                error: ErrorDetail {
                    message: "No image found in request".to_string(),
                    error_type: "invalid_request_error".to_string(),
                    code: None,
                },
            }),
        ));
    }

    // Process OCR with timing
    let start_time = Instant::now();
    log::info!("Starting OCR processing with model: {}", request.model);

    let ocr_result = process_ocr(image_data.unwrap(), &request.model, &state).await;

    let processing_time = start_time.elapsed();
    log::info!(
        "OCR processing completed in {}ms (model: {})",
        processing_time.as_millis(),
        request.model
    );

    let (detected_text, error) = match ocr_result {
        Ok(text) => (text, None),
        Err(e) => (String::new(), Some(e)),
    };

    // If there was an error, return it
    if let Some(err) = error {
        return Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: ErrorDetail {
                    message: format!("OCR processing failed: {}", err),
                    error_type: "internal_error".to_string(),
                    code: None,
                },
            }),
        ));
    }

    // Create response
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();

    let response = ChatCompletionResponse {
        id: format!("chatcmpl-{}", uuid::Uuid::new_v4()),
        object: "chat.completion".to_string(),
        created: timestamp,
        model: request.model,
        choices: vec![Choice {
            index: 0,
            message: Message {
                role: "assistant".to_string(),
                content: vec![
                    ContentItem {
                        item_type: "text".to_string(),
                        text: Some(detected_text),
                        image_url: None,
                    },
                ],
            },
            finish_reason: "stop".to_string(),
        }],
        usage: Usage {
            prompt_tokens: 0,
            completion_tokens: 0,
            total_tokens: 0,
        },
    };

    Ok(Json(response))
}

#[cfg(feature = "server")]
async fn process_ocr(image_data: Vec<u8>, model: &str, state: &AppState) -> Result<String, String> {
    use tokio::task;
    use tracing::{debug, error, info};

    let include_confidence = state.config.include_confidence;
    let image_size = image_data.len();
    let model_name = model.to_string();

    debug!("Processing image: size={} bytes, model={}", image_size, model);

    // Check if there is an available OCR pool
    if let Some(ocr_pool) = &state.ocr_pool {
        info!("Using pre-warmed OCR engine pool");

        let acquire_start = Instant::now();
        let mut engine_handle = ocr_pool.acquire().await
            .map_err(|e| format!("Failed to acquire OCR engine: {}", e))?;
        let acquire_time = acquire_start.elapsed();
        debug!("OCR engine acquisition time: {}ms", acquire_time.as_millis());

        let load_start = Instant::now();
        let img = image::load_from_memory(&image_data)
            .map_err(|e| format!("Failed to load image from memory: {}", e))?
            .to_rgb8();
        let load_time = load_start.elapsed();
        debug!("Image loading time: {}ms", load_time.as_millis());

        // Execute OCR recognition
        let detect_start = Instant::now();
        let result = engine_handle.detect(
            &img,
            50,     // padding
            1024,   // max_side_len
            0.5,    // box_score_thresh
            0.3,    // box_thresh
            1.6,    // un_clip_ratio
            false,  // do_angle
            false,  // most_angle
        );
        let detect_time = detect_start.elapsed();
        debug!("OCR recognition time: {}ms", detect_time.as_millis());

        // Process results
        match result {
            Ok(res) => {
                let block_count = res.text_blocks.len();
                info!(
                    "OCR recognition completed: {} text blocks, total time: {}ms (acquire: {}ms, recognition: {}ms)",
                    block_count,
                    acquire_time.as_millis() + detect_time.as_millis(),
                    acquire_time.as_millis(),
                    detect_time.as_millis()
                );

                let mut text_results = Vec::new();

                for block in res.text_blocks {
                    if include_confidence {
                        text_results.push(format!("{} (confidence: {:.2})", block.text, block.text_score));
                    } else {
                        text_results.push(block.text);
                    }
                }
                Ok(text_results.join("\n"))
            }
            Err(e) => {
                error!("OCR processing failed: {}", e);
                Err(format!("OCR detection failed: {}", e))
            }
        }
    } else {
        info!("Not using OCR engine pool, will reinitialize model");

        let model_paths = (
            state.config.det_model_path.clone(),
            state.config.cls_model_path.clone(),
            state.config.rec_model_path.clone(),
        );

        task::spawn_blocking(move || {
            let include_confidence = include_confidence;
            let model_name = model_name;

            let init_start = Instant::now();
            let mut ocr = crate::core::ocr_lite::OcrLite::new();

            if let Err(e) = ocr.init_models(
                &model_paths.0.to_string_lossy(),
                &model_paths.1.to_string_lossy(),
                &model_paths.2.to_string_lossy(),
                model_name.contains("server").then_some(2).unwrap_or(4),
            ) {
                return Err(format!("Failed to initialize OCR models: {}", e));
            }

            let init_time = init_start.elapsed();
            debug!("OCR model initialization time: {}ms", init_time.as_millis());

            // Process image
            let load_start = Instant::now();
            let img = image::load_from_memory(&image_data)
                .map_err(|e| format!("Failed to load image from memory: {}", e))?
                .to_rgb8();
            let load_time = load_start.elapsed();
            debug!("Image loading time: {}ms", load_time.as_millis());

            let detect_start = Instant::now();
            let result = ocr.detect(
                &img,
                50,     // padding
                1024,   // max_side_len
                0.5,    // box_score_thresh
                0.3,    // box_thresh
                1.6,    // un_clip_ratio
                false,  // do_angle
                false,  // most_angle
            );
            let detect_time = detect_start.elapsed();
            debug!("OCR detection time: {}ms", detect_time.as_millis());

            let total_time = init_start.elapsed();

            match result {
                Ok(res) => {
                    let block_count = res.text_blocks.len();
                    debug!(
                        "OCR detected {} text blocks, total processing time: {}ms (init: {}ms, load: {}ms, detection: {}ms)",
                        block_count,
                        total_time.as_millis(),
                        init_time.as_millis(),
                        load_time.as_millis(),
                        detect_time.as_millis()
                    );

                    let mut text_results = Vec::new();

                    for block in res.text_blocks {
                        if include_confidence {
                            text_results.push(format!("{} (confidence: {:.2})", block.text, block.text_score));
                        } else {
                            text_results.push(block.text);
                        }
                    }
                    Ok(text_results.join("\n"))
                }
                Err(e) => {
                    error!("OCR processing failed after {}ms: {}", total_time.as_millis(), e);
                    Err(format!("OCR processing error: {}", e))
                }
            }
        }).await.map_err(|e| format!("Task join error: {}", e))?
    }
}