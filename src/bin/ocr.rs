use pp_ocr_rs::cli::{args::{Args, Commands, OcrArgs}, batch::BatchProcessor, config::Config, engine::OcrEngine, output::OutputFormatter};
use clap::Parser;
use std::error::Error;
use std::process;
use std::sync::Arc;

fn main() {
    let args = Args::parse();

    match args.command {
        Commands::Ocr(ocr_args) => {
            handle_ocr_command(ocr_args);
        }
        #[cfg(feature = "server")]
        Commands::Serve(server_args) => {
            handle_serve_command(server_args);
        }
    }
}

#[cfg(not(feature = "server"))]
fn handle_serve_command(_args: std::convert::Infallible) {
    eprintln!("Error: Server feature is not enabled. Please build with --features server");
    process::exit(1);
}

fn handle_ocr_command(args: OcrArgs) {
    if let Err(e) = validate_ocr_args(&args) {
        eprintln!("Argument error: {}", e);
        process::exit(1);
    }

    let log_level = if args.verbose {
        log::LevelFilter::Debug
    } else if args.quiet {
        log::LevelFilter::Error
    } else {
        log::LevelFilter::Info
    };

    env_logger::Builder::from_default_env()
        .filter_level(log_level)
        .init();

    // Load configuration
    let mut config = if let Some(config_path) = &args.config {
        match Config::from_file(config_path) {
            Ok(c) => {
                if !args.quiet {
                    eprintln!("Configuration file loaded: {}", config_path.display());
                }
                c
            }
            Err(e) => {
                eprintln!("Failed to load configuration file {}: {}", config_path.display(), e);
                process::exit(1);
            }
        }
    } else {
        Config::default()
    };

    config.merge_with_ocr_args(&args);

    // Get output settings from merged configuration or command line arguments
    // Command line arguments have higher priority
    let pretty_json = args.pretty_json || config.output
        .as_ref()
        .and_then(|o| o.pretty_json)
        .unwrap_or(false);
    let include_confidence = args.include_confidence || config.output
        .as_ref()
        .and_then(|o| o.include_confidence)
        .unwrap_or(false);
    let include_processing_time = args.include_processing_time || config.output
        .as_ref()
        .and_then(|o| o.include_processing_time)
        .unwrap_or(false);

    // Create OCR engine
    let mut engine = match OcrEngine::new(config) {
        Ok(e) => e,
        Err(e) => {
            eprintln!("Failed to initialize OCR engine: {}", e);

            // If it's an ONNX Runtime error, provide more debugging information
            if let Some(source) = e.source() {
                eprintln!("\nDetailed error information:");
                eprintln!("{}", source);

                // Provide solutions based on error type
                let error_str = source.to_string().to_lowercase();
                if error_str.contains("library") || error_str.contains("dylib") {
                    eprintln!("\nPossible solutions:");
                    eprintln!("1. Set ONNX Runtime library path:");
                    eprintln!("   export DYLD_LIBRARY_PATH=$PWD/lib:$DYLD_LIBRARY_PATH");
                    eprintln!("2. Ensure system architecture matches (current: {})", std::env::consts::ARCH);
                    eprintln!("3. Try reinstalling dependencies: cargo clean && cargo build --release");
                } else if error_str.contains("file") || error_str.contains("not found") {
                    eprintln!("\nPossible solutions:");
                    eprintln!("1. Check if model files exist:");
                    eprintln!("   ls -la models/");
                    eprintln!("2. Ensure configuration file path is correct");
                    eprintln!("3. Check file permissions");
                } else if error_str.contains("memory") || error_str.contains("out of memory") {
                    eprintln!("\nPossible solutions:");
                    eprintln!("1. Increase available system memory");
                    eprintln!("2. Try using a smaller model");
                    eprintln!("3. Close other memory-consuming programs");
                }
            }

            process::exit(1);
        }
    };

    let formatter = Arc::new(OutputFormatter::new(pretty_json, include_confidence, include_processing_time));

    let processor = BatchProcessor::new(formatter.clone(), Arc::new(args.clone()));

    let result = if args.input.is_file() {
        if !args.quiet {
            eprintln!("Processing single file: {}", args.input.display());
        }
        processor.process_single_file(&args.input, &mut engine)
    } else if args.input.is_dir() {
        if !args.quiet {
            eprintln!("Processing directory: {}", args.input.display());
        }
        processor.process_directory(&args.input, &mut engine)
    } else {
        eprintln!("Error: Input path is neither a file nor a directory");
        process::exit(1);
    };

    if let Err(e) = result {
        eprintln!("Processing failed: {}", e);
        process::exit(1);
    }
}

