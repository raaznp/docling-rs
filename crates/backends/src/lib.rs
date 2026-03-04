use docling_core::base_models::InputFormat;
use docling_core::doc_types::DoclingDocument;
use docling_core::errors::{DoclingError, Result};
use std::io::Read;
use std::path::PathBuf;

pub mod asciidoc;
pub mod csv;
pub mod html;
pub mod image;
pub mod json;
pub mod latex;
pub mod markdown;
pub mod noop;
pub mod pdf;
pub mod webvtt;
pub mod xml;

#[cfg(feature = "office")]
pub mod docx;
#[cfg(feature = "office")]
pub mod pptx;
#[cfg(feature = "office")]
pub mod xlsx;

// ============================================================
// Backend traits
// ============================================================

/// The core trait all document backends must implement.
pub trait DocumentBackend: Send + Sync {
    /// Returns whether this backend successfully loaded the document.
    fn is_valid(&self) -> bool;

    /// Formats this backend supports.
    fn supported_formats() -> &'static [InputFormat]
    where
        Self: Sized;

    /// Free underlying resources (file handles, memory maps, etc.)
    fn unload(&mut self);
}

/// A backend that can directly produce a `DoclingDocument` from structured content
/// (e.g., HTML, Markdown, DOCX) without needing an ML pipeline.
pub trait DeclarativeBackend: DocumentBackend {
    /// Convert the loaded document to a `DoclingDocument`.
    fn convert(&mut self) -> Result<DoclingDocument>;
}

/// A backend for paginated documents (like PDF) that exposes per-page rendering.
pub trait PaginatedBackend: DocumentBackend {
    /// Total number of pages.
    fn page_count(&self) -> usize;

    /// Load and return page data for a specific page (1-indexed).
    fn load_page(&self, page_no: usize) -> Result<PageData>;
}

// ============================================================
// Page data from paginated backends
// ============================================================

/// Raw page data produced by a `PaginatedBackend`.
#[derive(Debug)]
pub struct PageData {
    pub page_no: usize,
    pub width: f64,
    pub height: f64,
    /// Native text layer cells (empty if scanned / no native text).
    pub text_cells: Vec<NativeTextCell>,
    /// Rendered bitmap image (if available).
    pub image: Option<Vec<u8>>,
    pub image_width: u32,
    pub image_height: u32,
}

/// A text cell extracted from a native PDF text layer.
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

// ============================================================
// Backend source — path or bytes
// ============================================================

/// The input source for a backend.
pub enum BackendSource {
    Path(PathBuf),
    Bytes(Vec<u8>, String), // (data, filename)
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

// ============================================================
// Backend registry helper
// ============================================================

/// Detect the format from a file extension.
pub fn detect_format(filename: &str) -> Option<InputFormat> {
    let ext = filename.rsplit('.').next()?.to_lowercase();
    InputFormat::from_extension(&ext)
}
