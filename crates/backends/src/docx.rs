use docling_core::{
    base_models::InputFormat,
    doc_types::{
        CodeItem, DoclingDocument, DocumentOrigin, ListItem, PictureItem, SectionHeaderItem,
        TableData, TableItem, TextItem,
    },
    errors::{DoclingError, Result},
    LayoutLabel,
};

use crate::{BackendSource, DeclarativeBackend, DocumentBackend};

/// Microsoft Word (DOCX) backend.
///
/// Parses DOCX files by treating them as ZIP archives containing word/document.xml,
/// using quick-xml for SAX-style parsing.
/// Mirrors `docling/backend/msword_backend.py`.
#[cfg(feature = "office")]
pub struct DocxBackend {
    source: BackendSource,
    valid: bool,
}

#[cfg(feature = "office")]
impl DocxBackend {
    pub fn new(source: BackendSource) -> Self {
        Self {
            source,
            valid: true,
        }
    }

    fn parse_docx(&self, data: &[u8], name: &str) -> Result<DoclingDocument> {
        use quick_xml::events::Event;
        use quick_xml::Reader;
        use std::io::{Cursor, Read};
        use zip::ZipArchive;

        let cursor = Cursor::new(data);
        let mut archive = ZipArchive::new(cursor)
            .map_err(|e| DoclingError::backend(format!("ZIP open error: {}", e)))?;

        // Read the main document XML
        let xml = {
            let mut entry = archive.by_name("word/document.xml").map_err(|e| {
                DoclingError::backend(format!("word/document.xml not found: {}", e))
            })?;
            let mut s = String::new();
            entry
                .read_to_string(&mut s)
                .map_err(|e| DoclingError::IoError { source: e })?;
            s
        };

        let mut doc = DoclingDocument::new(name);
        doc.origin = Some(DocumentOrigin {
            filename: name.to_string(),
            mime_type: "application/vnd.openxmlformats-officedocument.wordprocessingml.document"
                .to_string(),
            binary_hash: None,
            uri: None,
        });

        let mut reader = Reader::from_str(&xml);
        reader.config_mut().trim_text(false);

        let mut buf = Vec::new();
        let mut tag_stack: Vec<String> = Vec::new();
        let mut current_paragraph = String::new();
        let mut heading_level: Option<u8> = None;

        // Track whether we're inside a paragraph's properties (w:pPr)
        let mut in_ppr = false;
        let mut in_rpr = false;
        let mut style_id: Option<String> = None;
        let mut is_bold = false;

        loop {
            match reader.read_event_into(&mut buf) {
                Ok(Event::Start(ref e)) => {
                    let tag = local_name(e.name().as_ref());
                    tag_stack.push(tag.clone());

                    match tag.as_str() {
                        "p" => {
                            current_paragraph.clear();
                            heading_level = None;
                            is_bold = false;
                            style_id = None;
                        }
                        "pPr" => in_ppr = true,
                        "rPr" => in_rpr = true,
                        _ => {}
                    }

                    // Detect heading style from w:pStyle val attribute
                    if tag == "pStyle" && in_ppr {
                        for attr in e.attributes().flatten() {
                            if local_name(attr.key.as_ref()) == "val" {
                                let val = String::from_utf8_lossy(&attr.value).to_string();
                                if let Some(lvl) = parse_heading_style(&val) {
                                    heading_level = Some(lvl);
                                }
                                style_id = Some(val);
                            }
                        }
                    }
                }

                Ok(Event::End(ref e)) => {
                    let tag = local_name(e.name().as_ref());
                    tag_stack.pop();

                    match tag.as_str() {
                        "pPr" => in_ppr = false,
                        "rPr" => in_rpr = false,
                        "p" => {
                            let text = current_paragraph.trim().to_string();
                            if !text.is_empty() {
                                if let Some(level) = heading_level {
                                    let id = format!("#/texts/{}", doc.body.len());
                                    doc.add_header(SectionHeaderItem {
                                        id,
                                        text,
                                        level,
                                        label: LayoutLabel::SectionHeader,
                                        prov: vec![],
                                    });
                                } else {
                                    let id = format!("#/texts/{}", doc.body.len());
                                    let label = match style_id.as_deref() {
                                        Some(s) if s.to_lowercase().contains("list") => {
                                            LayoutLabel::ListItem
                                        }
                                        _ => LayoutLabel::Text,
                                    };
                                    doc.add_text(TextItem {
                                        id,
                                        text,
                                        label,
                                        prov: vec![],
                                        orig: None,
                                        enumerated: None,
                                        marker: None,
                                    });
                                }
                            }
                            current_paragraph.clear();
                        }
                        _ => {}
                    }
                }

                Ok(Event::Text(ref e)) => {
                    // Only add text inside <w:t> elements
                    if tag_stack.last().map(|t| t.as_str()) == Some("t") {
                        if let Ok(text) = e.unescape() {
                            current_paragraph.push_str(&text);
                        }
                    }
                }

                Ok(Event::Eof) => break,
                Err(e) => return Err(DoclingError::backend(format!("DOCX XML error: {}", e))),
                _ => {}
            }
            buf.clear();
        }

        Ok(doc)
    }
}

/// Extract local XML tag name (strip namespace prefix).
fn local_name(name: &[u8]) -> String {
    let s = String::from_utf8_lossy(name).to_string();
    if let Some(pos) = s.find(':') {
        s[pos + 1..].to_string()
    } else {
        s
    }
}

/// Parse OOXML heading style ID to heading level (1-6).
/// E.g. "Heading1" → Some(1), "heading2" → Some(2).
fn parse_heading_style(style: &str) -> Option<u8> {
    let lower = style.to_lowercase();
    if lower.starts_with("heading") {
        let rest = &lower["heading".len()..];
        rest.trim_start_matches(|c: char| c == ' ' || c == '-')
            .parse::<u8>()
            .ok()
            .filter(|&l| (1..=6).contains(&l))
    } else {
        None
    }
}

#[cfg(feature = "office")]
impl DocumentBackend for DocxBackend {
    fn is_valid(&self) -> bool {
        self.valid
    }
    fn supported_formats() -> &'static [InputFormat] {
        &[InputFormat::Docx]
    }
    fn unload(&mut self) {
        self.valid = false;
    }
}

#[cfg(feature = "office")]
impl DeclarativeBackend for DocxBackend {
    fn convert(&mut self) -> Result<DoclingDocument> {
        let bytes = self.source.read_bytes()?;
        let name = self.source.name().to_string();
        self.parse_docx(&bytes, &name)
    }
}