#[cfg(feature = "server")]
fn handle_serve_command(args: pp_ocr_rs::cli::args::ServerArgs) {
    use std::net::SocketAddr;
    use std::str::FromStr;
    use pp_ocr_rs::server::{Server, config::ServerConfig};

    fn ort_level_to_filter(level: &str) -> log::LevelFilter {
        match level.to_lowercase().as_str() {
            "verbose" | "trace" => log::LevelFilter::Trace,
            "debug" => log::LevelFilter::Debug,
            "info" => log::LevelFilter::Info,
            "warning" | "warn" => log::LevelFilter::Warn,
            "error" | "fatal" => log::LevelFilter::Error,
            _ => log::LevelFilter::Warn, // 默认警告级别
        }
    }

    // Load configuration
    let mut config = if let Some(config_path) = &args.config {
        match pp_ocr_rs::cli::config::Config::from_file(config_path) {
            Ok(c) => {
                eprintln!("Configuration file loaded: {}", config_path.display());
                c
            }
            Err(e) => {
                eprintln!("Failed to load configuration file {}: {}", config_path.display(), e);
                process::exit(1);
            }
        }
    } else {
        pp_ocr_rs::cli::config::Config::default()
    };

    config.merge_with_server_args(&args);

    let log_level = if args.verbose {
        log::LevelFilter::Debug
    } else if args.quiet {
        log::LevelFilter::Error
    } else if let Some(server_config) = &config.server {
        if let Some(level_str) = &server_config.log_level {
            match log::LevelFilter::from_str(level_str.to_lowercase().as_str()) {
                Ok(level) => level,
                Err(_) => {
                    eprintln!("Invalid log level in config: '{}', using default: info", level_str);
                    log::LevelFilter::Info
                }
            }
        } else {
            log::LevelFilter::Info
        }
    } else {
        log::LevelFilter::Info
    };

    // Initialize logger for server with ORT filtering
    let mut builder = env_logger::Builder::from_default_env();
    builder.filter_level(log_level);

    if let Some(ref ort_level) = config.ort_log_level {
        let ort_filter_level = ort_level_to_filter(ort_level);

        builder.filter_module("ort", ort_filter_level);
        builder.filter_module("ort::logging", ort_filter_level);
        builder.filter_module("ort::session", ort_filter_level);
        builder.filter_module("ort::environment", ort_filter_level);

        eprintln!("ORT Runtime init log level: {}", ort_level);
    }

    builder.format_timestamp_secs().init();

    log::info!("Starting pp-ocr API server with log level: {:?}", log_level);

    let bind_addr_str = config.get_server_bind_addr();
    let bind_addr: SocketAddr = match bind_addr_str.parse() {
        Ok(addr) => addr,
        Err(e) => {
            eprintln!("Invalid bind address '{}': {}", bind_addr_str, e);
            process::exit(1);
        }
    };

    let mut server_config = ServerConfig::default();
    server_config.bind_addr = bind_addr;
    server_config.num_threads = config.get_server_num_threads();
    server_config.ort_log_level = config.ort_log_level.clone();

    server_config.det_model_path = config.get_det_model_path();
    server_config.cls_model_path = config.get_cls_model_path();
    server_config.rec_model_path = config.get_rec_model_path();

    server_config.include_confidence = config.output
        .as_ref()
        .and_then(|o| o.include_confidence)
        .unwrap_or(false);

    if !args.verbose && !args.quiet {
        if let Some(server_config_yaml) = &config.server {
            if let Some(level_str) = &server_config_yaml.log_level {
                if let Err(e) = server_config.set_log_level_from_str(level_str) {
                    eprintln!("Warning: {}", e);
                }
            }
        }
    }

    let server = Server::new(server_config);

    println!("Starting pp-ocr API server on http://{}", bind_addr);
    log::info!("Server configured to bind on: {}", bind_addr);
    log::info!("Access logs will be displayed for incoming requests");

    if let Err(e) = tokio::runtime::Runtime::new()
        .unwrap()
        .block_on(server.run())
    {
        log::error!("Server error: {}", e);
        eprintln!("Server error: {}", e);
        process::exit(1);
    }
}

fn validate_ocr_args(args: &OcrArgs) -> Result<(), String> {
    if !args.input.exists() {
        return Err(format!("Input path does not exist: {}", args.input.display()));
    }

    if args.format != "json" && args.format != "text" {
        return Err("Output format must be 'json' or 'text'".to_string());
    }

    if args.box_limit == 0 {
        return Err("box_limit must be greater than 0".to_string());
    }

    if args.max_box_size == 0 {
        return Err("max_box_size must be greater than 0".to_string());
    }

    if !(0.0..=1.0).contains(&args.box_thresh) {
        return Err("box_thresh must be between 0.0 and 1.0".to_string());
    }

    if !(0.0..=1.0).contains(&args.min_box_size) {
        return Err("min_box_size must be between 0.0 and 1.0".to_string());
    }

    if args.unclip_ratio <= 0.0 {
        return Err("unclip_ratio must be greater than 0".to_string());
    }

    Ok(())
}