//! Content Enhancement Module - V1000 Experimental
//!
//! Provides content enhancement features including:
//! - Reader mode with customization
//! - Media player enhancement
//! - Content extraction and transformation
//! - Accessibility enhancements
//! - Language detection and text analysis

mod reader_mode;
mod media_player;
mod content_transformer;
mod accessibility;
mod language_detector;
mod text_analyzer;
mod manager;

pub use reader_mode::*;
pub use media_player::*;
pub use content_transformer::*;
pub use accessibility::*;
pub use language_detector::*;
pub use text_analyzer::*;
pub use manager::*;
