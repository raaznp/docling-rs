use docling_core::{
    base_models::InputFormat,
    doc_types::{DoclingDocument, DocumentOrigin, KeyValueItem, TextItem},
    errors::{DoclingError, Result},
    LayoutLabel,
};
use quick_xml::events::Event;
use quick_xml::Reader;

use crate::{BackendSource, DeclarativeBackend, DocumentBackend};

/// XBRL (eXtensible Business Reporting Language) backend.
/// Parses XBRL documents into DoclingDocument key-value pairs.
/// Mirrors `docling/backend/xml/xbrl_backend.py`.
pub struct XbrlBackend {
    source: BackendSource,
    valid: bool,
}

impl XbrlBackend {
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
            mime_type: "application/xbrl+xml".to_string(),
            binary_hash: None,
            uri: None,
        });

        let mut reader = Reader::from_str(xml);
        reader.config_mut().trim_text(true);

        let mut buf = Vec::new();
        let mut current_tag = String::new();
        let mut current_text = String::new();

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
                Ok(Event::End(_)) => {
                    let text = current_text.trim().to_string();
                    if !text.is_empty() && !current_tag.is_empty() {
                        // XBRL facts are key → value pairs
                        let id = format!("#/texts/{}", doc.body.len());
                        doc.body.push(docling_core::DocItem::KeyValue(KeyValueItem {
                            id,
                            label: LayoutLabel::KeyValueRegion,
                            prov: vec![],
                            key: current_tag.clone(),
                            value: text,
                        }));
                    }
                    current_text.clear();
                }
                Ok(Event::Eof) => break,
                Err(e) => return Err(DoclingError::backend(format!("XBRL parse error: {}", e))),
                _ => {}
            }
            buf.clear();
        }

        Ok(doc)
    }
}

impl DocumentBackend for XbrlBackend {
    fn is_valid(&self) -> bool {
        self.valid
    }
    fn supported_formats() -> &'static [InputFormat] {
        &[InputFormat::XmlXbrl]
    }
    fn unload(&mut self) {
        self.valid = false;
    }
}

impl DeclarativeBackend for XbrlBackend {
    fn convert(&mut self) -> Result<DoclingDocument> {
        let bytes = self.source.read_bytes()?;
        let xml = String::from_utf8_lossy(&bytes).into_owned();
        let name = self.source.name().to_string();
        self.parse_xml(&xml, &name)
    }
}
