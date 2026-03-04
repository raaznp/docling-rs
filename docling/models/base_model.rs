use crate::datamodel::base_models::Page;
use crate::datamodel::document::{ConversionResult, DoclingDocument};
use crate::errors::Result;

// ── BuildModel trait ────────────────────────────────────────────

/// A model that runs per-page during the build phase (OCR, layout, table).
pub trait BuildModel: Send + Sync {
    fn is_enabled(&self) -> bool;
    fn process_pages(&self, conv_res: &mut ConversionResult, pages: &mut Vec<Page>) -> Result<()>;
}

// ── EnrichmentModel trait ───────────────────────────────────────

pub trait EnrichmentModel: Send + Sync {
    fn is_enabled(&self) -> bool;
    fn prepare_element(&self, item: &crate::datamodel::document::DocItem) -> bool;
    fn process_batch(&self, doc: &mut DoclingDocument, item_indices: &[usize]) -> Result<()>;
}
