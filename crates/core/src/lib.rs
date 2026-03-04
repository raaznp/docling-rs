//! Core data models for Docling-rs.
//!
//! This crate provides all shared data structures, enums, and error types
//! used across the other docling crates.

pub mod backend_options;
pub mod base_models;
pub mod doc_types;
pub mod document;
pub mod errors;
pub mod pipeline_options;
pub mod settings;

// Re-export most commonly used types
pub use base_models::{
    BoundingBox, Cell, ConversionStatus, DoclingComponentType, DocumentLimits, DocumentStream,
    ErrorItem, InputFormat, LayoutCluster, LayoutLabel, OcrCell, Page, PageSize, ProvenanceItem,
    Size, Timings,
};
pub use doc_types::{
    CodeItem, DocItem, DoclingDocument, DocumentMetadata, DocumentOrigin, FormulaItem,
    KeyValueItem, ListItem, PageRef, PictureItem, ProvenanceRef, RefItem, SectionHeaderItem,
    TableData, TableItem, TextItem,
};
pub use document::{ConversionResult, InputDocument};
pub use errors::DoclingError;
pub use pipeline_options::{ConvertPipelineOptions, PdfPipelineOptions, PipelineOptions};
pub use settings::{Settings, SETTINGS};
