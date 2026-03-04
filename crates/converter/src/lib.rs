use docling_backends::{detect_format, BackendSource, DeclarativeBackend};
use docling_core::{
    base_models::{ConversionStatus, DocumentLimits, DocumentStream, ErrorItem, InputFormat},
    errors::{DoclingError, Result},
    ConversionResult, DoclingError as CoreError, InputDocument,
};
use docling_pipeline::{BasePipeline, SimplePipeline, StandardPdfPipeline};
use rayon::prelude::*;
use std::{
    collections::HashMap,
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
};

// ============================================================
// Format options
// ============================================================

/// Options for a particular input format.
#[derive(Clone)]
pub struct FormatOption {
    pub pipeline: PipelineKind,
    pub backend: BackendKind,
}

#[derive(Clone, Debug)]
pub enum PipelineKind {
    Simple,
    StandardPdf,
}

#[derive(Clone, Debug)]
pub enum BackendKind {
    Html,
    Markdown,
    Csv,
    Docx,
    Xlsx,
    Pptx,
    Pdf,
    Image,
    Latex,
    AsciiDoc,
    Noop,
    DoclingJson,
    WebVtt,
    XmlJats,
    XmlUspto,
    XmlXbrl,
}

// ============================================================
// DocumentConverter
// ============================================================

/// The main entry point for converting documents.
///
/// Mirrors `docling/document_converter.py::DocumentConverter`.
pub struct DocumentConverter {
    pub allowed_formats: Vec<InputFormat>,
    pub format_options: HashMap<InputFormat, FormatOption>,
    pub artifacts_path: Option<PathBuf>,
    pub do_ocr: bool,
    pub do_table_structure: bool,
    pub document_timeout: Option<f64>,
    /// Pipeline cache to avoid re-initialising for each document.
    pipeline_cache: Mutex<HashMap<String, Arc<dyn BasePipeline>>>,
}

impl DocumentConverter {
    /// Create a new converter with default settings.
    pub fn new() -> Self {
        Self {
            allowed_formats: all_formats(),
            format_options: default_format_options(),
            artifacts_path: None,
            do_ocr: true,
            do_table_structure: true,
            document_timeout: None,
            pipeline_cache: Mutex::new(HashMap::new()),
        }
    }

    /// Builder: set artifacts path.
    pub fn with_artifacts_path(mut self, path: PathBuf) -> Self {
        self.artifacts_path = Some(path);
        self
    }

    /// Builder: disable OCR.
    pub fn without_ocr(mut self) -> Self {
        self.do_ocr = false;
        self
    }

    /// Builder: set document timeout seconds.
    pub fn with_timeout(mut self, secs: f64) -> Self {
        self.document_timeout = Some(secs);
        self
    }

    /// Builder: restrict to specific formats.
    pub fn with_formats(mut self, formats: Vec<InputFormat>) -> Self {
        self.allowed_formats = formats;
        self
    }

    // ── Single document conversion ───────────────────────

    /// Convert a document from a file path.
    pub fn convert<P: AsRef<Path>>(&self, path: P) -> Result<ConversionResult> {
        let path = path.as_ref().to_path_buf();
        let format = self.detect_format_for_path(&path)?;
        let limits = DocumentLimits::default();
        let input = InputDocument::from_path(path, format, limits)?;
        self.process_document(input)
    }

    /// Convert a document from in-memory bytes.
    pub fn convert_stream(&self, stream: DocumentStream) -> Result<ConversionResult> {
        let format = self
            .detect_format_for_name(&stream.name)
            .ok_or_else(|| DoclingError::UnsupportedFormat(stream.name.clone()))?;
        let limits = DocumentLimits::default();
        let input = InputDocument::from_stream(stream, format, limits)?;
        self.process_document(input)
    }

    /// Convert a Markdown or HTML string.
    pub fn convert_string(&self, content: &str, format: InputFormat) -> Result<ConversionResult> {
        let ext = match format {
            InputFormat::Md => ".md",
            InputFormat::Html => ".html",
            _ => return Err(DoclingError::UnsupportedFormat(format.to_string())),
        };
        let stream = DocumentStream::new(format!("document{}", ext), content.as_bytes().to_vec());
        self.convert_stream(stream)
    }

