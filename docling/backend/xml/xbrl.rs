use crate::backend::{BackendSource, DeclarativeBackend, DocumentBackend};
use crate::datamodel::base_models::{InputFormat, LayoutLabel};
use crate::datamodel::document::{DoclingDocument, DocumentOrigin, TextItem};
use crate::errors::{DoclingError, Result};
use quick_xml::{events::Event, Reader};

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
        let xml_str = String::from_utf8_lossy(&bytes).to_string();
        let name = self.source.name().to_string();
        let mut doc = DoclingDocument::new(&name);
        doc.origin = Some(DocumentOrigin {
            filename: name,
            mime_type: "application/xml".into(),
            binary_hash: None,
            uri: None,
        });

        let mut reader = Reader::from_str(&xml_str);
        let mut buf = Vec::new();
        let mut in_fact = false;
        let mut idx = 0usize;

        loop {
            match reader.read_event_into(&mut buf) {
                Ok(Event::Start(ref e)) => {
                    let name_bytes = e.name();
                    let tag_name = std::str::from_utf8(name_bytes.as_ref()).unwrap_or("");
                    in_fact = !tag_name.starts_with("xbrli:") && !tag_name.starts_with("link:");
                }
                Ok(Event::Text(ref e)) if in_fact => {
                    let text = e.unescape().unwrap_or_default().trim().to_string();
                    if !text.is_empty() {
                        doc.add_text(TextItem {
                            id: format!("#/texts/{}", idx),
                            text: text.to_string(),
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
