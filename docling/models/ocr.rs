use super::BuildModel;
use crate::datamodel::base_models::Page;
use crate::datamodel::document::ConversionResult;
use crate::errors::Result;
use std::path::PathBuf;

pub struct OcrModel {
    enabled: bool,
    _lang: Vec<String>,
    _det_model: Option<PathBuf>,
    _rec_model: Option<PathBuf>,
}

impl OcrModel {
    pub fn new(
        enabled: bool,
        _force_full_page: bool,
        lang: Vec<String>,
        path: Option<PathBuf>,
    ) -> Result<Self> {
        Ok(Self {
            enabled,
            _lang: lang,
            _det_model: path.clone(),
            _rec_model: path,
        })
    }
}

impl BuildModel for OcrModel {
    fn is_enabled(&self) -> bool {
        self.enabled
    }
    fn process_pages(
        &self,
        _conv_res: &mut ConversionResult,
        _pages: &mut Vec<Page>,
    ) -> Result<()> {
        // TODO(onnx feature): run OCR detection+recognition on page bitmaps.
        Ok(())
    }
}
