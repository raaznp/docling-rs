use thiserror::Error;

/// Top-level error type for docling operations.
#[derive(Debug, Error)]
pub enum DoclingError {
    #[error("Conversion failed: {0}")]
    ConversionError(String),

    #[error("Backend error: {0}")]
    BackendError(String),

    #[error("Pipeline error: {0}")]
    PipelineError(String),

    #[error("Unsupported format: {0}")]
    UnsupportedFormat(String),

    #[error("Invalid document: {0}")]
    InvalidDocument(String),

    #[error("Model inference error: {0}")]
    ModelError(String),

    #[error("IO error: {source}")]
    IoError {
        #[from]
        source: std::io::Error,
    },

    #[error("JSON error: {source}")]
    JsonError {
        #[from]
        source: serde_json::Error,
    },

    #[error("HTTP error: {0}")]
    HttpError(String),

    #[error("Timeout: document processing exceeded {seconds}s")]
    Timeout { seconds: f64 },

    #[error("Format not allowed: {format}")]
    FormatNotAllowed { format: String },
}

impl DoclingError {
    pub fn conversion(msg: impl Into<String>) -> Self {
        Self::ConversionError(msg.into())
    }

    pub fn backend(msg: impl Into<String>) -> Self {
        Self::BackendError(msg.into())
    }

    pub fn pipeline(msg: impl Into<String>) -> Self {
        Self::PipelineError(msg.into())
    }

    pub fn invalid_doc(msg: impl Into<String>) -> Self {
        Self::InvalidDocument(msg.into())
    }

    pub fn model(msg: impl Into<String>) -> Self {
        Self::ModelError(msg.into())
    }
}

pub type Result<T> = std::result::Result<T, DoclingError>;
