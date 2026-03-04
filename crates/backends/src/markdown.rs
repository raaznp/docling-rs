use docling_core::{
    base_models::InputFormat,
    doc_types::{
        CodeItem, DoclingDocument, DocumentOrigin, FormulaItem, ListItem, PictureItem, RefItem,
        SectionHeaderItem, TableData, TableItem, TextItem,
    },
    errors::Result,
    LayoutLabel,
};
use pulldown_cmark::{Event, HeadingLevel, Options, Parser, Tag, TagEnd};

use crate::{BackendSource, DeclarativeBackend, DocumentBackend};

/// Markdown document backend.
///
/// Converts Markdown content to `DoclingDocument` using `pulldown-cmark`.
/// Mirrors `docling/backend/md_backend.py`.
pub struct MarkdownBackend {
    source: BackendSource,
    valid: bool,
    parse_front_matter: bool,
}

impl MarkdownBackend {
    pub fn new(source: BackendSource, parse_front_matter: bool) -> Self {
        Self {
            source,
            valid: true,
            parse_front_matter,
        }
    }

    fn parse_markdown(&self, md: &str, name: &str) -> Result<DoclingDocument> {
        let (front_matter, content) = if self.parse_front_matter {
            extract_front_matter(md)
        } else {
            (None, md)
        };

        let mut doc = DoclingDocument::new(name);
        doc.origin = Some(DocumentOrigin {
            filename: name.to_string(),
            mime_type: "text/markdown".to_string(),
            binary_hash: None,
            uri: None,
        });

        if let Some(fm) = front_matter {
            // Try to extract title from front matter
            if let Some(title_line) = fm.lines().find(|l| l.trim_start().starts_with("title:")) {
                doc.metadata.title = Some(
                    title_line
                        .trim_start_matches("title:")
                        .trim()
                        .trim_matches('"')
                        .trim_matches('\'')
                        .to_string(),
                );
            }
        }

        let opts = Options::all();
        let parser = Parser::new_ext(content, opts);

        let mut current_text = String::new();
        let mut current_heading_level: Option<u8> = None;
        let mut in_code_block = false;
        let mut code_lang: Option<String> = None;
        let mut code_text = String::new();
        let mut in_list = false;
        let mut list_is_ordered = false;
        let mut list_counter = 1u32;
        let mut in_table = false;
        let mut table_rows: Vec<Vec<String>> = Vec::new();
        let mut current_table_row: Vec<String> = Vec::new();
        let mut in_table_head = false;

        for event in parser {
            match event {
                // ── Headings ─────────────────────────────────────
                Event::Start(Tag::Heading { level, .. }) => {
                    current_heading_level = Some(heading_level_to_u8(level));
                    current_text.clear();
                }
                Event::End(TagEnd::Heading(_)) => {
                    if let Some(level) = current_heading_level.take() {
                        let id = format!("#/texts/{}", doc.body.len());
                        doc.add_header(SectionHeaderItem {
                            id,
                            text: current_text.trim().to_string(),
                            level,
                            label: LayoutLabel::SectionHeader,
                            prov: vec![],
                        });
                        current_text.clear();
                    }
                }

                // ── Paragraphs ───────────────────────────────────
                Event::Start(Tag::Paragraph) => {
                    current_text.clear();
                }
                Event::End(TagEnd::Paragraph) => {
                    let t = current_text.trim().to_string();
                    if !t.is_empty() {
                        let id = format!("#/texts/{}", doc.body.len());
                        doc.add_text(TextItem {
                            id,
                            text: t,
                            label: LayoutLabel::Text,
                            prov: vec![],
                            orig: None,
                            enumerated: None,
                            marker: None,
                        });
                    }
                    current_text.clear();
                }

                // ── Code blocks ──────────────────────────────────
                Event::Start(Tag::CodeBlock(kind)) => {
                    in_code_block = true;
                    code_lang = match kind {
                        pulldown_cmark::CodeBlockKind::Fenced(lang) => {
                            let s = lang.to_string();
                            if s.is_empty() {
                                None
                            } else {
                                Some(s)
                            }
                        }
                        _ => None,
                    };
                    code_text.clear();
                }
                Event::Code(text) if in_code_block => {
                    code_text.push_str(&text);
                }
                Event::End(TagEnd::CodeBlock) => {
                    let id = format!("#/texts/{}", doc.body.len());
                    doc.add_code(CodeItem {
                        id,
                        text: code_text.clone(),
                        label: LayoutLabel::Code,
                        prov: vec![],
                        code_language: code_lang.take(),
                    });
                    in_code_block = false;
                    code_text.clear();
                }

                // ── Lists ────────────────────────────────────────
                Event::Start(Tag::List(start)) => {
                    in_list = true;
                    list_is_ordered = start.is_some();
                    list_counter = start.unwrap_or(1) as u32;
                }
                Event::End(TagEnd::List(_)) => {
                    in_list = false;
                }
                Event::Start(Tag::Item) => {
                    current_text.clear();
                }
                Event::End(TagEnd::Item) => {
                    let t = current_text.trim().to_string();
                    if !t.is_empty() {
                        let marker = if list_is_ordered {
                            let m = format!("{}.", list_counter);
                            list_counter += 1;
                            m
                        } else {
                            "-".to_string()
                        };
                        let id = format!("#/texts/{}", doc.body.len());
                        doc.add_list_item(ListItem {
                            id,
                            text: t,
                            label: LayoutLabel::ListItem,
                            prov: vec![],
                            enumerated: Some(list_is_ordered),
                            marker: Some(marker),
                        });
                    }
                    current_text.clear();
                }

                // ── Tables ───────────────────────────────────────
                Event::Start(Tag::Table(_)) => {
                    in_table = true;
                    table_rows.clear();
                }
                Event::End(TagEnd::Table) => {
                    let num_rows = table_rows.len() as u32;
                    let num_cols = table_rows.first().map(|r| r.len() as u32).unwrap_or(0);
                    let cells: Vec<docling_core::Cell> = table_rows
                        .iter()
                        .enumerate()
                        .flat_map(|(row_i, row)| {
                            row.iter().enumerate().map(move |(col_i, cell_text)| {
                                docling_core::Cell {
                                    id: col_i as u32,
                                    text: cell_text.clone(),
                                    bbox: docling_core::BoundingBox::new(0.0, 0.0, 0.0, 0.0),
                                    row_span: 1,
                                    col_span: 1,
                                    start_row: row_i as u32,
                                    end_row: row_i as u32,
                                    start_col: col_i as u32,
                                    end_col: col_i as u32,
                                    column_header: row_i == 0,
                                    row_header: false,
                                    row_section: false,
                                }
                            })
                        })
                        .collect();

                    let id = format!("#/tables/{}", doc.body.len());
                    doc.add_table(TableItem {
                        id,
                        label: LayoutLabel::Table,
                        prov: vec![],
                        data: TableData {
                            num_rows,
                            num_cols,
                            table_cells: cells,
                            grid: None,
                        },
                        captions: None,
                    });
                    in_table = false;
                    table_rows.clear();
                }
                Event::Start(Tag::TableHead) => {
                    in_table_head = true;
                }
                Event::End(TagEnd::TableHead) => {
                    in_table_head = false;
                }
                Event::Start(Tag::TableRow) => {
                    current_table_row.clear();
                }
                Event::End(TagEnd::TableRow) => {
                    table_rows.push(current_table_row.clone());
                    current_table_row.clear();
                }
                Event::Start(Tag::TableCell) => {
                    current_text.clear();
                }
                Event::End(TagEnd::TableCell) => {
                    current_table_row.push(current_text.trim().to_string());
                    current_text.clear();
                }

                // ── Images ───────────────────────────────────────
                Event::Start(Tag::Image {
                    dest_url,
                    title,
                    alt_text,
                    ..
                }) => {
                    let id = format!("#/pictures/{}", doc.body.len());
                    let alt = alt_text.to_string();
                    let caption = if !alt.is_empty() {
                        Some(vec![RefItem {
                            ref_id: alt.clone(),
                        }])
                    } else {
                        None
                    };
                    doc.add_picture(PictureItem {
                        id,
                        label: LayoutLabel::Picture,
                        prov: vec![],
                        captions: caption,
                        description: if !alt.is_empty() { Some(alt) } else { None },
                        image_data: None,
                        classification: None,
                    });
                }

                // ── Inline math ──────────────────────────────────
                Event::InlineMath(math) | Event::DisplayMath(math) => {
                    let id = format!("#/texts/{}", doc.body.len());
                    doc.add_formula(FormulaItem {
                        id,
                        text: math.to_string(),
                        label: LayoutLabel::Formula,
                        prov: vec![],
                    });
                }

                // ── Text accumulation ────────────────────────────
                Event::Text(text) => {
                    current_text.push_str(&text);
                }
                Event::Code(text) => {
                    current_text.push_str(&text);
                }
                Event::SoftBreak => {
                    current_text.push(' ');
                }
                Event::HardBreak => {
                    current_text.push('\n');
                }

                _ => {}
            }
        }

        Ok(doc)
    }
}

