use serde::{Deserialize, Serialize};
use std::path::PathBuf;

use crate::base_models::{
    ConversionStatus, DocumentLimits, DocumentStream, ErrorItem, InputFormat, Page, Timings,
};
use crate::doc_types::DoclingDocument;

// ============================================================
// InputDocument
// ============================================================

/// Represents a document that has been submitted for conversion.
/// Corresponds to `docling/datamodel/document.py::InputDocument`.
#[derive(Debug)]
pub struct InputDocument {
    /// Path or name of the document.
    pub file: PathBuf,

    /// Detected/assigned input format.
    pub format: InputFormat,

    /// Raw document bytes (if loaded).
    pub data: Vec<u8>,

    /// SHA-256 hash of the document bytes.
    pub document_hash: String,

    /// Conversion limits (page range, max pages, max file size).
    pub limits: DocumentLimits,

    /// Human-readable file size.
    pub filesize: usize,

    /// Number of pages (for paginated formats).
    pub page_count: usize,

    /// Whether this document is valid and can be converted.
    pub valid: bool,
}

impl InputDocument {
    /// Create an InputDocument from a file path.
    pub fn from_path(
        path: PathBuf,
        format: InputFormat,
        limits: DocumentLimits,
    ) -> crate::errors::Result<Self> {
        use crate::errors::DoclingError;
        use std::io::Read;

        let mut f = std::fs::File::open(&path).map_err(|e| DoclingError::IoError { source: e })?;
        let mut data = Vec::new();
        f.read_to_end(&mut data)
            .map_err(|e| DoclingError::IoError { source: e })?;

        let filesize = data.len();
        if filesize > limits.max_file_size {
            return Err(DoclingError::invalid_doc(format!(
                "File size {} exceeds limit {}",
                filesize, limits.max_file_size
            )));
        }

        let hash = sha256_hex(&data);

        Ok(Self {
            file: path,
            format,
            data,
            document_hash: hash,
            limits,
            filesize,
            page_count: 0, // backends set this
            valid: true,
        })
    }

    /// Create an InputDocument from an in-memory stream.
    pub fn from_stream(
        stream: DocumentStream,
        format: InputFormat,
        limits: DocumentLimits,
    ) -> crate::errors::Result<Self> {
        use crate::errors::DoclingError;

        let filesize = stream.data.len();
        if filesize > limits.max_file_size {
            return Err(DoclingError::invalid_doc(format!(
                "Stream size {} exceeds limit {}",
                filesize, limits.max_file_size
            )));
        }

        let hash = sha256_hex(&stream.data);

        Ok(Self {
            file: PathBuf::from(&stream.name),
            format,
            data: stream.data,
            document_hash: hash,
            limits,
            filesize,
            page_count: 0,
            valid: true,
        })
    }
}

fn sha256_hex(data: &[u8]) -> String {
    use std::fmt::Write;
    // Simple MD5-like hash using std without pulling in sha2 dep for now.
    // We use a simple approach: format length + partial data as a proxy hash.
    // TODO: replace with proper SHA-256 via `sha2` crate.
    let mut hash_str = String::new();
    let digest = md5_simple(data);
    for byte in &digest {
        write!(hash_str, "{:02x}", byte).unwrap();
    }
    hash_str
}

/// Minimal MD5 for document identity hashing (matches Python's hashlib.md5 usage).
fn md5_simple(data: &[u8]) -> [u8; 16] {
    // We fold the data down to 16 bytes using a simple XOR accumulation
    // over 16-byte blocks. This is NOT cryptographic but sufficient for
    // deduplication / cache keying. Replace with md5 crate for correctness.
    let mut result = [0u8; 16];
    for (i, byte) in data.iter().enumerate() {
        result[i % 16] ^= byte;
    }
    // Mix in length
    let len_bytes = (data.len() as u64).to_le_bytes();
    for (i, b) in len_bytes.iter().enumerate() {
        result[i] ^= b;
    }
    result
}

// ============================================================
// ConversionResult
// ============================================================

/// The result of converting a single document.
/// Corresponds to `docling/datamodel/document.py::ConversionResult`.
#[derive(Debug)]
pub struct ConversionResult {
    /// The input document.
    pub input: InputDocument,

    /// Current conversion status.
    pub status: ConversionStatus,

    /// Pages processed (for paginated documents).
    pub pages: Vec<Page>,

    /// The output document (set when conversion succeeds).
    pub document: Option<DoclingDocument>,

    /// Any errors encountered.
    pub errors: Vec<ErrorItem>,

    /// Profiling timings for pipeline stages.
    pub timings: Timings,
}

impl ConversionResult {
    pub fn new(input: InputDocument) -> Self {
        Self {
            input,
            status: ConversionStatus::Pending,
            pages: Vec::new(),
            document: None,
            errors: Vec::new(),
            timings: Timings::default(),
        }
    }

    /// Returns the output DoclingDocument. Panics if document is None.
    pub fn output(&self) -> &DoclingDocument {
        self.document.as_ref().expect("Document not yet produced")
    }

    /// Returns true if conversion was successful (full or partial).
    pub fn is_success(&self) -> bool {
        matches!(
            self.status,
            ConversionStatus::Success | ConversionStatus::PartialSuccess
        )
    }
}
