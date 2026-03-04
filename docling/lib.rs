//! Docling-rs — Rust port of Docling
//!
//! Universal document conversion for AI pipelines.
//! Mirrors the Python `docling` package structure.
//!
//! # Quick start
//! ```rust,ignore
//! use docling::DocumentConverter;
//!
//! let converter = DocumentConverter::default();
//! let result = converter.convert("document.pdf").unwrap();
//! println!("{}", result.document.unwrap().export_to_markdown());
//! ```

#![allow(dead_code, unused_imports, unused_variables)]

// ── Core data models ─────────────────────────────────────────
pub mod datamodel;

// ── Document backends ────────────────────────────────────────
pub mod backend;

// ── ML models ────────────────────────────────────────────────
pub mod models;

// ── Pipelines ────────────────────────────────────────────────
pub mod pipeline;

// ── Chunking ─────────────────────────────────────────────────
pub mod chunking;

// ── Utilities ────────────────────────────────────────────────
pub mod utils;

// ── Errors ───────────────────────────────────────────────────
pub mod errors;

// ── Top-level DocumentConverter ──────────────────────────────
pub mod document_converter;

// ── Convenience re-exports ───────────────────────────────────
pub use datamodel::base_models::{
    BoundingBox, Cell, ConversionStatus, DocumentLimits, DocumentStream, Formatting, InputFormat,
    LayoutLabel, OcrCell, Page, PageSize,
};
pub use datamodel::document::{ConversionResult, InputDocument};
pub use document_converter::DocumentConverter;
pub use errors::{DoclingError, Result};
