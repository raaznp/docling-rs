use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

// ============================================================
// Input Format
// ============================================================

/// Supported input document formats.
/// Mirrors Python `InputFormat` enum in `docling/datamodel/base_models.py`.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum InputFormat {
    Pdf,
    Docx,
    Pptx,
    Xlsx,
    Html,
    Md,
    Asciidoc,
    Csv,
    Image,
    Latex,
    XmlJats,
    XmlUspto,
    XmlXbrl,
    MetsGbs,
    JsonDocling,
    Audio,
    Vtt,
}

impl InputFormat {
    /// Returns the canonical file extensions for this format.
    pub fn extensions(&self) -> &[&str] {
        match self {
            InputFormat::Pdf => &["pdf"],
            InputFormat::Docx => &["docx"],
            InputFormat::Pptx => &["pptx"],
            InputFormat::Xlsx => &["xlsx"],
            InputFormat::Html => &["html", "htm", "xhtml"],
            InputFormat::Md => &["md", "markdown"],
            InputFormat::Asciidoc => &["adoc", "asciidoc", "asc"],
            InputFormat::Csv => &["csv"],
            InputFormat::Image => &["png", "jpg", "jpeg", "tiff", "bmp", "gif", "webp"],
            InputFormat::Latex => &["tex", "latex"],
            InputFormat::XmlJats => &["jats", "xml"],
            InputFormat::XmlUspto => &["xml"],
            InputFormat::XmlXbrl => &["xbrl", "xml"],
            InputFormat::MetsGbs => &["xml"],
            InputFormat::JsonDocling => &["json"],
            InputFormat::Audio => &["mp3", "wav", "ogg", "flac", "m4a"],
            InputFormat::Vtt => &["vtt"],
        }
    }

    /// Attempt to detect format from file extension.
    pub fn from_extension(ext: &str) -> Option<Self> {
        let ext = ext.to_lowercase();
        match ext.as_str() {
            "pdf" => Some(InputFormat::Pdf),
            "docx" => Some(InputFormat::Docx),
            "pptx" => Some(InputFormat::Pptx),
            "xlsx" => Some(InputFormat::Xlsx),
            "html" | "htm" | "xhtml" => Some(InputFormat::Html),
            "md" | "markdown" => Some(InputFormat::Md),
            "adoc" | "asciidoc" | "asc" => Some(InputFormat::Asciidoc),
            "csv" => Some(InputFormat::Csv),
            "png" | "jpg" | "jpeg" | "tiff" | "tif" | "bmp" | "gif" | "webp" => {
                Some(InputFormat::Image)
            }
            "tex" | "latex" => Some(InputFormat::Latex),
            "vtt" => Some(InputFormat::Vtt),
            "mp3" | "wav" | "ogg" | "flac" | "m4a" => Some(InputFormat::Audio),
            _ => None,
        }
    }
}

impl std::fmt::Display for InputFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

// ============================================================
// Conversion Status
// ============================================================

/// Status of a document conversion.
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

// ============================================================
// Component types (for error attribution)
// ============================================================

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum DoclingComponentType {
    Pipeline,
    DocumentBackend,
    Model,
    UserInput,
    Unknown,
}

// ============================================================
// Error item
// ============================================================

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

// ============================================================
// Geometry / Bounding Box
// ============================================================

/// A 2D bounding box using the coordinate system: (l, t, r, b) from top-left origin.
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

    pub fn center_x(&self) -> f64 {
        (self.l + self.r) / 2.0
    }

    pub fn center_y(&self) -> f64 {
        (self.t + self.b) / 2.0
    }

    /// Check if this bbox overlaps with another.
    pub fn overlaps(&self, other: &BoundingBox) -> bool {
        self.l < other.r && self.r > other.l && self.t < other.b && self.b > other.t
    }

    /// Compute intersection-over-union (IoU).
    pub fn iou(&self, other: &BoundingBox) -> f64 {
        let inter_l = self.l.max(other.l);
        let inter_t = self.t.max(other.t);
        let inter_r = self.r.min(other.r);
        let inter_b = self.b.min(other.b);

        if inter_r <= inter_l || inter_b <= inter_t {
            return 0.0;
        }

        let inter_area = (inter_r - inter_l) * (inter_b - inter_t);
        let union_area = self.area() + other.area() - inter_area;
        if union_area == 0.0 {
            0.0
        } else {
            inter_area / union_area
        }
    }

    /// Normalize coordinates to [0, 1] given page dimensions.
    pub fn normalized(&self, page_width: f64, page_height: f64) -> Self {
        Self {
            l: self.l / page_width,
            t: self.t / page_height,
            r: self.r / page_width,
            b: self.b / page_height,
        }
    }
}

// ============================================================
// Page size / dimensions
// ============================================================

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PageSize {
    pub width: f64,
    pub height: f64,
}

/// Alias for clarity
pub type Size = PageSize;

// ============================================================
// OCR Cell
// ============================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OcrCell {
    pub id: u32,
    pub text: String,
    pub confidence: f32,
    pub bbox: BoundingBox,
    pub from_ocr: bool,
}

/// A table cell.
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

// ============================================================
// Provenance
// ============================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProvenanceItem {
    pub page_no: u32,
    pub bbox: BoundingBox,
    pub charspan: [usize; 2],
}

// ============================================================
// Page
// ============================================================

/// Represents a single page during processing.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Page {
    pub page_no: u32,
    pub size: Option<PageSize>,

    /// Raw OCR cells found on this page
    pub cells: Vec<OcrCell>,

    /// Layout predictions on this page
    pub predictions: PagePredictions,

    /// Parsed text blocks (from PDF native text layer)
    pub parsed_cells: Vec<OcrCell>,
}

impl Page {
    pub fn new(page_no: u32) -> Self {
        Self {
            page_no,
            size: None,
            cells: Vec::new(),
            predictions: PagePredictions::default(),
            parsed_cells: Vec::new(),
        }
    }
}

/// ML layout predictions for a single page.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PagePredictions {
    pub layout: Option<LayoutPrediction>,
}

/// Output from the layout detection model.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LayoutPrediction {
    pub clusters: Vec<LayoutCluster>,
}

/// A single detected layout cluster (e.g. paragraph, table, figure).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LayoutCluster {
    pub id: u32,
    pub label: LayoutLabel,
    pub confidence: f32,
    pub bbox: BoundingBox,
    pub cells: Vec<OcrCell>,
}

/// Layout label categories (mirrors docling-ibm-models taxonomy).
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

// ============================================================
// Document limits
// ============================================================

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

// ============================================================
// Document stream (in-memory input)
// ============================================================

/// An in-memory document source (name + bytes).
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

    pub fn with_mime(mut self, mime: impl Into<String>) -> Self {
        self.mime_type = Some(mime.into());
        self
    }
}

// ============================================================
// Profiling timings
// ============================================================

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Timings(pub HashMap<String, f64>);

impl Timings {
    pub fn record(&mut self, label: impl Into<String>, elapsed_secs: f64) {
        self.0.insert(label.into(), elapsed_secs);
    }

    pub fn get(&self, label: &str) -> Option<f64> {
        self.0.get(label).copied()
    }
}
