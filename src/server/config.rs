use std::net::SocketAddr;
use std::path::PathBuf;
use std::str::FromStr;
use log::LevelFilter;

#[derive(Debug, Clone)]
pub struct ServerConfig {
    pub bind_addr: SocketAddr,
    pub det_model_path: PathBuf,
    pub cls_model_path: PathBuf,
    pub rec_model_path: PathBuf,
    pub num_threads: i32,
    pub include_confidence: bool,
    pub log_level: LevelFilter,
    pub ocr_pool_size: Option<usize>,
    pub warmup_on_startup: bool,
    pub ort_log_level: Option<String>, 
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            bind_addr: "0.0.0.0:8080".parse().unwrap(),
            det_model_path: PathBuf::from("./models/ch_PP-OCRv5_mobile_det.onnx"),
            cls_model_path: PathBuf::from("./models/ch_ppocr_mobile_v2.0_cls_infer.onnx"),
            rec_model_path: PathBuf::from("./models/ch_PP-OCRv5_rec_mobile_infer.onnx"),
            num_threads: 4,
            include_confidence: false,
            log_level: LevelFilter::Info,
            ocr_pool_size: None, // Default CPU core number
            warmup_on_startup: true,
            ort_log_level: None, 
        }
    }
}

impl ServerConfig {
    /// Set log level from string
    pub fn set_log_level_from_str(&mut self, level_str: &str) -> Result<(), String> {
        match LevelFilter::from_str(level_str.to_lowercase().as_str()) {
            Ok(level) => {
                self.log_level = level;
                Ok(())
            }
            Err(_) => Err(format!(
                "Invalid log level '{}'. Valid levels: error, warn, info, debug, trace",
                level_str
            )),
        }
    }

  }