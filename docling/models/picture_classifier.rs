use super::EnrichmentModel;
use crate::datamodel::document::{DocItem, DoclingDocument};
use crate::errors::Result;

pub struct PictureClassifierModel {
    enabled: bool,
}

impl PictureClassifierModel {
    pub fn new(enabled: bool) -> Self {
        Self { enabled }
    }
}

impl EnrichmentModel for PictureClassifierModel {
    fn is_enabled(&self) -> bool {
        self.enabled
    }
    fn prepare_element(&self, item: &DocItem) -> bool {
        matches!(item, DocItem::Picture(_))
    }
    fn process_batch(&self, _doc: &mut DoclingDocument, _item_indices: &[usize]) -> Result<()> {
        // TODO(onnx): classify picture type (chart, diagram, photo, etc.)
        Ok(())
    }
}
