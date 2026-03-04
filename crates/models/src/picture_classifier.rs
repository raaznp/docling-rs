use docling_core::{
    doc_types::{DocItem, DoclingDocument},
    errors::Result,
};

use crate::base_model::EnrichmentModel;

/// Picture classifier enrichment model.
/// Classifies pictures into categories (chart, photograph, diagram, logo, etc.)
/// Mirrors `docling/models/stages/picture_classifier/`.
pub struct PictureClassifierModel {
    enabled: bool,
}

impl PictureClassifierModel {
    pub fn new(enabled: bool) -> Self {
        Self { enabled }
    }
}

impl EnrichmentModel for PictureClassifierModel {
    fn name(&self) -> &str {
        "PictureClassifier"
    }

    fn is_enabled(&self) -> bool {
        self.enabled
    }

    fn prepare_element(&self, item: &DocItem) -> Option<DocItem> {
        match item {
            DocItem::Picture(_) => Some(item.clone()),
            _ => None,
        }
    }

    fn process_batch(&self, doc: &mut DoclingDocument, batch: &[DocItem]) -> Result<()> {
        // TODO: For each picture in batch:
        // 1. Load image data from doc page images
        // 2. Run ONNX classifier → label probabilities
        // 3. Update picture item's classification field in doc.body
        Ok(())
    }
}
