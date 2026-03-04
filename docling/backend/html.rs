use crate::backend::{BackendSource, DeclarativeBackend, DocumentBackend};
use crate::datamodel::base_models::InputFormat;
use crate::datamodel::base_models::LayoutLabel;
use crate::datamodel::document::{DoclingDocument, DocumentOrigin, TextItem};
use crate::errors::{DoclingError, Result};

use scraper::{Html, Selector};

pub struct HtmlBackend {
    source: BackendSource,
    valid: bool,
}

impl HtmlBackend {
    pub fn new(source: BackendSource) -> Self {
        Self {
            source,
            valid: true,
        }
    }
}

impl DocumentBackend for HtmlBackend {
    fn is_valid(&self) -> bool {
        self.valid
    }
    fn supported_formats() -> &'static [InputFormat] {
        &[InputFormat::Html]
    }
    fn unload(&mut self) {
        self.valid = false;
    }
}

impl DeclarativeBackend for HtmlBackend {
    fn convert(&mut self) -> Result<DoclingDocument> {
        let bytes = self.source.read_bytes()?;
        let html_str = String::from_utf8_lossy(&bytes);
        let name = self.source.name().to_string();
        let mut doc = DoclingDocument::new(&name);
        doc.origin = Some(DocumentOrigin {
            filename: name.clone(),
            mime_type: "text/html".into(),
            binary_hash: None,
            uri: None,
        });

        let parsed = Html::parse_document(&html_str);
        let body_sel = Selector::parse("body").unwrap();
        let text_sel =
            Selector::parse("p, h1, h2, h3, h4, h5, h6, li, pre, code, blockquote").unwrap();

        let body = parsed
            .select(&body_sel)
            .next()
            .unwrap_or_else(|| parsed.root_element());
        let mut idx = 0usize;
        for el in body.select(&text_sel) {
            let tag = el.value().name();
            let text = el.text().collect::<String>().trim().to_string();
            if text.is_empty() {
                continue;
            }

            match tag {
                "h1" | "h2" | "h3" | "h4" | "h5" | "h6" => {
                    let level = tag[1..].parse::<u32>().unwrap_or(1);
                    doc.add_header(crate::datamodel::document::SectionHeaderItem {
                        id: format!("#/texts/{}", idx),
                        text: text.to_string(),
                        level,
                        label: LayoutLabel::SectionHeader,
                        prov: vec![],
                        formatting: None,
                        hyperlink: None,
                        annotations: vec![],
                    });
                }
                "li" => {
                    doc.add_list_item(crate::datamodel::document::ListItem {
                        id: format!("#/texts/{}", idx),
                        text: text.to_string(),
                        level: 0,
                        label: LayoutLabel::ListItem,
                        prov: vec![],
                        enumerated: Some(false),
                        marker: Some("-".into()),
                        formatting: None,
                        hyperlink: None,
                        annotations: vec![],
                    });
                }
                "pre" | "code" => {
                    doc.add_code(crate::datamodel::document::CodeItem {
                        id: format!("#/texts/{}", idx),
                        text: text.to_string(),
                        label: LayoutLabel::Code,
                        prov: vec![],
                        code_language: None,
                        formatting: None,
                        hyperlink: None,
                        annotations: vec![],
                    });
                }
                _ => {
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
                }
            }
            idx += 1;
        }
        Ok(doc)
    }
}
