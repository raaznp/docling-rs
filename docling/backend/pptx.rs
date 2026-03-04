use crate::backend::{BackendSource, DeclarativeBackend, DocumentBackend};
use crate::datamodel::base_models::{InputFormat, LayoutLabel};
use crate::datamodel::document::{DoclingDocument, DocumentOrigin, SectionHeaderItem, TextItem};
use crate::errors::{DoclingError, Result};

pub struct PptxBackend {
    source: BackendSource,
    valid: bool,
}
impl PptxBackend {
    pub fn new(source: BackendSource) -> Self {
        Self {
            source,
            valid: true,
        }
    }
}

impl DocumentBackend for PptxBackend {
    fn is_valid(&self) -> bool {
        self.valid
    }
    fn supported_formats() -> &'static [InputFormat] {
        &[InputFormat::Pptx]
    }
    fn unload(&mut self) {
        self.valid = false;
    }
}

impl DeclarativeBackend for PptxBackend {
    fn convert(&mut self) -> Result<DoclingDocument> {
        use std::io::Cursor;
        let bytes = self.source.read_bytes()?;
        let name = self.source.name().to_string();
        let mut doc = DoclingDocument::new(&name);
        doc.origin = Some(DocumentOrigin {
            filename: name,
            mime_type: "application/vnd.openxmlformats-officedocument.presentationml.presentation"
                .into(),
            binary_hash: None,
            uri: None,
        });

        let cursor = Cursor::new(bytes);
        let mut archive = zip::ZipArchive::new(cursor)
            .map_err(|e| DoclingError::backend(format!("ZIP error: {}", e)))?;

        let mut slide_names: Vec<String> = (0..archive.len())
            .filter_map(|i| archive.by_index(i).ok().map(|f| f.name().to_string()))
            .filter(|n| n.starts_with("ppt/slides/slide") && n.ends_with(".xml"))
            .collect();
        slide_names.sort();

        let mut idx = 0usize;
        for slide_name in slide_names {
            let mut slide_file = archive
                .by_name(&slide_name)
                .map_err(|e| DoclingError::backend(format!("Slide error: {}", e)))?;
            let mut content = String::new();
            use std::io::Read;
            slide_file.read_to_string(&mut content).ok();

            // Simple text extraction from <a:t> tags
            let slide_num = idx + 1;
            doc.add_header(SectionHeaderItem {
                id: format!("#/texts/{}", idx),
                text: format!("Slide {}", slide_num),
                level: 2,
                label: LayoutLabel::SectionHeader,
                prov: vec![],
                formatting: None,
                hyperlink: None,
                annotations: vec![],
            });
            idx += 1;

            let mut pos = 0;
            while let Some(start) = content[pos..].find("<a:t>") {
                let abs_start = pos + start + 5;
                if let Some(end) = content[abs_start..].find("</a:t>") {
                    let text = content[abs_start..abs_start + end].trim().to_string();
                    if !text.is_empty() {
                        doc.add_text(TextItem {
                            id: format!("#/texts/{}", idx),
                            text,
                            label: LayoutLabel::Text,
                            prov: vec![],
                            orig: None,
                            enumerated: None,
                            marker: None,
                            formatting: None,
                            hyperlink: None,
                            annotations: vec![],
                        });
                        idx += 1;
                    }
                    pos = abs_start + end + 6;
                } else {
                    break;
                }
            }
        }
        Ok(doc)
    }
}
