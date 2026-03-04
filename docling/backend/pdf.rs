use crate::backend::{
    BackendSource, DeclarativeBackend, DocumentBackend, PageData, PaginatedBackend,
};
use crate::datamodel::base_models::{BoundingBox, InputFormat, OcrCell};
use crate::datamodel::document::{DoclingDocument, DocumentOrigin};
use crate::errors::{DoclingError, Result};
use lopdf::Document;

pub struct PdfBackend {
    source: BackendSource,
    valid: bool,
    doc: Option<Document>,
}

impl PdfBackend {
    pub fn new(source: BackendSource) -> Result<Self> {
        let bytes = source.read_bytes()?;
        let doc = Document::load_mem(&bytes).ok();
        let valid = doc.is_some();
        Ok(Self { source, valid, doc })
    }
}

impl DocumentBackend for PdfBackend {
    fn is_valid(&self) -> bool {
        self.valid
    }
    fn supported_formats() -> &'static [InputFormat] {
        &[InputFormat::Pdf]
    }
    fn unload(&mut self) {
        self.doc = None;
        self.valid = false;
    }
}

impl PaginatedBackend for PdfBackend {
    fn page_count(&self) -> usize {
        self.doc.as_ref().map(|d| d.get_pages().len()).unwrap_or(0)
    }

    fn load_page(&self, page_no: usize) -> Result<PageData> {
        let doc = self
            .doc
            .as_ref()
            .ok_or_else(|| DoclingError::backend("PDF not loaded"))?;
        let pages = doc.get_pages();

        // page_no is 1-indexed
        let page_id = pages
            .get(&(page_no as u32))
            .ok_or_else(|| DoclingError::backend(format!("Page {} not found", page_no)))?;

        // Extract native text layer
        let text = doc.extract_text(&[page_no as u32]).unwrap_or_default();
        let cells: Vec<OcrCell> = text
            .lines()
            .enumerate()
            .map(|(i, line)| OcrCell {
                id: i as u32,
                text: line.to_string(),
                confidence: 1.0,
                bbox: BoundingBox::new(0.0, i as f64 * 12.0, 595.0, (i + 1) as f64 * 12.0),
                from_ocr: false,
            })
            .collect();

        Ok(PageData {
            page_no,
            width: 595.0, // A4 default until lopdf exposes page dimensions
            height: 842.0,
            text_cells: cells
                .iter()
                .map(|c| crate::backend::NativeTextCell {
                    text: c.text.clone(),
                    l: c.bbox.l,
                    t: c.bbox.t,
                    r: c.bbox.r,
                    b: c.bbox.b,
                    font_size: 12.0,
                    bold: false,
                    italic: false,
                })
                .collect(),
            image: None,
            image_width: 0,
            image_height: 0,
        })
    }
}
