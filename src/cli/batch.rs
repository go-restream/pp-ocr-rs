use crate::cli::engine::OcrEngine;
use crate::cli::output::OutputFormatter;
use crate::cli::args::OcrArgs;
use std::path::{Path, PathBuf};
use std::ffi::OsStr;
use indicatif::{ProgressBar, ProgressStyle};
use std::sync::Arc;

pub struct BatchProcessor {
    formatter: Arc<OutputFormatter>,
    args: Arc<OcrArgs>,
}

impl BatchProcessor {
    pub fn new(formatter: Arc<OutputFormatter>, args: Arc<OcrArgs>) -> Self {
        Self {
            formatter,
            args,
        }
    }

    pub fn process_directory(&self, dir_path: &Path, engine: &mut OcrEngine) -> Result<(), Box<dyn std::error::Error>> {
        let image_files = self.collect_image_files(dir_path)?;

        if image_files.is_empty() {
            eprintln!("Warning: No supported image files found in directory {}", dir_path.display());
            return Ok(());
        }

        if !self.args.quiet {
            eprintln!("Found {} image files", image_files.len());
        }

        let progress_bar = if self.args.quiet {
            None
        } else {
            let pb = ProgressBar::new(image_files.len() as u64);
            pb.set_style(
                ProgressStyle::default_bar()
                    .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} ({eta}) {msg}")
                    .unwrap()
                    .progress_chars("#>-")
            );
            pb.set_message("Processing images...");
            Some(pb)
        };

        let mut results = Vec::new();

        for image_path in image_files {
            let image_path_str = image_path.clone();
            let result = self.process_single_image(&image_path, engine);
            results.push((image_path, result));

            if let Some(pb) = &progress_bar {
                pb.inc(1);
                pb.set_message(format!("Processing: {}", image_path_str.file_name().unwrap().to_string_lossy()));
            }
        }

        if let Some(pb) = &progress_bar {
            pb.finish_with_message("Processing completed");
        }

        // Output results
        self.output_results(results)?;

        Ok(())
    }

    pub fn process_single_file(&self, file_path: &Path, engine: &mut OcrEngine) -> Result<(), Box<dyn std::error::Error>> {
        let result = self.process_single_image(file_path, engine);
        self.output_results(vec![(file_path.to_path_buf(), result)])?;
        Ok(())
    }

    fn collect_image_files(&self, dir_path: &Path) -> Result<Vec<PathBuf>, Box<dyn std::error::Error>> {
        let mut image_files = Vec::new();
        let _supported_extensions = ["png", "jpg", "jpeg", "bmp", "tiff", "tif", "webp"];

        if self.args.recursive {
            // Recursive search
            let pattern = format!("{}/**/*", dir_path.display());
            for entry in glob::glob(&pattern)? {
                let entry = entry?;
                if entry.is_file() && self.is_image_file(&entry) {
                    image_files.push(entry);
                }
            }
        } else {
            // Search only current directory
            for entry in std::fs::read_dir(dir_path)? {
                let entry = entry?;
                let path = entry.path();
                if path.is_file() && self.is_image_file(&path) {
                    image_files.push(path);
                }
            }
        }

        image_files.sort();
        Ok(image_files)
    }

    fn is_image_file(&self, path: &Path) -> bool {
        if let Some(extension) = path.extension().and_then(OsStr::to_str) {
            matches!(extension.to_lowercase().as_str(), "png" | "jpg" | "jpeg" | "bmp" | "tiff" | "tif" | "webp")
        } else {
            false
        }
    }

    fn process_single_image(&self, image_path: &Path, engine: &mut OcrEngine) -> Result<(crate::ocr_result::OcrResult, u64), Box<dyn std::error::Error>> {
        let start_time = std::time::Instant::now();
        let (result, _) = engine.process_image(image_path, &self.args)?;
        let processing_time = start_time.elapsed().as_millis() as u64;
        Ok((result, processing_time))
    }

    fn output_results(&self, results: Vec<(PathBuf, Result<(crate::ocr_result::OcrResult, u64), Box<dyn std::error::Error>>)>) -> Result<(), Box<dyn std::error::Error>> {
        let output_path = self.args.output.as_deref();

        if self.args.format == "json" {
            if results.len() == 1 && !self.args.append {
                // Single file, output single JSON object directly
                let (image_path, result) = &results[0];
                match result {
                    Ok((ocr_result, processing_time)) => {
                        let json_output = self.formatter.format_json(
                            image_path,
                            &Ok(ocr_result.clone()),
                            *processing_time,
                        )?;
                        crate::cli::output::write_to_file_or_stdout(&json_output, output_path, self.args.append)?;
                    }
                    Err(e) => {
                        let json_output = self.formatter.format_json(
                            image_path,
                            &Err(format!("{}", e)),
                            0,
                        )?;
                        crate::cli::output::write_to_file_or_stdout(&json_output, output_path, self.args.append)?;
                    }
                }
            } else {
                // Multiple files or append mode, output JSON array
                let mut json_results = Vec::new();

                for (image_path, result) in results {
                    match result {
                        Ok((ocr_result, processing_time)) => {
                            let json_output = self.formatter.format_json(
                                &image_path,
                                &Ok(ocr_result.clone()),
                                processing_time,
                            )?;
                            json_results.push(json_output);
                        }
                        Err(e) => {
                            let json_output = self.formatter.format_json(
                                &image_path,
                                &Err(format!("{}", e)),
                                0,
                            )?;
                            json_results.push(json_output);
                        }
                    }
                }

                let combined_output = if json_results.len() == 1 {
                    json_results.into_iter().next().unwrap()
                } else {
                    format!("[\n  {}\n]", json_results.join(",\n  "))
                };

                crate::cli::output::write_to_file_or_stdout(&combined_output, output_path, self.args.append)?;
            }
        } else {
            // Text format
            let mut text_output = String::new();

            for (image_path, result) in results {
                match result {
                    Ok((ocr_result, processing_time)) => {
                        text_output.push_str(&self.formatter.format_text(
                            &image_path,
                            &Ok(ocr_result),
                            processing_time,
                        ));
                    }
                    Err(e) => {
                        text_output.push_str(&self.formatter.format_text(
                            &image_path,
                            &Err(format!("{}", e)),
                            0,
                        ));
                    }
                }
            }

            crate::cli::output::write_to_file_or_stdout(&text_output, output_path, self.args.append)?;
        }

        Ok(())
    }
}