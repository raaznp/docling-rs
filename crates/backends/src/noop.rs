use docling_core::{
    base_models::InputFormat,
    doc_types::DoclingDocument,
    errors::{DoclingError, Result},
};

use crate::{BackendSource, DeclarativeBackend, DocumentBackend};

/// A no-op backend that accepts any document but produces an empty DoclingDocument.
/// Used for formats where the pipeline (not the backend) is responsible for
/// all content extraction (e.g., Audio → ASR pipeline).
/// Mirrors `docling/backend/noop_backend.py`.
pub struct NoopBackend {
    source: BackendSource,
    valid: bool,
}

impl NoopBackend {
    pub fn new(source: BackendSource) -> Self {
        Self {
            source,
            valid: true,
        }
    }
}

impl DocumentBackend for NoopBackend {
    fn is_valid(&self) -> bool {
        self.valid
    }

    fn supported_formats() -> &'static [InputFormat] {
        &[]
    }

    fn unload(&mut self) {
        self.valid = false;
    }
}

impl DeclarativeBackend for NoopBackend {
    fn convert(&mut self) -> Result<DoclingDocument> {
        Ok(DoclingDocument::new(self.source.name()))
    }
}
