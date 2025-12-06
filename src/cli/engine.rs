use crate::cli::config::Config;
use crate::core::ocr_error::OcrError;
use crate::core::ocr_lite::OcrLite;
use crate::core::ocr_result::OcrResult;
use crate::cli::args::OcrArgs;
use std::path::Path;
use std::time::Instant;

pub struct OcrEngine {
    ocr: OcrLite,
    config: Config,
}

impl OcrEngine {
    pub fn new(config: Config) -> Result<Self, OcrError> {
        let mut ocr = OcrLite::new();

        // Initialize models
        if let Some(dict_path) = config.get_dict_path() {
            // Use custom dictionary
            ocr.init_models_with_dict(
                config.get_det_model_path().to_str().unwrap(),
                config.get_cls_model_path().to_str().unwrap(),
                config.get_rec_model_path().to_str().unwrap(),
                dict_path.to_str().unwrap(),
                2, // Default to use 2 threads
            )?;
        } else {
            // Use default dictionary
            ocr.init_models(
                config.get_det_model_path().to_str().unwrap(),
                config.get_cls_model_path().to_str().unwrap(),
                config.get_rec_model_path().to_str().unwrap(),
                2, // Default to use 2 threads
            )?;
        }

        Ok(Self { ocr, config })
    }

    pub fn process_image(&mut self, image_path: &Path, args: &OcrArgs) -> Result<(OcrResult, u64), OcrError> {
        let start_time = Instant::now();

        let result = self.ocr.detect_from_path(
            image_path.to_str().unwrap(),
            args.box_limit,
            args.max_box_size,
            args.box_thresh,
            args.min_box_size,
            args.unclip_ratio,
            args.use_angle_cls,
            args.use_direction_cls,
        );

        let processing_time = start_time.elapsed().as_millis() as u64;

        match result {
            Ok(ocr_result) => Ok((ocr_result, processing_time)),
            Err(e) => Err(e),
        }
    }

    pub fn process_image_from_bytes(&mut self, image_data: &[u8], args: &OcrArgs) -> Result<(OcrResult, u64), OcrError> {
        let start_time = Instant::now();

        // Load image from byte data
        let img = image::load_from_memory(image_data)?
            .to_rgb8();

        let result = self.ocr.detect(
            &img,
            args.box_limit,
            args.max_box_size,
            args.box_thresh,
            args.min_box_size,
            args.unclip_ratio,
            args.use_angle_cls,
            args.use_direction_cls,
        );

        let processing_time = start_time.elapsed().as_millis() as u64;

        match result {
            Ok(ocr_result) => Ok((ocr_result, processing_time)),
            Err(e) => Err(e),
        }
    }

    /// Get a reference to the engine configuration
    pub fn config(&self) -> &Config {
        &self.config
    }
}