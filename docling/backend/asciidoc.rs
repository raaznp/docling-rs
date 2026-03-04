use crate::backend::{BackendSource, DeclarativeBackend, DocumentBackend};
use crate::datamodel::base_models::{InputFormat, LayoutLabel};
use crate::datamodel::document::{
    DoclingDocument, DocumentOrigin, FormulaItem, ListItem, SectionHeaderItem, TextItem,
};
use crate::errors::Result;

pub struct AsciiDocBackend {
    source: BackendSource,
    valid: bool,
}
impl AsciiDocBackend {
    pub fn new(source: BackendSource) -> Self {
        Self {
            source,
            valid: true,
        }
    }
}

impl DocumentBackend for AsciiDocBackend {
    fn is_valid(&self) -> bool {
        self.valid
    }
    fn supported_formats() -> &'static [InputFormat] {
        &[InputFormat::Asciidoc]
    }
    fn unload(&mut self) {
        self.valid = false;
    }
}

impl DeclarativeBackend for AsciiDocBackend {
    fn convert(&mut self) -> Result<DoclingDocument> {
        let bytes = self.source.read_bytes()?;
        let content = String::from_utf8_lossy(&bytes).to_string();
        let name = self.source.name().to_string();
        let mut doc = DoclingDocument::new(&name);
        doc.origin = Some(DocumentOrigin {
            filename: name,
            mime_type: "text/asciidoc".into(),
            binary_hash: None,
            uri: None,
        });

        let mut idx = 0usize;
        for line in content.lines() {
            if line.starts_with("= ") {
                doc.add_header(SectionHeaderItem {
                    id: format!("#/texts/{}", idx),
                    text: line[2..].trim().to_string(),
                    level: 1,
                    label: LayoutLabel::Title,
                    prov: vec![],
                    formatting: None,
                    hyperlink: None,
                    annotations: vec![],
                });
            } else if line.starts_with("== ") {
                doc.add_header(SectionHeaderItem {
                    id: format!("#/texts/{}", idx),
                    text: line[3..].trim().to_string(),
                    level: 2,
                    label: LayoutLabel::SectionHeader,
                    prov: vec![],
                    formatting: None,
                    hyperlink: None,
                    annotations: vec![],
                });
            } else if line.starts_with("=== ") {
                doc.add_header(SectionHeaderItem {
                    id: format!("#/texts/{}", idx),
                    text: line[4..].trim().to_string(),
                    level: 3, // Corrected level back to 3
                    label: LayoutLabel::SectionHeader,
                    prov: vec![],
                    formatting: None,
                    hyperlink: None,
                    annotations: vec![],
                });
            } else if line.starts_with("* ") || line.starts_with("- ") {
                doc.add_list_item(ListItem {
                    id: format!("#/texts/{}", idx),
                    text: line
                        .trim_start_matches(|c| c == '*' || c == '-' || c == '.')
                        .trim()
                        .to_string(),
                    level: 0,
                    label: LayoutLabel::ListItem,
                    prov: vec![],
                    enumerated: Some(line.chars().next().map_or(false, |c| c.is_digit(10))),
                    marker: Some(line.chars().next().unwrap_or('-').into()),
                    formatting: None,
                    hyperlink: None,
                    annotations: vec![],
                });
            } else if !line.trim().is_empty() && !line.starts_with("//") {
                doc.add_text(TextItem {
                    id: format!("#/texts/{}", idx),
                    text: line.to_string(), // Corrected back to 'line'
                    label: LayoutLabel::Text,
                    prov: vec![],
                    orig: None,
                    enumerated: None,
                    marker: None,
                    formatting: None,
                    hyperlink: None,
                    annotations: vec![],
                });
            } else {
                continue;
            }
            idx += 1;
        }
        Ok(doc)
    }
}
