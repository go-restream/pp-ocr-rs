use std::{
    collections::VecDeque,
    path::PathBuf,
    sync::{Arc, Mutex},
};

use tokio::sync::{Semaphore, SemaphorePermit};
use tracing::{debug, error, info, warn};

use crate::core::{ocr_error::OcrError, ocr_lite::OcrLite};
use crate::ocr_result::OcrResult;

/// OCR engine wrapper
#[derive(Debug)]
pub struct OcrEngine {
    /// OCR instance
    pub(crate) ocr: OcrLite,
    /// Model name
    pub(crate) model_name: String,
}

impl OcrEngine {
    /// Create a new OCR engine
    pub fn new(
        det_path: &str,
        cls_path: &str,
        rec_path: &str,
        model_name: String,
        num_thread: usize,
    ) -> Result<Self, OcrError> {
        let mut ocr = OcrLite::new();
        ocr.init_models(det_path, cls_path, rec_path, num_thread)?;

        Ok(Self { ocr, model_name })
    }

    /// Perform OCR recognition
    pub fn detect(
        &mut self,
        image: &image::RgbImage,
        padding: u32,
        max_side_len: u32,
        box_score_thresh: f32,
        box_thresh: f32,
        unclip_ratio: f32,
        do_angle: bool,
        most_angle: bool,
    ) -> Result<OcrResult, OcrError> {
        self.ocr.detect(
            image,
            padding,
            max_side_len,
            box_score_thresh,
            box_thresh,
            unclip_ratio,
            do_angle,
            most_angle,
        )
    }
}

/// OCR engine pool
#[derive(Debug)]
pub struct OcrPool {
    /// Engine pool
    pool: Arc<Mutex<VecDeque<OcrEngine>>>,
    /// Model type (mobile or server)
    model_type: String,
    /// Model file paths
    model_paths: (PathBuf, PathBuf, PathBuf),
    /// Semaphore to control concurrency
    semaphore: Arc<Semaphore>,
    /// Maximum pool size
    max_size: usize,
}

impl OcrPool {
    /// Create a new OCR engine pool
    pub fn new(
        model_type: String,
        model_paths: (PathBuf, PathBuf, PathBuf),
        pool_size: usize,
    ) -> Self {
        Self {
            pool: Arc::new(Mutex::new(VecDeque::with_capacity(pool_size))),
            model_type,
            model_paths,
            semaphore: Arc::new(Semaphore::new(pool_size)),
            max_size: pool_size,
        }
    }

    /// Warm up the engine pool (called on server startup)
    pub async fn warmup(&self) -> Result<(), OcrError> {
        info!(
            "Starting to warm up {} OCR engine pool, size: {}",
            self.model_type, self.max_size
        );

        let mut handles = vec![];

        // Create engines in parallel
        for i in 0..self.max_size {
            let model_paths = self.model_paths.clone();
            let model_type = self.model_type.clone();
            let num_thread = if model_type.contains("server") { 2 } else { 4 };

            let handle = tokio::task::spawn_blocking(move || {
                let start = std::time::Instant::now();
                match OcrEngine::new(
                    &model_paths.0.to_string_lossy(),
                    &model_paths.1.to_string_lossy(),
                    &model_paths.2.to_string_lossy(),
                    format!("{}-{}", model_type, i),
                    num_thread,
                ) {
                    Ok(engine) => {
                        debug!(
                            "OCR engine {}-{} warmup completed, took: {:?}",
                            model_type, i, start.elapsed()
                        );
                        Ok(engine)
                    }
                    Err(e) => {
                        error!("OCR engine {}-{} warmup failed: {:?}", model_type, i, e);
                        Err(e)
                    }
                }
            });

            handles.push(handle);
        }

        // Wait for all engines to be created
        let mut engines = Vec::with_capacity(self.max_size);
        for handle in handles {
            match handle.await.unwrap() {
                Ok(engine) => engines.push(engine),
                Err(e) => {
                    error!("Failed during warmup: {:?}", e);
                    return Err(e);
                }
            }
        }

        // Add engines to the pool
        {
            let mut pool = self.pool.lock().unwrap();
            for engine in engines {
                pool.push_back(engine);
            }
        }

        info!(
            "{} OCR engine pool warmup completed, {} engines total",
            self.model_type, self.max_size
        );

        Ok(())
    }

