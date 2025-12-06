#![allow(clippy::too_many_arguments)]

pub mod cli;
pub mod core;
#[cfg(feature = "server")]
pub mod server;

pub use core::{
    angle_net, base_net, crnn_net, db_net, ocr_error, ocr_lite,
    ocr_result, ocr_utils, scale_param,
};