fn heading_level_to_u8(level: HeadingLevel) -> u8 {
    match level {
        HeadingLevel::H1 => 1,
        HeadingLevel::H2 => 2,
        HeadingLevel::H3 => 3,
        HeadingLevel::H4 => 4,
        HeadingLevel::H5 => 5,
        HeadingLevel::H6 => 6,
    }
}

/// Extract YAML front matter from a Markdown string.
/// Returns (front_matter_str, rest_of_content).
fn extract_front_matter(md: &str) -> (Option<String>, &str) {
    if !md.starts_with("---") {
        return (None, md);
    }
    let rest = &md[3..];
    if let Some(end) = rest.find("\n---") {
        let fm = rest[..end].to_string();
        let content = &rest[end + 4..];
        (Some(fm), content)
    } else {
        (None, md)
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
        let md = String::from_utf8_lossy(&bytes).into_owned();
        let name = self.source.name().to_string();
        self.parse_markdown(&md, &name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_markdown_basic() {
        let md = "# Title\n\nHello world.\n\n- item one\n- item two\n";
        let source = BackendSource::Bytes(md.as_bytes().to_vec(), "test.md".to_string());
        let mut backend = MarkdownBackend::new(source, false);
        let doc = backend.convert().expect("conversion failed");
        // heading + paragraph + 2 list items
        assert_eq!(doc.body.len(), 4);
    }
}