    // ── Batch conversion ─────────────────────────────────

    /// Convert multiple documents in parallel (rayon-based).
    pub fn convert_all<P: AsRef<Path> + Send + Sync>(
        &self,
        paths: &[P],
        raises_on_error: bool,
    ) -> Vec<ConversionResult> {
        paths
            .par_iter()
            .map(|p| {
                self.convert(p.as_ref()).unwrap_or_else(|e| {
                    // Return a failure result
                    let dummy_stream =
                        DocumentStream::new(p.as_ref().to_string_lossy().to_string(), vec![]);
                    let fake_input = InputDocument::from_stream(
                        dummy_stream,
                        InputFormat::Pdf,
                        DocumentLimits::default(),
                    )
                    .expect("dummy input");
                    let mut cr = ConversionResult::new(fake_input);
                    cr.status = ConversionStatus::Failure;
                    cr.errors.push(ErrorItem::new(
                        docling_core::base_models::DoclingComponentType::Pipeline,
                        "DocumentConverter",
                        e.to_string(),
                    ));
                    cr
                })
            })
            .collect()
    }

    // ── Internal helpers ─────────────────────────────────

    fn detect_format_for_path(&self, path: &Path) -> Result<InputFormat> {
        path.extension()
            .and_then(|e| e.to_str())
            .and_then(|ext| InputFormat::from_extension(ext))
            .ok_or_else(|| DoclingError::UnsupportedFormat(path.to_string_lossy().to_string()))
    }

    fn detect_format_for_name(&self, name: &str) -> Option<InputFormat> {
        detect_format(name)
    }

    fn process_document(&self, input: InputDocument) -> Result<ConversionResult> {
        let format = input.format.clone();

        if !self.allowed_formats.contains(&format) {
            return Err(DoclingError::FormatNotAllowed {
                format: format.to_string(),
            });
        }

        let format_opt = self
            .format_options
            .get(&format)
            .ok_or_else(|| DoclingError::UnsupportedFormat(format.to_string()))?;

        let source =
            BackendSource::Bytes(input.data.clone(), input.file.to_string_lossy().to_string());

        // For declarative backends → use SimplePipeline
        match format_opt.pipeline {
            PipelineKind::Simple => {
                let mut doc = create_declarative_output(source, &format_opt.backend, &input)?;
                let mut conv_res = ConversionResult::new(input);
                conv_res.document = Some(doc);
                conv_res.status = ConversionStatus::Success;
                Ok(conv_res)
            }
            PipelineKind::StandardPdf => {
                let pipeline = self.get_or_create_pdf_pipeline()?;
                let conv_res = ConversionResult::new(input);
                Ok(pipeline.execute(conv_res, true))
            }
        }
    }

    fn get_or_create_pdf_pipeline(&self) -> Result<Arc<dyn BasePipeline>> {
        let cache_key = format!(
            "pdf-ocr{}-table{}-timeout{:?}",
            self.do_ocr, self.do_table_structure, self.document_timeout
        );

        let mut cache = self.pipeline_cache.lock().unwrap();
        if let Some(p) = cache.get(&cache_key) {
            return Ok(p.clone());
        }

        let pipeline = Arc::new(StandardPdfPipeline::new(
            self.artifacts_path.clone(),
            self.do_ocr,
            self.do_table_structure,
            false, // picture classification
            self.document_timeout,
        )?) as Arc<dyn BasePipeline>;

        cache.insert(cache_key, pipeline.clone());
        Ok(pipeline)
    }
}

