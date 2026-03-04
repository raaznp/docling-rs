use crate::backend::{detect_format, BackendSource, DeclarativeBackend};
use crate::datamodel::base_models::{ConversionStatus, DocumentLimits, InputFormat};
use crate::datamodel::document::{ConversionResult, InputDocument};
use crate::errors::{DoclingError, Result};
use crate::pipeline::base::BasePipeline;
use crate::pipeline::simple::SimplePipeline;
use crate::pipeline::standard_pdf::StandardPdfPipeline;
use std::path::PathBuf;

/// Main entry point for document conversion.
/// Mirrors `docling/document_converter.py::DocumentConverter`.
pub struct DocumentConverter {
    pub limits: DocumentLimits,
    pub artifacts_path: Option<PathBuf>,
    pub do_ocr: bool,
    pub do_table_structure: bool,
}

impl Default for DocumentConverter {
    fn default() -> Self {
        Self {
            limits: DocumentLimits::default(),
            artifacts_path: None,
            do_ocr: true,
            do_table_structure: true,
        }
    }
}

impl DocumentConverter {
    pub fn new() -> Self {
        Self::default()
    }

    /// Convert a single file by path.
    pub fn convert(&self, path: impl Into<PathBuf>) -> Result<ConversionResult> {
        let path = path.into();
        let fmt = path
            .extension()
            .and_then(|e| e.to_str())
            .and_then(|e| InputFormat::from_extension(e))
            .ok_or_else(|| DoclingError::unsupported(path.display().to_string()))?;

        let input = InputDocument::from_path(path, fmt.clone(), self.limits.clone())?;
        self.convert_input(input, &fmt)
    }

    /// Convert from an `InputDocument` with an explicit format.
    pub fn convert_input(
        &self,
        input: InputDocument,
        fmt: &InputFormat,
    ) -> Result<ConversionResult> {
        let mut result = ConversionResult::new(input);

        match fmt {
            InputFormat::Pdf => {
                let pipeline = StandardPdfPipeline::new(
                    self.artifacts_path.clone(),
                    self.do_ocr,
                    self.do_table_structure,
                    false,
                    None,
                )?;
                result = pipeline.execute(result, false);
            }
            InputFormat::Html => {
                let source = BackendSource::Bytes(
                    result.input.data.clone(),
                    result.input.file.display().to_string(),
                );
                let doc = crate::backend::html::HtmlBackend::new(source).convert()?;
                result.document = Some(doc);
                result.status = ConversionStatus::Success;
            }
            InputFormat::Md => {
                let source = BackendSource::Bytes(
                    result.input.data.clone(),
                    result.input.file.display().to_string(),
                );
                let doc = crate::backend::markdown::MarkdownBackend::new(source).convert()?;
                result.document = Some(doc);
                result.status = ConversionStatus::Success;
            }
            InputFormat::Csv => {
                let source = BackendSource::Bytes(
                    result.input.data.clone(),
                    result.input.file.display().to_string(),
                );
                let doc = crate::backend::csv::CsvBackend::new(source).convert()?;
                result.document = Some(doc);
                result.status = ConversionStatus::Success;
            }
            InputFormat::Asciidoc => {
                let source = BackendSource::Bytes(
                    result.input.data.clone(),
                    result.input.file.display().to_string(),
                );
                let doc = crate::backend::asciidoc::AsciiDocBackend::new(source).convert()?;
                result.document = Some(doc);
                result.status = ConversionStatus::Success;
            }
            InputFormat::Latex => {
                let source = BackendSource::Bytes(
                    result.input.data.clone(),
                    result.input.file.display().to_string(),
                );
                let doc = crate::backend::latex::LatexBackend::new(source).convert()?;
                result.document = Some(doc);
                result.status = ConversionStatus::Success;
            }
            InputFormat::Vtt => {
                let source = BackendSource::Bytes(
                    result.input.data.clone(),
                    result.input.file.display().to_string(),
                );
                let doc = crate::backend::webvtt::WebVttBackend::new(source).convert()?;
                result.document = Some(doc);
                result.status = ConversionStatus::Success;
            }
            InputFormat::JsonDocling => {
                let source = BackendSource::Bytes(
                    result.input.data.clone(),
                    result.input.file.display().to_string(),
                );
                let doc = crate::backend::json::JsonBackend::new(source).convert()?;
                result.document = Some(doc);
                result.status = ConversionStatus::Success;
            }
            f if f.is_audio() => {
                let source = BackendSource::Bytes(
                    result.input.data.clone(),
                    result.input.file.display().to_string(),
                );
                let doc = crate::backend::audio::AudioBackend::new(source).convert()?;
                result.document = Some(doc);
                result.status = ConversionStatus::Success;
            }
            f if f.is_video() => {
                let source = BackendSource::Bytes(
                    result.input.data.clone(),
                    result.input.file.display().to_string(),
                );
                let doc = crate::backend::video::VideoBackend::new(source).convert()?;
                result.document = Some(doc);
                result.status = ConversionStatus::Success;
            }
            _ => {
                return Err(DoclingError::unsupported(fmt.to_string()));
            }
        }

        Ok(result)
    }
}
