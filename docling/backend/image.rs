use crate::backend::{
    BackendSource, DeclarativeBackend, DocumentBackend, PageData, PaginatedBackend,
};
use crate::datamodel::base_models::{InputFormat, LayoutLabel};
use crate::datamodel::document::{DoclingDocument, DocumentOrigin, PictureItem};
use crate::errors::{DoclingError, Result};

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
        &[
            InputFormat::Png,
            InputFormat::Jpeg,
            InputFormat::Tiff,
            InputFormat::Bmp,
            InputFormat::Webp,
        ]
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
            return Err(DoclingError::backend("ImageBackend: only 1 page"));
        }
        let bytes = self.source.read_bytes()?;
        let img = image::load_from_memory(&bytes)
            .map_err(|e| DoclingError::backend(format!("Image load error: {}", e)))?;
        let (w, h) = (img.width(), img.height());
        let mut buf: Vec<u8> = Vec::new();
        img.write_to(&mut std::io::Cursor::new(&mut buf), image::ImageFormat::Png)
            .map_err(|e| DoclingError::backend(format!("Image encode error: {}", e)))?;
        Ok(PageData {
            page_no: 1,
            width: w as f64,
            height: h as f64,
            text_cells: vec![],
            image: Some(buf),
            image_width: w,
            image_height: h,
        })
    }
}

impl DeclarativeBackend for ImageBackend {
    fn convert(&mut self) -> Result<DoclingDocument> {
        let name = self.source.name().to_string();
        let mut doc = DoclingDocument::new(&name);
        doc.origin = Some(DocumentOrigin {
            filename: name.clone(),
            mime_type: "image/*".into(),
            binary_hash: None,
            uri: None,
        });
        doc.add_picture(PictureItem {
            id: "#/pictures/0".into(),
            label: LayoutLabel::Picture,
            prov: vec![],
            captions: None,
            description: None,
            image_data: None,
            classification: None,
        });
        Ok(doc)
    }
}
