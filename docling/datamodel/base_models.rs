use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// ── InputFormat ────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum InputFormat {
    // Document formats
    Pdf,
    Docx,
    Pptx,
    Xlsx,
    Html,
    Md,
    Asciidoc,
    Csv,
    Latex,
    // Image formats
    Png,
    Jpeg,
    Tiff,
    Bmp,
    Webp,
    // XML schemas
    XmlJats,
    XmlUspto,
    XmlXbrl,
    MetsGbs,
    // Other
    JsonDocling,
    Vtt,
    // Audio (asr feature)
    Wav,
    Mp3,
    M4a,
    Aac,
    Ogg,
    Flac,
    // Video (asr feature + ffmpeg)
    Mp4,
    Avi,
    Mov,
}

impl InputFormat {
    pub fn from_extension(ext: &str) -> Option<Self> {
        match ext.to_lowercase().as_str() {
            "pdf" => Some(Self::Pdf),
            "docx" => Some(Self::Docx),
            "pptx" => Some(Self::Pptx),
            "xlsx" => Some(Self::Xlsx),
            "html" | "htm" | "xhtml" => Some(Self::Html),
            "md" | "markdown" => Some(Self::Md),
            "adoc" | "asciidoc" | "asc" => Some(Self::Asciidoc),
            "csv" => Some(Self::Csv),
            "tex" | "latex" => Some(Self::Latex),
            "png" => Some(Self::Png),
            "jpg" | "jpeg" => Some(Self::Jpeg),
            "tiff" | "tif" => Some(Self::Tiff),
            "bmp" => Some(Self::Bmp),
            "webp" => Some(Self::Webp),
            "json" => Some(Self::JsonDocling),
            "vtt" => Some(Self::Vtt),
            "wav" => Some(Self::Wav),
            "mp3" => Some(Self::Mp3),
            "m4a" => Some(Self::M4a),
            "aac" => Some(Self::Aac),
            "ogg" => Some(Self::Ogg),
            "flac" => Some(Self::Flac),
            "mp4" => Some(Self::Mp4),
            "avi" => Some(Self::Avi),
            "mov" => Some(Self::Mov),
            _ => None,
        }
    }

    pub fn is_image(&self) -> bool {
        matches!(
            self,
            Self::Png | Self::Jpeg | Self::Tiff | Self::Bmp | Self::Webp
        )
    }

    pub fn is_audio(&self) -> bool {
        matches!(
            self,
            Self::Wav | Self::Mp3 | Self::M4a | Self::Aac | Self::Ogg | Self::Flac
        )
    }

    pub fn is_video(&self) -> bool {
        matches!(self, Self::Mp4 | Self::Avi | Self::Mov)
    }
}

impl std::fmt::Display for InputFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

// ── OutputFormat ────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum OutputFormat {
    /// Rich HTML with embedded images or references
    Html,
    /// GitHub Flavored Markdown
    Markdown,
    /// Lossless JSON serialization of DoclingDocument
    Json,
    /// Plain text without markup
    Text,
    /// DocTags markup for efficient content+layout representation
    Doctags,
    /// WebVTT timed text (for audio/video transcripts)
    Vtt,
}

// ── ConversionStatus ────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ConversionStatus {
    #[default]
    Pending,
    Started,
    Success,
    PartialSuccess,
    Failure,
    Skipped,
}

// ── Component types ─────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum DoclingComponentType {
    Pipeline,
    DocumentBackend,
    Model,
    UserInput,
    Unknown,
}

// ── ErrorItem ───────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorItem {
    pub component_type: DoclingComponentType,
    pub module_name: String,
    pub error_message: String,
}

impl ErrorItem {
    pub fn new(
        component_type: DoclingComponentType,
        module_name: impl Into<String>,
        error_message: impl Into<String>,
    ) -> Self {
        Self {
            component_type,
            module_name: module_name.into(),
            error_message: error_message.into(),
        }
    }
}

// ── Geometry ────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BoundingBox {
    pub l: f64,
    pub t: f64,
    pub r: f64,
    pub b: f64,
}

impl BoundingBox {
    pub fn new(l: f64, t: f64, r: f64, b: f64) -> Self {
        Self { l, t, r, b }
    }
    pub fn width(&self) -> f64 {
        (self.r - self.l).abs()
    }
    pub fn height(&self) -> f64 {
        (self.b - self.t).abs()
    }
    pub fn area(&self) -> f64 {
        self.width() * self.height()
    }
    pub fn overlaps(&self, other: &BoundingBox) -> bool {
        self.l < other.r && self.r > other.l && self.t < other.b && self.b > other.t
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PageSize {
    pub width: f64,
    pub height: f64,
}

// ── Cell types ─────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OcrCell {
    pub id: u32,
    pub text: String,
    pub confidence: f32,
    pub bbox: BoundingBox,
    pub from_ocr: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Cell {
    pub id: u32,
    pub text: String,
    pub bbox: BoundingBox,
    pub row_span: u32,
    pub col_span: u32,
    pub start_row: u32,
    pub end_row: u32,
    pub start_col: u32,
    pub end_col: u32,
    pub column_header: bool,
    pub row_header: bool,
    pub row_section: bool,
}

// ── Page ────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Page {
    pub page_no: u32,
    pub size: Option<PageSize>,
    pub cells: Vec<OcrCell>,
    pub predictions: PagePredictions,
    pub parsed_cells: Vec<OcrCell>,
}

impl Page {
    pub fn new(page_no: u32) -> Self {
        Self {
            page_no,
            size: None,
            cells: vec![],
            predictions: PagePredictions::default(),
            parsed_cells: vec![],
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PagePredictions {
    pub layout: Option<LayoutPrediction>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LayoutPrediction {
    pub clusters: Vec<LayoutCluster>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LayoutCluster {
    pub id: u32,
    pub label: LayoutLabel,
    pub confidence: f32,
    pub bbox: BoundingBox,
    pub cells: Vec<OcrCell>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LayoutLabel {
    Text,
    Title,
    SectionHeader,
    Caption,
    Footnote,
    Formula,
    ListItem,
    PageHeader,
    PageFooter,
    Table,
    Figure,
    Picture,
    Code,
    Form,
    KeyValueRegion,
    Document,
    Unknown,
}

// ── Document limits ─────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentLimits {
    pub max_num_pages: usize,
    pub max_file_size: usize,
    pub page_range: (usize, usize),
}

impl Default for DocumentLimits {
    fn default() -> Self {
        Self {
            max_num_pages: usize::MAX,
            max_file_size: usize::MAX,
            page_range: (1, usize::MAX),
        }
    }
}

// ── DocumentStream ─────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct DocumentStream {
    pub name: String,
    pub data: Vec<u8>,
    pub mime_type: Option<String>,
}

impl DocumentStream {
    pub fn new(name: impl Into<String>, data: Vec<u8>) -> Self {
        Self {
            name: name.into(),
            data,
            mime_type: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct Formatting {
    pub bold: bool,
    pub italic: bool,
    pub strikethrough: bool,
    pub underline: bool,
}

impl Formatting {
    pub fn is_empty(&self) -> bool {
        !self.bold && !self.italic && !self.strikethrough && !self.underline
    }
}

// ── Profiling ───────────────────────────────────────────────────

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Timings(pub HashMap<String, f64>);

impl Timings {
    pub fn record(&mut self, label: impl Into<String>, secs: f64) {
        self.0.insert(label.into(), secs);
    }
    pub fn get(&self, label: &str) -> Option<f64> {
        self.0.get(label).copied()
    }
}
