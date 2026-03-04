//! ML model traits and implementations for Docling-rs.
//!
//! Models wrap ONNX Runtime sessions and implement either:
//! - `BuildModel` — page-level processing (layout detection, OCR)
//! - `EnrichmentModel` — document-level enrichment (picture classification, description)

pub mod base_model;
pub mod layout;
pub mod ocr;
pub mod picture_classifier;
pub mod picture_description;
pub mod table;

pub use base_model::{BuildModel, EnrichmentModel};
