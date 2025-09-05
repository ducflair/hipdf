//! # hipdf
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
//! use hipdf::ocg::{OCGManager, Layer, LayerContentBuilder, LayerOperations as Ops};
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
//! - [`hatching`] - Hatching and pattern support for PDF documents

pub mod embed_pdf;
pub mod hatching;
pub mod ocg;
pub use lopdf;

// Common type aliases and utilities
pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;
pub type Error = Box<dyn std::error::Error>;

#[cfg(test)]
mod tests {
    // Tests are in separate integration test files
}
