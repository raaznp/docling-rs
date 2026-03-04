use docling_core::{
    base_models::InputFormat,
    doc_types::{
        CodeItem, DoclingDocument, DocumentMetadata, DocumentOrigin, FormulaItem, KeyValueItem,
        ListItem, PictureItem, ProvenanceRef, RefItem, SectionHeaderItem, TableData, TableItem,
        TextItem,
    },
    errors::{DoclingError, Result},
    LayoutLabel,
};
use scraper::{Html, Selector};
use std::collections::HashMap;

use crate::{BackendSource, DeclarativeBackend, DocumentBackend};

/// HTML document backend.
///
/// Converts an HTML document to a `DoclingDocument` using CSS-selector
/// based parsing. Mirrors `docling/backend/html_backend.py`.
pub struct HtmlBackend {
    source: BackendSource,
    valid: bool,
    base_url: Option<String>,
}

impl HtmlBackend {
    pub fn new(source: BackendSource, base_url: Option<String>) -> Self {
        Self {
            source,
            valid: true,
            base_url,
        }
    }

    fn parse_html(&self, html_str: &str) -> Result<DoclingDocument> {
        let document = Html::parse_document(html_str);
        let name = self.source.name().to_string();
        let mut doc = DoclingDocument::new(&name);

        // Set metadata
        if let Ok(title_sel) = Selector::parse("title") {
            if let Some(el) = document.select(&title_sel).next() {
                doc.metadata.title = Some(el.text().collect::<String>().trim().to_string());
            }
        }

        doc.origin = Some(DocumentOrigin {
            filename: name,
            mime_type: "text/html".to_string(),
            binary_hash: None,
            uri: self.base_url.clone(),
        });

        // Walk the body
        if let Ok(body_sel) = Selector::parse("body") {
            if let Some(body) = document.select(&body_sel).next() {
                self.walk_element(&body, &mut doc, 1);
            } else {
                // No explicit body — walk entire document
                let root = document.root_element();
                self.walk_element(&root, &mut doc, 1);
            }
        }

        Ok(doc)
    }

    fn walk_element(&self, el: &scraper::ElementRef, doc: &mut DoclingDocument, heading_depth: u8) {
        let tag = el.value().name().to_lowercase();
        let text: String = el.text().collect::<Vec<_>>().join("").trim().to_string();

        match tag.as_str() {
            // Headings → SectionHeaderItem
            "h1" | "h2" | "h3" | "h4" | "h5" | "h6" => {
                let level = tag[1..].parse::<u8>().unwrap_or(1);
                if !text.is_empty() {
                    let id = format!("#/texts/{}", doc.body.len());
                    doc.add_header(SectionHeaderItem {
                        id,
                        text,
                        level,
                        label: LayoutLabel::SectionHeader,
                        prov: vec![],
                    });
                }
            }

            // Paragraphs / div / span → TextItem
            "p" | "article" | "section" | "div" | "span" | "blockquote" => {
                if !text.is_empty() && !self.is_structural(el) {
                    let id = format!("#/texts/{}", doc.body.len());
                    doc.add_text(TextItem {
                        id,
                        text,
                        label: if tag == "blockquote" {
                            LayoutLabel::Text
                        } else {
                            LayoutLabel::Text
                        },
                        prov: vec![],
                        orig: None,
                        enumerated: None,
                        marker: None,
                    });
                } else {
                    // Recurse into structural elements
                    for child in el.children() {
                        if let Some(child_el) = scraper::ElementRef::wrap(child) {
                            self.walk_element(&child_el, doc, heading_depth);
                        }
                    }
                }
            }

            // Lists → ListItem
            "ul" | "ol" => {
                let enumerated = tag == "ol";
                let mut counter = 1u32;
                if let Ok(li_sel) = Selector::parse("li") {
                    for li in el.select(&li_sel) {
                        let li_text: String =
                            li.text().collect::<Vec<_>>().join("").trim().to_string();
                        if !li_text.is_empty() {
                            let marker = if enumerated {
                                format!("{}.", counter)
                            } else {
                                "-".to_string()
                            };
                            counter += 1;
                            let id = format!("#/texts/{}", doc.body.len());
                            doc.add_list_item(ListItem {
                                id,
                                text: li_text,
                                label: LayoutLabel::ListItem,
                                prov: vec![],
                                enumerated: Some(enumerated),
                                marker: Some(marker),
                            });
                        }
                    }
                }
            }

            // Code blocks
            "pre" => {
                let code_text: String = el.text().collect::<Vec<_>>().join("");
                if !code_text.trim().is_empty() {
                    // Try to detect language from a nested <code class="language-xxx">
                    let lang = el
                        .select(&Selector::parse("code").unwrap())
                        .next()
                        .and_then(|c| c.value().attr("class"))
                        .and_then(|cls| {
                            cls.split_whitespace()
                                .find(|c| c.starts_with("language-"))
                                .map(|c| c.trim_start_matches("language-").to_string())
                        });
                    let id = format!("#/texts/{}", doc.body.len());
                    doc.add_code(CodeItem {
                        id,
                        text: code_text,
                        label: LayoutLabel::Code,
                        prov: vec![],
                        code_language: lang,
                    });
                }
            }

            // Tables → TableItem
            "table" => {
                if let Some(table_item) = self.parse_table(el, doc.body.len()) {
                    doc.add_table(table_item);
                }
            }

            // Images → PictureItem
            "img" => {
                let src = el.value().attr("src").unwrap_or("").to_string();
                let alt = el.value().attr("alt").unwrap_or("").to_string();
                let id = format!("#/pictures/{}", doc.body.len());
                doc.add_picture(PictureItem {
                    id,
                    label: LayoutLabel::Picture,
                    prov: vec![],
                    captions: if !alt.is_empty() {
                        Some(vec![RefItem {
                            ref_id: alt.clone(),
                        }])
                    } else {
                        None
                    },
                    description: if !alt.is_empty() { Some(alt) } else { None },
                    image_data: None,
                    classification: None,
                });
            }

            // Math / formula (MathML or LaTeX in <script>/<span>)
            "math" => {
                if !text.is_empty() {
                    let id = format!("#/texts/{}", doc.body.len());
                    doc.add_formula(FormulaItem {
                        id,
                        text,
                        label: LayoutLabel::Formula,
                        prov: vec![],
                    });
                }
            }

            // Recurse into everything else
            _ => {
                for child in el.children() {
                    if let Some(child_el) = scraper::ElementRef::wrap(child) {
                        self.walk_element(&child_el, doc, heading_depth);
                    }
                }
            }
        }
    }

