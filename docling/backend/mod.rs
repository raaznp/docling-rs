//! backend — document format parsers.
//! Mirrors `docling/backend/`.

pub mod asciidoc;
pub mod audio;
pub mod csv;
pub mod html;
pub mod image;
pub mod json;
pub mod latex;
pub mod markdown;
pub mod noop;
pub mod pdf;
pub mod video;
pub mod webvtt;
pub mod xml;

#[cfg(feature = "office")]
pub mod docx;
#[cfg(feature = "office")]
pub mod pptx;
#[cfg(feature = "office")]
pub mod xlsx;

use crate::datamodel::base_models::InputFormat;
use crate::datamodel::document::DoclingDocument;
use crate::errors::{DoclingError, Result};
use std::io::Read;
use std::path::PathBuf;

// ── Backend traits ───────────────────────────────────────────

/// The core trait all document backends implement.
pub trait DocumentBackend: Send + Sync {
    fn is_valid(&self) -> bool;
    fn supported_formats() -> &'static [InputFormat]
    where
        Self: Sized;
    fn unload(&mut self);
}

/// A backend that directly produces a `DoclingDocument` from structured content.
pub trait DeclarativeBackend: DocumentBackend {
    fn convert(&mut self) -> Result<DoclingDocument>;
}

/// A backend for paginated documents (PDF, images) that exposes per-page data.
pub trait PaginatedBackend: DocumentBackend {
    fn page_count(&self) -> usize;
    fn load_page(&self, page_no: usize) -> Result<PageData>;
}

// ── Page data ────────────────────────────────────────────────

#[derive(Debug)]
pub struct PageData {
    pub page_no: usize,
    pub width: f64,
    pub height: f64,
    pub text_cells: Vec<NativeTextCell>,
    pub image: Option<Vec<u8>>,
    pub image_width: u32,
    pub image_height: u32,
}

#[derive(Debug, Clone)]
pub struct NativeTextCell {
    pub text: String,
    pub l: f64,
    pub t: f64,
    pub r: f64,
    pub b: f64,
    pub font_size: f32,
    pub bold: bool,
    pub italic: bool,
}

// ── Backend source ───────────────────────────────────────────

pub enum BackendSource {
    Path(PathBuf),
    Bytes(Vec<u8>, String),
}

impl BackendSource {
    pub fn name(&self) -> &str {
        match self {
            BackendSource::Path(p) => p.file_name().and_then(|n| n.to_str()).unwrap_or("unknown"),
            BackendSource::Bytes(_, name) => name.as_str(),
        }
    }

    pub fn read_bytes(&self) -> Result<Vec<u8>> {
        match self {
            BackendSource::Bytes(data, _) => Ok(data.clone()),
            BackendSource::Path(p) => {
                let mut f =
                    std::fs::File::open(p).map_err(|e| DoclingError::IoError { source: e })?;
                let mut buf = Vec::new();
                f.read_to_end(&mut buf)
                    .map_err(|e| DoclingError::IoError { source: e })?;
                Ok(buf)
            }
        }
    }
}

/// Detect the input format from a file extension.
pub fn detect_format(filename: &str) -> Option<InputFormat> {
    let ext = filename.rsplit('.').next()?.to_lowercase();
    InputFormat::from_extension(&ext)
}
