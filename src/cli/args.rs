use clap::Parser;
use std::path::PathBuf;

#[derive(Parser, Debug, Clone)]
#[command(name = "ocr")]
#[command(about = "A command-line OCR tool using pp-ocr-rs", long_about = None)]
#[command(version)]
pub struct Args {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Parser, Debug, Clone)]
pub enum Commands {
    /// Process OCR on images
    Ocr(OcrArgs),
    /// Start API server
    #[cfg(feature = "server")]
    Serve(ServerArgs),
}

#[derive(Parser, Debug, Clone)]
pub struct OcrArgs {
    /// Image file path or directory containing images
    #[arg(help = "Image file path or directory containing images")]
    pub input: PathBuf,

    /// YAML configuration file path
    #[arg(short, long, help = "YAML configuration file path")]
    pub config: Option<PathBuf>,

    /// ORT日志级别 (verbose, info, warning, error, fatal)
    #[arg(long, default_value = "warning", help = "ORT日志级别")]
    pub ort_log_level: Option<String>,

    /// Detection model path
    #[arg(long, help = "Detection model path")]
    pub det_model: Option<String>,

    /// Angle classification model path
    #[arg(long, help = "Angle classification model path")]
    pub cls_model: Option<String>,

    /// Recognition model path
    #[arg(long, help = "Recognition model path")]
    pub rec_model: Option<String>,

    /// Character dictionary file path
    #[arg(long, help = "Character dictionary file path")]
    pub dict_path: Option<String>,

    /// Use angle classification
    #[arg(long, help = "Use angle classification")]
    pub use_angle_cls: bool,

    /// Use direction classification
    #[arg(long, help = "Use direction classification")]
    pub use_direction_cls: bool,

    /// Output format (json or text)
    #[arg(short, long, default_value = "text", help = "Output format (json or text)")]
    pub format: String,

    /// Output file path (if not specified, output to stdout)
    #[arg(short, long, help = "Output file path (if not specified, output to stdout)")]
    pub output: Option<PathBuf>,

    /// Append mode (only valid when output file is specified)
    #[arg(long, help = "Append mode (only valid when output file is specified)")]
    pub append: bool,

    /// Recursively search subdirectories
    #[arg(short, long, help = "Recursively search subdirectories")]
    pub recursive: bool,

    /// Quiet mode (do not output progress information)
    #[arg(short, long, help = "Quiet mode (do not output progress information)")]
    pub quiet: bool,

    /// Verbose mode (output more debug information)
    #[arg(short, long, help = "Verbose mode (output more debug information)")]
    pub verbose: bool,

    /// Box limit
    #[arg(long, default_value = "50", help = "Box limit")]
    pub box_limit: u32,

    /// Maximum box size
    #[arg(long, default_value = "1024", help = "Maximum box size")]
    pub max_box_size: u32,

    /// Box threshold
    #[arg(long, default_value = "0.5", help = "Box threshold")]
    pub box_thresh: f32,

    /// Minimum box size
    #[arg(long, default_value = "0.3", help = "Minimum box size")]
    pub min_box_size: f32,

    /// Box threshold ratio
    #[arg(long, default_value = "1.6", help = "Box threshold ratio")]
    pub unclip_ratio: f32,

    /// Pretty JSON output
    #[arg(long, help = "Pretty JSON output")]
    pub pretty_json: bool,

    /// Include confidence information
    #[arg(long, help = "Include confidence information")]
    pub include_confidence: bool,

    /// Include processing time information
    #[arg(long, help = "Include processing time information")]
    pub include_processing_time: bool,
}

#[cfg(feature = "server")]
#[derive(Parser, Debug, Clone)]
pub struct ServerArgs {
    /// YAML configuration file path
    #[arg(short, long, help = "YAML configuration file path")]
    pub config: Option<PathBuf>,

    /// ORT日志级别 (verbose, info, warning, error, fatal)
    #[arg(long, default_value = "warning", help = "ORT日志级别")]
    pub ort_log_level: Option<String>,

    /// Bind address (default: 0.0.0.0:8080)
    #[arg(short, long, help = "Bind address")]
    pub bind: Option<String>,

    /// Detection model path
    #[arg(long, help = "Detection model path")]
    pub det_model: Option<String>,

    /// Angle classification model path
    #[arg(long, help = "Angle classification model path")]
    pub cls_model: Option<String>,

    /// Recognition model path
    #[arg(long, help = "Recognition model path")]
    pub rec_model: Option<String>,

    /// Number of threads for processing
    #[arg(short, long, help = "Number of threads for processing")]
    pub threads: Option<i32>,

    /// Quiet mode (do not output progress information)
    #[arg(short, long, help = "Quiet mode (do not output progress information)")]
    pub quiet: bool,

    /// Verbose mode (output more debug information)
    #[arg(short, long, help = "Verbose mode (output more debug information)")]
    pub verbose: bool,
}