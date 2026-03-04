use docling_core::{
    base_models::InputFormat,
    doc_types::{DoclingDocument, DocumentOrigin, SectionHeaderItem, TextItem},
    errors::{DoclingError, Result},
    LayoutLabel,
};
use quick_xml::events::Event;
use quick_xml::Reader;

use crate::{BackendSource, DeclarativeBackend, DocumentBackend};

/// JATS XML backend.
/// Parses JATS (Journal Article Tag Suite) XML into DoclingDocument.
/// Mirrors `docling/backend/xml/jats_backend.py`.
pub struct JatsBackend {
    source: BackendSource,
    valid: bool,
}

impl JatsBackend {
    pub fn new(source: BackendSource) -> Self {
        Self {
            source,
            valid: true,
        }
    }

    fn parse_xml(&self, xml: &str, name: &str) -> Result<DoclingDocument> {
        let mut doc = DoclingDocument::new(name);
        doc.origin = Some(DocumentOrigin {
            filename: name.to_string(),
            mime_type: "application/xml".to_string(),
            binary_hash: None,
            uri: None,
        });

        let mut reader = Reader::from_str(xml);
        reader.config_mut().trim_text(true);

        let mut buf = Vec::new();
        let mut current_tag = String::new();
        let mut current_text = String::new();
        let mut heading_level = 1u8;

        loop {
            match reader.read_event_into(&mut buf) {
                Ok(Event::Start(ref e)) => {
                    current_tag = String::from_utf8_lossy(e.name().as_ref()).to_string();
                    current_text.clear();
                }
                Ok(Event::Text(ref e)) => {
                    if let Ok(text) = e.unescape() {
                        current_text.push_str(&text);
                    }
                }
                Ok(Event::End(ref e)) => {
                    let tag = String::from_utf8_lossy(e.name().as_ref()).to_string();
                    let text = current_text.trim().to_string();

                    match tag.as_str() {
                        "article-title" | "title" => {
                            if !text.is_empty() {
                                doc.metadata.title = Some(text.clone());
                                let id = format!("#/texts/{}", doc.body.len());
                                doc.add_header(SectionHeaderItem {
                                    id,
                                    text,
                                    level: 1,
                                    label: LayoutLabel::Title,
                                    prov: vec![],
                                });
                            }
                        }
                        "sec" => {
                            heading_level = (heading_level + 1).min(6);
                        }
                        "p" | "abstract" => {
                            if !text.is_empty() {
                                let id = format!("#/texts/{}", doc.body.len());
                                doc.add_text(TextItem {
                                    id,
                                    text,
                                    label: LayoutLabel::Text,
                                    prov: vec![],
                                    orig: None,
                                    enumerated: None,
                                    marker: None,
                                });
                            }
                        }
                        _ => {}
                    }
                    current_text.clear();
                }
                Ok(Event::Eof) => break,
                Err(e) => {
                    return Err(DoclingError::backend(format!("XML parse error: {}", e)));
                }
                _ => {}
            }
            buf.clear();
        }

        Ok(doc)
    }
}

impl DocumentBackend for JatsBackend {
    fn is_valid(&self) -> bool {
        self.valid
    }
    fn supported_formats() -> &'static [InputFormat] {
        &[InputFormat::XmlJats]
    }
    fn unload(&mut self) {
        self.valid = false;
    }
}

impl DeclarativeBackend for JatsBackend {
    fn convert(&mut self) -> Result<DoclingDocument> {
        let bytes = self.source.read_bytes()?;
        let xml = String::from_utf8_lossy(&bytes).into_owned();
        let name = self.source.name().to_string();
        self.parse_xml(&xml, &name)
    }
}
