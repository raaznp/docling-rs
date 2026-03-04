use docling_core::{base_models::Page, errors::Result, ConversionResult};
use std::path::PathBuf;

use crate::base_model::{find_model_artifact, BuildModel};

/// OCR model.
///
/// Wraps the RapidOCR ONNX models (det + rec) for text detection and recognition.
/// Called on pages where the native PDF text layer is absent or insufficient.
/// Mirrors `docling/models/base_ocr_model.py`.
pub struct OcrModel {
    enabled: bool,
    force_full_page: bool,
    lang: Vec<String>,
    det_model_path: Option<PathBuf>,
    rec_model_path: Option<PathBuf>,
}

impl OcrModel {
    pub fn new(
        enabled: bool,
        force_full_page: bool,
        lang: Vec<String>,
        artifacts_path: Option<PathBuf>,
    ) -> Result<Self> {
        let (det_model_path, rec_model_path) = if enabled {
            (
                find_model_artifact("rapidocr_det.onnx", artifacts_path.as_deref()),
                find_model_artifact("rapidocr_rec.onnx", artifacts_path.as_deref()),
            )
        } else {
            (None, None)
        };

        if enabled && (det_model_path.is_none() || rec_model_path.is_none()) {
            log::warn!(
                "OcrModel: one or more model artifacts not found. \
                 OCR will be skipped. Run `docling tools download-models` first."
            );
        }

        Ok(Self {
            enabled,
            force_full_page,
            lang,
            det_model_path,
            rec_model_path,
        })
    }

    fn needs_ocr(&self, page: &Page) -> bool {
        if self.force_full_page {
            return true;
        }
        page.parsed_cells.is_empty() || page.parsed_cells.len() < 5
    }
}

impl BuildModel for OcrModel {
    fn name(&self) -> &str {
        "OcrModel"
    }

    fn is_enabled(&self) -> bool {
        self.enabled
    }

    fn process_pages(&self, _conv_res: &mut ConversionResult, pages: &mut Vec<Page>) -> Result<()> {
        if !self.enabled {
            return Ok(());
        }

        for page in pages.iter_mut() {
            if !self.needs_ocr(page) {
                page.cells = page.parsed_cells.clone();
                continue;
            }

            // TODO: Run RapidOCR det + rec pipeline:
            // 1. Load page.image bytes
            // 2. Run det_model_path ONNX session → text region bounding boxes
            // 3. Crop each region from image
            // 4. Run rec_model_path ONNX session → text + confidence
            // 5. Push OcrCell objects into page.cells
            page.cells = page.parsed_cells.clone();
        }

        Ok(())
    }
}
