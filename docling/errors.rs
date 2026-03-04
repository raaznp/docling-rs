//! Error types for Docling-rs.
//! Mirrors `docling/exceptions.py`.

use thiserror::Error;

pub type Result<T> = std::result::Result<T, DoclingError>;

#[derive(Debug, Error)]
pub enum DoclingError {
    #[error("I/O error: {source}")]
    IoError {
        #[from]
        source: std::io::Error,
    },

    #[error("Backend error: {0}")]
    BackendError(String),

    #[error("Invalid document: {0}")]
    InvalidDocument(String),

    #[error("Unsupported format: {0}")]
    UnsupportedFormat(String),

    #[error("Model error: {0}")]
    ModelError(String),

    #[error("Conversion error: {0}")]
    ConversionError(String),

    #[error("Serialization error: {0}")]
    SerializationError(String),
}

impl DoclingError {
    pub fn backend(msg: impl Into<String>) -> Self {
        Self::BackendError(msg.into())
    }

    pub fn invalid_doc(msg: impl Into<String>) -> Self {
        Self::InvalidDocument(msg.into())
    }

    pub fn unsupported(format: impl Into<String>) -> Self {
        Self::UnsupportedFormat(format.into())
    }

    pub fn model(msg: impl Into<String>) -> Self {
        Self::ModelError(msg.into())
    }
}