    /// Returns true if the element is a structural container (has children that
    /// should be recursed into rather than treated as a flat text block).
    fn is_structural(&self, el: &scraper::ElementRef) -> bool {
        for child in el.children() {
            if let Some(child_el) = scraper::ElementRef::wrap(child) {
                let tag = child_el.value().name();
                if matches!(
                    tag,
                    "p" | "h1"
                        | "h2"
                        | "h3"
                        | "h4"
                        | "h5"
                        | "h6"
                        | "ul"
                        | "ol"
                        | "table"
                        | "pre"
                        | "blockquote"
                        | "div"
                        | "article"
                        | "section"
                ) {
                    return true;
                }
            }
        }
        false
    }

    /// Parse an HTML `<table>` element into a `TableItem`.
    fn parse_table(&self, el: &scraper::ElementRef, idx: usize) -> Option<TableItem> {
        let tr_sel = Selector::parse("tr").ok()?;
        let td_sel = Selector::parse("td, th").ok()?;

        let mut rows: Vec<Vec<docling_core::Cell>> = Vec::new();
        let mut row_idx = 0u32;

        for tr in el.select(&tr_sel) {
            let mut row_cells = Vec::new();
            let mut col_idx = 0u32;

            for td in tr.select(&td_sel) {
                let text: String = td.text().collect::<Vec<_>>().join("").trim().to_string();
                let is_header = td.value().name() == "th";
                let col_span: u32 = td
                    .value()
                    .attr("colspan")
                    .and_then(|v| v.parse().ok())
                    .unwrap_or(1);
                let row_span: u32 = td
                    .value()
                    .attr("rowspan")
                    .and_then(|v| v.parse().ok())
                    .unwrap_or(1);

                row_cells.push(docling_core::Cell {
                    id: col_idx,
                    text,
                    bbox: docling_core::BoundingBox::new(0.0, 0.0, 0.0, 0.0),
                    row_span,
                    col_span,
                    start_row: row_idx,
                    end_row: row_idx + row_span - 1,
                    start_col: col_idx,
                    end_col: col_idx + col_span - 1,
                    column_header: is_header && row_idx == 0,
                    row_header: is_header && col_idx == 0,
                    row_section: false,
                });
                col_idx += col_span;
            }

            rows.push(row_cells);
            row_idx += 1;
        }

        if rows.is_empty() {
            return None;
        }

        let num_rows = rows.len() as u32;
        let num_cols = rows.iter().map(|r| r.len() as u32).max().unwrap_or(0);
        let all_cells: Vec<docling_core::Cell> = rows.into_iter().flatten().collect();

        Some(TableItem {
            id: format!("#/tables/{}", idx),
            label: LayoutLabel::Table,
            prov: vec![],
            data: TableData {
                num_rows,
                num_cols,
                table_cells: all_cells,
                grid: None,
            },
            captions: None,
        })
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
        let html_str = String::from_utf8_lossy(&bytes).into_owned();
        self.parse_html(&html_str)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_html_heading() {
        let html = r#"<html><body><h1>Hello</h1><p>World</p></body></html>"#;
        let source = BackendSource::Bytes(html.as_bytes().to_vec(), "test.html".to_string());
        let mut backend = HtmlBackend::new(source, None);
        let doc = backend.convert().expect("conversion should succeed");
        assert_eq!(doc.body.len(), 2);
    }
}
