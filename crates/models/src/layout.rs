use docling_core::{
    base_models::Page,
    errors::{DoclingError, Result},
    ConversionResult,
};
use std::path::PathBuf;

use crate::base_model::{find_model_artifact, BuildModel};

/// Layout detection model.
///
/// Wraps the ONNX layout detection model (docling-ibm-models layout model).
/// Full ONNX inference is implemented once the ort stable API is used.
/// For now, model loading is deferred to a TODO in `process_pages`.
pub struct LayoutModel {
    enabled: bool,
    model_path: Option<PathBuf>,
}

impl LayoutModel {
    pub fn new(enabled: bool, artifacts_path: Option<PathBuf>) -> Result<Self> {
        let model_path = if enabled {
            find_model_artifact("layout_model.onnx", artifacts_path.as_deref())
                .or_else(|| find_model_artifact("models/layout.onnx", artifacts_path.as_deref()))
        } else {
            None
        };

        if enabled && model_path.is_none() {
            log::warn!(
                "LayoutModel: model artifact not found. \
                 Run `docling tools download-models` first. Inference will be skipped."
            );
        }

        Ok(Self {
            enabled,
            model_path,
        })
    }
}

impl BuildModel for LayoutModel {
    fn name(&self) -> &str {
        "LayoutModel"
    }

    fn is_enabled(&self) -> bool {
        self.enabled
    }

    fn process_pages(&self, _conv_res: &mut ConversionResult, pages: &mut Vec<Page>) -> Result<()> {
        if !self.enabled {
            return Ok(());
        }

        // TODO: For each page with an image:
        // 1. Load image bytes from page.image
        // 2. Preprocess: resize to 640×640, normalize to [0,1] float32 tensor
        // 3. Run ONNX inference via `ort` → output tensors (bboxes, class ids, scores)
        // 4. Post-process: apply NMS, map class ids → LayoutLabel
        // 5. Push resulting LayoutCluster objects into page.predictions.layout
        //
        // Model path: self.model_path

        Ok(())
    }
}
