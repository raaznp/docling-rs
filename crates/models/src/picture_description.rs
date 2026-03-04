use docling_core::{
    doc_types::{DocItem, DoclingDocument},
    errors::Result,
};

use crate::base_model::EnrichmentModel;

/// Picture description enrichment model.
/// Generates natural language captions for pictures.
/// Full VLM inference is feature-gated; the base impl is a no-op.
/// Mirrors `docling/models/picture_description_base_model.py`.
pub struct PictureDescriptionModel {
    enabled: bool,
    kind: PictureDescriptionKind,
}

pub enum PictureDescriptionKind {
    Disabled,
    ApiVlm {
        endpoint: String,
        api_key: String,
    },
    #[cfg(feature = "vlm")]
    LocalVlm {
        model_path: std::path::PathBuf,
    },
}

impl PictureDescriptionModel {
    pub fn disabled() -> Self {
        Self {
            enabled: false,
            kind: PictureDescriptionKind::Disabled,
        }
    }

    pub fn api_vlm(endpoint: String, api_key: String) -> Self {
        Self {
            enabled: true,
            kind: PictureDescriptionKind::ApiVlm { endpoint, api_key },
        }
    }
}

impl EnrichmentModel for PictureDescriptionModel {
    fn name(&self) -> &str {
        "PictureDescription"
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
        // TODO: For each picture:
        // - ApiVlm: POST image to OpenAI-compatible endpoint, receive description
        // - LocalVlm: Run local VLM inference (feature-gated)
        Ok(())
    }
}