impl Default for DocumentConverter {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================
// Backend dispatch
// ============================================================

fn create_declarative_output(
    source: BackendSource,
    backend: &BackendKind,
    _input: &InputDocument,
) -> Result<docling_core::DoclingDocument> {
    use docling_backends::*;

    match backend {
        BackendKind::Html => {
            let mut b = html::HtmlBackend::new(source, None);
            b.convert()
        }
        BackendKind::Markdown => {
            let mut b = markdown::MarkdownBackend::new(source, true);
            b.convert()
        }
        BackendKind::Csv => {
            let mut b = csv::CsvBackend::new(source);
            b.convert()
        }
        BackendKind::Latex => {
            let mut b = latex::LatexBackend::new(source, true);
            b.convert()
        }
        BackendKind::AsciiDoc => {
            let mut b = asciidoc::AsciiDocBackend::new(source);
            b.convert()
        }
        BackendKind::Noop => {
            let mut b = noop::NoopBackend::new(source);
            b.convert()
        }
        BackendKind::DoclingJson => {
            let mut b = json::DoclingJsonBackend::new(source);
            b.convert()
        }
        BackendKind::WebVtt => {
            let mut b = webvtt::WebVttBackend::new(source);
            b.convert()
        }
        BackendKind::XmlJats => {
            let mut b = xml::jats::JatsBackend::new(source);
            b.convert()
        }
        BackendKind::XmlUspto => {
            let mut b = xml::uspto::UsptoBackend::new(source);
            b.convert()
        }
        BackendKind::XmlXbrl => {
            let mut b = xml::xbrl::XbrlBackend::new(source);
            b.convert()
        }
        #[cfg(feature = "office")]
        BackendKind::Docx => {
            let mut b = docx::DocxBackend::new(source);
            b.convert()
        }
        #[cfg(feature = "office")]
        BackendKind::Xlsx => {
            let mut b = xlsx::XlsxBackend::new(source);
            b.convert()
        }
        #[cfg(feature = "office")]
        BackendKind::Pptx => {
            let mut b = pptx::PptxBackend::new(source);
            b.convert()
        }
        _ => Err(DoclingError::UnsupportedFormat(format!("{:?}", backend))),
    }
}

// ============================================================
// Default format/pipeline registry
// ============================================================

fn all_formats() -> Vec<InputFormat> {
    vec![
        InputFormat::Pdf,
        InputFormat::Docx,
        InputFormat::Pptx,
        InputFormat::Xlsx,
        InputFormat::Html,
        InputFormat::Md,
        InputFormat::Asciidoc,
        InputFormat::Csv,
        InputFormat::Image,
        InputFormat::Latex,
        InputFormat::XmlJats,
        InputFormat::XmlUspto,
        InputFormat::XmlXbrl,
        InputFormat::JsonDocling,
        InputFormat::Vtt,
    ]
}

fn default_format_options() -> HashMap<InputFormat, FormatOption> {
    let mut map = HashMap::new();
    let simple = |b: BackendKind| FormatOption {
        pipeline: PipelineKind::Simple,
        backend: b,
    };
    let pdf = |b: BackendKind| FormatOption {
        pipeline: PipelineKind::StandardPdf,
        backend: b,
    };

    map.insert(InputFormat::Html, simple(BackendKind::Html));
    map.insert(InputFormat::Md, simple(BackendKind::Markdown));
    map.insert(InputFormat::Csv, simple(BackendKind::Csv));
    map.insert(InputFormat::Latex, simple(BackendKind::Latex));
    map.insert(InputFormat::Asciidoc, simple(BackendKind::AsciiDoc));
    map.insert(InputFormat::JsonDocling, simple(BackendKind::DoclingJson));
    map.insert(InputFormat::Vtt, simple(BackendKind::WebVtt));
    map.insert(InputFormat::XmlJats, simple(BackendKind::XmlJats));
    map.insert(InputFormat::XmlUspto, simple(BackendKind::XmlUspto));
    map.insert(InputFormat::XmlXbrl, simple(BackendKind::XmlXbrl));
    map.insert(InputFormat::Docx, simple(BackendKind::Docx));
    map.insert(InputFormat::Xlsx, simple(BackendKind::Xlsx));
    map.insert(InputFormat::Pptx, simple(BackendKind::Pptx));
    map.insert(InputFormat::Pdf, pdf(BackendKind::Pdf));
    map.insert(InputFormat::Image, pdf(BackendKind::Image));

    map
}
