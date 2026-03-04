use crate::backend::{BackendSource, DeclarativeBackend, DocumentBackend};
use crate::datamodel::base_models::{BoundingBox, Cell, InputFormat, LayoutLabel};
use crate::datamodel::document::{DoclingDocument, DocumentOrigin, TableData, TableItem};
use crate::errors::{DoclingError, Result};

pub struct CsvBackend {
    source: BackendSource,
    valid: bool,
    delimiter: u8,
    has_header: bool,
}

impl CsvBackend {
    pub fn new(source: BackendSource) -> Self {
        Self {
            source,
            valid: true,
            delimiter: b',',
            has_header: true,
        }
    }
    pub fn with_delimiter(mut self, d: u8) -> Self {
        self.delimiter = d;
        self
    }
}

impl DocumentBackend for CsvBackend {
    fn is_valid(&self) -> bool {
        self.valid
    }
    fn supported_formats() -> &'static [InputFormat] {
        &[InputFormat::Csv]
    }
    fn unload(&mut self) {
        self.valid = false;
    }
}

impl DeclarativeBackend for CsvBackend {
    fn convert(&mut self) -> Result<DoclingDocument> {
        let bytes = self.source.read_bytes()?;
        let name = self.source.name().to_string();
        let mut doc = DoclingDocument::new(&name);
        doc.origin = Some(DocumentOrigin {
            filename: name.clone(),
            mime_type: "text/csv".into(),
            binary_hash: None,
            uri: None,
        });

        let mut rdr = csv::Reader::from_reader(bytes.as_slice());
        let mut rows: Vec<Vec<String>> = Vec::new();
        let has_header = self.has_header;

        if has_header {
            if let Ok(hdrs) = rdr.headers() {
                rows.push(hdrs.iter().map(|h| h.to_string()).collect());
            }
        }
        for result in rdr.records() {
            let record = result.map_err(|e| DoclingError::backend(e.to_string()))?;
            rows.push(record.iter().map(|f| f.to_string()).collect());
        }

        if rows.is_empty() {
            return Ok(doc);
        }

        let num_rows = rows.len() as u32;
        let num_cols = rows.first().map(|r| r.len() as u32).unwrap_or(0);

        let cells: Vec<Cell> = rows
            .iter()
            .enumerate()
            .flat_map(|(row_i, row)| {
                row.iter().enumerate().map(move |(col_i, text)| Cell {
                    id: col_i as u32,
                    text: text.clone(),
                    bbox: BoundingBox::new(0.0, 0.0, 0.0, 0.0),
                    row_span: 1,
                    col_span: 1,
                    start_row: row_i as u32,
                    end_row: row_i as u32,
                    start_col: col_i as u32,
                    end_col: col_i as u32,
                    column_header: row_i == 0 && has_header,
                    row_header: false,
                    row_section: false,
                })
            })
            .collect();

        doc.add_table(TableItem {
            id: "#/tables/0".into(),
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
        Ok(doc)
    }
}
