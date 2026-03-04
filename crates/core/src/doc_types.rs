use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

use crate::base_models::{BoundingBox, Cell, LayoutLabel, PageSize};

// ============================================================
// DoclingDocument — the output document representation
// ============================================================
//
// Mirrors the schema from docling-core (DoclingDocument Pydantic model).
// This is what all backends ultimately produce.
// ============================================================

/// A text item within the document body.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextItem {
    #[serde(rename = "self_ref")]
    pub id: String,
    pub text: String,
    pub label: LayoutLabel,
    pub prov: Vec<ProvenanceRef>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub orig: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enumerated: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub marker: Option<String>,
}

/// A table within the document body.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TableItem {
    #[serde(rename = "self_ref")]
    pub id: String,
    pub label: LayoutLabel,
    pub prov: Vec<ProvenanceRef>,
    pub data: TableData,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub captions: Option<Vec<RefItem>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TableData {
    pub num_rows: u32,
    pub num_cols: u32,
    pub table_cells: Vec<Cell>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub grid: Option<Vec<Vec<Option<u32>>>>,
}

/// A picture/figure within the document.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PictureItem {
    #[serde(rename = "self_ref")]
    pub id: String,
    pub label: LayoutLabel,
    pub prov: Vec<ProvenanceRef>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub captions: Option<Vec<RefItem>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub image_data: Option<String>, // base64 encoded image
    #[serde(skip_serializing_if = "Option::is_none")]
    pub classification: Option<HashMap<String, f32>>,
}

/// A section heading item.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SectionHeaderItem {
    #[serde(rename = "self_ref")]
    pub id: String,
    pub text: String,
    pub level: u8,
    pub label: LayoutLabel,
    pub prov: Vec<ProvenanceRef>,
}

/// A key-value region (forms, metadata).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyValueItem {
    #[serde(rename = "self_ref")]
    pub id: String,
    pub label: LayoutLabel,
    pub prov: Vec<ProvenanceRef>,
    pub key: String,
    pub value: String,
}

/// A list item.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListItem {
    #[serde(rename = "self_ref")]
    pub id: String,
    pub text: String,
    pub label: LayoutLabel,
    pub prov: Vec<ProvenanceRef>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enumerated: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub marker: Option<String>,
}

/// A code block.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeItem {
    #[serde(rename = "self_ref")]
    pub id: String,
    pub text: String,
    pub label: LayoutLabel,
    pub prov: Vec<ProvenanceRef>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub code_language: Option<String>,
}

/// A formula (math).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FormulaItem {
    #[serde(rename = "self_ref")]
    pub id: String,
    pub text: String,
    pub label: LayoutLabel,
    pub prov: Vec<ProvenanceRef>,
}

/// Reference to another doc item by ID.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RefItem {
    #[serde(rename = "$ref")]
    pub ref_id: String,
}

/// Provenance (where in the source document this item came from).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProvenanceRef {
    pub page_no: u32,
    pub bbox: BoundingBox,
    pub charspan: [usize; 2],
}

/// Document body item — any element that can appear in the document body.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum DocItem {
    Text(TextItem),
    SectionHeader(SectionHeaderItem),
    Table(TableItem),
    Picture(PictureItem),
    ListItem(ListItem),
    Code(CodeItem),
    Formula(FormulaItem),
    KeyValue(KeyValueItem),
    Ref(RefItem),
}

impl DocItem {
    pub fn id(&self) -> &str {
        match self {
            DocItem::Text(i) => &i.id,
            DocItem::SectionHeader(i) => &i.id,
            DocItem::Table(i) => &i.id,
            DocItem::Picture(i) => &i.id,
            DocItem::ListItem(i) => &i.id,
            DocItem::Code(i) => &i.id,
            DocItem::Formula(i) => &i.id,
            DocItem::KeyValue(i) => &i.id,
            DocItem::Ref(i) => &i.ref_id,
        }
    }
}

/// A page of the source document (for provenance tracking in the output).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PageRef {
    pub page_no: u32,
    pub size: PageSize,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub image: Option<String>, // base64 encoded page image
}

