use crate::backend::{BackendSource, DeclarativeBackend, DocumentBackend};
use crate::datamodel::base_models::{InputFormat, LayoutLabel};
use crate::datamodel::document::{DoclingDocument, DocumentOrigin, TextItem};
use crate::errors::Result;

pub struct NoopBackend {
    source: BackendSource,
}

impl NoopBackend {
    pub fn new(source: BackendSource) -> Self {
        Self { source }
    }
}

impl DocumentBackend for NoopBackend {
    fn is_valid(&self) -> bool {
        true
    }
    fn supported_formats() -> &'static [InputFormat] {
        &[]
    }
    fn unload(&mut self) {}
}

impl DeclarativeBackend for NoopBackend {
    fn convert(&mut self) -> Result<DoclingDocument> {
        Ok(DoclingDocument::new(self.source.name()))
    }
}