    /// Get OCR engine from pool
    pub async fn acquire(&self) -> Result<OcrEngineHandle<'_>, OcrError> {
        // Acquire semaphore permit (blocks until permit is available)
        let permit = self.semaphore.acquire().await.map_err(|e| {
            error!("Failed to acquire semaphore permit: {:?}", e);
            OcrError::Custom(format!("Failed to acquire semaphore permit: {}", e))
        })?;

        // Get engine from pool
        let engine = {
            let mut pool = self.pool.lock().unwrap();
            pool.pop_front().ok_or_else(|| {
                OcrError::Custom("OCR engine pool is empty".to_string())
            })?
        };

        debug!("Retrieved engine {} from {} pool", engine.model_name, self.model_type);

        Ok(OcrEngineHandle {
            engine: Some(engine),
            pool: self,
            permit: Some(permit),
        })
    }

    /// Return engine to pool
    fn release_engine(&self, engine: OcrEngine) {
        let mut pool = self.pool.lock().unwrap();
        if pool.len() < self.max_size {
            let model_name = engine.model_name.clone();
            pool.push_back(engine);
            debug!("Engine {} returned to {} pool", model_name, self.model_type);
        } else {
            warn!("Pool is full, discarding engine {}", engine.model_name);
        }
    }

    /// Get current pool status
    pub fn pool_status(&self) -> PoolStatus {
        let pool = self.pool.lock().unwrap();
        PoolStatus {
            available: pool.len(),
            max_size: self.max_size,
            active: self.max_size - pool.len(),
            model_type: self.model_type.clone(),
        }
    }
}

/// OCR engine handle for automatically returning engine to pool
pub struct OcrEngineHandle<'a> {
    engine: Option<OcrEngine>,
    pool: &'a OcrPool,
    // Semaphore permit, automatically released when handle is dropped
    #[allow(dead_code)]
    permit: Option<SemaphorePermit<'a>>,
}

impl<'a> OcrEngineHandle<'a> {
    /// Get reference to the engine
    pub fn engine(&mut self) -> &mut OcrLite {
        &mut self.engine.as_mut().unwrap().ocr
    }

    /// Perform OCR recognition
    pub fn detect(
        &mut self,
        image: &image::RgbImage,
        padding: u32,
        max_side_len: u32,
        box_score_thresh: f32,
        box_thresh: f32,
        unclip_ratio: f32,
        do_angle: bool,
        most_angle: bool,
    ) -> Result<OcrResult, OcrError> {
        let engine = self.engine.as_mut().unwrap();
        engine.detect(image, padding, max_side_len, box_score_thresh, box_thresh, unclip_ratio, do_angle, most_angle)
    }
}

impl<'a> Drop for OcrEngineHandle<'a> {
    fn drop(&mut self) {
        if let Some(engine) = self.engine.take() {
            self.pool.release_engine(engine);
        }
        // permit will be automatically released
    }
}

/// Pool status information
#[derive(Debug, Clone)]
pub struct PoolStatus {
    /// Number of available engines
    pub available: usize,
    /// Maximum pool size
    pub max_size: usize,
    /// Number of active engines
    pub active: usize,
    /// Model type
    pub model_type: String,
}

impl PoolStatus {
    /// Get usage ratio (0.0 - 1.0)
    pub fn usage_ratio(&self) -> f32 {
        self.active as f32 / self.max_size as f32
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[tokio::test]
    #[ignore] // Requires model files, ignored in CI
    async fn test_ocr_pool_basic() {
        // Ensure model files exist
        let det_path = PathBuf::from("./models/ch_PP-OCRv5_mobile_det.onnx");
        let cls_path = PathBuf::from("./models/ch_ppocr_mobile_v2.0_cls_infer.onnx");
        let rec_path = PathBuf::from("./models/ch_PP-OCRv5_rec_mobile_infer.onnx");

        if !det_path.exists() || !cls_path.exists() || !rec_path.exists() {
            println!("Skipping test - model files not found");
            return;
        }

        // Create pool
        let pool = OcrPool::new(
            "mobile".to_string(),
            (det_path, cls_path, rec_path),
            2,
        );

        // Warm up
        pool.warmup().await.unwrap();

        // Check status
        let status = pool.pool_status();
        assert_eq!(status.available, 2);
        assert_eq!(status.active, 0);

        // Get engine
        let mut handle1 = pool.acquire().await.unwrap();
        let status = pool.pool_status();
        assert_eq!(status.available, 1);
        assert_eq!(status.active, 1);

        // Release engine
        drop(handle1);
        let status = pool.pool_status();
        assert_eq!(status.available, 2);
        assert_eq!(status.active, 0);
    }
}