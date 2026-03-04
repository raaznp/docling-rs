use crate::datamodel::document::ConversionResult;
use crate::errors::Result;
use crate::pipeline::base::BasePipeline;

/// SimplePipeline for declarative backends (HTML, Markdown, DOCX, etc.)
pub struct SimplePipeline;

impl BasePipeline for SimplePipeline {
    fn name(&self) -> &str {
        "SimplePipeline"
    }

    fn build_document(&self, conv_res: ConversionResult) -> Result<ConversionResult> {
        Ok(conv_res)
    }
}
