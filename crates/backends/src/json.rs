use docling_core::{
    base_models::InputFormat,
    doc_types::{DoclingDocument, DocumentOrigin},
    errors::{DoclingError, Result},
};
use serde_json::Value;

use crate::{BackendSource, DeclarativeBackend, DocumentBackend};

/// Backend that loads an already-serialised `DoclingDocument` from JSON.
/// Mirrors `docling/backend/json/docling_json_backend.py`.
pub struct DoclingJsonBackend {
    source: BackendSource,
    valid: bool,
}

impl DoclingJsonBackend {
    pub fn new(source: BackendSource) -> Self {
        Self {
            source,
            valid: true,
        }
    }
}

impl DocumentBackend for DoclingJsonBackend {
    fn is_valid(&self) -> bool {
        self.valid
    }

    fn supported_formats() -> &'static [InputFormat] {
        &[InputFormat::JsonDocling]
    }

    fn unload(&mut self) {
        self.valid = false;
    }
}

impl DeclarativeBackend for DoclingJsonBackend {
    fn convert(&mut self) -> Result<DoclingDocument> {
        let bytes = self.source.read_bytes()?;
        let doc: DoclingDocument = serde_json::from_slice(&bytes)
            .map_err(|e| DoclingError::backend(format!("Invalid Docling JSON: {}", e)))?;
        Ok(doc)
    }
}
