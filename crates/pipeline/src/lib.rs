//! Pipeline crate for Docling-rs.
//!
//! Pipelines orchestrate the document build → assemble → enrich workflow.

pub mod base;
pub mod simple;
pub mod standard_pdf;

pub use base::{BasePipeline, PaginatedPipeline};
pub use simple::SimplePipeline;
pub use standard_pdf::StandardPdfPipeline;
