use docling_core::{
    base_models::InputFormat,
    doc_types::{
        CodeItem, DoclingDocument, DocumentOrigin, FormulaItem, ListItem, PictureItem,
        SectionHeaderItem, TableData, TableItem, TextItem,
    },
    errors::{DoclingError, Result},
    LayoutLabel,
};

use crate::{BackendSource, DeclarativeBackend, DocumentBackend};

/// AsciiDoc backend.
/// Basic AsciiDoc parser that handles common block elements.
/// Mirrors `docling/backend/asciidoc_backend.py`.
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

    fn parse(&self, content: &str, name: &str) -> Result<DoclingDocument> {
        let mut doc = DoclingDocument::new(name);
        doc.origin = Some(DocumentOrigin {
            filename: name.to_string(),
            mime_type: "text/asciidoc".to_string(),
            binary_hash: None,
            uri: None,
        });

        let mut lines = content.lines().peekable();
        let mut in_code_block = false;
        let mut code_lines: Vec<String> = Vec::new();
        let mut code_lang: Option<String> = None;
        let mut paragraph_lines: Vec<String> = Vec::new();

        let flush_paragraph = |doc: &mut DoclingDocument, lines: &mut Vec<String>| {
            let text = lines.join(" ").trim().to_string();
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
            lines.clear();
        };

        while let Some(line) = lines.next() {
            // Code block delimiters
            if line.starts_with("----") {
                if in_code_block {
                    // End code block
                    let text = code_lines.join("\n");
                    let id = format!("#/texts/{}", doc.body.len());
                    doc.add_code(CodeItem {
                        id,
                        text,
                        label: LayoutLabel::Code,
                        prov: vec![],
                        code_language: code_lang.take(),
                    });
                    in_code_block = false;
                    code_lines.clear();
                } else {
                    flush_paragraph(&mut doc, &mut paragraph_lines);
                    in_code_block = true;
                }
                continue;
            }

            if in_code_block {
                code_lines.push(line.to_string());
                continue;
            }

            // Headings: = Title, == Section, === Subsection
            if let Some(rest) = line.strip_prefix('=') {
                let mut level = 1u8;
                let mut rem = rest;
                while rem.starts_with('=') {
                    level += 1;
                    rem = &rem[1..];
                }
                if rem.starts_with(' ') {
                    flush_paragraph(&mut doc, &mut paragraph_lines);
                    let text = rem.trim().to_string();
                    if !text.is_empty() {
                        let id = format!("#/texts/{}", doc.body.len());
                        // Level 1 is the document title
                        if level == 1 {
                            doc.metadata.title = Some(text.clone());
                        }
                        doc.add_header(SectionHeaderItem {
                            id,
                            text,
                            level,
                            label: LayoutLabel::SectionHeader,
                            prov: vec![],
                        });
                    }
                    continue;
                }
            }

            // List items: * bullet, . numbered
            if line.starts_with("* ") || line.starts_with(". ") {
                flush_paragraph(&mut doc, &mut paragraph_lines);
                let enumerated = line.starts_with(". ");
                let text = line[2..].trim().to_string();
                let marker = if enumerated {
                    "1.".to_string()
                } else {
                    "-".to_string()
                };
                let id = format!("#/texts/{}", doc.body.len());
                doc.add_list_item(ListItem {
                    id,
                    text,
                    label: LayoutLabel::ListItem,
                    prov: vec![],
                    enumerated: Some(enumerated),
                    marker: Some(marker),
                });
                continue;
            }

            // Block attribute [source,lang] before code block
            if line.starts_with("[source") {
                if let Some(lang) = line
                    .find(',')
                    .map(|i| line[i + 1..].trim_end_matches(']').trim().to_string())
                {
                    code_lang = Some(lang);
                }
                continue;
            }

            // Block title .Title
            if line.starts_with('.') && !line.starts_with("..") {
                // treat as caption / title — skip for now
                continue;
            }

            // Empty line = end of paragraph
            if line.trim().is_empty() {
                flush_paragraph(&mut doc, &mut paragraph_lines);
                continue;
            }

            // Accumulate paragraph text
            paragraph_lines.push(line.to_string());
        }

        flush_paragraph(&mut doc, &mut paragraph_lines);

        Ok(doc)
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
        let content = String::from_utf8_lossy(&bytes).into_owned();
        let name = self.source.name().to_string();
        self.parse(&content, &name)
    }
}
