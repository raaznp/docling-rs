use docling_core::{
    base_models::Page,
    doc_types::{DocItem, DoclingDocument},
    errors::Result,
    ConversionResult,
};
use std::path::Path;

// ============================================================
// BuildModel trait
// ============================================================

/// A model that operates on batches of pages during the build phase.
/// Corresponds to the Python pipeline `build_pipe` elements.
pub trait BuildModel: Send + Sync {
    fn name(&self) -> &str;
    fn is_enabled(&self) -> bool;

    /// Process a batch of pages in-place.
    fn process_pages(&self, conv_res: &mut ConversionResult, pages: &mut Vec<Page>) -> Result<()>;
}

// ============================================================
// EnrichmentModel trait
// ============================================================

/// A model that enriches individual document items after assembly.
/// Corresponds to the Python pipeline `enrichment_pipe` elements.
pub trait EnrichmentModel: Send + Sync {
    fn name(&self) -> &str;
    fn is_enabled(&self) -> bool;
    fn batch_size(&self) -> usize {
        8
    }

    /// Prepare an element for processing (filter/transform). Returns None to skip.
    fn prepare_element(&self, item: &DocItem) -> Option<DocItem>;

    /// Process a batch of prepared elements and update the document in-place.
    fn process_batch(&self, doc: &mut DoclingDocument, batch: &[DocItem]) -> Result<()>;
}

// ============================================================
// Model artifacts path helper
// ============================================================

/// Returns the path to a model artifact by name.
/// Checks: (1) explicit `artifacts_path`, (2) `DOCLING_ARTIFACTS_PATH` env var,
/// (3) `~/.docling/models/`.
pub fn find_model_artifact(
    model_name: &str,
    artifacts_path: Option<&Path>,
) -> Option<std::path::PathBuf> {
    // 1. Explicit override
    if let Some(base) = artifacts_path {
        let p = base.join(model_name);
        if p.exists() {
            return Some(p);
        }
    }

    // 2. Environment variable
    if let Ok(env_path) = std::env::var("DOCLING_ARTIFACTS_PATH") {
        let p = std::path::Path::new(&env_path).join(model_name);
        if p.exists() {
            return Some(p);
        }
    }

    // 3. Default: ~/.docling/models/<name>
    if let Some(home) = dirs::home_dir() {
        let p = home.join(".docling").join("models").join(model_name);
        if p.exists() {
            return Some(p);
        }
    }

    None
}
