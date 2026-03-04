use crate::backend::{BackendSource, DeclarativeBackend, DocumentBackend};
use crate::datamodel::base_models::InputFormat;
use crate::datamodel::document::DoclingDocument;
use crate::errors::{DoclingError, Result};

pub struct JsonBackend {
    source: BackendSource,
    valid: bool,
}
impl JsonBackend {
    pub fn new(source: BackendSource) -> Self {
        Self {
            source,
            valid: true,
        }
    }
}

impl DocumentBackend for JsonBackend {
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

impl DeclarativeBackend for JsonBackend {
    fn convert(&mut self) -> Result<DoclingDocument> {
        let bytes = self.source.read_bytes()?;
        serde_json::from_slice(&bytes)
            .map_err(|e| DoclingError::backend(format!("JSON parse error: {}", e)))
    }
}
