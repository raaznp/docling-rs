use super::BuildModel;
use crate::datamodel::base_models::Page;
use crate::datamodel::document::ConversionResult;
use crate::errors::Result;
use std::path::PathBuf;

#[derive(Debug, Clone, Copy)]
pub enum TableFormerMode {
    Fast,
    Accurate,
}

pub struct TableStructureModel {
    enabled: bool,
    _mode: TableFormerMode,
    _model_path: Option<PathBuf>,
}

impl TableStructureModel {
    pub fn new(enabled: bool, mode: TableFormerMode, path: Option<PathBuf>) -> Result<Self> {
        Ok(Self {
            enabled,
            _mode: mode,
            _model_path: path,
        })
    }
}

impl BuildModel for TableStructureModel {
    fn is_enabled(&self) -> bool {
        self.enabled
    }
    fn process_pages(
        &self,
        _conv_res: &mut ConversionResult,
        _pages: &mut Vec<Page>,
    ) -> Result<()> {
        // TODO(onnx feature): run TableFormer ONNX model.
        Ok(())
    }
}
