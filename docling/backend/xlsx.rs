use crate::backend::{BackendSource, DeclarativeBackend, DocumentBackend};
use crate::datamodel::base_models::{BoundingBox, Cell, InputFormat, LayoutLabel};
use crate::datamodel::document::{
    DoclingDocument, DocumentOrigin, SectionHeaderItem, TableData, TableItem,
};
use crate::errors::{DoclingError, Result};

#[cfg(feature = "office")]
use calamine::{open_workbook_auto_from_rs, Data, Reader, Sheets};

pub struct XlsxBackend {
    source: BackendSource,
    valid: bool,
}
impl XlsxBackend {
    pub fn new(source: BackendSource) -> Self {
        Self {
            source,
            valid: true,
        }
    }
}

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

impl DeclarativeBackend for XlsxBackend {
    fn convert(&mut self) -> Result<DoclingDocument> {
        let bytes = self.source.read_bytes()?;
        let name = self.source.name().to_string();
        let mut doc = DoclingDocument::new(&name);
        doc.origin = Some(DocumentOrigin {
            filename: name.clone(),
            mime_type: "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet".into(),
            binary_hash: None,
            uri: None,
        });

        #[cfg(feature = "office")]
        {
            use std::io::Cursor;
            let cursor = Cursor::new(bytes.as_slice());
            let mut wb: Sheets<_> = open_workbook_auto_from_rs(cursor)
                .map_err(|e| DoclingError::backend(format!("Workbook error: {}", e)))?;

            let sheets: Vec<String> = wb.sheet_names().to_vec();
            for sheet_name in sheets {
                let range = match wb.worksheet_range(&sheet_name) {
                    Ok(r) => r,
                    Err(e) => {
                        log::warn!("Skipping sheet '{}': {}", sheet_name, e);
                        continue;
                    }
                };
                let idx = doc.body.len();
                doc.add_header(SectionHeaderItem {
                    id: format!("#/texts/{}", idx),
                    text: sheet_name.clone(),
                    level: 1,
                    label: LayoutLabel::SectionHeader,
                    prov: vec![],
                    formatting: None,
                    hyperlink: None,
                    annotations: vec![],
                });
                let nr = range.height() as u32;
                let nc = range.width() as u32;
                let mut cells = Vec::new();
                for (ri, row) in range.rows().enumerate() {
                    for (ci, cell) in row.iter().enumerate() {
                        cells.push(Cell {
                            id: ci as u32,
                            text: cell_to_str(cell),
                            bbox: BoundingBox::new(0.0, 0.0, 0.0, 0.0),
                            row_span: 1,
                            col_span: 1,
                            start_row: ri as u32,
                            end_row: ri as u32,
                            start_col: ci as u32,
                            end_col: ci as u32,
                            column_header: ri == 0,
                            row_header: false,
                            row_section: false,
                        });
                    }
                }
                doc.add_table(TableItem {
                    id: format!("#/tables/{}", doc.body.len()),
                    label: LayoutLabel::Table,
                    prov: vec![],
                    data: TableData {
                        num_rows: nr,
                        num_cols: nc,
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
fn cell_to_str(cell: &Data) -> String {
    match cell {
        Data::String(s) => s.clone(),
        Data::Float(f) => f.to_string(),
        Data::Int(i) => i.to_string(),
        Data::Bool(b) => b.to_string(),
        Data::DateTimeIso(dt) => dt.to_string(),
        Data::DurationIso(d) => d.to_string(),
        Data::Error(e) => format!("{:?}", e),
        _ => String::new(),
    }
}
