use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// Detection model path
    pub det_model_path: Option<PathBuf>,
    /// Angle classification model path
    pub cls_model_path: Option<PathBuf>,
    /// Recognition model path
    pub rec_model_path: Option<PathBuf>,
    /// Character dictionary path
    pub dict_path: Option<PathBuf>,
    /// ORT日志级别配置
    pub ort_log_level: Option<String>,
    /// Whether to use angle classification
    pub use_angle_cls: Option<bool>,
    /// Whether to use direction classification
    pub use_direction_cls: Option<bool>,
    /// Detection parameters
    pub detection: Option<DetectionParams>,
    /// Recognition parameters
    pub recognition: Option<RecognitionParams>,
    /// Output parameters
    pub output: Option<OutputParams>,
    /// Server parameters
    pub server: Option<ServerParams>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DetectionParams {
    /// Detection box limit
    pub box_limit: Option<u32>,
    /// Detection maximum box size
    pub box_thresh: Option<f32>,
    /// Detection minimum box size
    pub min_box_size: Option<u32>,
    /// Box threshold ratio
    pub unclip_ratio: Option<f32>,
    /// Whether to use direction classification
    pub use_dilation: Option<bool>,
    /// Whether to use polygon score
    pub use_polygon_score: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecognitionParams {
    /// Whether to use log
    pub use_log: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutputParams {
    /// Whether to include confidence information
    pub include_confidence: Option<bool>,
    /// Whether to format JSON output prettily
    pub pretty_json: Option<bool>,
    /// Whether to include processing time information
    pub include_processing_time: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerParams {
    /// Bind address
    pub bind_addr: Option<String>,
    /// Number of threads
    pub num_threads: Option<i32>,
    /// Log level (error, warn, info, debug, trace)
    pub log_level: Option<String>,
}

impl Default for OutputParams {
    fn default() -> Self {
        Self {
            include_confidence: Some(false),
            pretty_json: Some(false),
            include_processing_time: Some(false),
        }
    }
}

impl Default for ServerParams {
    fn default() -> Self {
        Self {
            bind_addr: None,
            num_threads: None,
            log_level: None,
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            det_model_path: None,
            cls_model_path: None,
            rec_model_path: None,
            dict_path: None,
            ort_log_level: Some("warning".to_string()),
            use_angle_cls: Some(false),
            use_direction_cls: Some(false),
            detection: None,
            recognition: None,
            output: None,
            server: None,
        }
    }
}

impl Config {
    pub fn from_file(path: &PathBuf) -> Result<Self, Box<dyn std::error::Error>> {
        let content = std::fs::read_to_string(path)?;
        let config: Config = serde_yaml::from_str(&content)?;
        Ok(config)
    }

    pub fn merge_with_ocr_args(&mut self, args: &crate::cli::args::OcrArgs) {
        // 合并ORT日志级别
        if let Some(ref ort_log_level) = args.ort_log_level {
            self.ort_log_level = Some(ort_log_level.clone());
        }
        if let Some(det_model) = &args.det_model {
            self.det_model_path = Some(PathBuf::from(det_model));
        }
        if let Some(cls_model) = &args.cls_model {
            self.cls_model_path = Some(PathBuf::from(cls_model));
        }
        if let Some(rec_model) = &args.rec_model {
            self.rec_model_path = Some(PathBuf::from(rec_model));
        }
        if let Some(dict) = &args.dict_path {
            self.dict_path = Some(PathBuf::from(dict));
        }
        if args.use_angle_cls {
            self.use_angle_cls = Some(true);
        }
        if args.use_direction_cls {
            self.use_direction_cls = Some(true);
        }

        // Handle output parameters
        if self.output.is_none() {
            self.output = Some(OutputParams::default());
        }

        if args.include_confidence {
            self.output.as_mut().unwrap().include_confidence = Some(true);
        }
        if args.pretty_json {
            self.output.as_mut().unwrap().pretty_json = Some(true);
        }
        if args.include_processing_time {
            self.output.as_mut().unwrap().include_processing_time = Some(true);
        }
    }

    pub fn get_det_model_path(&self) -> PathBuf {
        self.det_model_path
            .clone()
            .unwrap_or_else(|| PathBuf::from("./models/ch_PP-OCRv5_mobile_det.onnx"))
    }

    pub fn get_cls_model_path(&self) -> PathBuf {
        self.cls_model_path
            .clone()
            .unwrap_or_else(|| PathBuf::from("./models/ch_ppocr_mobile_v2.0_cls_infer.onnx"))
    }

    pub fn get_rec_model_path(&self) -> PathBuf {
        self.rec_model_path
            .clone()
            .unwrap_or_else(|| PathBuf::from("./models/ch_PP-OCRv5_rec_mobile_infer.onnx"))
    }

    pub fn get_dict_path(&self) -> Option<PathBuf> {
        self.dict_path.clone()
    }

    pub fn get_use_angle_cls(&self) -> bool {
        self.use_angle_cls.unwrap_or(false)
    }

    pub fn get_use_direction_cls(&self) -> bool {
        self.use_direction_cls.unwrap_or(false)
    }

    #[cfg(feature = "server")]
    pub fn merge_with_server_args(&mut self, args: &crate::cli::args::ServerArgs) {
        // 合并ORT日志级别
        if let Some(ref ort_log_level) = args.ort_log_level {
            self.ort_log_level = Some(ort_log_level.clone());
        }

        // Merge server parameters
        if self.server.is_none() {
            self.server = Some(ServerParams::default());
        }

        if let Some(ref mut server_config) = self.server {
            if let Some(bind_addr) = &args.bind {
                server_config.bind_addr = Some(bind_addr.clone());
            }
            if let Some(threads) = args.threads {
                server_config.num_threads = Some(threads);
            }
        }

        // Merge model paths if provided
        if let Some(det_model) = &args.det_model {
            self.det_model_path = Some(PathBuf::from(det_model));
        }
        if let Some(cls_model) = &args.cls_model {
            self.cls_model_path = Some(PathBuf::from(cls_model));
        }
        if let Some(rec_model) = &args.rec_model {
            self.rec_model_path = Some(PathBuf::from(rec_model));
        }
    }

    #[cfg(feature = "server")]
    pub fn get_server_bind_addr(&self) -> String {
        self.server
            .as_ref()
            .and_then(|s| s.bind_addr.clone())
            .unwrap_or_else(|| "0.0.0.0:8080".to_string())
    }

    #[cfg(feature = "server")]
    pub fn get_server_num_threads(&self) -> i32 {
        self.server
            .as_ref()
            .and_then(|s| s.num_threads)
            .unwrap_or(4)
    }

  }

