//! datamodel/pipeline_options.rs — pipeline configuration options.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PipelineOptions {
    pub do_ocr: bool,
    pub do_table_structure: bool,
    pub do_picture_classification: bool,
    pub do_picture_description: bool,
    pub generate_page_images: bool,
    pub generate_picture_images: bool,
}

impl Default for PipelineOptions {
    fn default() -> Self {
        Self {
            do_ocr: true,
            do_table_structure: true,
            do_picture_classification: false,
            do_picture_description: false,
            generate_page_images: false,
            generate_picture_images: false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PdfPipelineOptions {
    pub pipeline: PipelineOptions,
    pub artifacts_path: Option<std::path::PathBuf>,
    pub document_timeout: Option<f64>,
}
