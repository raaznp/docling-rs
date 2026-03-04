use crate::backend::{BackendSource, DeclarativeBackend, DocumentBackend};
use crate::datamodel::base_models::{InputFormat, LayoutLabel};
use crate::datamodel::document::{
    Annotation, AnnotationKind, CodeItem, DoclingDocument, DocumentOrigin, ListItem,
    SectionHeaderItem, TextItem,
};
use crate::errors::Result;
use pulldown_cmark::{Event, Options, Parser, Tag, TagEnd};

pub struct MarkdownBackend {
    source: BackendSource,
    valid: bool,
}

impl MarkdownBackend {
    pub fn new(source: BackendSource) -> Self {
        Self {
            source,
            valid: true,
        }
    }
}

impl DocumentBackend for MarkdownBackend {
    fn is_valid(&self) -> bool {
        self.valid
    }
    fn supported_formats() -> &'static [InputFormat] {
        &[InputFormat::Md]
    }
    fn unload(&mut self) {
        self.valid = false;
    }
}

impl DeclarativeBackend for MarkdownBackend {
    fn convert(&mut self) -> Result<DoclingDocument> {
        let bytes = self.source.read_bytes()?;
        let md_str = String::from_utf8_lossy(&bytes).to_string();
        let name = self.source.name().to_string();
        let mut doc = DoclingDocument::new(&name);
        doc.origin = Some(DocumentOrigin {
            filename: name.clone(),
            mime_type: "text/markdown".into(),
            binary_hash: None,
            uri: None,
        });

        let mut options = Options::empty();
        options.insert(Options::ENABLE_TABLES);
        options.insert(Options::ENABLE_STRIKETHROUGH);
        options.insert(Options::ENABLE_TASKLISTS);
        let parser = Parser::new_ext(&md_str, options);
        let mut cur_text = String::new();
        let mut cur_heading: Option<u32> = None;
        let mut cur_annotations: Vec<Annotation> = Vec::new();
        // Stack of (index_in_cur_annotations, kind, start_offset)
        let mut tag_stack: Vec<(usize, AnnotationKind, usize)> = Vec::new();
        let mut list_depth = 0u32;
        let mut in_item = false;
        let mut is_enumerated = false;
        let mut code_lang = String::new();
        let mut idx = 0usize;
        let mut in_table = false;
        let mut table_cells: Vec<crate::datamodel::base_models::Cell> = Vec::new();
        let mut cur_row = 0;
        let mut cur_col = 0;

        for event in parser {
            match event {
                Event::Start(tag) => {
                    match tag {
                        Tag::Table(_) => {
                            in_table = true;
                            table_cells.clear();
                            cur_row = 0;
                            cur_col = 0;
                        }
                        Tag::TableHead | Tag::TableRow => {
                            cur_col = 0;
                        }
                        Tag::TableCell => {
                            cur_text.clear();
                            cur_annotations.clear();
                            tag_stack.clear();
                        }
                        Tag::Heading { level, .. } => {
                            cur_heading = Some(level as u32);
                            cur_text.clear();
                            cur_annotations.clear();
                            tag_stack.clear();
                        }
                        Tag::Paragraph => {
                            cur_text.clear();
                            cur_annotations.clear();
                            tag_stack.clear();
                        }
                        Tag::List(start) => {
                            if in_item && !cur_text.is_empty() {
                                // Commit parent item text before starting sub-list
                                doc.add_list_item(ListItem {
                                    id: format!("#/texts/{}", idx),
                                    text: cur_text.clone(),
                                    level: list_depth.saturating_sub(1),
                                    label: LayoutLabel::ListItem,
                                    prov: vec![],
                                    enumerated: Some(is_enumerated),
                                    marker: if is_enumerated {
                                        Some("1.".into())
                                    } else {
                                        Some("-".into())
                                    },
                                    formatting: None,
                                    hyperlink: None,
                                    annotations: cur_annotations.clone(),
                                });
                                idx += 1;
                                cur_text.clear();
                                cur_annotations.clear();
                            }
                            list_depth += 1;
                            is_enumerated = start.is_some();
                        }
                        Tag::Item => {
                            in_item = true;
                            cur_text.clear();
                            cur_annotations.clear();
                            tag_stack.clear();
                        }
                        Tag::Emphasis => {
                            let start = cur_text.chars().count();
                            let ann_idx = cur_annotations.len();
                            // Push placeholder
                            cur_annotations.push(Annotation {
                                start,
                                end: start,
                                kind: AnnotationKind::Italic,
                            });
                            tag_stack.push((ann_idx, AnnotationKind::Italic, start));
                        }
                        Tag::Strong => {
                            let start = cur_text.chars().count();
                            let ann_idx = cur_annotations.len();
                            cur_annotations.push(Annotation {
                                start,
                                end: start,
                                kind: AnnotationKind::Bold,
                            });
                            tag_stack.push((ann_idx, AnnotationKind::Bold, start));
                        }
                        Tag::Strikethrough => {
                            let start = cur_text.chars().count();
                            let ann_idx = cur_annotations.len();
                            cur_annotations.push(Annotation {
                                start,
                                end: start,
                                kind: AnnotationKind::Strikethrough,
                            });
                            tag_stack.push((ann_idx, AnnotationKind::Strikethrough, start));
                        }
                        Tag::Link { dest_url, .. } => {
                            let start = cur_text.chars().count();
                            let ann_idx = cur_annotations.len();
                            cur_annotations.push(Annotation {
                                start,
                                end: start,
                                kind: AnnotationKind::Link {
                                    url: dest_url.to_string(),
                                },
                            });
                            tag_stack.push((
                                ann_idx,
                                AnnotationKind::Link {
                                    url: dest_url.to_string(),
                                },
                                start,
                            ));
                        }
                        Tag::CodeBlock(kind) => {
                            code_lang = match kind {
                                pulldown_cmark::CodeBlockKind::Fenced(lang) => lang.to_string(),
                                _ => String::new(),
                            };
                            cur_text.clear();
                        }
                        _ => {}
                    }
                }
                Event::End(tag_end) => match tag_end {
                    TagEnd::Table => {
                        in_table = false;
                        doc.add_table(crate::datamodel::document::TableItem {
                            id: format!("#/texts/{}", idx),
                            label: LayoutLabel::Table,
                            prov: vec![],
                            data: crate::datamodel::document::TableData {
                                table_cells: table_cells.clone(),
                                num_rows: (cur_row) as u32,
                                num_cols: if table_cells.is_empty() {
                                    0
                                } else {
                                    table_cells.iter().map(|c| c.end_col).max().unwrap_or(0) as u32
                                },
                                grid: None,
                            },
                            captions: None,
                        });
                        idx += 1;
                    }
                    TagEnd::TableHead | TagEnd::TableRow => {
                        cur_row += 1;
                    }
                    TagEnd::TableCell => {
                        table_cells.push(crate::datamodel::base_models::Cell {
                            id: table_cells.len() as u32,
                            text: cur_text.clone(),
                            bbox: crate::datamodel::base_models::BoundingBox::new(
                                0.0, 0.0, 0.0, 0.0,
                            ),
                            start_row: cur_row as u32,
                            end_row: cur_row as u32 + 1,
                            start_col: cur_col as u32,
                            end_col: cur_col as u32 + 1,
                            column_header: cur_row == 0,
                            row_header: false,
                            row_section: false,
                            row_span: 1,
                            col_span: 1,
                        });
                        cur_col += 1;
                        cur_text.clear();
                    }
                    TagEnd::Heading(_) => {
                        if let Some(level) = cur_heading.take() {
                            doc.add_header(SectionHeaderItem {
                                id: format!("#/texts/{}", idx),
                                text: cur_text.clone(),
                                level,
                                label: LayoutLabel::SectionHeader,
                                prov: vec![],
                                formatting: None,
                                hyperlink: None,
                                annotations: cur_annotations.clone(),
                            });
                            idx += 1;
                        }
                        cur_text.clear();
                        cur_annotations.clear();
                    }
                    TagEnd::Paragraph => {
                        if !cur_text.is_empty() && !in_table {
                            doc.add_text(TextItem {
                                id: format!("#/texts/{}", idx),
                                text: cur_text.clone(),
                                label: LayoutLabel::Text,
                                prov: vec![],
                                orig: None,
                                enumerated: None,
                                marker: None,
                                formatting: None,
                                hyperlink: None,
                                annotations: cur_annotations.clone(),
                            });
                            idx += 1;
                        }
                        cur_text.clear();
                        cur_annotations.clear();
                    }
                    TagEnd::List(_) => {
                        list_depth = list_depth.saturating_sub(1);
                    }
                    TagEnd::Item => {
                        if !cur_text.is_empty() {
                            doc.add_list_item(ListItem {
                                id: format!("#/texts/{}", idx),
                                text: cur_text.clone(),
                                level: list_depth.saturating_sub(1),
                                label: LayoutLabel::ListItem,
                                prov: vec![],
                                enumerated: Some(is_enumerated),
                                marker: if is_enumerated {
                                    Some("1.".into())
                                } else {
                                    Some("-".into())
                                },
                                formatting: None,
                                hyperlink: None,
                                annotations: cur_annotations.clone(),
                            });
                            idx += 1;
                        }
                        cur_text.clear();
                        cur_annotations.clear();
                        in_item = false;
                    }
                    TagEnd::Emphasis | TagEnd::Strong | TagEnd::Strikethrough | TagEnd::Link => {
                        if let Some((ann_idx, _kind, _start)) = tag_stack.pop() {
                            cur_annotations[ann_idx].end = cur_text.chars().count();
                        }
                    }
                    TagEnd::CodeBlock => {
                        doc.add_code(CodeItem {
                            id: format!("#/texts/{}", idx),
                            text: cur_text.clone(),
                            label: LayoutLabel::Code,
                            prov: vec![],
                            code_language: if code_lang.is_empty() {
                                None
                            } else {
                                Some(code_lang.clone())
                            },
                            formatting: None,
                            hyperlink: None,
                            annotations: vec![],
                        });
                        idx += 1;
                        cur_text.clear();
                    }
                    _ => {}
                },
                Event::Text(t) => cur_text.push_str(&t),
                Event::InlineMath(t) => {
                    cur_text.push_str("$");
                    cur_text.push_str(&t);
                    cur_text.push_str("$");
                }
                Event::DisplayMath(t) => {
                    cur_text.push_str("$$");
                    cur_text.push_str(&t);
                    cur_text.push_str("$$");
                }
                Event::Code(t) => {
                    if in_item || tag_stack.len() > 0 || !cur_text.is_empty() {
                        let start = cur_text.chars().count();
                        cur_text.push_str(&t);
                        cur_annotations.push(Annotation {
                            start,
                            end: cur_text.chars().count(),
                            kind: AnnotationKind::Code,
                        });
                    } else {
                        // For isolated code in a paragraph, docling parity often uses CodeItem
                        doc.add_code(CodeItem {
                            id: format!("#/texts/{}", idx),
                            text: t.to_string(),
                            label: LayoutLabel::Code,
                            prov: vec![],
                            code_language: None,
                            formatting: None,
                            hyperlink: None,
                            annotations: vec![],
                        });
                        idx += 1;
                    }
                }
                Event::Html(t) => {
                    // Commit current text if any before HTML
                    if !cur_text.is_empty() {
                        doc.add_text(TextItem {
                            id: format!("#/texts/{}", idx),
                            text: cur_text.clone(),
                            label: LayoutLabel::Text,
                            prov: vec![],
                            orig: None,
                            enumerated: None,
                            marker: None,
                            formatting: None,
                            hyperlink: None,
                            annotations: cur_annotations.clone(),
                        });
                        idx += 1;
                        cur_text.clear();
                        cur_annotations.clear();
                    }

                    let stripped = t
                        .replace("<div title=\"\">", "")
                        .replace("<div>", "")
                        .trim()
                        .to_string();

                    if !stripped.is_empty() && !stripped.starts_with("<!--") {
                        doc.add_text(TextItem {
                            id: format!("#/texts/{}", idx),
                            text: stripped,
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
                Event::SoftBreak => cur_text.push(' '),
                Event::HardBreak => cur_text.push('\n'),
                _ => {}
            }
        }
        Ok(doc)
    }
}
