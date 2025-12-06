use crate::core::ocr_result::OcrResult;
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};
use std::io::Write;

#[derive(Debug, Serialize, Deserialize)]
pub struct JsonOcrResult {
    /// Image path
    pub image: String,
    /// Processing timestamp
    pub timestamp: String,
    /// OCR recognition results
    pub results: Vec<JsonTextBlock>,
    /// Processing time in milliseconds
    pub processing_time_ms: u64,
    /// Error message (if any)
    pub error: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct JsonTextBlock {
    /// Recognized text
    pub text: String,
    /// Confidence
    pub confidence: f32,
    /// Text box coordinate points
    pub r#box: JsonTextBox,
    /// Angle information
    pub angle: Option<JsonAngle>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct JsonTextBox {
    /// Four corner coordinate points
    pub points: Vec<(u32, u32)>,
    /// Detection confidence
    pub confidence: f32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct JsonAngle {
    /// Angle index
    pub index: i32,
    /// Angle confidence
    pub confidence: f32,
}

pub struct OutputFormatter {
    pretty: bool,
    include_confidence: bool,
    include_processing_time: bool,
}

impl OutputFormatter {
    pub fn new(pretty: bool, include_confidence: bool, include_processing_time: bool) -> Self {
        Self {
            pretty,
            include_confidence,
            include_processing_time,
        }
    }

    pub fn format_json(
        &self,
        image_path: &Path,
        result: &Result<OcrResult, String>,
        processing_time_ms: u64,
    ) -> Result<String, Box<dyn std::error::Error>> {
        let image_path_str = image_path.to_string_lossy().to_string();
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)?
            .as_secs()
            .to_string();

        let json_result = match result {
            Ok(ocr_result) => JsonOcrResult {
                image: image_path_str,
                timestamp,
                results: ocr_result
                    .text_blocks
                    .iter()
                    .map(|block| JsonTextBlock {
                        text: block.text.clone(),
                        confidence: block.text_score,
                        r#box: JsonTextBox {
                            points: block
                                .box_points
                                .iter()
                                .map(|p| (p.x, p.y))
                                .collect(),
                            confidence: block.box_score,
                        },
                        angle: if block.angle_index >= 0 {
                            Some(JsonAngle {
                                index: block.angle_index,
                                confidence: block.angle_score,
                            })
                        } else {
                            None
                        },
                    })
                    .collect(),
                processing_time_ms,
                error: None,
            },
            Err(e) => JsonOcrResult {
                image: image_path_str,
                timestamp,
                results: vec![],
                processing_time_ms,
                error: Some(e.to_string()),
            },
        };

        if self.pretty {
            Ok(serde_json::to_string_pretty(&json_result)?)
        } else {
            Ok(serde_json::to_string(&json_result)?)
        }
    }

    pub fn format_text(
        &self,
        image_path: &Path,
        result: &Result<OcrResult, String>,
        processing_time_ms: u64,
    ) -> String {
        let mut output = String::new();

        // Add image path
        output.push_str(&format!("Image: {}\n", image_path.display()));

        match result {
            Ok(ocr_result) => {
                if ocr_result.text_blocks.is_empty() {
                    output.push_str("No text detected\n");
                } else {
                    for (i, block) in ocr_result.text_blocks.iter().enumerate() {
                        output.push_str(&format!("Text block {}\n", i + 1));
                        output.push_str(&format!("  Text: {}\n", block.text));

                        if self.include_confidence {
                            output.push_str(&format!("  Confidence: {:.2}%\n", block.text_score * 100.0));
                            output.push_str(&format!("  Box confidence: {:.2}%\n", block.box_score * 100.0));

                            if block.angle_index >= 0 {
                                output.push_str(&format!(
                                    "  Angle: index={}, confidence={:.2}%\n",
                                    block.angle_index,
                                    block.angle_score * 100.0
                                ));
                            }
                        }

                        output.push('\n');
                    }
                }
            }
            Err(e) => {
                output.push_str(&format!("Error: {}\n", e));
            }
        }

        // Show processing time if enabled
        if self.include_processing_time {
            if processing_time_ms < 1000 {
                output.push_str(&format!("Processing time: {} ms\n", processing_time_ms));
            } else {
                output.push_str(&format!("Processing time: {:.2} s\n", processing_time_ms as f64 / 1000.0));
            }
        }

        output.push_str("---\n");
        output
    }
}

pub fn write_to_file_or_stdout(
    content: &str,
    output_path: Option<&Path>,
    append: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    match output_path {
        Some(path) => {
            // Ensure parent directory exists
            if let Some(parent) = path.parent() {
                std::fs::create_dir_all(parent)?;
            }

            if append {
                std::fs::OpenOptions::new()
                    .create(true)
                    .append(true)
                    .open(path)?
                    .write_all(content.as_bytes())?;
            } else {
                std::fs::write(path, content)?;
            }

            if !atty::is(atty::Stream::Stdout) {
                eprintln!("Results written to: {}", path.display());
            }
        }
        None => {
            print!("{}", content);
        }
    }

    Ok(())
}