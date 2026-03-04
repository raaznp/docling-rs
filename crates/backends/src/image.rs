use docling_core::{
    base_models::InputFormat,
    doc_types::{DoclingDocument, DocumentOrigin, PictureItem},
    errors::{DoclingError, Result},
    LayoutLabel,
};

use crate::{BackendSource, DeclarativeBackend, DocumentBackend, PageData, PaginatedBackend};

/// Image document backend.
///
/// Treats a single image file as a one-page document. The image is available
/// for downstream OCR and layout analysis stages.
/// Mirrors `docling/backend/image_backend.py`.
pub struct ImageBackend {
    source: BackendSource,
    valid: bool,
}

impl ImageBackend {
    pub fn new(source: BackendSource) -> Self {
        Self {
            source,
            valid: true,
        }
    }
}

impl DocumentBackend for ImageBackend {
    fn is_valid(&self) -> bool {
        self.valid
    }

    fn supported_formats() -> &'static [InputFormat] {
        &[InputFormat::Image]
    }

    fn unload(&mut self) {
        self.valid = false;
    }
}

impl PaginatedBackend for ImageBackend {
    fn page_count(&self) -> usize {
        1
    }

    fn load_page(&self, page_no: usize) -> Result<PageData> {
        if page_no != 1 {
            return Err(DoclingError::backend(format!(
                "ImageBackend only has 1 page, requested {}",
                page_no
            )));
        }

        let bytes = self.source.read_bytes()?;
        let img = image::load_from_memory(&bytes)
            .map_err(|e| DoclingError::backend(format!("Image load error: {}", e)))?;

        let width = img.width();
        let height = img.height();

        // Encode back to PNG bytes for downstream use
        let mut png_bytes: Vec<u8> = Vec::new();
        img.write_to(
            &mut std::io::Cursor::new(&mut png_bytes),
            image::ImageFormat::Png,
        )
        .map_err(|e| DoclingError::backend(format!("Image encode error: {}", e)))?;

        Ok(PageData {
            page_no: 1,
            width: width as f64,
            height: height as f64,
            text_cells: vec![],
            image: Some(png_bytes),
            image_width: width,
            image_height: height,
        })
    }
}

impl DeclarativeBackend for ImageBackend {
    fn convert(&mut self) -> Result<DoclingDocument> {
        let name = self.source.name().to_string();
        let mut doc = DoclingDocument::new(&name);
        doc.origin = Some(DocumentOrigin {
            filename: name.clone(),
            mime_type: "image/*".to_string(),
            binary_hash: None,
            uri: None,
        });

        // The image itself becomes a single picture item
        // Actual OCR/layout happens in the pipeline, not the backend
        doc.add_picture(PictureItem {
            id: "#/pictures/0".to_string(),
            label: LayoutLabel::Picture,
            prov: vec![],
            captions: None,
            description: None,
            image_data: None, // set by pipeline after OCR
            classification: None,
        });

        Ok(doc)
    }
}
