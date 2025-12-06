#[cfg(feature = "server")]
use axum::{
    middleware,
    routing::{get, post},
    Router,
};
#[cfg(feature = "server")]
use std::sync::Arc;
#[cfg(feature = "server")]
use tokio::signal;
#[cfg(feature = "server")]
use tower::ServiceBuilder;
#[cfg(feature = "server")]
use tower_http::trace::TraceLayer;
#[cfg(feature = "server")]
use tracing::{error, info};

#[cfg(feature = "server")]
use super::{
    config::ServerConfig,
    handlers::{chat_completions, health, get_models},
    middleware::access_log_middleware,
};

#[cfg(feature = "server")]
#[derive(Clone)]
pub struct AppState {
    pub config: ServerConfig,
    /// OCR engine pool
    pub ocr_pool: Option<Arc<crate::server::ocr_pool::OcrPool>>,
}

#[cfg(feature = "server")]
pub struct Server {
    config: ServerConfig,
}

#[cfg(feature = "server")]
impl Server {
    pub fn new(config: ServerConfig) -> Self {
        Self { config }
    }

    pub async fn run(self) -> Result<(), Box<dyn std::error::Error>> {
        // Create OCR engine pool
        let ocr_pool = if self.config.warmup_on_startup {
            info!("Initializing OCR engine pool...");

            // Determine pool size
            let pool_size = self.config.ocr_pool_size.unwrap_or_else(|| {
                num_cpus::get() // Use CPU core count as default
            });

            // Create OCR engine pool
            let pool = crate::server::ocr_pool::OcrPool::new(
                "mobile".to_string(),
                (
                    self.config.det_model_path.clone(),
                    self.config.cls_model_path.clone(),
                    self.config.rec_model_path.clone(),
                ),
                pool_size,
            );

            let pool = Arc::new(pool);

            // Warm up engine pool
            match pool.warmup().await {
                Ok(()) => {
                    info!("OCR engine pool warmup completed, pool size: {}", pool_size);
                    Some(pool)
                }
                Err(e) => {
                    error!("OCR engine pool warmup failed: {:?}", e);
                    error!("Server will run without preheated models, first request may be slower");
                    None
                }
            }
        } else {
            info!("Model warmup not enabled, models will be loaded on first request");
            None
        };

        let app_state = AppState {
            config: self.config.clone(),
            ocr_pool,
        };

        let app = Router::new()
            .route("/v1/chat/completions", post(chat_completions))
            .route("/v1/health", get(health))
            .route("/v1/models", get(get_models))
            .layer(
                ServiceBuilder::new()
                    .layer(TraceLayer::new_for_http())
                    .layer(middleware::from_fn_with_state(app_state.clone(), access_log_middleware))
            )
            .with_state(app_state);

        let addr = self.config.bind_addr;
        let listener = tokio::net::TcpListener::bind(addr).await?;
        println!("API server listening on http://{}", addr);
        log::info!("Server started successfully on http://{}", addr);

        // Handle graceful shutdown
        log::info!("Server is running with graceful shutdown support");
        axum::serve(listener, app)
            .with_graceful_shutdown(shutdown_signal())
            .await?;

        log::info!("Server shutdown completed");
        Ok(())
    }
}

#[cfg(feature = "server")]
async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {
            println!("received ctrl+c");
            log::info!("Received Ctrl+C signal, shutting down gracefully");
        },
        _ = terminate => {
            println!("received terminate signal");
            log::info!("Received terminate signal, shutting down gracefully");
        },
    }
}