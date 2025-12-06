#[cfg(feature = "server")]
pub mod config;
#[cfg(feature = "server")]
pub mod handlers;
#[cfg(feature = "server")]
pub mod middleware;
#[cfg(feature = "server")]
pub mod models;
#[cfg(feature = "server")]
pub mod ocr_pool;
#[cfg(feature = "server")]
pub mod server;
#[cfg(feature = "server")]
pub mod utils;

#[cfg(feature = "server")]
pub use server::Server;