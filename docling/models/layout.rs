use super::BuildModel;
use crate::datamodel::base_models::Page;
use crate::datamodel::document::ConversionResult;
use crate::errors::Result;
use std::path::PathBuf;

pub struct LayoutModel {
    enabled: bool,
    _model_path: Option<PathBuf>,
}

impl LayoutModel {
    pub fn new(enabled: bool, model_path: Option<PathBuf>) -> Result<Self> {
        Ok(Self {
            enabled,
            _model_path: model_path,
        })
    }
}

impl BuildModel for LayoutModel {
    fn is_enabled(&self) -> bool {
        self.enabled
    }
    fn process_pages(
        &self,
        _conv_res: &mut ConversionResult,
        _pages: &mut Vec<Page>,
    ) -> Result<()> {
        // TODO(onnx feature): run layout detection ONNX model on page bitmaps.
        Ok(())
    }
}
