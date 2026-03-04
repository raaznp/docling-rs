#[cfg(feature = "office")]
use crate::backend::{BackendSource, DeclarativeBackend, DocumentBackend};
use crate::datamodel::base_models::InputFormat;
use crate::datamodel::base_models::LayoutLabel;
use crate::datamodel::document::{
    CodeItem, DoclingDocument, DocumentOrigin, SectionHeaderItem, TextItem,
};
use crate::errors::{DoclingError, Result};

pub struct DocxBackend {
    source: BackendSource,
    valid: bool,
}
impl DocxBackend {
    pub fn new(source: BackendSource) -> Self {
        Self {
            source,
            valid: true,
        }
    }
}

impl crate::backend::DocumentBackend for DocxBackend {
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

impl crate::backend::DeclarativeBackend for DocxBackend {
    fn convert(&mut self) -> Result<DoclingDocument> {
        let bytes = self.source.read_bytes()?;
        let name = self.source.name().to_string();
        let mut doc = DoclingDocument::new(&name);
        doc.origin = Some(DocumentOrigin {
            filename: name,
            mime_type: "application/vnd.openxmlformats-officedocument.wordprocessingml.document"
                .into(),
            binary_hash: None,
            uri: None,
        });

        let docx = docx_rs::read_docx(&bytes)
            .map_err(|e| DoclingError::backend(format!("DOCX parse error: {:?}", e)))?;
        let mut idx = 0usize;
        for child in &docx.document.children {
            if let docx_rs::DocumentChild::Paragraph(p) = child {
                let text: String = p
                    .children
                    .iter()
                    .filter_map(|c| {
                        if let docx_rs::ParagraphChild::Run(r) = c {
                            Some(
                                r.children
                                    .iter()
                                    .filter_map(|rc| {
                                        if let docx_rs::RunChild::Text(t) = rc {
                                            Some(t.text.as_str())
                                        } else {
                                            None
                                        }
                                    })
                                    .collect::<String>(),
                            )
                        } else {
                            None
                        }
                    })
                    .collect();

                let text = text.trim().to_string();
                if text.is_empty() {
                    continue;
                }

                let style = p
                    .property
                    .style
                    .as_ref()
                    .map(|s| s.val.as_str())
                    .unwrap_or("");
                match style {
                    "Heading1" => {
                        doc.add_header(SectionHeaderItem {
                            id: format!("#/texts/{}", idx),
                            text: text.to_string(),
                            level: 1,
                            label: LayoutLabel::SectionHeader,
                            prov: vec![],
                            formatting: None,
                            hyperlink: None,
                            annotations: vec![],
                        });
                    }
                    "Heading2" => {
                        doc.add_header(SectionHeaderItem {
                            id: format!("#/texts/{}", idx),
                            text: text.clone(),
                            level: 2,
                            label: LayoutLabel::SectionHeader,
                            prov: vec![],
                            formatting: None,
                            hyperlink: None,
                            annotations: vec![],
                        });
                    }
                    "Heading3" => {
                        doc.add_header(SectionHeaderItem {
                            id: format!("#/texts/{}", idx),
                            text: text.clone(),
                            level: 3,
                            label: LayoutLabel::SectionHeader,
                            prov: vec![],
                            formatting: None,
                            hyperlink: None,
                            annotations: vec![],
                        });
                    }
                    _ => {
                        doc.add_code(CodeItem {
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
                }
                idx += 1;
            }
        }
        Ok(doc)
    }
}
