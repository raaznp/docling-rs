use super::EnrichmentModel;
use crate::datamodel::document::{DocItem, DoclingDocument};
use crate::errors::Result;

pub struct PictureDescriptionModel {
    enabled: bool,
}

impl PictureDescriptionModel {
    pub fn new() -> Self {
        Self { enabled: true }
    }
    pub fn disabled() -> Self {
        Self { enabled: false }
    }
}

impl EnrichmentModel for PictureDescriptionModel {
    fn is_enabled(&self) -> bool {
        self.enabled
    }
    fn prepare_element(&self, item: &DocItem) -> bool {
        matches!(item, DocItem::Picture(_))
    }
    fn process_batch(&self, _doc: &mut DoclingDocument, _item_indices: &[usize]) -> Result<()> {
        // TODO(onnx): run VLM to generate picture descriptions.
        Ok(())
    }
}
