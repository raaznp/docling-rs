use serde::{Deserialize, Serialize};
use std::path::PathBuf;

// ============================================================
// Accelerator options
// ============================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum AcceleratorDevice {
    Cpu,
    Cuda,
    Mps,
    Auto,
}

impl Default for AcceleratorDevice {
    fn default() -> Self {
        Self::Auto
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AcceleratorOptions {
    pub device: AcceleratorDevice,
    /// Number of threads for CPU inference.
    pub num_threads: Option<usize>,
}

// ============================================================
// OCR options
// ============================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OcrOptions {
    pub enabled: bool,
    /// Force OCR even when native text layer is present.
    pub force_full_page_ocr: bool,
    /// Language hints for the OCR engine (e.g., ["en", "fr"]).
    pub lang: Vec<String>,
    /// Maximum number of OCR threads.
    pub max_workers: Option<usize>,
}

impl Default for OcrOptions {
    fn default() -> Self {
        Self {
            enabled: true,
            force_full_page_ocr: false,
            lang: vec!["en".to_string()],
            max_workers: None,
        }
    }
}

// ============================================================
// Table structure options
// ============================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TableStructureOptions {
    pub enabled: bool,
    pub mode: TableFormerMode,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TableFormerMode {
    Fast,
    Accurate,
}

impl Default for TableStructureOptions {
    fn default() -> Self {
        Self {
            enabled: true,
            mode: TableFormerMode::Fast,
        }
    }
}

// ============================================================
// Picture classifier options
// ============================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PictureClassificationOptions {
    pub enabled: bool,
}

impl Default for PictureClassificationOptions {
    fn default() -> Self {
        Self { enabled: true }
    }
}

// ============================================================
// Picture description options
// ============================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PictureDescriptionKind {
    /// Disabled picture description.
    Disabled,
    /// Use a local VLM via ONNX/ORT.
    LocalVlm,
    /// Use a remote API (OpenAI-compatible).
    ApiVlm,
}

impl Default for PictureDescriptionKind {
    fn default() -> Self {
        Self::Disabled
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PictureDescriptionOptions {
    pub kind: PictureDescriptionKind,
    pub max_tokens: Option<usize>,
    pub prompt: Option<String>,
}

// ============================================================
// Base PipelineOptions
// ============================================================

/// Base options common to all pipelines.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PipelineOptions {
    /// Path overriding the default model artifact path.
    pub artifacts_path: Option<PathBuf>,
    /// Overall document timeout in seconds (None = no limit).
    pub document_timeout: Option<f64>,
    /// Whether to allow external plugin implementations.
    pub allow_external_plugins: bool,
}

impl Default for PipelineOptions {
    fn default() -> Self {
        Self {
            artifacts_path: None,
            document_timeout: None,
            allow_external_plugins: false,
        }
    }
}

// ============================================================
// ConvertPipelineOptions
// ============================================================

/// Options for the enrichment-capable conversion pipelines.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ConvertPipelineOptions {
    pub base: PipelineOptions,

    pub do_ocr: bool,
    pub ocr_options: OcrOptions,

    pub do_table_structure: bool,
    pub table_structure_options: TableStructureOptions,

    pub do_picture_classification: bool,
    pub picture_classification_options: PictureClassificationOptions,

    pub do_picture_description: bool,
    pub picture_description_options: PictureDescriptionOptions,

    pub do_chart_extraction: bool,

    pub accelerator_options: AcceleratorOptions,

    pub enable_remote_services: bool,
}

// ============================================================
// PdfPipelineOptions
// ============================================================

/// Additional options specific to the PDF pipeline.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PdfPipelineOptions {
    pub base: ConvertPipelineOptions,

    /// Generate page-level images (for picture extraction).
    pub generate_page_images: bool,
    /// DPI to render PDF pages at.
    pub images_scale: f32,
    /// Whether to return parsed page objects in ConversionResult.
    pub generate_parsed_pages: bool,
    /// Whether to generate images for figures in the output document.
    pub generate_picture_images: bool,
    /// Whether to generate images for table segments.
    pub generate_table_images: bool,
}

impl Default for PdfPipelineOptions {
    fn default() -> Self {
        Self {
            base: ConvertPipelineOptions {
                do_ocr: true,
                do_table_structure: true,
                ..Default::default()
            },
            generate_page_images: false,
            images_scale: 2.0,
            generate_parsed_pages: false,
            generate_picture_images: false,
            generate_table_images: false,
        }
    }
}
