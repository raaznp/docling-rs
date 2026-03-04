use crate::backend::{BackendSource, DeclarativeBackend, DocumentBackend};
use crate::datamodel::base_models::{InputFormat, LayoutLabel};
use crate::datamodel::document::{DoclingDocument, DocumentOrigin, SectionHeaderItem, TextItem};
use crate::errors::Result;

pub struct WebVttBackend {
    source: BackendSource,
    valid: bool,
}
impl WebVttBackend {
    pub fn new(source: BackendSource) -> Self {
        Self {
            source,
            valid: true,
        }
    }
}

impl DocumentBackend for WebVttBackend {
    fn is_valid(&self) -> bool {
        self.valid
    }
    fn supported_formats() -> &'static [InputFormat] {
        &[InputFormat::Vtt]
    }
    fn unload(&mut self) {
        self.valid = false;
    }
}

impl DeclarativeBackend for WebVttBackend {
    fn convert(&mut self) -> Result<DoclingDocument> {
        let bytes = self.source.read_bytes()?;
        let text = String::from_utf8_lossy(&bytes).to_string();
        let name = self.source.name().to_string();
        let mut doc = DoclingDocument::new(&name);
        doc.origin = Some(DocumentOrigin {
            filename: name,
            mime_type: "text/vtt".into(),
            binary_hash: None,
            uri: None,
        });

        for (i, line) in text.lines().enumerate() {
            let line = line.trim();
            if line.is_empty()
                || line == "WEBVTT"
                || line.contains("-->")
                || line.parse::<usize>().is_ok()
            {
                continue;
            }

            // Check for section headers (e.g., starting with "### ")
            if line.starts_with("### ") {
                doc.add_header(SectionHeaderItem {
                    id: format!("#/texts/{}", i),
                    text: line[4..].trim().to_string(),
                    level: 1,
                    label: LayoutLabel::SectionHeader,
                    prov: vec![],
                    formatting: None,
                    hyperlink: None,
                    annotations: vec![],
                });
                continue; // Skip adding as regular text
            }

            doc.add_text(TextItem {
                id: format!("#/texts/{}", i),
                text: line.to_string(),
                label: LayoutLabel::Text,
                prov: vec![],
                orig: None,
                enumerated: None,
                marker: None,
                formatting: None,
                hyperlink: None,
                annotations: vec![],
            });
        }
        Ok(doc)
    }
}
