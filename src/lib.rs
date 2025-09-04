//! # HiPDF
//!
//! A high-level PDF manipulation library built on lopdf, focusing on ease of use
//! and powerful abstractions for common PDF operations.
//!
//! ## Features
//!
//! - **OCG (Optional Content Groups) Support**: Easy creation and management of PDF layers
//! - **Layer Management**: High-level API for organizing content into toggleable layers
//! - **Content Building**: Fluent API for building layered PDF content
//! - **Type Safety**: Strongly typed interfaces with compile-time guarantees
//!
//! ## Example
//!
//! ```rust
//! use hipdf::{OCGManager, Layer, LayerContentBuilder, LayerOperations as Ops};
//! use lopdf::{Document, Object};
//!
//! // Create a new PDF with layers
//! let mut doc = Document::with_version("1.5");
//! let mut ocg_manager = OCGManager::with_config(Default::default());
//!
//! // Add layers
//! ocg_manager.add_layer(Layer::new("Background", true));
//! ocg_manager.add_layer(Layer::new("Main Content", true));
//!
//! // Initialize layers in document
//! ocg_manager.initialize(&mut doc);
//! ```
//!
//! ## Modules
//!
//! - [`ocg`] - Optional Content Groups (layers) functionality
//! - [`layer`] - Layer management and utilities
//! - [`content`] - Content building for layered PDFs



pub mod ocg;

// Re-export commonly used types for convenience
pub use ocg::{OCGManager, Layer, LayerContentBuilder, LayerOperations, OCGConfig};





#[cfg(test)]
mod tests {
    #[test]
    fn test_library_compilation() {
        // Basic test to ensure library compiles correctly
        assert!(true);
    }
}

// Common type aliases and utilities
pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;
pub type Error = Box<dyn std::error::Error>;
