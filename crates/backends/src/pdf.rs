use docling_core::{
    base_models::InputFormat,
    errors::{DoclingError, Result},
};

use crate::{BackendSource, DocumentBackend, NativeTextCell, PageData, PaginatedBackend};

/// PDF document backend using `lopdf`.
///
/// Extracts the native text layer from PDFs without rendering.
/// Page images are not available with this backend — the pipeline
/// will need a rendering backend (e.g. pdfium-render behind a feature flag
/// or an external PDF-to-image tool) for image-based OCR.
///
/// Mirrors `docling/backend/pypdfium2_backend.py` (text-layer path only).
pub struct PdfBackend {
    source: BackendSource,
    valid: bool,
}

impl PdfBackend {
    pub fn new(source: BackendSource) -> Result<Self> {
        Ok(Self {
            source,
            valid: true,
        })
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
        self.valid = false;
    }
}

impl PaginatedBackend for PdfBackend {
    fn page_count(&self) -> usize {
        let bytes = match self.source.read_bytes() {
            Ok(b) => b,
            Err(_) => return 0,
        };
        match lopdf::Document::load_mem(&bytes) {
            Ok(doc) => doc.get_pages().len(),
            Err(_) => 0,
        }
    }

    fn load_page(&self, page_no: usize) -> Result<PageData> {
        let bytes = self.source.read_bytes()?;
        let pdf = lopdf::Document::load_mem(&bytes)
            .map_err(|e| DoclingError::backend(format!("lopdf load error: {}", e)))?;

        let pages = pdf.get_pages();
        if page_no == 0 || page_no > pages.len() {
            return Err(DoclingError::backend(format!(
                "Page {} is out of range (document has {} pages)",
                page_no,
                pages.len()
            )));
        }

        // Get the page ID for this page number
        let page_id = pages
            .get(&(page_no as u32))
            .copied()
            .ok_or_else(|| DoclingError::backend(format!("Page {} not found", page_no)))?;

        // Extract text from the page
        let text = pdf.extract_text(&[page_no as u32]).unwrap_or_default();

        // Get page dimensions (MediaBox)
        let (width, height) = get_page_dimensions(&pdf, page_id);

        // Produce simple text cells from extracted text lines
        let text_cells: Vec<NativeTextCell> = text
            .lines()
            .filter(|l| !l.trim().is_empty())
            .enumerate()
            .map(|(i, line)| NativeTextCell {
                text: line.to_string(),
                l: 0.0,
                t: i as f64 * 12.0,
                r: width,
                b: i as f64 * 12.0 + 12.0,
                font_size: 12.0,
                bold: false,
                italic: false,
            })
            .collect();

        Ok(PageData {
            page_no,
            width,
            height,
            text_cells,
            image: None, // Rendering requires pdfium-render (future feature)
            image_width: width as u32,
            image_height: height as u32,
        })
    }
}

/// Extract MediaBox dimensions from a page dictionary.
fn get_page_dimensions(pdf: &lopdf::Document, page_id: lopdf::ObjectId) -> (f64, f64) {
    const DEFAULT_WIDTH: f64 = 595.0; // A4 width in points
    const DEFAULT_HEIGHT: f64 = 842.0; // A4 height in points

    let page = match pdf.get_object(page_id) {
        Ok(lopdf::Object::Dictionary(d)) => d,
        _ => return (DEFAULT_WIDTH, DEFAULT_HEIGHT),
    };

    let media_box = match page.get(b"MediaBox") {
        Ok(lopdf::Object::Array(arr)) => arr,
        _ => return (DEFAULT_WIDTH, DEFAULT_HEIGHT),
    };

    if media_box.len() < 4 {
        return (DEFAULT_WIDTH, DEFAULT_HEIGHT);
    }

    let get_f = |obj: &lopdf::Object| -> f64 {
        match obj {
            lopdf::Object::Real(f) => *f as f64,
            lopdf::Object::Integer(i) => *i as f64,
            _ => 0.0,
        }
    };

    let llx = get_f(&media_box[0]);
    let lly = get_f(&media_box[1]);
    let urx = get_f(&media_box[2]);
    let ury = get_f(&media_box[3]);

    ((urx - llx).abs(), (ury - lly).abs())
}
