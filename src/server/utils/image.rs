use base64::{engine::general_purpose, Engine as _};
use image::{ImageFormat, DynamicImage};
use crate::core::ocr_error::OcrError;

pub fn decode_base64_image(data_url: &str) -> Result<Vec<u8>, OcrError> {
    // Remove data URL prefix if present
    let base64_data = if data_url.starts_with("data:image/") {
        if let Some(comma_pos) = data_url.find(',') {
            &data_url[comma_pos + 1..]
        } else {
            data_url
        }
    } else {
        data_url
    };

    // Decode base64
    general_purpose::STANDARD
        .decode(base64_data)
        .map_err(|e| OcrError::Custom(format!("Invalid base64 data: {}", e)))
}

pub fn validate_image_format(image_data: &[u8]) -> Result<(), OcrError> {
    // Try to decode the image to verify it's valid
    image::load_from_memory(image_data)
        .map_err(|e| OcrError::Custom(format!("Invalid image format: {}", e)))?;
    Ok(())
}

pub fn dynamic_image_to_bytes(image: DynamicImage, format: ImageFormat) -> Result<Vec<u8>, OcrError> {
    let mut buffer = Vec::new();
    image.write_to(&mut std::io::Cursor::new(&mut buffer), format)
        .map_err(|e| OcrError::Custom(format!("Failed to encode image: {}", e)))?;
    Ok(buffer)
}