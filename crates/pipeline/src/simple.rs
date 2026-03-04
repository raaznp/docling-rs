use docling_backends::DeclarativeBackend;
use docling_core::{base_models::ConversionStatus, errors::Result, ConversionResult};

use crate::base::BasePipeline;

/// Simple pipeline for declarative backends (HTML, Markdown, DOCX, XLSX, PPTX, CSV, etc.)
///
/// The backend itself produces the final DoclingDocument, so this pipeline
/// just calls `backend.convert()` during build_document.
/// Mirrors `docling/pipeline/simple_pipeline.py`.
pub struct SimplePipeline;

impl BasePipeline for SimplePipeline {
    fn name(&self) -> &str {
        "SimplePipeline"
    }

    fn build_document(&self, mut conv_res: ConversionResult) -> Result<ConversionResult> {
        // The backend should be set up to do declarative conversion.
        // In the converter, the backend is stored in conv_res.input.
        // Here we just mark the status as started; the actual conversion
        // was already done when the InputDocument was constructed with
        // the DeclarativeBackend and its convert() was called.
        //
        // In the full implementation, the DocumentConverter would call
        // backend.convert() and store the result before calling
        // pipeline.execute(). This is the pattern used here.
        conv_res.status = ConversionStatus::Started;
        Ok(conv_res)
    }

    fn determine_status(&self, conv_res: &ConversionResult) -> ConversionStatus {
        if conv_res.document.is_some() {
            ConversionStatus::Success
        } else {
            ConversionStatus::Failure
        }
    }
}
