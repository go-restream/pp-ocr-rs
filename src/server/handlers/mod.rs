pub mod ocr;
pub mod health;
pub mod models;

pub use ocr::chat_completions;
pub use health::health;
pub use models::get_models;