/// Document metadata.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DocumentMetadata {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub authors: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subject: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub keywords: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub language: Option<String>,
    /// Catch-all for format-specific metadata.
    #[serde(flatten)]
    pub extra: IndexMap<String, Value>,
}

// ============================================================
// DoclingDocument — top-level output
// ============================================================

/// The unified output document produced by any Docling backend/pipeline.
///
/// This is the Rust equivalent of `docling_core.types.doc.DoclingDocument`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DoclingDocument {
    pub schema_name: String,
    pub version: String,
    pub name: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub origin: Option<DocumentOrigin>,

    pub metadata: DocumentMetadata,

    /// Ordered list of document body items.
    pub body: Vec<DocItem>,

    /// All pages in the source document (for coordinate reference).
    pub pages: IndexMap<u32, PageRef>,

    /// Named groups (e.g., header group, body group).
    #[serde(skip_serializing_if = "IndexMap::is_empty")]
    pub groups: IndexMap<String, Vec<RefItem>>,
}

impl DoclingDocument {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            schema_name: "DoclingDocument".to_string(),
            version: "1.0.0".to_string(),
            name: name.into(),
            origin: None,
            metadata: DocumentMetadata::default(),
            body: Vec::new(),
            pages: IndexMap::new(),
            groups: IndexMap::new(),
        }
    }

    /// Export to JSON string (pretty-printed).
    pub fn to_json_pretty(&self) -> serde_json::Result<String> {
        serde_json::to_string_pretty(self)
    }

    /// Export to Markdown-like text (simple version — full export is in `docling-utils`).
    pub fn export_to_markdown(&self) -> String {
        let mut out = String::new();
        for item in &self.body {
            match item {
                DocItem::SectionHeader(h) => {
                    let hashes = "#".repeat(h.level as usize);
                    out.push_str(&format!("{} {}\n\n", hashes, h.text));
                }
                DocItem::Text(t) => {
                    out.push_str(&t.text);
                    out.push_str("\n\n");
                }
                DocItem::ListItem(li) => {
                    let marker = li.marker.as_deref().unwrap_or("-");
                    out.push_str(&format!("{} {}\n", marker, li.text));
                }
                DocItem::Code(c) => {
                    let lang = c.code_language.as_deref().unwrap_or("");
                    out.push_str(&format!("```{}\n{}\n```\n\n", lang, c.text));
                }
                DocItem::Formula(f) => {
                    out.push_str(&format!("$${}$$\n\n", f.text));
                }
                DocItem::Table(_) => {
                    out.push_str("<!-- table -->\n\n");
                }
                DocItem::Picture(_) => {
                    out.push_str("<!-- figure -->\n\n");
                }
                _ => {}
            }
        }
        out
    }

    /// Iterate body items with a depth level (for hierarchical traversal).
    pub fn iter_items(&self) -> impl Iterator<Item = &DocItem> {
        self.body.iter()
    }

    /// Add a text item to the body.
    pub fn add_text(&mut self, item: TextItem) {
        self.body.push(DocItem::Text(item));
    }

    /// Add a section header to the body.
    pub fn add_header(&mut self, item: SectionHeaderItem) {
        self.body.push(DocItem::SectionHeader(item));
    }

    /// Add a table to the body.
    pub fn add_table(&mut self, item: TableItem) {
        self.body.push(DocItem::Table(item));
    }

    /// Add a picture to the body.
    pub fn add_picture(&mut self, item: PictureItem) {
        self.body.push(DocItem::Picture(item));
    }

    /// Add a list item to the body.
    pub fn add_list_item(&mut self, item: ListItem) {
        self.body.push(DocItem::ListItem(item));
    }

    /// Add a code block to the body.
    pub fn add_code(&mut self, item: CodeItem) {
        self.body.push(DocItem::Code(item));
    }

    pub fn add_formula(&mut self, item: FormulaItem) {
        self.body.push(DocItem::Formula(item));
    }
}

/// Where a document was loaded from.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentOrigin {
    pub filename: String,
    pub mime_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub binary_hash: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub uri: Option<String>,
}
