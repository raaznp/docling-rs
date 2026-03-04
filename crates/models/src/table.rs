use docling_core::{base_models::Page, errors::Result, ConversionResult};
use std::path::PathBuf;

use crate::base_model::{find_model_artifact, BuildModel};

/// Table structure recognition model.
///
/// Wraps the TableFormer ONNX model that predicts row/column structure
/// from a detected table region image.
/// Mirrors `docling/models/base_table_model.py`.
pub struct TableStructureModel {
    enabled: bool,
    mode: TableFormerMode,
    model_path: Option<PathBuf>,
}

pub enum TableFormerMode {
    Fast,
    Accurate,
}

impl TableStructureModel {
    pub fn new(
        enabled: bool,
        mode: TableFormerMode,
        artifacts_path: Option<PathBuf>,
    ) -> Result<Self> {
        let model_name = match mode {
            TableFormerMode::Fast => "tableformer_fast.onnx",
            TableFormerMode::Accurate => "tableformer_accurate.onnx",
        };

        let model_path = if enabled {
            find_model_artifact(model_name, artifacts_path.as_deref())
        } else {
            None
        };

        if enabled && model_path.is_none() {
            log::warn!(
                "TableStructureModel: model artifact '{}' not found. \
                 Table structure recognition will be skipped.",
                model_name
            );
        }

        Ok(Self {
            enabled,
            mode,
            model_path,
        })
    }
}

impl BuildModel for TableStructureModel {
    fn name(&self) -> &str {
        "TableStructureModel"
    }

    fn is_enabled(&self) -> bool {
        self.enabled
    }

    fn process_pages(&self, _conv_res: &mut ConversionResult, pages: &mut Vec<Page>) -> Result<()> {
        if !self.enabled {
            return Ok(());
        }

        for page in pages.iter_mut() {
            let _table_clusters: Vec<_> = page
                .predictions
                .layout
                .as_ref()
                .map(|lp| {
                    lp.clusters
                        .iter()
                        .filter(|c| matches!(c.label, docling_core::LayoutLabel::Table))
                        .collect::<Vec<_>>()
                })
                .unwrap_or_default();

            // TODO: For each table cluster:
            // 1. Crop table region from page image
            // 2. Run TableFormer ONNX model (model_path)
            // 3. Decode row/column structure → TableData cells
            // 4. Attach cells to cluster
        }

        Ok(())
    }
}
