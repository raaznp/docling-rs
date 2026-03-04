use calamine::{open_workbook_auto_from_rs, DataType, Reader, Xlsx};
use docling_core::{
    base_models::InputFormat,
    doc_types::{
        DoclingDocument, DocumentOrigin, SectionHeaderItem, TableData, TableItem, TextItem,
    },
    errors::{DoclingError, Result},
    LayoutLabel,
};

use crate::{BackendSource, DeclarativeBackend, DocumentBackend};

/// Microsoft Excel (XLSX) backend.
/// Converts Excel workbooks to DoclingDocument with one table per sheet.
/// Mirrors `docling/backend/msexcel_backend.py`.
#[cfg(feature = "office")]
pub struct XlsxBackend {
    source: BackendSource,
    valid: bool,
}

#[cfg(feature = "office")]
impl XlsxBackend {
    pub fn new(source: BackendSource) -> Self {
        Self {
            source,
            valid: true,
        }
    }

    fn parse_xlsx(&self, data: &[u8], name: &str) -> Result<DoclingDocument> {
        use std::io::Cursor;

        let mut doc = DoclingDocument::new(name);
        doc.origin = Some(DocumentOrigin {
            filename: name.to_string(),
            mime_type: "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet"
                .to_string(),
            binary_hash: None,
            uri: None,
        });

        let cursor = Cursor::new(data);
        let mut workbook: Xlsx<_> = open_workbook_auto_from_rs(cursor)
            .map_err(|e| DoclingError::backend(format!("XLSX open error: {}", e)))?;

        let sheet_names: Vec<String> = workbook.sheet_names().to_vec();

        for sheet_name in sheet_names {
            if let Ok(range) = workbook.worksheet_range(&sheet_name) {
                // Add sheet name as a section heading
                let heading_id = format!("#/texts/{}", doc.body.len());
                doc.add_header(SectionHeaderItem {
                    id: heading_id,
                    text: sheet_name.clone(),
                    level: 2,
                    label: LayoutLabel::SectionHeader,
                    prov: vec![],
                });

                let num_rows = range.height() as u32;
                let num_cols = range.width() as u32;

                let cells: Vec<docling_core::Cell> = range
                    .rows()
                    .enumerate()
                    .flat_map(|(row_i, row)| {
                        row.iter().enumerate().map(move |(col_i, cell)| {
                            let text = match cell {
                                DataType::String(s) => s.clone(),
                                DataType::Float(f) => f.to_string(),
                                DataType::Int(i) => i.to_string(),
                                DataType::Bool(b) => b.to_string(),
                                DataType::DateTime(dt) => dt.to_string(),
                                DataType::Error(e) => format!("{:?}", e),
                                DataType::Empty => String::new(),
                                _ => String::new(),
                            };
                            docling_core::Cell {
                                id: col_i as u32,
                                text,
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

                let table_id = format!("#/tables/{}", doc.body.len());
                doc.add_table(TableItem {
                    id: table_id,
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
            }
        }

        Ok(doc)
    }
}

#[cfg(feature = "office")]
impl DocumentBackend for XlsxBackend {
    fn is_valid(&self) -> bool {
        self.valid
    }
    fn supported_formats() -> &'static [InputFormat] {
        &[InputFormat::Xlsx]
    }
    fn unload(&mut self) {
        self.valid = false;
    }
}

#[cfg(feature = "office")]
impl DeclarativeBackend for XlsxBackend {
    fn convert(&mut self) -> Result<DoclingDocument> {
        let bytes = self.source.read_bytes()?;
        let name = self.source.name().to_string();
        self.parse_xlsx(&bytes, &name)
    }
}
