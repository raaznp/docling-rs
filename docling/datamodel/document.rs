use serde::{Deserialize, Serialize};
use std::path::PathBuf;

use crate::datamodel::base_models::{
    BoundingBox, Cell, ConversionStatus, DocumentLimits, DocumentStream, ErrorItem, Formatting,
    InputFormat, LayoutLabel, OcrCell, Page, PageSize, Timings,
};

// ── DoclingDocument ─────────────────────────────────────────────

/// Types of document body items.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum DocItem {
    Text(TextItem),
    SectionHeader(SectionHeaderItem),
    ListItem(ListItem),
    Table(TableItem),
    Picture(PictureItem),
    Code(CodeItem),
    Formula(FormulaItem),
    KeyValue(KeyValueItem),
    Reference(RefItem),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Annotation {
    pub start: usize,
    pub end: usize,
    pub kind: AnnotationKind,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AnnotationKind {
    Bold,
    Italic,
    Strikethrough,
    Underline,
    Code,
    Link { url: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextItem {
    pub id: String,
    pub text: String,
    pub label: crate::datamodel::base_models::LayoutLabel,
    pub prov: Vec<ProvenanceRef>,
    pub orig: Option<String>,
    pub enumerated: Option<bool>,
    pub marker: Option<String>,
    pub formatting: Option<Formatting>,
    pub hyperlink: Option<String>,
    pub annotations: Vec<Annotation>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SectionHeaderItem {
    pub id: String,
    pub text: String,
    pub level: u32,
    pub label: crate::datamodel::base_models::LayoutLabel,
    pub prov: Vec<ProvenanceRef>,
    pub formatting: Option<Formatting>,
    pub hyperlink: Option<String>,
    pub annotations: Vec<Annotation>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListItem {
    pub id: String,
    pub text: String,
    pub level: u32,
    pub label: crate::datamodel::base_models::LayoutLabel,
    pub prov: Vec<ProvenanceRef>,
    pub enumerated: Option<bool>,
    pub marker: Option<String>,
    pub formatting: Option<Formatting>,
    pub hyperlink: Option<String>,
    pub annotations: Vec<Annotation>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TableItem {
    pub id: String,
    pub label: crate::datamodel::base_models::LayoutLabel,
    pub prov: Vec<ProvenanceRef>,
    pub data: TableData,
    pub captions: Option<Vec<RefItem>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TableData {
    pub num_rows: u32,
    pub num_cols: u32,
    pub table_cells: Vec<crate::datamodel::base_models::Cell>,
    pub grid: Option<Vec<Vec<usize>>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PictureItem {
    pub id: String,
    pub label: crate::datamodel::base_models::LayoutLabel,
    pub prov: Vec<ProvenanceRef>,
    pub captions: Option<Vec<RefItem>>,
    pub description: Option<String>,
    pub image_data: Option<Vec<u8>>,
    pub classification: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeItem {
    pub id: String,
    pub text: String,
    pub label: crate::datamodel::base_models::LayoutLabel,
    pub prov: Vec<ProvenanceRef>,
    pub code_language: Option<String>,
    pub formatting: Option<Formatting>,
    pub hyperlink: Option<String>,
    pub annotations: Vec<Annotation>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FormulaItem {
    pub id: String,
    pub text: String,
    pub label: crate::datamodel::base_models::LayoutLabel,
    pub prov: Vec<ProvenanceRef>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyValueItem {
    pub id: String,
    pub key: String,
    pub value: String,
    pub prov: Vec<ProvenanceRef>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RefItem {
    pub ref_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProvenanceRef {
    pub page_no: u32,
    pub bbox: crate::datamodel::base_models::BoundingBox,
    pub charspan: [usize; 2],
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PageRef {
    pub page_no: u32,
    pub size: crate::datamodel::base_models::PageSize,
    pub image: Option<Vec<u8>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentMetadata {
    pub title: Option<String>,
    pub author: Option<String>,
    pub language: Option<String>,
    pub description: Option<String>,
    pub created: Option<String>,
    pub modified: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentOrigin {
    pub filename: String,
    pub mime_type: String,
    pub binary_hash: Option<String>,
    pub uri: Option<String>,
}

/// The central document representation — equivalent to Python's `DoclingDocument`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DoclingDocument {
    pub name: String,
    pub origin: Option<DocumentOrigin>,
    pub metadata: Option<DocumentMetadata>,
    pub body: Vec<DocItem>,
    pub pages: std::collections::HashMap<u32, PageRef>,
}

impl DoclingDocument {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            origin: None,
            metadata: None,
            body: Vec::new(),
            pages: std::collections::HashMap::new(),
        }
    }

    pub fn add_text(&mut self, item: TextItem) {
        self.body.push(DocItem::Text(item));
    }
    pub fn add_header(&mut self, item: SectionHeaderItem) {
        self.body.push(DocItem::SectionHeader(item));
    }
    pub fn add_list_item(&mut self, item: ListItem) {
        self.body.push(DocItem::ListItem(item));
    }
    pub fn add_table(&mut self, item: TableItem) {
        self.body.push(DocItem::Table(item));
    }
    pub fn add_picture(&mut self, item: PictureItem) {
        self.body.push(DocItem::Picture(item));
    }
    pub fn add_code(&mut self, item: CodeItem) {
        self.body.push(DocItem::Code(item));
    }
    pub fn add_formula(&mut self, item: FormulaItem) {
        self.body.push(DocItem::Formula(item));
    }

    /// Export to Markdown (`.export_to_markdown()` from architecture diagram).
    pub fn export_to_markdown(&self) -> String {
        crate::utils::export::to_markdown(self)
    }

    /// Export to dict/JSON (`.export_to_dict()` from architecture diagram).
    pub fn export_to_dict(&self) -> serde_json::Value {
        serde_json::to_value(self).unwrap_or(serde_json::Value::Null)
    }

    /// Export to DocTags (`.export_to_document_tokens()` from architecture diagram).
    pub fn export_to_document_tokens(&self) -> String {
        crate::utils::export::to_doctags(self)
    }

    /// Export to plain text.
    pub fn export_to_text(&self) -> String {
        crate::utils::export::to_text(self)
    }

    /// Export to HTML.
    pub fn export_to_html(&self) -> String {
        crate::utils::export::to_html(self)
    }
}

// ── InputDocument ───────────────────────────────────────────────

#[derive(Debug)]
pub struct InputDocument {
    pub file: PathBuf,
    pub format: InputFormat,
    pub data: Vec<u8>,
    pub document_hash: String,
    pub limits: DocumentLimits,
    pub filesize: usize,
    pub page_count: usize,
    pub valid: bool,
}

impl InputDocument {
    pub fn from_path(
        path: PathBuf,
        format: InputFormat,
        limits: DocumentLimits,
    ) -> crate::errors::Result<Self> {
        use crate::errors::DoclingError;
        use std::io::Read;

        let mut f = std::fs::File::open(&path).map_err(|e| DoclingError::IoError { source: e })?;
        let mut data = Vec::new();
        f.read_to_end(&mut data)
            .map_err(|e| DoclingError::IoError { source: e })?;

        let filesize = data.len();
        if filesize > limits.max_file_size {
            return Err(DoclingError::invalid_doc(format!(
                "File size {} exceeds limit {}",
                filesize, limits.max_file_size
            )));
        }

        Ok(Self {
            file: path,
            format,
            data,
            document_hash: String::new(),
            limits,
            filesize,
            page_count: 0,
            valid: true,
        })
    }

    pub fn empty_failure() -> Self {
        Self {
            file: PathBuf::from("__error__"),
            format: InputFormat::Pdf,
            data: vec![],
            document_hash: String::new(),
            limits: DocumentLimits::default(),
            filesize: 0,
            page_count: 0,
            valid: false,
        }
    }
}

// ── ConversionResult ────────────────────────────────────────────

#[derive(Debug)]
pub struct ConversionResult {
    pub input: InputDocument,
    pub status: ConversionStatus,
    pub pages: Vec<Page>,
    pub document: Option<DoclingDocument>,
    pub errors: Vec<ErrorItem>,
    pub timings: Timings,
}

impl ConversionResult {
    pub fn new(input: InputDocument) -> Self {
        Self {
            input,
            status: ConversionStatus::Pending,
            pages: Vec::new(),
            document: None,
            errors: Vec::new(),
            timings: Timings::default(),
        }
    }

    pub fn empty_failure() -> Self {
        Self::new(InputDocument::empty_failure())
    }

    pub fn output(&self) -> &DoclingDocument {
        self.document.as_ref().expect("Document not yet produced")
    }

    pub fn is_success(&self) -> bool {
        matches!(
            self.status,
            ConversionStatus::Success | ConversionStatus::PartialSuccess
        )
    }
}
