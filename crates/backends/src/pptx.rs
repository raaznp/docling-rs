use docling_core::{
    base_models::InputFormat,
    doc_types::{DoclingDocument, DocumentOrigin, PictureItem, SectionHeaderItem, TextItem},
    errors::{DoclingError, Result},
    LayoutLabel,
};
use quick_xml::events::Event;
use quick_xml::Reader;

use crate::{BackendSource, DeclarativeBackend, DocumentBackend};

/// Microsoft PowerPoint (PPTX) backend.
///
/// Parses PPTX files by iterating slides from the ZIP archive and
/// extracting text from shapes using quick-xml.
/// Mirrors `docling/backend/mspowerpoint_backend.py`.
#[cfg(feature = "office")]
pub struct PptxBackend {
    source: BackendSource,
    valid: bool,
}

#[cfg(feature = "office")]
impl PptxBackend {
    pub fn new(source: BackendSource) -> Self {
        Self {
            source,
            valid: true,
        }
    }

    fn parse_pptx(&self, data: &[u8], name: &str) -> Result<DoclingDocument> {
        use std::io::{Cursor, Read};
        use zip::ZipArchive;

        let cursor = Cursor::new(data);
        let mut archive = ZipArchive::new(cursor)
            .map_err(|e| DoclingError::backend(format!("ZIP error: {}", e)))?;

        let mut doc = DoclingDocument::new(name);
        doc.origin = Some(DocumentOrigin {
            filename: name.to_string(),
            mime_type: "application/vnd.openxmlformats-officedocument.presentationml.presentation"
                .to_string(),
            binary_hash: None,
            uri: None,
        });

        // Collect all slide XML file paths
        let slide_paths: Vec<String> = (0..archive.len())
            .filter_map(|i| {
                let entry = archive.by_index(i).ok()?;
                let path = entry.name().to_string();
                if path.starts_with("ppt/slides/slide") && path.ends_with(".xml") {
                    Some(path)
                } else {
                    None
                }
            })
            .collect();

        let mut slide_count = 0u32;

        for slide_path in &slide_paths {
            let xml = {
                let mut entry = archive.by_name(slide_path).map_err(|e| {
                    DoclingError::backend(format!("Entry {} not found: {}", slide_path, e))
                })?;
                let mut s = String::new();
                entry
                    .read_to_string(&mut s)
                    .map_err(|e| DoclingError::IoError { source: e })?;
                s
            };

            slide_count += 1;

            // Add slide heading
            let id = format!("#/texts/{}", doc.body.len());
            doc.add_header(SectionHeaderItem {
                id,
                text: format!("Slide {}", slide_count),
                level: 2,
                label: LayoutLabel::SectionHeader,
                prov: vec![],
            });

            // Extract text from all <a:t> elements in the slide
            let slide_texts = extract_slide_texts(&xml)?;

            for text in slide_texts {
                if !text.trim().is_empty() {
                    let id = format!("#/texts/{}", doc.body.len());
                    doc.add_text(TextItem {
                        id,
                        text: text.trim().to_string(),
                        label: LayoutLabel::Text,
                        prov: vec![],
                        orig: None,
                        enumerated: None,
                        marker: None,
                    });
                }
            }

            // Count images / pictures on slide (a:blip references)
            let pic_count = count_images_in_xml(&xml);
            for _ in 0..pic_count {
                let id = format!("#/pictures/{}", doc.body.len());
                doc.add_picture(PictureItem {
                    id,
                    label: LayoutLabel::Picture,
                    prov: vec![],
                    captions: None,
                    description: None,
                    image_data: None,
                    classification: None,
                });
            }
        }

        Ok(doc)
    }
}

/// Extract all text runs (<a:t>) from a slide XML.
fn extract_slide_texts(xml: &str) -> Result<Vec<String>> {
    let mut reader = Reader::from_str(xml);
    reader.config_mut().trim_text(true);

    let mut buf = Vec::new();
    let mut texts: Vec<String> = Vec::new();
    let mut in_text = false;

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(ref e)) => {
                let tag = String::from_utf8_lossy(e.name().as_ref()).to_string();
                if tag.ends_with(":t") || tag == "t" {
                    in_text = true;
                }
            }
            Ok(Event::End(ref e)) => {
                let tag = String::from_utf8_lossy(e.name().as_ref()).to_string();
                if tag.ends_with(":t") || tag == "t" {
                    in_text = false;
                }
            }
            Ok(Event::Text(ref e)) if in_text => {
                if let Ok(text) = e.unescape() {
                    texts.push(text.to_string());
                }
            }
            Ok(Event::Eof) => break,
            Err(e) => {
                return Err(DoclingError::backend(format!(
                    "PPTX XML parse error: {}",
                    e
                )))
            }
            _ => {}
        }
        buf.clear();
    }

    Ok(texts)
}

/// Count `<a:blip` references in slide XML (images).
fn count_images_in_xml(xml: &str) -> usize {
    xml.matches("<a:blip ").count() + xml.matches("<p:blipFill").count()
}

#[cfg(feature = "office")]
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

#[cfg(feature = "office")]
impl DeclarativeBackend for PptxBackend {
    fn convert(&mut self) -> Result<DoclingDocument> {
        let bytes = self.source.read_bytes()?;
        let name = self.source.name().to_string();
        self.parse_pptx(&bytes, &name)
    }
}
